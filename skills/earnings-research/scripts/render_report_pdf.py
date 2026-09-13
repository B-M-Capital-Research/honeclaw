#!/usr/bin/env python3

from __future__ import annotations

import argparse
import base64
import html
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import uuid
from datetime import date
from pathlib import Path


MAX_REPORT_CHARS = 240_000
RENDER_ATTEMPTS = 2
RENDER_TIMEOUT_SECONDS = 40
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
    # Math is data; protect its source from Markdown substitutions. Avoid
    # treating ordinary dollar-denominated prices as paired math delimiters.
    math = []
    def protect(match):
        token = match.group(0)
        display = token.startswith("$$")
        body = token[2:-2] if display else token[1:-1]
        if not display and not re.search(r"\\[a-zA-Z]|[_^=]", body):
            return token
        math.append(f'<span class="hone-math" data-display="{str(display).lower()}">{html.escape(body)}</span>')
        return f"HONEMATHPLACEHOLDER{len(math)-1}END"
    value = re.sub(r"\$\$[\s\S]+?\$\$|(?<!\\)\$[^$\n]+\$", protect, value)
    escaped = html.escape(value, quote=True)
    escaped = re.sub(r"&lt;br\s*/?&gt;", "<br>", escaped, flags=re.I)
    escaped = re.sub(r"`([^`]+)`", r"<code>\1</code>", escaped)
    escaped = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", escaped)
    escaped = re.sub(
        r"\[([^\]]+)\]\((https?://[^)]+)\)",
        r'<a href="\2">\1</a>',
        escaped,
    )
    for index, fragment in enumerate(math):
        escaped = escaped.replace(f"HONEMATHPLACEHOLDER{index}END", fragment)
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

        if line.startswith("```"):
            flush_paragraph()
            close_list()
            language = line[3:].strip().lower()
            index += 1
            code_lines = []
            while index < len(lines) and not lines[index].strip().startswith("```"):
                code_lines.append(lines[index])
                index += 1
            if index < len(lines):
                index += 1
            class_name = "hone-mermaid" if language == "mermaid" else "code-block"
            chunks.append(f'<pre class="{class_name}"><code>{html.escape(chr(10).join(code_lines))}</code></pre>')
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

        if re.fullmatch(r"(?:-{3,}|\*{3,}|_{3,})", line):
            flush_paragraph()
            close_list()
            chunks.append("<hr>")
            index += 1
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
    # macOS desktop Chrome can print empty CJK glyphs in headless mode.
    # The purpose-built headless shell keeps its font initialization intact.
    if sys.platform == "darwin":
        available.sort(key=lambda path: "chrome-headless-shell" not in str(path))
    return available


def run_chromium(command: list[str]) -> subprocess.CompletedProcess:
    # The script's host deadline is 120s. Two bounded attempts leave time for
    # cleanup and JSON delivery; never leave browser children running on timeout.
    output = Path(next(arg.split("=", 1)[1] for arg in command if arg.startswith("--print-to-pdf=")))
    deadline = time.monotonic() + RENDER_TIMEOUT_SECONDS
    with subprocess.Popen(
        command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True, start_new_session=True,
    ) as process:
        try:
            while True:
                try:
                    stdout, stderr = process.communicate(timeout=min(1, max(0.01, deadline - time.monotonic())))
                    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
                except subprocess.TimeoutExpired as pending:
                    # Some Chrome builds finish printing but keep background
                    # children/pipes alive. The complete local PDF is the
                    # artifact boundary; teardown must not discard it.
                    stdout = pending.stdout or b""
                    stderr = pending.stderr or b""
                    if isinstance(stdout, bytes):
                        stdout = stdout.decode("utf-8", errors="replace")
                    if isinstance(stderr, bytes):
                        stderr = stderr.decode("utf-8", errors="replace")
                    dom_ready = "--dump-dom" not in command or re.search(
                        r'<html\b[^>]*data-hone-diagrams="ready"', stdout
                    )
                    if is_complete_pdf(output) and dom_ready:
                        return subprocess.CompletedProcess(command, 0, stdout, stderr)
                    if time.monotonic() >= deadline:
                        raise
        finally:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.communicate()


def is_complete_pdf(path: Path) -> bool:
    if not path.is_file() or path.stat().st_size < 1_000:
        return False
    with path.open("rb") as stream:
        if stream.read(5) != b"%PDF-":
            return False
        stream.seek(max(0, path.stat().st_size - 1024))
        return stream.read().rstrip().endswith(b"%%EOF")


def render_pdf_with_chromium(rendered_html: str, pdf_path: Path) -> list[str]:
    browsers = chromium_candidates()
    if not browsers:
        raise RuntimeError("Chromium/Chrome executable not found")
    failures: list[str] = []
    # Only atomically publish a completed PDF. Failed attempts cannot expose
    # partially written artifacts or overwrite an already delivered report.
    with tempfile.TemporaryDirectory(prefix=".hone-earnings-pdf-", dir=pdf_path.parent) as temp_dir:
        html_path = Path(temp_dir) / "report.html"
        html_path.write_text(rendered_html, encoding="utf-8")
        staged_pdf = Path(temp_dir) / "report.pdf"
        for attempt in range(RENDER_ATTEMPTS):
            chrome = browsers[min(attempt, len(browsers) - 1)]
            command = [
                str(chrome),
                "--headless",
                "--disable-gpu",
                "--disable-dev-shm-usage",
                "--disable-extensions",
                "--no-sandbox",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-background-networking",
                f"--user-data-dir={Path(temp_dir) / f'profile-{attempt}'}",
                "--no-pdf-header-footer",
                "--print-to-pdf-no-header",
                f"--print-to-pdf={staged_pdf}",
                html_path.as_uri(),
            ]
            diagrams = 'class="hone-mermaid"' in rendered_html or 'class="hone-math"' in rendered_html
            if diagrams:
                command[1:1] = ["--virtual-time-budget=10000", "--dump-dom"]
            try:
                completed = run_chromium(command)
            except subprocess.TimeoutExpired:
                failures.append(f"{chrome.name} attempt {attempt + 1} timed out")
                staged_pdf.unlink(missing_ok=True)
                continue
            except OSError as exc:
                failures.append(f"{chrome.name} could not start: {exc}")
                staged_pdf.unlink(missing_ok=True)
                continue
            if completed.returncode == 0 and is_complete_pdf(staged_pdf):
                warnings = []
                if diagrams:
                    root = re.search(r"<html\b[^>]*>", completed.stdout)
                    if not root or 'data-hone-diagrams="ready"' not in root.group():
                        failures.append("diagram layout did not finish before PDF printing")
                        staged_pdf.unlink(missing_ok=True)
                        continue
                    fallback = re.search(r'data-hone-diagram-fallbacks="(\d+)"', root.group())
                    if fallback and int(fallback.group(1)):
                        warnings.append(f"{fallback.group(1)} 处图表或公式无法绘制，已保留原始内容。")
                os.replace(staged_pdf, pdf_path)
                return warnings
            detail = (completed.stderr or completed.stdout).strip()[-500:]
            failures.append(f"{chrome.name} exited {completed.returncode}: {detail}")
            staged_pdf.unlink(missing_ok=True)
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
    body_html = markdown_to_html(body)
    scripts = ""
    math_css = ""
    script_policy = ""
    has_math = 'class="hone-math"' in body_html
    if 'class="hone-mermaid"' in body_html or has_math:
        bundle = Path(__file__).parent / "vendor/mermaid-11.12.3.min.js"
        if not bundle.is_file():
            raise RuntimeError("bundled Mermaid renderer is unavailable")
        nonce = uuid.uuid4().hex
        script_policy = f"; script-src 'nonce-{nonce}'"
        bundle_uri = "data:text/javascript;base64," + base64.b64encode(bundle.read_bytes()).decode()
        katex_script = ""
        if has_math:
            katex_root = Path(__file__).parent / "vendor"
            katex_uri = "data:text/javascript;base64," + base64.b64encode((katex_root / "katex-0.16.22.min.js").read_bytes()).decode()
            katex_script = f'<script nonce="{nonce}" src="{katex_uri}"></script>'
            math_css = (katex_root / "katex-0.16.22.inline.css").read_text()
        scripts = f'<script nonce="{nonce}" src="{bundle_uri}"></script>{katex_script}<script nonce="{nonce}">' + r"""
(async () => {
  mermaid.initialize({startOnLoad: false, securityLevel: 'strict', maxTextSize: 120000,
    secure: ['securityLevel', 'startOnLoad', 'maxTextSize', 'maxEdges', 'secure'], maxEdges: 3000,
    theme: 'neutral', fontFamily: 'Noto Sans CJK SC, PingFang SC, Hiragino Sans GB, Heiti SC, sans-serif',
    flowchart: {htmlLabels: false, useMaxWidth: true}});
  await document.fonts.ready;
  let failures = 0;
  for (const element of document.querySelectorAll('.hone-math')) {
    try {
      katex.render(element.textContent || '', element, {displayMode: element.dataset.display === 'true',
        throwOnError: false, trust: false, strict: 'warn', maxExpand: 1000, maxSize: 40});
      if (element.querySelector('.katex-error')) failures += 1;
    } catch (_) { failures += 1; }
  }
  for (const [index, source] of Array.from(document.querySelectorAll('.hone-mermaid')).entries()) {
    const id = `honeDiagram${index}`;
    try {
      const result = await mermaid.render(id, source.textContent || '');
      const figure = document.createElement('figure');
      figure.className = 'report-diagram';
      figure.innerHTML = result.svg;
      source.replaceWith(figure);
    } catch (_) {
      failures += 1;
      document.getElementById('d' + id)?.remove();
      source.className = 'code-block diagram-fallback';
      const note = document.createElement('p');
      note.textContent = '图表暂未绘制，原始内容如下：';
      source.before(note);
    }
  }
  await document.fonts.ready;
  document.documentElement.setAttribute('data-hone-diagram-fallbacks', String(failures));
  document.documentElement.setAttribute('data-hone-diagrams', 'ready');
})();
""" + "</script>"
    share_block = (
        f'<img src="data:image/jpeg;base64,{base64.b64encode(share_image.read_bytes()).decode()}" alt="知识星球分享图">'
        if share_image
        else '<div class="share-fallback">知识星球 · 深度投研社区</div>'
    )
    return f"""<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; img-src data:; font-src data:{script_policy}"><style>
@page {{ size: A4; margin: 19mm 18mm 20mm; @top-left {{ content: "{html.escape(mode_label)}"; color: #777; font-size: 9pt; }} @top-right {{ content: "HONE  {date.today().isoformat()}"; color: #555; font-size: 9pt; }} @bottom-left {{ content: "HONE 深度研究"; color: #999; font-size: 8pt; }} @bottom-right {{ content: "第 " counter(page) " 页 / 共 " counter(pages) " 页"; color: #777; font-size: 8pt; }} }}
* {{ box-sizing: border-box; }}
body {{ margin: 0; color: #202b3a; font-family: "Noto Sans CJK SC", "PingFang SC", "Hiragino Sans GB", "Heiti SC", "Microsoft YaHei", sans-serif; font-size: 11.5pt; line-height: 1.72; text-align: justify; }}
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
.code-block {{ white-space: pre-wrap; overflow-wrap: anywhere; font-size: 8pt; line-height: 1.4; }}
.report-diagram {{ margin: 4mm 0; break-inside: avoid; text-align: center; }}
.report-diagram svg {{ display: block; width: 100%; max-width: 100%; max-height: 190mm; height: auto; margin: auto; }}
a {{ color: #245d73; word-break: break-word; }} code {{ padding: 1px 4px; background: #eef2f1; }}
.table-wrap {{ margin: 3mm 0 5mm; }}
table {{ width: 100%; table-layout: fixed; border-collapse: collapse; font-size: 8.8pt; line-height: 1.45; }}
thead {{ display: table-header-group; }} tr {{ break-inside: avoid; }}
th, td {{ padding: 2mm; border-right: 1px solid #d9e4e1; border-bottom: 1px solid #d9e4e1; text-align: left; vertical-align: top; overflow-wrap: anywhere; }}
th {{ background: #eaf3f1; color: #174f47; }} th:last-child, td:last-child {{ border-right: 0; }} tbody tr:last-child td {{ border-bottom: 0; }}
.share-page {{ break-before: page; min-height: 230mm; display: flex; flex-direction: column; align-items: center; justify-content: flex-start; padding-top: 8mm; text-align: center; }}
.share-page img {{ width: 145mm; max-height: 184mm; object-fit: contain; }}
.share-fallback {{ width: 108mm; padding: 36mm 10mm; background: #e9f5ef; border: 1px solid #9bc8b5; border-radius: 8px; color: #17634f; font-size: 18pt; font-weight: 800; }}
.disclaimer {{ margin-top: 7mm; padding-top: 3mm; border-top: 1px dashed #ddd; color: #888; font-size: 8pt; line-height: 1.5; text-align: left; }}
{math_css}
.katex-display {{ overflow-wrap: normal; max-width: 100%; font-size: .95em; }}
hr {{ border: 0; border-top: 1px solid #d9e4e1; margin: 5mm 0; }}
</style></head><body>
<div class="watermark">知识星球：巴芒科技</div>
<header class="report-header"><div class="report-title">{html.escape(title)}</div><div class="report-meta">{html.escape(company)} · {html.escape(mode_label)}</div></header>
<main>{body_html}</main>
<section class="share-page">{share_block}<div class="disclaimer">免责声明：本报告仅供研究交流，不构成投资建议。数据、预测与判断可能存在错误或时效限制，请以公司正式披露及权威来源为准。</div></section>
{scripts}</body></html>"""


def main() -> int:
    try:
        spec = load_spec()
        company = str(spec.get("company", "")).strip()
        mode = str(spec.get("mode", "")).strip().lower()
        report = str(spec.get("report_markdown", "")).strip()
        if not company:
            raise ValueError("company is required")
        if mode not in {"preview", "analysis", "company"}:
            raise ValueError("mode must be preview, analysis or company")
        validate_report(report)

        out_dir = output_directory()
        base = safe_name(str(spec.get("output_name", "")) or f"{company}-{mode}")
        pdf_path = out_dir / f"{base}-{uuid.uuid4().hex[:8]}.pdf"
        mode_label = {"preview": "财报前瞻", "analysis": "财报分析", "company": "公司完整分析"}[mode]
        warnings = render_pdf_with_chromium(
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
                "warnings": warnings,
            }
        )
    except Exception as exc:
        return fail(str(exc))


if __name__ == "__main__":
    sys.exit(main())
