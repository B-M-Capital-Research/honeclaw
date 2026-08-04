#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
renderer="$repo_root/skills/earnings-research/scripts/render_report_pdf.py"
pdfinfo_bin="${PDFINFO_BIN:-$(command -v pdfinfo || true)}"
pdftoppm_bin="${PDFTOPPM_BIN:-$(command -v pdftoppm || true)}"

if [[ -z "$pdfinfo_bin" || -z "$pdftoppm_bin" ]]; then
  echo "pdfinfo and pdftoppm are required; set PDFINFO_BIN/PDFTOPPM_BIN when they are not on PATH" >&2
  exit 1
fi

test_root="$(mktemp -d "${TMPDIR:-/tmp}/hone-earnings-pdf-regression.XXXXXX")"
trap 'rm -rf "$test_root"' EXIT
mkdir -p "$test_root/out"

spec='{"company":"NVIDIA","mode":"analysis","output_name":"nvidia-regression","report_markdown":"# NVIDIA财报分析\n\n这是一份关于 NVIDIA FY2026 Q1 财务报表的解读，金额统一为亿美元。\n\n## 1. 利润表（Income Statement）解读：收入与利润同步增长\n\n- 收入：季度收入保持增长。\n\n## 2. 资产负债表（Balance Sheet）解读：流动性保持充裕\n\n- 现金：期末现金保持稳定。\n\n## 3. 现金流量表（Cash Flow Statement）解读：经营现金流改善\n\n- 现金流：经营现金流同比改善。\n\n## 4. 补充财务增长指标（Financial Growth）\n\n- 增长：利润增速快于收入。\n\n## 数据总结\n\n收入增长、利润扩张和现金流改善相互印证。"}'
result="$(HONE_SKILL_OUTPUT_DIR="$test_root/out" python3 "$renderer" "$spec")"
pdf_path="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["artifacts"][0]["path"])' <<<"$result")"

[[ -s "$pdf_path" ]]
"$pdfinfo_bin" "$pdf_path" | grep -Eq '^Pages:[[:space:]]+[2-9][0-9]*$'
"$pdfinfo_bin" "$pdf_path" | grep -Eq '^Page size:.*\(A4\)$'
"$pdftoppm_bin" -f 1 -singlefile -png -r 90 "$pdf_path" "$test_root/first" >/dev/null
pages="$("$pdfinfo_bin" "$pdf_path" | awk '/^Pages:/ {print $2}')"
"$pdftoppm_bin" -f "$pages" -singlefile -png -r 90 "$pdf_path" "$test_root/last" >/dev/null
[[ -s "$test_root/first.png" && -s "$test_root/last.png" ]]

echo "earnings research PDF regression passed: $pages pages"
