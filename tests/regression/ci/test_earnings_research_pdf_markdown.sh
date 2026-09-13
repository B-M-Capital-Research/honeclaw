#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
renderer="$repo_root/skills/earnings-research/scripts/render_report_pdf.py"

RENDERER_PATH="$renderer" python3 - <<'PY'
import importlib.util
import os
import subprocess
import sys
import time
import tempfile
from pathlib import Path

spec = importlib.util.spec_from_file_location("earnings_pdf_renderer", os.environ["RENDERER_PATH"])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

# Content ownership belongs to the migrated BamangResearch prompt, not the
# renderer. Arbitrary useful headings and prose must remain valid without a
# source schema, evidence manifest, or preview_audit.
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
    "近期只保留了一条可由公司公告核验的重大事件，没有为凑数量补写新闻。\n\n"
    "来源：[CoreWeave Investor Relations](https://investors.coreweave.com/)\n"
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
    "# 10. 结论\n\n结论只基于已取得的财报与电话会材料。\n\n"
    "来源：[SEC filing](https://www.sec.gov/Archives/edgar/data/1769628/)\n"
)
module.validate_report(analysis)
assert "<h1>10. 结论</h1>" in module.markdown_to_html(analysis)

module.validate_report("# 原工作流报告\n\n这是无需证据清单即可排版的完整正文。")
module.validate_report(report + "\n{financial_information}")

for invalid, expected in [("", "required"), ("x" * 240_001, "exceeds")]:
    try:
        module.validate_report(invalid)
    except ValueError as exc:
        assert expected in str(exc)
    else:
        raise AssertionError(f"expected technical rejection containing {expected}")

# Keep the technical Chromium retry deterministic without requiring a browser
# in the CI-safe contract test.
original_candidates = module.chromium_candidates
original_run = module.run_chromium
render_calls = []
try:
    module.chromium_candidates = lambda: [Path("/fake/chrome")]

    def fake_run(command, **kwargs):
        render_calls.append(command)
        if len(render_calls) == 1:
            return subprocess.CompletedProcess(command, 21, "", "")
        output = next(item.split("=", 1)[1] for item in command if item.startswith("--print-to-pdf="))
        Path(output).write_bytes(b"%PDF-1.4\n" + b"x" * 1200 + b"\n%%EOF\n")
        return subprocess.CompletedProcess(command, 0, "", "")

    module.run_chromium = fake_run
    with tempfile.TemporaryDirectory(prefix="hone-earnings-render-test-") as temp_dir:
        output = Path(temp_dir) / "report.pdf"
        module.render_pdf_with_chromium(workflow_html, output)
        assert output.stat().st_size >= 1000
finally:
    module.chromium_candidates = original_candidates
    module.run_chromium = original_run

assert len(render_calls) == 2
assert "--disable-extensions" in render_calls[0]
profiles = [next(arg for arg in call if arg.startswith("--user-data-dir=")) for call in render_calls]
assert len(set(profiles)) == 2
assert module.RENDER_ATTEMPTS * module.RENDER_TIMEOUT_SECONDS < 120

# A failed browser must neither publish a partial artifact nor replace an
# existing artifact. A zero exit with HTML/truncated PDF is still a failure.
original_run = module.run_chromium
original_candidates = module.chromium_candidates
try:
    module.chromium_candidates = lambda: [Path("/fake/chrome")]
    for invalid_pdf in (b"<html>" + b"x" * 1200, b"%PDF-1.7\n" + b"x" * 1200):
        def incomplete(command):
            target = next(arg.split("=", 1)[1] for arg in command if arg.startswith("--print-to-pdf="))
            Path(target).write_bytes(invalid_pdf)
            return subprocess.CompletedProcess(command, 0, "", "")
        module.run_chromium = incomplete
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "delivered.pdf"
            output.write_bytes(b"existing delivered artifact")
            try:
                module.render_pdf_with_chromium(workflow_html, output)
                raise AssertionError("partial PDF must fail")
            except RuntimeError:
                pass
            assert output.read_bytes() == b"existing delivered artifact"
            assert list(Path(directory).iterdir()) == [output]
finally:
    module.run_chromium = original_run
    module.chromium_candidates = original_candidates

# Attribute-looking text cannot turn a report link into executable HTML.
injected = module.inline_markup('[source](https://example.com/" onclick="alert)')
assert ' onclick="' not in injected
# Regression for Chrome writing a complete report but never closing its pipes.
with tempfile.TemporaryDirectory() as directory:
    target = Path(directory) / "report.pdf"
    script = "import sys,time;from pathlib import Path;Path(sys.argv[1].split('=',1)[1]).write_bytes(b'%PDF-1.7\\n'+b'x'*1200+b'\\n%%EOF\\n');time.sleep(60)"
    started = time.monotonic()
    result = module.run_chromium([sys.executable, "-c", script, f"--print-to-pdf={target}"])
    assert result.returncode == 0
    assert module.is_complete_pdf(target)
    assert time.monotonic() - started < 5

assert "table-layout: fixed" in workflow_html
assert "table-header-group" in workflow_html
assert "overflow: hidden; break-inside: avoid" not in workflow_html
PY

echo "earnings PDF content-preserving renderer regression passed"
