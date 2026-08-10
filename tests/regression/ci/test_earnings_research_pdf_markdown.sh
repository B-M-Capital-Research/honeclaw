#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
renderer="$repo_root/skills/earnings-research/scripts/render_report_pdf.py"

RENDERER_PATH="$renderer" python3 - <<'PY'
import importlib.util
import os
import subprocess
import tempfile
from pathlib import Path

spec = importlib.util.spec_from_file_location("earnings_pdf_renderer", os.environ["RENDERER_PATH"])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

# Content ownership belongs to the migrated BamangResearch prompt, not the
# renderer. Arbitrary useful headings, a short news appendix, and an explicit
# evidence gap must all remain valid without preview_audit.
report = (
    "# CRWV公司财报前瞻分析\n\n"
    "# 1. 整体分析\n\n"
    "## 1.1 核心股价因素\n\n"
    "供电与资本效率决定AI云产能兑现速度。\n\n"
    "## 1.2 业绩指引 vs 机构观点\n\n"
    "未找到可核验来源支持最新一致目标价，因此不列示该数字。\n\n"
    "| 指标 | 当前判断 |\n|---|---|\n| 收入 | 以公司指引为锚 |\n\n"
    "# 附录：近期新闻时间线分析\n\n"
    "## 新闻解读\n\n"
    "近期只保留了一条可由公司公告核验的重大事件，没有为凑数量补写新闻。\n"
)
module.validate_report(report)

rendered = module.markdown_to_html(report)
assert "<h1>1. 整体分析</h1>" in rendered
assert "<table>" in rendered
assert "未找到可核验来源" in rendered
assert "preview_audit" not in rendered

workflow_html = module.build_html("CRWV", "财报前瞻", report, None)
assert "知识星球：巴芒科技" in workflow_html
assert '<div class="report-title">CRWV公司财报前瞻分析</div>' in workflow_html
assert "近期只保留了一条" in workflow_html
assert "@top-left" in workflow_html

analysis = (
    "# CRWV公司财报分析总结\n\n"
    "# 1. 财报摘要\n\n本季度收入高于公司此前指引。\n\n"
    "# 10. 结论\n\n结论只基于已取得的财报与电话会材料。\n"
)
module.validate_report(analysis)
assert "<h1>10. 结论</h1>" in module.markdown_to_html(analysis)

for invalid, expected in [
    (report + "\nAnonymous Institution gave a buy rating.", "anonymous source"),
    (report + "\n来源：https://example.com/fake", "placeholder URL"),
    (report + "\n{financial_information}", "unresolved placeholder"),
]:
    try:
        module.validate_report(invalid)
    except ValueError as exc:
        assert expected in str(exc)
    else:
        raise AssertionError(f"expected rejection containing {expected}")

# Keep the technical Chromium retry deterministic without requiring a browser
# in the CI-safe contract test.
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
        output = Path(temp_dir) / "report.pdf"
        module.render_pdf_with_chromium(workflow_html, output)
        assert output.stat().st_size >= 1000
finally:
    module.chromium_candidates = original_candidates
    module.subprocess.run = original_run

assert len(render_calls) == 2
assert "--disable-extensions" in render_calls[0]
PY

echo "earnings PDF content-preserving renderer regression passed"
