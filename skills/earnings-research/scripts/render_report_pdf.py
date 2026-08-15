#!/usr/bin/env python3

from __future__ import annotations

import argparse
import html
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import uuid
from datetime import date
from pathlib import Path


MAX_REPORT_CHARS = 240_000
def emit(payload: dict) -> int:
    print(json.dumps(payload, ensure_ascii=False))
    return 0


def fail(message: str) -> int:
    return emit(
        {
            "success": False,
            "error": message,
            "fallback_message": "PDF 生成失败；请按错误修正后重新渲染。",
            "artifacts": [],
            "warnings": [],
        }
    )


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


def safe_name(value: str) -> str:
    cleaned = re.sub(r"[^0-9A-Za-z._-]+", "-", value.strip()).strip("-._")
    return cleaned[:80] or "earnings-report"


def validate_report(report: str) -> None:
    if not report:
        raise ValueError("report_markdown is required")
    if len(report) > MAX_REPORT_CHARS:
        raise ValueError(f"report_markdown exceeds {MAX_REPORT_CHARS} characters")


def inline_markup(value: str) -> str:
    escaped = html.escape(value, quote=False)
    escaped = re.sub(r"`([^`]+)`", r"<code>\1</code>", escaped)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    escaped = re.sub(
        r"\[([^\]]+)\]\((https?://[^)]+)\)",
        r'<a href="\2">\1</a>',
        escaped,
    )
    return escaped


def markdown_table_cells(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def is_markdown_table_separator(line: str) -> bool:
    cells = markdown_table_cells(line)
    return bool(cells) and all(re.fullmatch(r":?-{3,}:?", cell) for cell in cells)


def split_report_title(markdown: str) -> tuple[str, str]:
    lines = markdown.replace("\r\n", "\n").split("\n")
    for index, line in enumerate(lines):
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("# "):
            return stripped[2:].strip(), "\n".join(lines[index + 1 :]).strip()
        return stripped.lstrip("# ").strip(), "\n".join(lines[index + 1 :]).strip()
    return "财报研究报告", ""


def markdown_to_html(markdown: str) -> str:
    lines = markdown.replace("\r\n", "\n").split("\n")
    chunks: list[str] = []
    paragraph: list[str] = []
    list_kind: str | None = None
    index = 0

    def flush_paragraph() -> None:
        if paragraph:
            chunks.append(f"<p>{inline_markup(' '.join(paragraph))}</p>")
            paragraph.clear()

    def close_list() -> None:
        nonlocal list_kind
        if list_kind:
            chunks.append(f"</{list_kind}>")
            list_kind = None

    while index < len(lines):
        line = lines[index].strip()
        if not line:
            flush_paragraph()
            close_list()
            index += 1
            continue

        if "|" in line and index + 1 < len(lines) and is_markdown_table_separator(lines[index + 1]):
            flush_paragraph()
            close_list()
            headers = markdown_table_cells(line)
            index += 2
            rows: list[list[str]] = []
            while index < len(lines) and "|" in lines[index] and lines[index].strip():
                rows.append(markdown_table_cells(lines[index]))
                index += 1
            head = "".join(f"<th>{inline_markup(cell)}</th>" for cell in headers)
            body = "".join(
                "<tr>"
                + "".join(
                    f"<td>{inline_markup(row[column] if column < len(row) else '')}</td>"
                    for column in range(len(headers))
                )
                + "</tr>"
                for row in rows
            )
            chunks.append(
                f'<div class="table-wrap"><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>'
            )
            continue

        heading = re.match(r"^(#{1,4})\s+(.+)$", line)
        if heading:
            flush_paragraph()
            close_list()
            level = len(heading.group(1))
            chunks.append(f"<h{level}>{inline_markup(heading.group(2))}</h{level}>")
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
                chunks.append(f"<{wanted}>")
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
    for command in ("chromium", "chromium-browser", "google-chrome", "google-chrome-stable"):
        if located := shutil.which(command):
            candidates.append(Path(located))
    candidates.extend(
        [
            Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            Path("/Applications/Chromium.app/Contents/MacOS/Chromium"),
        ]
    )
    for root in (Path.home() / "Library/Caches/ms-playwright", Path.home() / ".cache/ms-playwright"):
        for pattern in (
            "chromium-*/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
            "chromium-*/chrome-linux/chrome",
            "chromium-*/chrome-linux64/chrome",
            "chromium_headless_shell-*/chrome-headless-shell-mac-arm64/chrome-headless-shell",
            "chromium_headless_shell-*/chrome-headless-shell-linux64/chrome-headless-shell",
        ):
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
        html_path = Path(temp_dir) / "report.html"
        html_path.write_text(rendered_html, encoding="utf-8")
        for chrome in browsers:
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
                    failures.append(f"{chrome.name} attempt {attempt + 1} timed out")
                    pdf_path.unlink(missing_ok=True)
                    continue
                if completed.returncode == 0 and pdf_path.is_file() and pdf_path.stat().st_size >= 1_000:
                    return
                detail = (completed.stderr or completed.stdout).strip()[-500:]
                failures.append(f"{chrome.name} exited {completed.returncode}: {detail}")
                pdf_path.unlink(missing_ok=True)
    raise RuntimeError("Chromium PDF render failed: " + " | ".join(failures[-4:]))


def resolve_share_image() -> Path | None:
    explicit = os.environ.get("HONE_ZSXQ_SHARE_IMAGE", "").strip()
    if explicit and Path(explicit).is_file():
        return Path(explicit).resolve()
    candidate = Path(__file__).resolve().parents[3] / "packages/app/public/membership_zsxq.jpg"
    return candidate if candidate.is_file() else None


def output_directory() -> Path:
    explicit = os.environ.get("HONE_SKILL_OUTPUT_DIR", "").strip()
    root = Path(explicit).expanduser() if explicit else Path.cwd() / "earnings-reports"
    root.mkdir(parents=True, exist_ok=True)
    return root.resolve()


def build_html(company: str, mode_label: str, report: str, share_image: Path | None) -> str:
    title, body = split_report_title(report)
    share_block = (
        f'<img src="{share_image.as_uri()}" alt="知识星球分享图">'
        if share_image
        else '<div class="share-fallback">知识星球 · 深度投研社区</div>'
    )
    return f"""<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><style>
@page {{ size: A4; margin: 19mm 18mm 20mm; @top-left {{ content: "{html.escape(mode_label)}"; color: #777; font-size: 9pt; }} @top-right {{ content: "HONE  {date.today().isoformat()}"; color: #555; font-size: 9pt; }} @bottom-left {{ content: "HONE 深度研究"; color: #999; font-size: 8pt; }} @bottom-right {{ content: "第 " counter(page) " 页 / 共 " counter(pages) " 页"; color: #777; font-size: 8pt; }} }}
* {{ box-sizing: border-box; }}
body {{ margin: 0; color: #202b3a; font-family: -apple-system, BlinkMacSystemFont, "PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif; font-size: 11.5pt; line-height: 1.72; text-align: justify; }}
.watermark {{ position: fixed; inset: 38% auto auto 4%; width: 92%; text-align: center; transform: rotate(-48deg); color: rgba(50,57,66,.07); font-size: 36pt; font-weight: 800; z-index: -1; }}
.report-header {{ margin: 7mm 0 8mm; padding-bottom: 4mm; border-bottom: 1px solid #ecd4c5; }}
.report-title {{ font-size: 22pt; line-height: 1.3; font-weight: 800; }}
.report-meta {{ margin-top: 2mm; color: #777; font-size: 9pt; }}
h1 {{ margin: 8mm 0 4mm; color: #202b3a; font-size: 20pt; break-after: avoid; }}
h2 {{ margin: 7mm 0 3mm; padding: 2mm 2.5mm; border-radius: 2mm; background: #f9dfcc; font-size: 16pt; break-after: avoid; }}
h3 {{ margin: 6mm 0 2.5mm; color: #344052; font-size: 13.5pt; break-after: avoid; }}
h4 {{ margin: 5mm 0 2mm; font-size: 12pt; break-after: avoid; }}
p {{ margin: 0 0 4mm; orphans: 3; widows: 3; }}
ul, ol {{ margin: 2mm 0 4mm; padding-left: 1.6em; }} li {{ margin: 1.2mm 0; }}
blockquote {{ margin: 3mm 0; padding: 2.5mm 3mm; border-left: 3px solid #8ab6af; background: #f2f7f6; }}
a {{ color: #245d73; word-break: break-word; }} code {{ padding: 1px 4px; background: #eef2f1; }}
.table-wrap {{ margin: 3mm 0 5mm; border: 1px solid #d9e4e1; border-radius: 2mm; overflow: hidden; break-inside: avoid; }}
table {{ width: 100%; border-collapse: collapse; font-size: 8.8pt; line-height: 1.45; }}
th, td {{ padding: 2mm; border-right: 1px solid #d9e4e1; border-bottom: 1px solid #d9e4e1; text-align: left; vertical-align: top; overflow-wrap: anywhere; }}
th {{ background: #eaf3f1; color: #174f47; }} th:last-child, td:last-child {{ border-right: 0; }} tbody tr:last-child td {{ border-bottom: 0; }}
.share-page {{ break-before: page; min-height: 230mm; display: flex; flex-direction: column; align-items: center; justify-content: flex-start; padding-top: 8mm; text-align: center; }}
.share-page img {{ width: 145mm; max-height: 184mm; object-fit: contain; }}
.share-fallback {{ width: 108mm; padding: 36mm 10mm; background: #e9f5ef; border: 1px solid #9bc8b5; border-radius: 8px; color: #17634f; font-size: 18pt; font-weight: 800; }}
.disclaimer {{ margin-top: 7mm; padding-top: 3mm; border-top: 1px dashed #ddd; color: #888; font-size: 8pt; line-height: 1.5; text-align: left; }}
</style></head><body>
<div class="watermark">知识星球：巴芒科技</div>
<header class="report-header"><div class="report-title">{html.escape(title)}</div><div class="report-meta">{html.escape(company)} · {html.escape(mode_label)}</div></header>
<main>{markdown_to_html(body)}</main>
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
        validate_report(report)

        out_dir = output_directory()
        base = safe_name(str(spec.get("output_name", "")) or f"{company}-{mode}")
        pdf_path = out_dir / f"{base}-{uuid.uuid4().hex[:8]}.pdf"
        mode_label = "财报前瞻" if mode == "preview" else "财报分析"
        render_pdf_with_chromium(
            build_html(company, mode_label, report, resolve_share_image()),
            pdf_path,
        )
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
