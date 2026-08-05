#!/usr/bin/env python3

from __future__ import annotations

import argparse
from datetime import date, timedelta
import html
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import uuid
from pathlib import Path


MAX_REPORT_CHARS = 240_000
EXPECTATION_CALLS = ("超出分析师预期", "低于分析师预期", "与分析师持平")
AI_STYLE_MARKERS = (
    "数据时间",
    "行情口径",
    "事实：",
    "推断：",
    "结论：",
    "本轮",
    "研究行动",
    "证伪条件",
    "作为 AI",
    "作为AI",
    "根据工具",
    "以下内容仅供分析参考",
    "不要未经自己思考",
)


def emit(payload: dict) -> int:
    print(json.dumps(payload, ensure_ascii=False))
    return 0


def fail(message: str) -> int:
    return emit(
        {
            "success": False,
            "error": message,
            "fallback_message": "PDF 生成失败；请保留聊天中的完整报告文本。",
            "artifacts": [],
            "warnings": [],
        }
    )


def safe_name(value: str) -> str:
    cleaned = re.sub(r"[^0-9A-Za-z._-]+", "-", value.strip()).strip("-._")
    return cleaned[:80] or "earnings-report"


def load_spec() -> dict:
    parser = argparse.ArgumentParser()
    parser.add_argument("spec_json", nargs="?")
    parser.add_argument("--input", dest="input_path")
    args = parser.parse_args()
    if args.input_path:
        raw = Path(args.input_path).expanduser().read_text(encoding="utf-8")
    elif args.spec_json:
        raw = args.spec_json
    else:
        raise ValueError("missing JSON spec or --input path")
    value = json.loads(raw)
    if not isinstance(value, dict):
        raise ValueError("spec must be a JSON object")
    return value


def inline_markup(value: str) -> str:
    escaped = html.escape(value, quote=False)
    escaped = re.sub(r"`([^`]+)`", r"<code>\1</code>", escaped)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    escaped = re.sub(r"\[([^\]]+)\]\((https?://[^)]+)\)", r'<a href="\2">\1</a>', escaped)
    return escaped


def markdown_table_cells(line: str) -> list[str]:
    value = line.strip()
    if value.startswith("|"):
        value = value[1:]
    if value.endswith("|"):
        value = value[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", value)]


def is_markdown_table_separator(line: str) -> bool:
    cells = markdown_table_cells(line)
    return bool(cells) and all(re.fullmatch(r":?-{3,}:?", cell) for cell in cells)


def workflow_headings(markdown: str) -> list[str]:
    return [
        line.strip()
        for line in markdown.replace("\r\n", "\n").split("\n")
        if re.fullmatch(r"#{1,4}\s+(.+)", line.strip())
    ]


def expectation_call(value: str) -> str | None:
    normalized = value.lstrip()
    return next((item for item in EXPECTATION_CALLS if normalized.startswith(item)), None)


def expectation_calls_in(value: str) -> set[str]:
    return {item for item in EXPECTATION_CALLS if item in value}


def reject_ai_style_markers(report: str) -> None:
    found = [marker for marker in AI_STYLE_MARKERS if marker in report]
    if found:
        raise ValueError("report contains normal AI-answer meta language: " + ", ".join(found))


def finite_number(value: object, field: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{field} must be a finite number")
    number = float(value)
    if not math.isfinite(number):
        raise ValueError(f"{field} must be a finite number")
    return number


def first_report_number(value: str, field: str) -> float:
    match = re.search(r"[-+]?(?:\d+(?:\.\d*)?|\.\d+)", value.replace(",", ""))
    if not match:
        raise ValueError(f"{field} must contain a numeric value")
    return float(match.group(0))


def validate_report_scaled_value(
    *,
    raw_value: float,
    report_value: object,
    report_text: str,
    report_scale: float,
    report_unit: str,
    field: str,
) -> float:
    displayed = finite_number(report_value, field + "_value")
    expected = raw_value * report_scale
    epsilon = max(1e-9, abs(expected) * 1e-6)
    if abs(displayed - expected) > epsilon:
        raise ValueError(f"{field}_value must equal the audited value times report_scale")
    if report_unit not in report_text:
        raise ValueError(f"{field} must include report_unit")
    parsed = first_report_number(report_text, field)
    if abs(parsed - displayed) > epsilon:
        raise ValueError(f"{field} numeric text must match {field}_value")
    return displayed


def nonempty_string(value: object, field: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{field} must be a non-empty string")
    return value.strip()


def validate_iso_date(value: object, field: str) -> str:
    text = nonempty_string(value, field)
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", text):
        raise ValueError(f"{field} must use YYYY-MM-DD")
    return text


def parse_iso_date(value: object, field: str) -> date:
    text = validate_iso_date(value, field)
    try:
        return date.fromisoformat(text)
    except ValueError as exc:
        raise ValueError(f"{field} must be a valid calendar date") from exc


def validate_preview_audit(
    preview_audit: object,
) -> tuple[str, dict[str, dict[str, object]]]:
    if not isinstance(preview_audit, dict):
        raise ValueError("preview_audit is required for preview reports")

    nonempty_string(preview_audit.get("fiscal_period"), "preview_audit.fiscal_period")
    validate_iso_date(preview_audit.get("consensus_as_of"), "preview_audit.consensus_as_of")
    parse_iso_date(preview_audit.get("report_date"), "preview_audit.report_date")

    sources = preview_audit.get("consensus_sources")
    if not isinstance(sources, list) or not sources:
        raise ValueError("preview_audit.consensus_sources must contain at least one current source")
    for index, source in enumerate(sources):
        if not isinstance(source, dict):
            raise ValueError(f"preview_audit.consensus_sources[{index}] must be an object")
        nonempty_string(source.get("name"), f"preview_audit.consensus_sources[{index}].name")
        validate_iso_date(source.get("as_of"), f"preview_audit.consensus_sources[{index}].as_of")
    if len(sources) < 2:
        nonempty_string(
            preview_audit.get("consensus_limitations"),
            "preview_audit.consensus_limitations",
        )

    metrics = preview_audit.get("metrics")
    if not isinstance(metrics, dict) or "revenue" not in metrics:
        raise ValueError("preview_audit.metrics must contain revenue")
    profit_metrics = {"adjusted_eps", "gaap_eps", "operating_income", "ebitda", "net_income"}
    if not profit_metrics.intersection(metrics):
        raise ValueError("preview_audit.metrics must contain at least one profit metric")
    parsed_metrics: dict[str, dict[str, object]] = {}
    allowed_anchor_kinds = {
        "management_guidance_midpoint",
        "management_guidance_point",
        "segment_model",
        "margin_model",
    }
    for metric_name, metric in metrics.items():
        if not isinstance(metric, dict):
            raise ValueError(f"preview_audit.metrics.{metric_name} must be an object")
        anchor = finite_number(
            metric.get("anchor"), f"preview_audit.metrics.{metric_name}.anchor"
        )
        anchor_kind = nonempty_string(
            metric.get("anchor_kind"), f"preview_audit.metrics.{metric_name}.anchor_kind"
        )
        if anchor_kind not in allowed_anchor_kinds:
            raise ValueError(
                f"preview_audit.metrics.{metric_name}.anchor_kind must identify guidance or a model"
            )
        if metric_name == "revenue" and anchor_kind == "margin_model":
            raise ValueError("preview_audit.metrics.revenue cannot use a margin-model anchor")
        consensus = finite_number(
            metric.get("consensus"), f"preview_audit.metrics.{metric_name}.consensus"
        )
        forecast = finite_number(
            metric.get("forecast"), f"preview_audit.metrics.{metric_name}.forecast"
        )
        tolerance = finite_number(
            metric.get("tolerance"), f"preview_audit.metrics.{metric_name}.tolerance"
        )
        if tolerance <= 0:
            raise ValueError(f"preview_audit.metrics.{metric_name}.tolerance must be positive")
        tolerance_components = metric.get("tolerance_components")
        if not isinstance(tolerance_components, dict):
            raise ValueError(
                f"preview_audit.metrics.{metric_name}.tolerance_components must be an object"
            )
        component_values = []
        for component_name in (
            "estimate_dispersion",
            "revision_magnitude",
            "measurement_precision",
        ):
            component = finite_number(
                tolerance_components.get(component_name),
                f"preview_audit.metrics.{metric_name}.tolerance_components.{component_name}",
            )
            if component < 0:
                raise ValueError(
                    f"preview_audit.metrics.{metric_name}.tolerance_components.{component_name} "
                    "cannot be negative"
                )
            component_values.append(component)
        if component_values[-1] <= 0:
            raise ValueError(
                f"preview_audit.metrics.{metric_name}.tolerance_components.measurement_precision "
                "must be positive"
            )
        expected_tolerance = max(component_values)
        tolerance_epsilon = max(1e-9, abs(expected_tolerance) * 1e-6)
        if abs(tolerance - expected_tolerance) > tolerance_epsilon:
            raise ValueError(
                f"preview_audit.metrics.{metric_name}.tolerance must equal the largest evidenced "
                "tolerance component"
            )
        metric_unit = nonempty_string(
            metric.get("unit"), f"preview_audit.metrics.{metric_name}.unit"
        )
        report_scale = finite_number(
            metric.get("report_scale"), f"preview_audit.metrics.{metric_name}.report_scale"
        )
        if report_scale <= 0:
            raise ValueError(f"preview_audit.metrics.{metric_name}.report_scale must be positive")
        report_unit = nonempty_string(
            metric.get("report_unit"), f"preview_audit.metrics.{metric_name}.report_unit"
        )
        if metric_name == "revenue":
            if metric_unit != "USD millions":
                raise ValueError(
                    "preview_audit.metrics.revenue.unit must be exactly USD millions; "
                    "normalize source revenue before building the forecast bridge"
                )
            if report_unit != "亿美元" or abs(report_scale - 0.01) > 1e-12:
                raise ValueError(
                    "preview_audit.metrics.revenue must use report_unit=亿美元 and "
                    "report_scale=0.01 because 1 USD million equals 0.01 亿美元"
                )
        report_anchor = nonempty_string(
            metric.get("report_anchor"),
            f"preview_audit.metrics.{metric_name}.report_anchor",
        )
        report_consensus = nonempty_string(
            metric.get("report_consensus"),
            f"preview_audit.metrics.{metric_name}.report_consensus",
        )
        report_forecast = nonempty_string(
            metric.get("report_forecast"),
            f"preview_audit.metrics.{metric_name}.report_forecast",
        )
        report_tolerance = nonempty_string(
            metric.get("report_tolerance"),
            f"preview_audit.metrics.{metric_name}.report_tolerance",
        )
        validate_report_scaled_value(
            raw_value=anchor,
            report_value=metric.get("report_anchor_value"),
            report_text=report_anchor,
            report_scale=report_scale,
            report_unit=report_unit,
            field=f"preview_audit.metrics.{metric_name}.report_anchor",
        )
        validate_report_scaled_value(
            raw_value=consensus,
            report_value=metric.get("report_consensus_value"),
            report_text=report_consensus,
            report_scale=report_scale,
            report_unit=report_unit,
            field=f"preview_audit.metrics.{metric_name}.report_consensus",
        )
        validate_report_scaled_value(
            raw_value=forecast,
            report_value=metric.get("report_forecast_value"),
            report_text=report_forecast,
            report_scale=report_scale,
            report_unit=report_unit,
            field=f"preview_audit.metrics.{metric_name}.report_forecast",
        )
        validate_report_scaled_value(
            raw_value=tolerance,
            report_value=metric.get("report_tolerance_value"),
            report_text=report_tolerance,
            report_scale=report_scale,
            report_unit=report_unit,
            field=f"preview_audit.metrics.{metric_name}.report_tolerance",
        )
        parsed_metrics[str(metric_name)] = {
            "anchor": anchor,
            "anchor_kind": anchor_kind,
            "consensus": consensus,
            "forecast": forecast,
            "tolerance": tolerance,
            "report_scale": report_scale,
            "report_unit": report_unit,
            "report_anchor": report_anchor,
            "report_consensus": report_consensus,
            "report_forecast": report_forecast,
            "report_tolerance": report_tolerance,
        }

    decision_metrics = preview_audit.get("decision_metrics")
    if not isinstance(decision_metrics, list) or len(decision_metrics) < 2:
        raise ValueError("preview_audit.decision_metrics must contain at least two metrics")
    if len(set(decision_metrics)) != len(decision_metrics):
        raise ValueError("preview_audit.decision_metrics cannot contain duplicates")
    if "revenue" not in decision_metrics or not profit_metrics.intersection(decision_metrics):
        raise ValueError("preview_audit.decision_metrics must include revenue and a profit metric")

    states: list[str] = []
    for metric_name in decision_metrics:
        if metric_name not in parsed_metrics:
            raise ValueError(f"decision metric is missing from preview_audit.metrics: {metric_name}")
        metric = parsed_metrics[metric_name]
        consensus = float(metric["consensus"])
        forecast = float(metric["forecast"])
        tolerance = float(metric["tolerance"])
        delta = forecast - consensus
        if delta > tolerance:
            states.append("above")
        elif delta < -tolerance:
            states.append("below")
        else:
            states.append("inline")
    if "above" in states and "below" not in states:
        expected_call = "超出分析师预期"
    elif "below" in states and "above" not in states:
        expected_call = "低于分析师预期"
    else:
        expected_call = "与分析师持平"
    audit_call = nonempty_string(preview_audit.get("call"), "preview_audit.call")
    if audit_call not in EXPECTATION_CALLS:
        raise ValueError("preview_audit.call must be a supported expectation call")
    if audit_call != expected_call:
        raise ValueError(
            "preview_audit.call conflicts with forecast, consensus, or tolerance; "
            f"expected {expected_call}"
        )

    history = preview_audit.get("guidance_history")
    if not isinstance(history, list):
        raise ValueError("preview_audit.guidance_history must be a list")
    if len(history) < 3:
        nonempty_string(preview_audit.get("history_limitations"), "preview_audit.history_limitations")
    for index, item in enumerate(history):
        if not isinstance(item, dict):
            raise ValueError(f"preview_audit.guidance_history[{index}] must be an object")
        nonempty_string(item.get("period"), f"preview_audit.guidance_history[{index}].period")
        nonempty_string(item.get("source"), f"preview_audit.guidance_history[{index}].source")
        validate_iso_date(
            item.get("source_date"), f"preview_audit.guidance_history[{index}].source_date"
        )
        deviations = item.get("deviations_pct")
        if not isinstance(deviations, dict) or not deviations:
            raise ValueError(
                f"preview_audit.guidance_history[{index}].deviations_pct must not be empty"
            )
        for metric_name, value in deviations.items():
            finite_number(
                value,
                f"preview_audit.guidance_history[{index}].deviations_pct.{metric_name}",
            )

    inclusion = preview_audit.get("guidance_inclusion")
    if not isinstance(inclusion, list) or not inclusion:
        raise ValueError("preview_audit.guidance_inclusion must not be empty")
    allowed_statuses = {"included", "not_included", "partial", "unknown"}
    for index, item in enumerate(inclusion):
        if not isinstance(item, dict):
            raise ValueError(f"preview_audit.guidance_inclusion[{index}] must be an object")
        for field in ("catalyst", "affected_period", "evidence"):
            nonempty_string(item.get(field), f"preview_audit.guidance_inclusion[{index}].{field}")
        status = nonempty_string(
            item.get("status"), f"preview_audit.guidance_inclusion[{index}].status"
        )
        if status not in allowed_statuses:
            raise ValueError(
                f"preview_audit.guidance_inclusion[{index}].status must be one of "
                + ", ".join(sorted(allowed_statuses))
            )

    bridge = preview_audit.get("forecast_bridge")
    if not isinstance(bridge, list) or len(bridge) < 3:
        raise ValueError("preview_audit.forecast_bridge must contain at least three operating drivers")
    bridge_deltas: dict[str, list[float]] = {}
    has_revenue_history_bias = False
    allowed_categories = {
        "historical_bias",
        "volume",
        "price",
        "mix",
        "cost",
        "capacity",
        "product_ramp",
        "customer_timing",
        "fx",
        "other",
    }
    for index, item in enumerate(bridge):
        if not isinstance(item, dict):
            raise ValueError(f"preview_audit.forecast_bridge[{index}] must be an object")
        for field in ("driver", "metric", "affected_period", "evidence"):
            nonempty_string(item.get(field), f"preview_audit.forecast_bridge[{index}].{field}")
        direction = nonempty_string(
            item.get("direction"), f"preview_audit.forecast_bridge[{index}].direction"
        )
        if direction not in {"up", "down", "neutral"}:
            raise ValueError(
                f"preview_audit.forecast_bridge[{index}].direction must be up, down, or neutral"
            )
        category = nonempty_string(
            item.get("category"), f"preview_audit.forecast_bridge[{index}].category"
        )
        if category not in allowed_categories:
            raise ValueError(
                f"preview_audit.forecast_bridge[{index}].category must be a supported bridge category"
            )
        metric_name = nonempty_string(
            item.get("metric"), f"preview_audit.forecast_bridge[{index}].metric"
        )
        if metric_name not in parsed_metrics:
            raise ValueError(
                f"preview_audit.forecast_bridge[{index}].metric is missing from metrics"
            )
        delta = finite_number(
            item.get("delta"), f"preview_audit.forecast_bridge[{index}].delta"
        )
        report_delta = nonempty_string(
            item.get("report_delta"), f"preview_audit.forecast_bridge[{index}].report_delta"
        )
        metric = parsed_metrics[metric_name]
        validate_report_scaled_value(
            raw_value=delta,
            report_value=item.get("report_delta_value"),
            report_text=report_delta,
            report_scale=float(metric["report_scale"]),
            report_unit=str(metric["report_unit"]),
            field=f"preview_audit.forecast_bridge[{index}].report_delta",
        )
        if (delta > 0 and direction != "up") or (delta < 0 and direction != "down"):
            raise ValueError(
                f"preview_audit.forecast_bridge[{index}].direction conflicts with delta"
            )
        if delta == 0 and direction != "neutral":
            raise ValueError(
                f"preview_audit.forecast_bridge[{index}].zero delta must be neutral"
            )
        bridge_deltas.setdefault(metric_name, []).append(delta)
        item["_validated_report_delta"] = report_delta
        if metric_name == "revenue" and category == "historical_bias":
            has_revenue_history_bias = True

    if not has_revenue_history_bias:
        raise ValueError(
            "preview_audit.forecast_bridge must explicitly apply or reject the historical "
            "guidance bias for revenue"
        )
    for metric_name in decision_metrics:
        deltas = bridge_deltas.get(metric_name, [])
        if not deltas:
            raise ValueError(
                f"preview_audit.forecast_bridge must quantify at least one delta for {metric_name}"
            )
        if not any(delta != 0 for delta in deltas):
            raise ValueError(
                f"preview_audit.forecast_bridge cannot leave every {metric_name} adjustment at zero"
            )
        metric = parsed_metrics[metric_name]
        bridged_forecast = float(metric["anchor"]) + sum(deltas)
        forecast = float(metric["forecast"])
        bridge_epsilon = max(1e-9, abs(forecast) * 1e-6)
        if abs(bridged_forecast - forecast) > bridge_epsilon:
            raise ValueError(
                f"preview_audit.forecast_bridge does not reconcile anchor to forecast for {metric_name}"
            )
    return audit_call, parsed_metrics


def validate_workflow_report(
    company: str, mode: str, report: str, preview_audit: object | None = None
) -> None:
    reject_ai_style_markers(report)
    headings = workflow_headings(report)
    if mode == "preview":
        audit_call, parsed_metrics = validate_preview_audit(preview_audit)
        expected = [
            f"# {company}公司财报前瞻分析",
            "# 1. 整体分析",
            "## 1.1 核心股价因素",
            "## 1.2 业绩指引 vs 机构观点",
            "### 1.2.1 核心结论",
            "### 1.2.2 财报假设",
            "### 1.2.3 和机构分析对比",
            "## 1.3 近期新闻",
        ]
        if headings != expected:
            raise ValueError(
                "preview report must use the exact old Workflow headings and order: "
                + " -> ".join(expected)
            )
        if not re.match(
            rf"^# {re.escape(company)}公司财报前瞻分析\s*\n+# 1\. 整体分析\s*$",
            report,
            flags=re.MULTILINE,
        ):
            raise ValueError("preview must start directly with its title and 1. 整体分析")
        overall = re.search(
            r"^# 1\. 整体分析\s*$\n+(.+?)(?=^## 1\.1 核心股价因素\s*$)",
            report,
            flags=re.MULTILINE | re.DOTALL,
        )
        overall_call = expectation_call(overall.group(1)) if overall else None
        if overall_call is None:
            raise ValueError(
                "preview overall analysis must begin with 超出分析师预期、低于分析师预期、or 与分析师持平"
            )
        overall_text = overall.group(1).strip()
        compact_overall = re.sub(r"\s+", "", overall_text)
        if len(compact_overall) < 55 or len(re.findall(r"[。！？]", overall_text)) < 2:
            raise ValueError(
                "preview overall analysis must explain the call in at least two substantive sentences"
            )
        first_sentence = re.split(r"[。！？]", overall_text, maxsplit=1)[0].strip()
        if len(re.sub(r"\s+", "", first_sentence)) < 28 or not re.search(r"\d", first_sentence):
            raise ValueError(
                "preview overall analysis must attach a numerical reason to the call in its first sentence"
            )
        operating_terms = (
            "收入",
            "EPS",
            "每股收益",
            "利润",
            "毛利",
            "订单",
            "出货",
            "价格",
            "成本",
            "产能",
            "客户",
            "产品",
            "需求",
        )
        if sum(term in overall_text for term in operating_terms) < 2:
            raise ValueError(
                "preview overall analysis must name at least two company-relevant operating variables"
            )
        conclusion = re.search(
            r"^### 1\.2\.1 核心结论\s*$\n+(.+?)(?=^### 1\.2\.2 财报假设\s*$)",
            report,
            flags=re.MULTILINE | re.DOTALL,
        )
        conclusion_text = conclusion.group(1) if conclusion else ""
        conclusion_calls = expectation_calls_in(conclusion_text)
        if not conclusion_calls:
            raise ValueError(
                "preview 1.2.1 must make the expectation call unambiguous in its first paragraph"
            )
        if conclusion_calls != {overall_call}:
            raise ValueError("preview overall analysis and 1.2.1 must use the same expectation call")
        if overall_call != audit_call:
            raise ValueError("preview report call must match preview_audit.call")
        if re.search(r"(?m)^\s*\|", report):
            raise ValueError("preview must use connected prose instead of Markdown tables")
        if re.search(r"(?m)^\s*[-*]\s+", conclusion.group(1)):
            raise ValueError("preview 1.2.1 must be connected prose, not a checklist")
        assumptions = re.search(
            r"^### 1\.2\.2 财报假设\s*$\n+(.+?)(?=^### 1\.2\.3 和机构分析对比\s*$)",
            report,
            flags=re.MULTILINE | re.DOTALL,
        )
        assumptions_text = assumptions.group(1) if assumptions else ""
        required_assumption_groups = (
            ("机构预期",),
            ("独立预测",),
            ("收入", "营收"),
            ("%",),
        )
        if not all(any(term in assumptions_text for term in group) for group in required_assumption_groups):
            raise ValueError(
                "preview 1.2.2 must contain these literal fields: 机构预期, 独立预测, "
                "收入 or 营收, and a percentage gap containing %"
            )
        if not any(term in assumptions_text for term in ("EPS", "每股收益", "营业利润", "净利润", "EBITDA")):
            raise ValueError("preview 1.2.2 must include at least one profit forecast")
        if not any(term in assumptions_text for term in ("中性区间", "中性带", "持平区间", "容差")):
            raise ValueError(
                "preview 1.2.2 must contain one literal neutral-tolerance label: "
                "中性带, 中性区间, 持平区间, or 容差"
            )
        fiscal_period = nonempty_string(preview_audit.get("fiscal_period"), "preview_audit.fiscal_period")
        if fiscal_period not in assumptions_text:
            raise ValueError("preview 1.2.2 must use the audited fiscal period")
        for metric_name in preview_audit["decision_metrics"]:
            metric = parsed_metrics[metric_name]
            audited_display_values = {
                "report_anchor": str(metric["report_anchor"]),
                "report_consensus": str(metric["report_consensus"]),
                "report_forecast": str(metric["report_forecast"]),
                "report_tolerance": str(metric["report_tolerance"]),
            }
            missing_display_values = [
                f"{field}={value}"
                for field, value in audited_display_values.items()
                if value not in assumptions_text
            ]
            if missing_display_values:
                raise ValueError(
                    "preview 1.2.2 must contain these exact audited display strings for "
                    + metric_name
                    + ": "
                    + ", ".join(missing_display_values)
                )
        for item in preview_audit["forecast_bridge"]:
            if item.get("delta") != 0 and item.get("_validated_report_delta") not in assumptions_text:
                raise ValueError(
                    "preview 1.2.2 must publish every non-zero forecast-bridge delta in its audited display unit"
                )
        factor = re.search(
            r"^## 1\.1 核心股价因素\s*$\n+(.+?)(?=^## 1\.2 业绩指引 vs 机构观点\s*$)",
            report,
            flags=re.MULTILINE | re.DOTALL,
        )
        compact_factor = re.sub(r"\s+", "", factor.group(1)) if factor else ""
        if not compact_factor or len(compact_factor) > 30:
            raise ValueError("preview 1.1 must contain one company-specific sentence of at most 30 characters")
        comparison = re.search(
            r"^### 1\.2\.3 和机构分析对比\s*$\n+(.+?)(?=^## 1\.3 近期新闻\s*$)",
            report,
            flags=re.MULTILINE | re.DOTALL,
        )
        recent_event_terms = ("近期", "最近", "订单", "合同", "客户", "产品", "发布", "合作", "监管", "新闻")
        if not comparison or not any(term in comparison.group(1) for term in recent_event_terms):
            raise ValueError("preview 1.2.3 must connect a recent company event to the earnings assumptions")
        comparison_text = comparison.group(1)
        if "指引" not in comparison_text or not any(
            term in comparison_text for term in ("历史", "此前", "上一季", "过去", "前两季", "前三季")
        ):
            raise ValueError("preview 1.2.3 must compare historical guidance outcomes with current guidance")
        if not any(term in comparison_text for term in ("电话会", "业绩会", "演示材料", "投资者材料")):
            raise ValueError("preview 1.2.3 must incorporate the latest call or investor materials")
        if not any(term in comparison_text for term in ("已计入", "未计入", "部分计入", "是否计入")):
            raise ValueError("preview 1.2.3 must state whether major catalysts are included in guidance")
        news = re.search(
            r"^## 1\.3 近期新闻\s*$\n+(.+)$",
            report,
            flags=re.MULTILINE | re.DOTALL,
        )
        news_text = news.group(1) if news else ""
        news_items = re.findall(r"(?m)^[-*]\s+(.+)$", news_text)
        if not 8 <= len(news_items) <= 10:
            raise ValueError("preview 1.3 must contain eight to ten news bullets")
        news_pattern = re.compile(
            r"^(\d{4}-\d{2}-\d{2})｜类型：(公司|机构预期|同业|供应链|需求端)｜"
            r"事件：.+｜当季影响：.+｜指引计入："
            r"(?:已计入指引|未计入指引|部分计入指引|计入状态未知)｜"
            r"\[[^\]]+\]\(https?://[^)]+\)$"
        )
        operating_terms = ("收入", "利润", "毛利", "销量", "价格", "成本", "产能", "供给", "需求", "出货", "EPS", "本季")
        report_date = parse_iso_date(preview_audit.get("report_date"), "preview_audit.report_date")
        news_dates: list[date] = []
        news_categories: set[str] = set()
        for index, item in enumerate(news_items, start=1):
            match = news_pattern.fullmatch(item.strip())
            if not match:
                raise ValueError(
                    "each preview news item must include date, type, event, current-quarter impact, "
                    "guidance inclusion, and one source link"
                )
            if not any(term in item for term in operating_terms):
                raise ValueError(
                    f"preview news item {index} must explicitly use at least one operating or period "
                    f"impact term: {', '.join(operating_terms)}"
                )
            item_date = parse_iso_date(match.group(1), "preview news date")
            if item_date > report_date:
                raise ValueError("preview news items cannot be dated after the scheduled report date")
            news_dates.append(item_date)
            news_categories.add(match.group(2))
        if news_dates != sorted(news_dates, reverse=True):
            raise ValueError("preview news items must be in reverse chronological order")
        fresh_cutoff = report_date - timedelta(days=14)
        fresh_count = sum(fresh_cutoff <= item_date <= report_date for item_date in news_dates)
        if fresh_count * 2 < len(news_dates):
            raise ValueError(
                "at least half of preview news items must fall within 14 days before report_date"
            )
        required_families = (
            bool(news_categories.intersection({"公司", "机构预期"})),
            bool(news_categories.intersection({"同业", "供应链"})),
            "需求端" in news_categories,
        )
        if not all(required_families):
            raise ValueError(
                "preview news must cover company/expectations, peer/supply, and demand-side evidence"
            )
        return

    if mode == "analysis":
        if len(headings) != 6:
            raise ValueError("analysis report must contain exactly six old Workflow headings")
        if headings[0] != f"# {company}财报分析":
            raise ValueError(f"analysis title must be: {company}财报分析")
        patterns = [
            r"## 1\. 利润表（Income Statement）解读：.+",
            r"## 2\. 资产负债表（Balance Sheet）解读：.+",
            r"## 3\. 现金流量表（Cash Flow Statement）解读：.+",
            r"## 4\. 补充财务增长指标（Financial Growth）",
            r"## 数据总结",
        ]
        for heading, pattern in zip(headings[1:], patterns):
            if not re.fullmatch(pattern, heading):
                raise ValueError(
                    "analysis report must use the exact old Workflow section order; invalid heading: "
                    + heading
                )
        forbidden = (
            "Bull / Base / Bear",
            "估值与价格",
            "风险、证伪",
            "下一步跟踪",
            "投资建议",
            "利润表结论：",
            "资产负债表结论：",
            "现金流结论：",
        )
        if any(value in report for value in forbidden):
            raise ValueError("analysis report contains normal-Q&A sections forbidden by the old Workflow")
        return

    raise ValueError("mode must be preview or analysis")


def split_workflow_title(markdown: str) -> tuple[str, str]:
    title, separator, body = markdown.partition("\n")
    if separator and title.startswith("# "):
        return title[2:].strip(), body.lstrip("\n")
    return title.lstrip("# ").strip(), markdown


def markdown_to_html(markdown: str) -> str:
    lines = markdown.replace("\r\n", "\n").split("\n")
    chunks: list[str] = []
    paragraph: list[str] = []
    list_kind: str | None = None
    in_news_section = False

    def flush_paragraph() -> None:
        if paragraph:
            chunks.append(f"<p>{inline_markup(' '.join(paragraph))}</p>")
            paragraph.clear()

    def close_list() -> None:
        nonlocal list_kind
        if list_kind:
            chunks.append(f"</{list_kind}>")
            list_kind = None

    index = 0
    while index < len(lines):
        raw = lines[index]
        line = raw.strip()
        if not line:
            flush_paragraph()
            close_list()
            index += 1
            continue
        if (
            "|" in line
            and index + 1 < len(lines)
            and is_markdown_table_separator(lines[index + 1].strip())
        ):
            flush_paragraph()
            close_list()
            headers = markdown_table_cells(line)
            chunks.append("<div class=\"table-wrap\"><table><thead><tr>")
            chunks.extend(f"<th>{inline_markup(cell)}</th>" for cell in headers)
            chunks.append("</tr></thead><tbody>")
            index += 2
            while index < len(lines):
                row = lines[index].strip()
                if not row or "|" not in row:
                    break
                cells = markdown_table_cells(row)
                if len(cells) != len(headers):
                    break
                chunks.append("<tr>")
                chunks.extend(f"<td>{inline_markup(cell)}</td>" for cell in cells)
                chunks.append("</tr>")
                index += 1
            chunks.append("</tbody></table></div>")
            continue
        heading = re.match(r"^(#{1,4})\s+(.+)$", line)
        if heading:
            flush_paragraph()
            close_list()
            level = len(heading.group(1))
            heading_text = heading.group(2)
            in_news_section = heading_text == "1.3 近期新闻"
            heading_class = ' class="news-section"' if in_news_section else ""
            chunks.append(f"<h{level}{heading_class}>{inline_markup(heading_text)}</h{level}>")
            index += 1
            continue
        bullet = re.match(r"^[-*]\s+(.+)$", line)
        numbered = re.match(r"^\d+[.)]\s+(.+)$", line)
        if bullet or numbered:
            flush_paragraph()
            wanted = "ul" if bullet else "ol"
            if list_kind != wanted:
                close_list()
                list_kind = wanted
                list_class = ' class="news-list"' if in_news_section else ""
                chunks.append(f"<{wanted}{list_class}>")
            chunks.append(f"<li>{inline_markup((bullet or numbered).group(1))}</li>")
            index += 1
            continue
        if line.startswith(">"):
            flush_paragraph()
            close_list()
            chunks.append(f"<blockquote>{inline_markup(line.lstrip('> '))}</blockquote>")
            index += 1
            continue
        paragraph.append(line)
        index += 1

    flush_paragraph()
    close_list()
    return "\n".join(chunks)


def chromium_candidates() -> list[Path]:
    candidates: list[Path] = []
    for command in ["chromium", "chromium-browser", "google-chrome", "google-chrome-stable"]:
        located = shutil.which(command)
        if located:
            candidates.append(Path(located))
    candidates.extend(
        [
            Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            Path("/Applications/Chromium.app/Contents/MacOS/Chromium"),
        ]
    )
    home = Path.home()
    cache_roots = [home / "Library/Caches/ms-playwright", home / ".cache/ms-playwright"]
    patterns = [
        "chromium-*/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
        "chromium-*/chrome-linux/chrome",
        "chromium-*/chrome-linux64/chrome",
        "chromium_headless_shell-*/chrome-headless-shell-mac-arm64/chrome-headless-shell",
        "chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell",
    ]
    for root in cache_roots:
        for pattern in patterns:
            candidates.extend(sorted(root.glob(pattern), reverse=True))
    available: list[Path] = []
    seen: set[Path] = set()
    for path in candidates:
        resolved = path.resolve()
        if resolved in seen or not resolved.is_file() or not os.access(resolved, os.X_OK):
            continue
        seen.add(resolved)
        available.append(resolved)
    return available


def render_pdf_with_chromium(rendered_html: str, pdf_path: Path) -> None:
    browsers = chromium_candidates()
    if not browsers:
        raise RuntimeError("Chromium/Chrome executable not found")

    failures: list[str] = []
    with tempfile.TemporaryDirectory(prefix="hone-earnings-pdf-") as temp_dir:
        temp_root = Path(temp_dir)
        html_path = temp_root / "report.html"
        html_path.write_text(rendered_html, encoding="utf-8")
        for browser_index, chrome in enumerate(browsers):
            # Chrome can transiently reject a headless print with a blank
            # stderr. Retry once before falling through to another installed
            # Chromium build. `--disable-extensions` also avoids a fresh
            # headless profile staying alive on an injected extension page.
            for attempt in range(2):
                command = [
                    str(chrome),
                    "--headless",
                    "--disable-gpu",
                    "--disable-dev-shm-usage",
                    "--disable-extensions",
                    "--no-sandbox",
                    "--no-first-run",
                    "--allow-file-access-from-files",
                    "--no-pdf-header-footer",
                    "--print-to-pdf-no-header",
                    f"--print-to-pdf={pdf_path}",
                    html_path.as_uri(),
                ]
                try:
                    completed = subprocess.run(command, capture_output=True, text=True, timeout=45)
                except subprocess.TimeoutExpired:
                    failures.append(f"{chrome.name} attempt {attempt + 1} timed out after 45s")
                    pdf_path.unlink(missing_ok=True)
                    continue
                if completed.returncode == 0 and pdf_path.is_file() and pdf_path.stat().st_size >= 1_000:
                    return
                detail = (completed.stderr or completed.stdout).strip()[-800:]
                failures.append(
                    f"{chrome.name} attempt {attempt + 1} exited {completed.returncode}"
                    + (f": {detail}" if detail else "")
                )
                pdf_path.unlink(missing_ok=True)

    raise RuntimeError("Chromium PDF render failed: " + " | ".join(failures[-4:]))


def resolve_share_image() -> Path | None:
    explicit = os.environ.get("HONE_ZSXQ_SHARE_IMAGE", "").strip()
    if explicit and Path(explicit).is_file():
        return Path(explicit).resolve()
    repo_candidate = Path(__file__).resolve().parents[3] / "packages/app/public/membership_zsxq.jpg"
    return repo_candidate if repo_candidate.is_file() else None


def output_directory() -> Path:
    explicit = os.environ.get("HONE_SKILL_OUTPUT_DIR", "").strip()
    root = Path(explicit).expanduser() if explicit else Path.cwd() / "earnings-reports"
    root.mkdir(parents=True, exist_ok=True)
    return root.resolve()


def display_report_date(value: str | None) -> str:
    try:
        parsed = date.fromisoformat(value or "")
    except ValueError:
        parsed = date.today()
    return f"{parsed.year}/{parsed.month}/{parsed.day}"


def build_html(
    company: str,
    mode_label: str,
    report: str,
    share_image: Path | None,
    report_date: str | None = None,
) -> str:
    workflow_title, report_body = split_workflow_title(report)
    chrome_label = "1. 整体分析" if mode_label == "财报前瞻" else mode_label
    rendered_date = display_report_date(report_date)
    share_block = (
        f'<img src="{share_image.as_uri()}" alt="知识星球分享图">'
        if share_image
        else '<div class="share-fallback">知识星球 · 深度投研社区</div>'
    )
    return f"""<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><style>
@page {{
  size: A4;
  margin: 20mm 20mm 20mm;
  @top-left-corner {{ background: #fff6ee; border-bottom: .35mm solid #f2ded1; content: ""; }}
  @top-left {{
    content: "{html.escape(chrome_label)}";
    background: #fff6ee;
    border-bottom: .35mm solid #f2ded1;
    color: #68635f;
    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", sans-serif;
    font-size: 9pt;
  }}
  @top-center {{ background: #fff6ee; border-bottom: .35mm solid #f2ded1; content: ""; }}
  @top-right {{
    content: "HONE   {html.escape(rendered_date)}";
    background: #fff6ee;
    border-bottom: .35mm solid #f2ded1;
    color: #3f3b39;
    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", sans-serif;
    font-size: 11pt;
    font-weight: 800;
  }}
  @top-right-corner {{ background: #fff6ee; border-bottom: .35mm solid #f2ded1; content: ""; }}
  @bottom-left-corner {{ background: #fff6ee; border-top: .35mm solid #f2ded1; content: ""; }}
  @bottom-left {{
    content: "HONE 深度研究";
    background: #fff6ee;
    border-top: .35mm solid #f2ded1;
    color: #8e8a86;
    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", sans-serif;
    font-size: 8.5pt;
  }}
  @bottom-right {{
    content: "第 " counter(page) " 页 / 共 " counter(pages) " 页";
    background: #fff6ee;
    border-top: .35mm solid #f2ded1;
    color: #5d5a57;
    font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", sans-serif;
    font-size: 8.5pt;
    font-weight: 700;
  }}
  @bottom-center {{ background: #fff6ee; border-top: .35mm solid #f2ded1; content: ""; }}
  @bottom-right-corner {{ background: #fff6ee; border-top: .35mm solid #f2ded1; content: ""; }}
}}
* {{ box-sizing: border-box; }}
html {{ background: #fff; }}
body {{ margin: 0; color: #202b3a; font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif; font-size: 12.8pt; line-height: 1.82; text-align: justify; }}
.watermark {{ position: fixed; inset: 37% auto auto 3%; width: 94%; text-align: center; transform: rotate(-48deg); color: rgba(50, 57, 66, .075); font-size: 39pt; font-weight: 800; letter-spacing: .08em; z-index: -1; }}
.report-header {{ margin: 12mm 0 8mm; }}
.report-title {{ margin: 0; font-size: 17pt; line-height: 1.35; font-weight: 500; letter-spacing: .015em; }}
.report-meta {{ display: none; }}
h1 {{ margin: 0 0 5mm; padding-bottom: 2.5mm; border-bottom: .7mm solid #f1cdb6; color: #202b3a; font-size: 24pt; line-height: 1.25; font-weight: 800; break-after: avoid; }}
h2 {{ margin: 7mm 0 4mm; padding: 2.2mm 2.5mm; border-radius: 2mm; background: #f9dfcc; color: #202b3a; font-size: 17.5pt; line-height: 1.3; font-weight: 800; break-after: avoid; }}
.news-section {{ break-before: page; margin-top: 0; }}
.news-list {{ margin-top: 3mm; padding-left: 1.3em; font-size: 9.6pt; line-height: 1.48; text-align: left; }}
.news-list li {{ margin: 0 0 2.2mm; padding-left: .8mm; break-inside: avoid; }}
.news-list li:last-child {{ margin-bottom: 0; }}
h3 {{ margin: 2mm 0 4mm; padding: 1.7mm 2mm; border-radius: 1.8mm; background: #fff3e9; color: #344052; font-size: 14.5pt; line-height: 1.32; font-weight: 750; break-after: avoid; }}
h4 {{ margin: 5mm 0 2mm; font-size: 13pt; break-after: avoid; }}
p {{ margin: 0 0 4.6mm; orphans: 3; widows: 3; }}
ul, ol {{ margin: 3mm 0 5mm; padding-left: 1.65em; }}
li {{ margin: 1.5mm 0; }}
blockquote {{ margin: 12px 0; padding: 9px 12px; border-left: 3px solid #8ab6af; background: #f2f7f6; color: #44504e; }}
code {{ padding: 1px 4px; border-radius: 4px; background: #eef2f1; font-family: ui-monospace, monospace; }}
a {{ color: #202b3a; font-weight: 600; text-decoration: none; word-break: break-word; }}
.table-wrap {{ margin: 10px 0 16px; overflow: hidden; border: 1px solid #d9e4e1; border-radius: 7px; break-inside: avoid; }}
table {{ width: 100%; border-collapse: collapse; font-size: 10pt; line-height: 1.5; }}
th, td {{ padding: 7px 8px; border-right: 1px solid #d9e4e1; border-bottom: 1px solid #d9e4e1; text-align: left; vertical-align: top; overflow-wrap: anywhere; }}
th:last-child, td:last-child {{ border-right: 0; }}
tbody tr:last-child td {{ border-bottom: 0; }}
th {{ background: #eaf3f1; color: #174f47; font-weight: 750; }}
tbody tr:nth-child(even) {{ background: #f8faf9; }}
.disclaimer {{ margin-top: 7mm; padding-top: 3mm; border-top: .3mm dashed #ead8cb; color: #8a8580; font-size: 8.2pt; line-height: 1.55; text-align: left; }}
.share-page {{ break-before: page; min-height: 231mm; display: flex; flex-direction: column; align-items: center; justify-content: flex-start; padding-top: 8mm; text-align: center; }}
.share-page img {{ width: 145mm; max-height: 184mm; object-fit: contain; }}
.share-page .disclaimer {{ width: 100%; margin-top: 7mm; }}
.share-fallback {{ width: 108mm; padding: 36mm 10mm; background: #e9f5ef; border: 1px solid #9bc8b5; border-radius: 8px; color: #17634f; font-size: 18pt; font-weight: 800; }}
</style></head><body>
<div class="watermark">知识星球：巴芒科技</div>
<header class="report-header"><div class="report-title">{html.escape(workflow_title)}</div><div class="report-meta">{html.escape(company)} · {html.escape(mode_label)}</div></header>
<main>{markdown_to_html(report_body)}</main>
<section class="share-page">{share_block}<div class="disclaimer">免责声明：本报告仅供研究交流，不构成投资建议。数据、预测与判断可能存在错误或时效限制，请以公司正式披露及权威来源为准。</div></section>
</body></html>"""


def main() -> int:
    try:
        spec = load_spec()
        company = str(spec.get("company", "")).strip()
        mode = str(spec.get("mode", "")).strip().lower()
        report = str(spec.get("report_markdown", "")).strip()
        if not company:
            raise ValueError("company is required")
        if mode not in {"preview", "analysis"}:
            raise ValueError("mode must be preview or analysis")
        if not report:
            raise ValueError("report_markdown is required")
        if len(report) > MAX_REPORT_CHARS:
            raise ValueError(f"report_markdown exceeds {MAX_REPORT_CHARS} characters")
        validate_workflow_report(company, mode, report, spec.get("preview_audit"))

        out_dir = output_directory()
        base = safe_name(str(spec.get("output_name", "")) or f"{company}-{mode}")
        pdf_path = out_dir / f"{base}-{uuid.uuid4().hex[:8]}.pdf"
        mode_label = "财报前瞻" if mode == "preview" else "财报分析"
        report_date = None
        if mode == "preview" and isinstance(spec.get("preview_audit"), dict):
            report_date = str(spec["preview_audit"].get("consensus_as_of", "") or "")
        rendered_html = build_html(
            company,
            mode_label,
            report,
            resolve_share_image(),
            report_date=report_date,
        )

        render_pdf_with_chromium(rendered_html, pdf_path)
        return emit(
            {
                "success": True,
                "summary": f"已生成 {company} {mode_label} PDF",
                "artifacts": [
                    {
                        "kind": "document",
                        "path": str(pdf_path.resolve()),
                        "mime": "application/pdf",
                    }
                ],
                "warnings": [],
            }
        )
    except Exception as exc:
        return fail(str(exc))


if __name__ == "__main__":
    sys.exit(main())
