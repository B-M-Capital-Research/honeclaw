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
    "超出分析师预期。独立预测高于当前机构预期，订单和供给改善足以覆盖差额。\n"
    "## 1.1 核心股价因素\n"
    "数据中心GPU供需与毛利率\n"
    "## 1.2 业绩指引 vs 机构观点\n"
    "### 1.2.1 核心结论\n"
    "超出分析师预期。独立预测高于当前机构预期，订单与供给改善支撑收入和利润。\n"
    "### 1.2.2 财报假设\n"
    "FY2026 Q1 机构预期收入 450 亿美元、调整后 EPS 0.90 美元；独立预测收入 460 亿美元、调整后 EPS 0.95 美元，对应高出 2.2% 和 5.6%。收入中性带为 1.0%，EPS 中性区间为 2.0%。数据中心出货增加将推高收入，供应改善则有利于毛利率。\n"
    "### 1.2.3 和机构分析对比\n"
    "历史上过去三季实际收入均高于管理层指引上限。当前指引低于机构当前预期，但最新业绩会和演示材料显示，近期新产品发布与客户订单扩大了收入上行空间；其中已有订单已计入指引，额外供给改善部分计入，因此维持开头判断。\n"
    "## 1.3 近期新闻\n"
    "- 2026-08-03｜类型：公司｜事件：客户扩大订单｜当季影响：收入与需求上升｜指引计入：部分计入指引｜[公司](https://example.com/1)\n"
    "- 2026-08-01｜类型：机构预期｜事件：机构上修预期｜当季影响：收入门槛上升｜指引计入：计入状态未知｜[机构](https://example.com/2)\n"
    "- 2026-07-30｜类型：需求端｜事件：云厂商提高资本开支｜当季影响：数据中心需求增加｜指引计入：计入状态未知｜[客户](https://example.com/3)\n"
    "- 2026-07-28｜类型：同业｜事件：同业业绩超预期｜当季影响：验证行业收入与毛利改善｜指引计入：计入状态未知｜[同业](https://example.com/4)\n"
    "- 2026-07-25｜类型：供应链｜事件：供应持续偏紧｜当季影响：产品价格获得支撑｜指引计入：部分计入指引｜[行业](https://example.com/5)\n"
    "- 2026-07-20｜类型：公司｜事件：新产品开始出货｜当季影响：产品组合改善并推高毛利｜指引计入：已计入指引｜[公司](https://example.com/6)\n"
    "- 2026-07-10｜类型：供应链｜事件：供应恢复｜当季影响：出货约束减轻｜指引计入：未计入指引｜[公司](https://example.com/7)\n"
    "- 2026-07-02｜类型：公司｜事件：远期技术送样｜当季影响：本季收入没有贡献｜指引计入：计入状态未知｜[公司](https://example.com/8)"
)
preview_audit = {
    "fiscal_period": "FY2026 Q1",
    "report_date": "2026-08-05",
    "consensus_as_of": "2026-08-04",
    "consensus_sources": [
        {"name": "Provider A", "as_of": "2026-08-04"},
        {"name": "Provider B", "as_of": "2026-08-03"},
    ],
    "metrics": {
        "revenue": {"consensus": 45.0, "forecast": 46.0, "unit": "USD billions", "tolerance": 0.45, "report_consensus": "450 亿美元", "report_forecast": "460 亿美元"},
        "adjusted_eps": {"consensus": 0.90, "forecast": 0.95, "unit": "USD/share", "tolerance": 0.018, "report_consensus": "0.90 美元", "report_forecast": "0.95 美元"},
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
        {"driver": "volume", "metric": "revenue", "direction": "up", "affected_period": "FY2026 Q1", "evidence": "earnings deck"},
        {"driver": "mix", "metric": "adjusted_eps", "direction": "up", "affected_period": "FY2026 Q1", "evidence": "earnings call"},
        {"driver": "cost", "metric": "adjusted_eps", "direction": "down", "affected_period": "FY2026 Q1", "evidence": "company filing"},
    ],
}
module.validate_workflow_report("NVIDIA", "preview", preview, preview_audit)

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
            "超出分析师预期。独立预测高于当前机构预期，订单与供给改善支撑收入和利润。",
            "低于分析师预期。独立预测高于当前机构预期，订单与供给改善支撑收入和利润。",
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
contradictory["metrics"]["revenue"]["forecast"] = 44.0
contradictory["metrics"]["adjusted_eps"]["forecast"] = 0.85
try:
    module.validate_workflow_report("NVIDIA", "preview", preview, contradictory)
except ValueError as exc:
    assert "conflicts" in str(exc)
else:
    raise AssertionError("preview must reject a call that conflicts with the independent forecast")

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
    assert "published values" in str(exc)
else:
    raise AssertionError("published forecast values must match the structured audit")

try:
    module.validate_workflow_report(
        "NVIDIA", "preview", preview.replace("## 1.3 近期新闻", "## 1.3 新闻"), preview_audit
    )
except ValueError:
    pass
else:
    raise AssertionError("preview must require the exact recent-news section")

too_few_news = preview.rsplit("\n- ", 1)[0]
try:
    module.validate_workflow_report("NVIDIA", "preview", too_few_news, preview_audit)
except ValueError as exc:
    assert "eight to ten" in str(exc)
else:
    raise AssertionError("preview news page must contain at least eight items")

missing_news_impact = preview.replace("当季影响：收入与需求上升", "收入与需求上升")
try:
    module.validate_workflow_report("NVIDIA", "preview", missing_news_impact, preview_audit)
except ValueError as exc:
    assert "date, type, event" in str(exc)
else:
    raise AssertionError("each news item must include its current-quarter impact")

stale_news = (
    preview.replace("2026-07-20", "2026-07-17")
    .replace("2026-07-30", "2026-07-21")
    .replace("2026-07-28", "2026-07-19")
    .replace("2026-07-25", "2026-07-18")
)
try:
    module.validate_workflow_report("NVIDIA", "preview", stale_news, preview_audit)
except ValueError as exc:
    assert "within 14 days" in str(exc)
else:
    raise AssertionError("at least half of preview news must be fresh")

missing_demand = preview.replace("类型：需求端", "类型：公司")
try:
    module.validate_workflow_report("NVIDIA", "preview", missing_demand, preview_audit)
except ValueError as exc:
    assert "demand-side" in str(exc)
else:
    raise AssertionError("preview news must include downstream-demand evidence")

news_html = module.markdown_to_html("## 1.3 近期新闻\n\n- 2026-08-03｜类型：公司｜事件：订单｜当季影响：收入｜指引计入：已计入指引｜[来源](https://example.com)")
assert '<h2 class="news-section">1.3 近期新闻</h2>' in news_html
assert '<ul class="news-list">' in news_html
PY

echo "earnings PDF markdown table regression passed"
