#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
renderer="$repo_root/skills/earnings-research/scripts/render_report_pdf.py"

RENDERER_PATH="$renderer" python3 - <<'PY'
import importlib.util
import os
from copy import deepcopy
from pathlib import Path
import subprocess
import tempfile

spec = importlib.util.spec_from_file_location("earnings_pdf_renderer", os.environ["RENDERER_PATH"])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

rendered = module.markdown_to_html(
    "## 核心指标\n\n"
    "| 指标 | 实际 | 预期 |\n"
    "|---|---:|:---|\n"
    "| 收入 | **59.50 亿美元** | 47.20 亿美元 |\n"
    "| EPS | 23.41 美元 | 14.62 美元 |\n"
)

assert "<table>" in rendered
assert "<th>指标</th>" in rendered
assert "<strong>59.50 亿美元</strong>" in rendered
assert "|---|" not in rendered
assert rendered.count("<tr>") == 3

workflow_html = module.build_html(
    "NVIDIA",
    "财报前瞻",
    "# NVIDIA公司财报前瞻分析\n# 1. 整体分析\n超出分析师预期。",
    None,
    report_date="2026-08-04",
)
assert "知识星球：巴芒科技" in workflow_html
assert "@top-left" in workflow_html
assert 'content: "HONE   2026/8/4"' in workflow_html
assert "background: #fff6ee" in workflow_html
assert "background: #f9dfcc" in workflow_html
assert ".news-item { margin: 0 0 4mm" in workflow_html
assert "font-size: 10.2pt; line-height: 1.58" in workflow_html
assert ".news-item:last-of-type { margin-bottom: 0; }" in workflow_html
assert "WORKFLOW FORMAT" not in workflow_html
assert '<div class="report-title">NVIDIA公司财报前瞻分析</div>' in workflow_html

# A transient blank-stderr Chrome exit used to make the native workflow give
# up even though the next isolated launch would succeed. Keep the retry and
# per-attempt profile boundary deterministic without requiring a real browser
# in this CI-safe contract test.
original_candidates = module.chromium_candidates
original_run = module.subprocess.run
render_calls = []
try:
    module.chromium_candidates = lambda: [Path("/fake/chrome")]

    def fake_run(command, **kwargs):
        render_calls.append(command)
        if len(render_calls) == 1:
            return subprocess.CompletedProcess(command, 21, "", "")
        output = next(item.split("=", 1)[1] for item in command if item.startswith("--print-to-pdf="))
        Path(output).write_bytes(b"%PDF-1.4\n" + b"x" * 1200)
        return subprocess.CompletedProcess(command, 0, "", "")

    module.subprocess.run = fake_run
    with tempfile.TemporaryDirectory(prefix="hone-earnings-render-test-") as temp_dir:
        module.render_pdf_with_chromium("<html></html>", Path(temp_dir) / "report.pdf")
finally:
    module.chromium_candidates = original_candidates
    module.subprocess.run = original_run

assert len(render_calls) == 2
assert "--disable-extensions" in render_calls[0]

analysis = (
    "# NVIDIA财报分析\n\n"
    "本报告解读 NVIDIA FY2026 Q1 财务报表，单位为亿美元。\n\n"
    "## 1. 利润表（Income Statement）解读：收入与利润同步增长\n\n- 收入增长。\n\n"
    "## 2. 资产负债表（Balance Sheet）解读：流动性保持充裕\n\n- 现金稳定。\n\n"
    "## 3. 现金流量表（Cash Flow Statement）解读：经营现金流改善\n\n- 现金流增长。\n\n"
    "## 4. 补充财务增长指标（Financial Growth）\n\n- 增长指标。\n\n"
    "## 数据总结\n\n盈利质量改善。"
)
module.validate_workflow_report("NVIDIA", "analysis", analysis)

preview = (
    "# NVIDIA公司财报前瞻分析\n"
    "# 1. 整体分析\n"
    "超出分析师预期，独立预测收入高出当前共识 2.2%，每股收益高出 5.6%。客户订单兑现与供给改善共同抬高本季出货，但成本回落仍需验证；财报后的关键不是需求有无，而是产品组合能否把收入增量转成利润。\n"
    "## 1.1 核心股价因素\n"
    "数据中心GPU供需与毛利率\n"
    "## 1.2 业绩指引 vs 机构观点\n"
    "### 1.2.1 核心结论\n"
    "过去三季的指引偏差没有因基数抬高而消失，订单与供给改善仍支撑收入和利润同时越过市场门槛，因此维持超出分析师预期的判断。\n"
    "### 1.2.2 财报假设\n"
    "FY2026 Q1 管理层指引锚点为收入 450 亿美元、调整后 EPS 0.90 美元；机构预期收入 450 亿美元、调整后 EPS 0.90 美元；独立预测收入 460 亿美元、调整后 EPS 0.95 美元，对应高出 2.2% 和 5.6%。收入中性带为 4.5 亿美元，EPS 中性区间为 0.018 美元。收入桥把历史指引偏差计入 +4.0 亿美元，数据中心出货再增加 +6.0 亿美元；产品组合和成本分别为 EPS 带来 +0.04 美元和 +0.01 美元。\n"
    "### 1.2.3 和机构分析对比\n"
    "历史上过去三季实际收入均高于管理层指引上限。截至 2026-08-04，当前股价 123.45 美元已经反映较强增长预期。Morgan Stanley 维持增持评级，目标价 150 美元，认为本季收入约 455 亿美元、本季 EPS 约 0.92 美元，主要担心供给爬坡；Goldman Sachs 给出买入建议，目标价 160 美元，本季收入约 458 亿美元、本季 EPS 约 0.94 美元，更看重产品组合。两家机构的收入和利润预期都低于 460 亿美元与 0.95 美元的独立预测。最新业绩会和演示材料显示，近期新产品发布与客户订单扩大了收入上行空间；其中已有订单已计入指引，额外供给改善部分计入，因此维持开头判断。\n"
    "## 1.3 近期新闻\n"
    "**2026-08-03** Morgan Stanley 更新 NVIDIA 观点，认为供给爬坡限制本季收入，但维持增持评级；这项机构预期的计入状态未知。来源：Morgan Stanley。\n\n"
    "**2026-08-01** NVIDIA 披露客户订单扩大，新增需求支持本季收入与出货，其中只有部分计入指引。来源：NVIDIA Investor Relations。\n\n"
    "**2026-07-30** 主要云客户确认采购计划，已核实的合作关系使其资本开支直接支持 NVIDIA 本季需求，但计入状态未知。来源：Customer Filing。\n\n"
    "**2026-07-28** NVIDIA 更新新产品交付节奏，产品组合改善并推高本季毛利，这部分已计入指引。来源：NVIDIA Product Release。\n\n"
    "**2026-07-25** Goldman Sachs 给出买入建议，认为本季收入和 EPS 高于管理层指引，但其机构预期计入状态未知。来源：Goldman Sachs。\n\n"
    "**2026-07-23** NVIDIA 管理层说明新增产能开始释放，预计增加本季出货，这部分已计入指引。来源：NVIDIA Operations Update。\n\n"
    "**2026-07-20** 上次 NVIDIA 财报和电话会确认订单、收入及 EPS 基线，并说明当前产品爬坡已计入指引。来源：NVIDIA Earnings Release。\n\n"
    "**2026-07-10** 关键供应商恢复供给，经过 NVIDIA 采购关系核实后可判断本季出货约束减轻，但该变化未计入指引。来源：Supplier Release。"
)
preview_audit = {
    "fiscal_period": "FY2026 Q1",
    "report_date": "2026-08-05",
    "consensus_as_of": "2026-08-04",
    "consensus_sources": [
        {"name": "Provider A", "as_of": "2026-08-04"},
        {"name": "Provider B", "as_of": "2026-08-03"},
    ],
    "institution_views": [
        {"institution": "Morgan Stanley", "as_of": "2026-08-03", "rating_or_recommendation": "增持", "target_price": "目标价 150 美元", "revenue_view": "本季收入约 455 亿美元", "profit_view": "本季 EPS 约 0.92 美元", "rationale": "供给爬坡仍是限制", "source_name": "Morgan Stanley", "source_url": "https://example.com/ms"},
        {"institution": "Goldman Sachs", "as_of": "2026-07-25", "rating_or_recommendation": "买入", "target_price": "目标价 160 美元", "revenue_view": "本季收入约 458 亿美元", "profit_view": "本季 EPS 约 0.94 美元", "rationale": "产品组合改善", "source_name": "Goldman Sachs", "source_url": "https://example.com/gs"},
    ],
    "market_context": {"quote_value": 123.45, "report_quote": "123.45 美元", "quote_as_of": "2026-08-04", "quote_source_name": "Provider A"},
    "metrics": {
        "revenue": {"anchor": 45000, "anchor_kind": "management_guidance_midpoint", "consensus": 45000, "forecast": 46000, "unit": "USD millions", "tolerance": 450, "tolerance_components": {"estimate_dispersion": 300, "revision_magnitude": 450, "measurement_precision": 100}, "report_scale": 0.01, "report_unit": "亿美元", "report_anchor_value": 450, "report_consensus_value": 450, "report_forecast_value": 460, "report_tolerance_value": 4.5, "report_anchor": "450 亿美元", "report_consensus": "450 亿美元", "report_forecast": "460 亿美元", "report_tolerance": "4.5 亿美元"},
        "adjusted_eps": {"anchor": 0.90, "anchor_kind": "management_guidance_point", "consensus": 0.90, "forecast": 0.95, "unit": "USD/share", "tolerance": 0.018, "tolerance_components": {"estimate_dispersion": 0.010, "revision_magnitude": 0.018, "measurement_precision": 0.005}, "report_scale": 1, "report_unit": "美元", "report_anchor_value": 0.90, "report_consensus_value": 0.90, "report_forecast_value": 0.95, "report_tolerance_value": 0.018, "report_anchor": "0.90 美元", "report_consensus": "0.90 美元", "report_forecast": "0.95 美元", "report_tolerance": "0.018 美元"},
    },
    "decision_metrics": ["revenue", "adjusted_eps"],
    "call": "超出分析师预期",
    "guidance_history": [
        {"period": "FY2025 Q2", "source": "company", "source_date": "2025-05-01", "deviations_pct": {"revenue": 4.0}},
        {"period": "FY2025 Q3", "source": "company", "source_date": "2025-08-01", "deviations_pct": {"revenue": 5.0}},
        {"period": "FY2025 Q4", "source": "company", "source_date": "2025-11-01", "deviations_pct": {"revenue": 6.0}},
    ],
    "guidance_inclusion": [
        {"catalyst": "new product", "affected_period": "FY2026 Q1", "status": "partial", "evidence": "earnings call 2026-07-01"}
    ],
    "forecast_bridge": [
        {"driver": "repeatable guidance bias", "category": "historical_bias", "metric": "revenue", "delta": 400, "report_delta_value": 4.0, "report_delta": "+4.0 亿美元", "direction": "up", "affected_period": "FY2026 Q1", "evidence": "three comparable company releases"},
        {"driver": "volume", "category": "volume", "metric": "revenue", "delta": 600, "report_delta_value": 6.0, "report_delta": "+6.0 亿美元", "direction": "up", "affected_period": "FY2026 Q1", "evidence": "earnings deck"},
        {"driver": "mix", "category": "mix", "metric": "adjusted_eps", "delta": 0.04, "report_delta_value": 0.04, "report_delta": "+0.04 美元", "direction": "up", "affected_period": "FY2026 Q1", "evidence": "earnings call"},
        {"driver": "cost", "category": "cost", "metric": "adjusted_eps", "delta": 0.01, "report_delta_value": 0.01, "report_delta": "+0.01 美元", "direction": "up", "affected_period": "FY2026 Q1", "evidence": "company filing"},
    ],
    "news_evidence": [
        {"date": "2026-08-03", "event_kind": "institution_view", "relevance": "company_direct", "event_summary": "Morgan Stanley updates NVIDIA view", "affected_period": "FY2026 Q1", "operating_link": "sets the revenue and EPS bar", "company_link": "direct analyst view about NVIDIA", "guidance_status": "unknown", "source_name": "Morgan Stanley", "source_url": "https://example.com/1"},
        {"date": "2026-08-01", "event_kind": "company_operating_update", "relevance": "company_direct", "event_summary": "customer orders expand", "affected_period": "FY2026 Q1", "operating_link": "supports revenue and shipments", "company_link": "NVIDIA disclosed the order update", "guidance_status": "partial", "source_name": "NVIDIA Investor Relations", "source_url": "https://example.com/2"},
        {"date": "2026-07-30", "event_kind": "named_customer", "relevance": "named_customer", "event_summary": "customer confirms procurement", "affected_period": "FY2026 Q1", "operating_link": "supports current demand", "company_link": "the disclosed procurement relationship directly covers NVIDIA accelerators", "guidance_status": "unknown", "source_name": "Customer Filing", "source_url": "https://example.com/3"},
        {"date": "2026-07-28", "event_kind": "company_operating_update", "relevance": "company_direct", "event_summary": "product delivery update", "affected_period": "FY2026 Q1", "operating_link": "improves mix and gross margin", "company_link": "NVIDIA issued the product release", "guidance_status": "included", "source_name": "NVIDIA Product Release", "source_url": "https://example.com/4"},
        {"date": "2026-07-25", "event_kind": "institution_view", "relevance": "company_direct", "event_summary": "Goldman Sachs gives Buy view", "affected_period": "FY2026 Q1", "operating_link": "sets revenue and EPS expectations", "company_link": "direct analyst view about NVIDIA", "guidance_status": "unknown", "source_name": "Goldman Sachs", "source_url": "https://example.com/5"},
        {"date": "2026-07-23", "event_kind": "company_operating_update", "relevance": "company_direct", "event_summary": "capacity starts releasing", "affected_period": "FY2026 Q1", "operating_link": "increases current shipments", "company_link": "NVIDIA management disclosed the capacity update", "guidance_status": "included", "source_name": "NVIDIA Operations Update", "source_url": "https://example.com/6"},
        {"date": "2026-07-20", "event_kind": "previous_earnings", "relevance": "company_direct", "event_summary": "previous earnings and call", "affected_period": "FY2026 Q1", "operating_link": "sets orders revenue and EPS baseline", "company_link": "NVIDIA earnings release and call", "guidance_status": "included", "source_name": "NVIDIA Earnings Release", "source_url": "https://example.com/7"},
        {"date": "2026-07-10", "event_kind": "peer_supply_chain", "relevance": "peer_supply_chain", "event_summary": "supplier restores supply", "affected_period": "FY2026 Q1", "operating_link": "reduces shipment constraints", "company_link": "the supplier names NVIDIA procurement and the affected accelerator component", "guidance_status": "not_included", "source_name": "Supplier Release", "source_url": "https://example.com/8"},
    ],
}
module.validate_workflow_report("NVIDIA", "preview", preview, preview_audit)

canonical_non_disclosure_report = (
    preview.replace("目标价 150 美元", "未披露目标价", 1)
    .replace("本季收入约 455 亿美元", "未披露单季营收预测", 1)
    .replace("本季 EPS 约 0.92 美元", "未披露单季EPS预测", 1)
)
canonical_non_disclosure_audit = deepcopy(preview_audit)
for field in ("target_price", "revenue_view", "profit_view"):
    canonical_non_disclosure_audit["institution_views"][0].pop(field)
module.validate_workflow_report(
    "NVIDIA",
    "preview",
    canonical_non_disclosure_report,
    canonical_non_disclosure_audit,
)
assert canonical_non_disclosure_audit["institution_views"][0]["target_price"] == "未披露目标价"
assert canonical_non_disclosure_audit["institution_views"][0]["revenue_view"] == "未披露单季营收预测"
assert canonical_non_disclosure_audit["institution_views"][0]["profit_view"] == "未披露单季EPS预测"

try:
    module.validate_workflow_report(
        "NVIDIA",
        "preview",
        preview.replace("关键供应商恢复供给", "关键供应商发布公告").replace("本季出货约束减轻", "没有直接财务贡献"),
        preview_audit,
    )
except ValueError as exc:
    message = str(exc)
    assert "preview news item 8" in message
    assert "收入" in message
    assert "本季" in message
else:
    raise AssertionError("news impact feedback must identify the exact invalid bullet")

wrong_revenue_unit = deepcopy(preview_audit)
revenue = wrong_revenue_unit["metrics"]["revenue"]
for field in ("anchor", "consensus", "forecast", "tolerance"):
    revenue[field] = revenue[field] / 1000
for field in revenue["tolerance_components"]:
    revenue["tolerance_components"][field] /= 1000
revenue["unit"] = "USD billions"
revenue["report_scale"] = 10
for item in wrong_revenue_unit["forecast_bridge"]:
    if item["metric"] == "revenue":
        item["delta"] /= 1000
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, wrong_revenue_unit)
except ValueError as exc:
    message = str(exc)
    assert "exactly USD millions" in message
    assert "normalize source revenue" in message
else:
    raise AssertionError("preview must reject ambiguous billion-based revenue normalization")

revenue_synonym_preview = (
    preview.replace("锚点为收入", "锚点为营收")
    .replace("；机构预期收入", "；机构预期营收")
    .replace("；独立预测收入", "；独立预测营收")
    .replace("收入中性带", "营收中性带")
    .replace("收入桥", "营收桥")
)
module.validate_workflow_report("NVIDIA", "preview", revenue_synonym_preview, preview_audit)

try:
    module.validate_workflow_report(
        "NVIDIA",
        "preview",
        preview.replace("收入中性带", "收入中性宽容带").replace("EPS 中性区间", "EPS 宽容范围"),
        preview_audit,
    )
except ValueError as exc:
    message = str(exc)
    assert "literal neutral-tolerance label" in message
    assert "中性带" in message
else:
    raise AssertionError("renderer must return an actionable literal for a missing neutral-band label")

inline_audit = deepcopy(preview_audit)
inline_audit["call"] = "与分析师持平"
inline_audit["metrics"]["revenue"]["forecast"] = 45200
inline_audit["metrics"]["revenue"]["report_forecast_value"] = 452
inline_audit["metrics"]["revenue"]["report_forecast"] = "452 亿美元"
inline_audit["metrics"]["adjusted_eps"]["forecast"] = 0.91
inline_audit["metrics"]["adjusted_eps"]["report_forecast_value"] = 0.91
inline_audit["metrics"]["adjusted_eps"]["report_forecast"] = "0.91 美元"
inline_audit["forecast_bridge"][0]["delta"] = 100
inline_audit["forecast_bridge"][0]["report_delta_value"] = 1.0
inline_audit["forecast_bridge"][0]["report_delta"] = "+1.0 亿美元"
inline_audit["forecast_bridge"][1]["delta"] = 100
inline_audit["forecast_bridge"][1]["report_delta_value"] = 1.0
inline_audit["forecast_bridge"][1]["report_delta"] = "+1.0 亿美元"
inline_audit["forecast_bridge"][2]["delta"] = 0.005
inline_audit["forecast_bridge"][2]["report_delta_value"] = 0.005
inline_audit["forecast_bridge"][2]["report_delta"] = "+0.005 美元"
inline_audit["forecast_bridge"][3]["delta"] = 0.005
inline_audit["forecast_bridge"][3]["report_delta_value"] = 0.005
inline_audit["forecast_bridge"][3]["report_delta"] = "+0.005 美元"
assert module.validate_preview_audit(inline_audit)[0] == "与分析师持平"

miss_audit = deepcopy(preview_audit)
miss_audit["call"] = "低于分析师预期"
miss_audit["metrics"]["revenue"]["forecast"] = 44000
miss_audit["metrics"]["revenue"]["report_forecast_value"] = 440
miss_audit["metrics"]["revenue"]["report_forecast"] = "440 亿美元"
miss_audit["metrics"]["adjusted_eps"]["forecast"] = 0.85
miss_audit["metrics"]["adjusted_eps"]["report_forecast_value"] = 0.85
miss_audit["metrics"]["adjusted_eps"]["report_forecast"] = "0.85 美元"
for item in miss_audit["forecast_bridge"]:
    item["delta"] = -abs(item["delta"])
    item["direction"] = "down"
    metric = miss_audit["metrics"][item["metric"]]
    item["report_delta_value"] = item["delta"] * metric["report_scale"]
    item["report_delta"] = f'{item["report_delta_value"]} {metric["report_unit"]}'
assert module.validate_preview_audit(miss_audit)[0] == "低于分析师预期"

try:
    module.validate_workflow_report("NVIDIA", "preview", preview.replace("# 1. 整体分析", "## 1. 整体分析"), preview_audit)
except ValueError:
    pass
else:
    raise AssertionError("preview must reject incorrect heading levels")

try:
    module.validate_workflow_report(
        "NVIDIA",
        "preview",
        preview.replace(
            "因此维持超出分析师预期的判断",
            "因此改为低于分析师预期的判断",
        ),
        preview_audit,
    )
except ValueError:
    pass
else:
    raise AssertionError("preview must reject inconsistent expectation calls")

for marker in ("数据时间：北京时间 2026-08-04", "事实：收入增长", "本轮未核验"):
    try:
        module.validate_workflow_report("NVIDIA", "preview", preview + marker, preview_audit)
    except ValueError:
        pass
    else:
        raise AssertionError(f"AI-style marker must be rejected: {marker}")

try:
    module.validate_workflow_report(
        "NVIDIA",
        "analysis",
        "# NVIDIA财报分析\n## 1. 核心结论\n普通问答格式",
    )
except ValueError:
    pass
else:
    raise AssertionError("normal Q&A headings must be rejected")

try:
    module.validate_workflow_report("NVIDIA", "analysis", analysis + "\n## 估值与价格含义\n不应出现")
except ValueError:
    pass
else:
    raise AssertionError("analysis must reject valuation sections")

try:
    module.validate_workflow_report("NVIDIA", "analysis", analysis.replace("盈利质量改善。", "事实：盈利质量改善。"))
except ValueError:
    pass
else:
    raise AssertionError("analysis must reject fact/inference labels")

try:
    module.validate_workflow_report("NVIDIA", "preview", preview)
except ValueError as exc:
    assert "preview_audit" in str(exc)
else:
    raise AssertionError("preview must require a structured audit")

contradictory = deepcopy(preview_audit)
contradictory["metrics"]["revenue"]["forecast"] = 44000
contradictory["metrics"]["revenue"]["report_forecast_value"] = 440
contradictory["metrics"]["revenue"]["report_forecast"] = "440 亿美元"
contradictory["metrics"]["adjusted_eps"]["forecast"] = 0.85
contradictory["metrics"]["adjusted_eps"]["report_forecast_value"] = 0.85
contradictory["metrics"]["adjusted_eps"]["report_forecast"] = "0.85 美元"
for item in contradictory["forecast_bridge"]:
    item["delta"] = -abs(item["delta"])
    item["direction"] = "down"
    metric = contradictory["metrics"][item["metric"]]
    item["report_delta_value"] = item["delta"] * metric["report_scale"]
    item["report_delta"] = f'{item["report_delta_value"]} {metric["report_unit"]}'
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, contradictory)
except ValueError as exc:
    assert "conflicts" in str(exc)
else:
    raise AssertionError("preview must reject a call that conflicts with the independent forecast")

arbitrary_tolerance = deepcopy(preview_audit)
arbitrary_tolerance["metrics"]["revenue"]["tolerance"] = 900
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, arbitrary_tolerance)
except ValueError as exc:
    assert "largest evidenced" in str(exc)
else:
    raise AssertionError("preview must reject an arbitrary neutral band")

broken_bridge = deepcopy(preview_audit)
broken_bridge["metrics"]["revenue"]["forecast"] = 46200
broken_bridge["metrics"]["revenue"]["report_forecast_value"] = 462
broken_bridge["metrics"]["revenue"]["report_forecast"] = "462 亿美元"
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, broken_bridge)
except ValueError as exc:
    assert "does not reconcile" in str(exc)
else:
    raise AssertionError("preview bridge must reconcile anchor and numeric deltas to forecast")

missing_history_bias = deepcopy(preview_audit)
missing_history_bias["forecast_bridge"][0]["category"] = "volume"
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, missing_history_bias)
except ValueError as exc:
    assert "historical" in str(exc)
else:
    raise AssertionError("preview must explicitly assess historical guidance bias")

consensus_copy = deepcopy(preview_audit)
consensus_copy["call"] = "与分析师持平"
for metric in consensus_copy["metrics"].values():
    metric["forecast"] = metric["consensus"]
    metric["report_forecast_value"] = metric["report_consensus_value"]
    metric["report_forecast"] = metric["report_consensus"]
for item in consensus_copy["forecast_bridge"]:
    item["delta"] = 0
    item["direction"] = "neutral"
    item["report_delta_value"] = 0
    item["report_delta"] = f'0 {consensus_copy["metrics"][item["metric"]]["report_unit"]}'
try:
    module.validate_preview_audit(consensus_copy)
except ValueError as exc:
    assert "cannot leave every" in str(exc)
else:
    raise AssertionError("preview cannot copy consensus without a quantified independent bridge")

thin_opening = preview.replace(
    "超出分析师预期，独立预测收入高出当前共识 2.2%，每股收益高出 5.6%。客户订单兑现与供给改善共同抬高本季出货，但成本回落仍需验证；财报后的关键不是需求有无，而是产品组合能否把收入增量转成利润。",
    "超出分析师预期。收入和利润走强。",
)
try:
    module.validate_workflow_report("NVIDIA", "preview", thin_opening, preview_audit)
except ValueError as exc:
    assert "substantive sentences" in str(exc)
else:
    raise AssertionError("preview opening must explain the call instead of printing a bare label")

single_source = deepcopy(preview_audit)
single_source["consensus_sources"] = [{"name": "Provider A", "as_of": "2026-08-04"}]
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, single_source)
except ValueError as exc:
    assert "consensus_limitations" in str(exc)
else:
    raise AssertionError("a single consensus source must disclose its limitation")

thin_history = deepcopy(preview_audit)
thin_history["guidance_history"] = thin_history["guidance_history"][:1]
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, thin_history)
except ValueError as exc:
    assert "history_limitations" in str(exc)
else:
    raise AssertionError("short guidance history must disclose its limitation")

missing_inclusion = deepcopy(preview_audit)
missing_inclusion["guidance_inclusion"] = []
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, missing_inclusion)
except ValueError as exc:
    assert "guidance_inclusion" in str(exc)
else:
    raise AssertionError("preview must audit whether catalysts were included in guidance")

published_mismatch = deepcopy(preview_audit)
published_mismatch["metrics"]["revenue"]["report_forecast"] = "470 亿美元"
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, published_mismatch)
except ValueError as exc:
    assert "numeric text" in str(exc)
else:
    raise AssertionError("published forecast values must match the structured audit")

missing_audited_display = deepcopy(preview_audit)
missing_audited_display["metrics"]["revenue"]["forecast"] = 47000
missing_audited_display["metrics"]["revenue"]["report_forecast_value"] = 470
missing_audited_display["metrics"]["revenue"]["report_forecast"] = "470 亿美元"
missing_audited_display["forecast_bridge"][1]["delta"] = 1600
missing_audited_display["forecast_bridge"][1]["report_delta_value"] = 16.0
missing_audited_display["forecast_bridge"][1]["report_delta"] = "+16.0 亿美元"
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, missing_audited_display)
except ValueError as exc:
    message = str(exc)
    assert "exact audited display strings" in message
    assert "revenue" in message
    assert "report_forecast=470 亿美元" in message
else:
    raise AssertionError("renderer feedback must name the exact audited display value to publish")

unit_mismatch = deepcopy(preview_audit)
unit_mismatch["forecast_bridge"][0]["report_delta_value"] = 0.40
unit_mismatch["forecast_bridge"][0]["report_delta"] = "+0.40 亿美元"
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, unit_mismatch)
except ValueError as exc:
    assert "report_scale" in str(exc)
else:
    raise AssertionError("preview must reject a bridge delta that misses the display-unit scale")

try:
    module.validate_workflow_report(
        "NVIDIA", "preview", preview.replace("## 1.3 近期新闻", "## 1.3 新闻"), preview_audit
    )
except ValueError:
    pass
else:
    raise AssertionError("preview must require the exact recent-news section")

too_few_news = preview.rsplit("\n\n**", 1)[0]
try:
    module.validate_workflow_report("NVIDIA", "preview", too_few_news, preview_audit)
except ValueError as exc:
    assert "one natural paragraph" in str(exc)
else:
    raise AssertionError("preview news page must contain at least eight items")

linked_news = preview.replace(
    "来源：Morgan Stanley。",
    "来源：[Morgan Stanley](https://example.com/news)。",
)
try:
    module.validate_workflow_report("NVIDIA", "preview", linked_news, preview_audit)
except ValueError as exc:
    assert "plain source names" in str(exc)
else:
    raise AssertionError("preview news must not display hyperlinks")

stale_news = (
    preview.replace("2026-07-30", "2026-07-21")
    .replace("2026-07-28", "2026-07-19")
    .replace("2026-07-25", "2026-07-18")
    .replace("2026-07-23", "2026-07-17")
    .replace("2026-07-20", "2026-07-16")
)
stale_audit = deepcopy(preview_audit)
for item in stale_audit["news_evidence"]:
    item["date"] = {
        "2026-07-30": "2026-07-21",
        "2026-07-28": "2026-07-19",
        "2026-07-25": "2026-07-18",
        "2026-07-23": "2026-07-17",
        "2026-07-20": "2026-07-16",
    }.get(item["date"], item["date"])
try:
    module.validate_workflow_report("NVIDIA", "preview", stale_news, stale_audit)
except ValueError as exc:
    assert "within 14 days" in str(exc)
else:
    raise AssertionError("at least half of preview news must be fresh")

weak_news_audit = deepcopy(preview_audit)
weak_news_audit["news_evidence"][5].update(
    {
        "event_kind": "named_customer",
        "relevance": "named_customer",
        "company_link": "the named customer relationship directly affects NVIDIA current-quarter demand",
    }
)
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, weak_news_audit)
except ValueError as exc:
    assert "company_direct" in str(exc)
else:
    raise AssertionError("preview news must remain primarily company-direct")

aggregator_institution_audit = deepcopy(preview_audit)
aggregator_institution_audit["institution_views"][0]["institution"] = "Seeking Alpha"
try:
    module.validate_workflow_report(
        "NVIDIA", "preview", preview, aggregator_institution_audit
    )
except ValueError as exc:
    assert "issuing broker, bank, or research house" in str(exc)
else:
    raise AssertionError("publishers and aggregators must not masquerade as institutions")

vague_rating_audit = deepcopy(preview_audit)
vague_rating_audit["institution_views"][0]["rating_or_recommendation"] = "Maintains"
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, vague_rating_audit)
except ValueError as exc:
    assert "actual Buy/Hold/Sell/Outperform-style stance" in str(exc)
else:
    raise AssertionError("institution comparison must include the actual rating stance")

homepage_source_audit = deepcopy(preview_audit)
homepage_source_audit["institution_views"][0]["source_url"] = "https://example.com/"
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, homepage_source_audit)
except ValueError as exc:
    assert "specific rating or research page" in str(exc)
else:
    raise AssertionError("institution evidence must not use a site homepage")

conference_news_audit = deepcopy(preview_audit)
conference_news_audit["news_evidence"][1]["event_summary"] = (
    "management will present at a technology summit"
)
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, conference_news_audit)
except ValueError as exc:
    assert "conference, price-move, or generic sector chatter" in str(exc)
else:
    raise AssertionError("conference attendance must not pad preview news")

missing_institution_comparison = preview.replace(
    "Goldman Sachs 给出买入建议",
    "另一家机构给出买入建议",
    1,
)
try:
    module.validate_workflow_report(
        "NVIDIA", "preview", missing_institution_comparison, preview_audit
    )
except ValueError as exc:
    assert "Goldman Sachs" in str(exc)
else:
    raise AssertionError("1.2.3 must compare each audited named institution")

news_html = module.markdown_to_html(
    "## 1.3 近期新闻\n\n"
    "**2026-08-03** 公司订单支持本季收入，其中部分计入指引。来源：公司公告。"
)
assert '<h2 class="news-section">1.3 近期新闻</h2>' in news_html
assert '<p class="news-item">' in news_html
assert '<ul class="news-list">' not in news_html
assert '<a href=' not in news_html
PY

echo "earnings PDF markdown table regression passed"
