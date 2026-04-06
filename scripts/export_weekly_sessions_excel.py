#!/usr/bin/env python3
"""Export recent session messages into a multi-sheet Excel workbook.

Each sheet represents one day. By default the script exports the latest
7 calendar days ending at today's date in Asia/Shanghai.
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import zipfile
from collections import defaultdict
from dataclasses import dataclass
from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Any
from xml.sax.saxutils import escape

try:
    from zoneinfo import ZoneInfo
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Python 3.9+ with zoneinfo support is required") from exc


SHANGHAI_TZ = ZoneInfo("Asia/Shanghai")
HEADER = ["用户标识", "时间", "发送人", "发送消息"]


@dataclass
class ExportRow:
    session_id: str
    user_identifier: str
    timestamp: datetime
    sender: str
    message: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="导出近几天聊天记录到 Excel，每个日期一个 Sheet。"
    )
    parser.add_argument(
        "--sessions-dir",
        type=Path,
        default=Path("data/sessions"),
        help="会话 JSON 目录，默认 data/sessions",
    )
    parser.add_argument(
        "--sqlite-db",
        type=Path,
        default=Path("data/sessions.sqlite3"),
        help="会话 SQLite 路径，默认 data/sessions.sqlite3",
    )
    parser.add_argument(
        "--source",
        choices=("auto", "json", "sqlite", "both"),
        default="auto",
        help="导出数据来源：auto/json/sqlite/both，默认 auto",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="输出 xlsx 文件路径，默认 data/reports/sessions_YYYYMMDD_last7days.xlsx",
    )
    parser.add_argument(
        "--date",
        type=str,
        default=None,
        help="结束日期，格式 YYYYMMDD，默认今天（Asia/Shanghai）",
    )
    parser.add_argument(
        "--days",
        type=int,
        default=10,
        help="导出天数，默认 10",
    )
    return parser.parse_args()


def resolve_end_date(raw: str | None) -> date:
    if not raw:
        return datetime.now(SHANGHAI_TZ).date()
    return datetime.strptime(raw, "%Y%m%d").date()


def parse_timestamp(raw: Any) -> datetime | None:
    if not isinstance(raw, str) or not raw.strip():
        return None
    candidate = raw.strip().replace("Z", "+00:00")
    try:
        parsed = datetime.fromisoformat(candidate)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=SHANGHAI_TZ)
    return parsed.astimezone(SHANGHAI_TZ)


def stringify_content(content: Any) -> str:
    if isinstance(content, str):
        return content
    if isinstance(content, (dict, list)):
        return json.dumps(content, ensure_ascii=False)
    if content is None:
        return ""
    return str(content)


def choose_user_identifier(session: dict[str, Any], message: dict[str, Any]) -> str:
    metadata = message.get("metadata")
    if isinstance(metadata, dict):
        for key in ("mobile", "open_id", "user_id", "chat_id"):
            value = metadata.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()

    actor = session.get("actor")
    if isinstance(actor, dict):
        channel = actor.get("channel")
        user_id = actor.get("user_id")
        if isinstance(channel, str) and isinstance(user_id, str) and channel and user_id:
            return f"{channel}:{user_id}"
        if isinstance(user_id, str) and user_id.strip():
            return user_id.strip()

    session_id = session.get("id")
    if isinstance(session_id, str) and session_id.strip():
        return session_id.strip()
    return "unknown"


def sender_label(role: str) -> str | None:
    if role == "user":
        return "人"
    if role == "assistant":
        return "AI"
    return None


def collect_rows(
    sessions_dir: Path,
    sqlite_db: Path,
    start_date: date,
    end_date: date,
    source: str,
) -> dict[str, list[ExportRow]]:
    rows_by_sheet: dict[str, list[ExportRow]] = defaultdict(list)
    dedupe_keys: set[tuple[str, str, str, str]] = set()

    use_json = source in {"json", "both"} or (source == "auto" and sessions_dir.exists())
    use_sqlite = source in {"sqlite", "both"} or (source == "auto" and sqlite_db.exists())

    if not use_json and not use_sqlite:
        raise FileNotFoundError(
            f"No available data source. sessions_dir={sessions_dir} sqlite_db={sqlite_db}"
        )

    def append_row(row: ExportRow) -> None:
        dedupe_key = (
            row.session_id,
            row.timestamp.isoformat(),
            row.sender,
            row.message,
        )
        if dedupe_key in dedupe_keys:
            return
        dedupe_keys.add(dedupe_key)
        sheet_name = row.timestamp.strftime("%Y%m%d")
        rows_by_sheet[sheet_name].append(row)

    if use_json:
        if not sessions_dir.exists():
            raise FileNotFoundError(f"Sessions directory not found: {sessions_dir}")

        for path in sorted(sessions_dir.glob("*.json")):
            if path.name.startswith("."):
                continue
            try:
                session = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, json.JSONDecodeError):
                continue

            messages = session.get("messages")
            if not isinstance(messages, list):
                continue

            session_id = str(session.get("id") or path.stem)
            for message in messages:
                if not isinstance(message, dict):
                    continue
                sender = sender_label(str(message.get("role", "")))
                if sender is None:
                    continue

                timestamp = parse_timestamp(message.get("timestamp"))
                if timestamp is None:
                    continue
                current_date = timestamp.date()
                if current_date < start_date or current_date > end_date:
                    continue

                append_row(
                    ExportRow(
                        session_id=session_id,
                        user_identifier=choose_user_identifier(session, message),
                        timestamp=timestamp,
                        sender=sender,
                        message=stringify_content(message.get("content")),
                    )
                )

    if use_sqlite:
        if not sqlite_db.exists():
            raise FileNotFoundError(f"SQLite database not found: {sqlite_db}")

        conn = sqlite3.connect(str(sqlite_db))
        conn.row_factory = sqlite3.Row
        try:
            query = """
                SELECT
                    sm.session_id,
                    sm.role,
                    sm.content,
                    sm.timestamp,
                    sm.mobile,
                    sm.open_id,
                    sm.chat_id,
                    sm.channel,
                    s.actor_channel,
                    s.actor_user_id
                FROM session_messages sm
                LEFT JOIN sessions s ON s.session_id = sm.session_id
                WHERE sm.role IN ('user', 'assistant')
                  AND sm.timestamp IS NOT NULL
            """
            for record in conn.execute(query):
                timestamp = parse_timestamp(record["timestamp"])
                if timestamp is None:
                    continue
                current_date = timestamp.date()
                if current_date < start_date or current_date > end_date:
                    continue

                sender = sender_label(str(record["role"]))
                if sender is None:
                    continue

                user_identifier = ""
                for value in (
                    record["mobile"],
                    record["open_id"],
                    record["chat_id"],
                ):
                    if isinstance(value, str) and value.strip():
                        user_identifier = value.strip()
                        break
                if not user_identifier:
                    actor_channel = record["actor_channel"]
                    actor_user_id = record["actor_user_id"]
                    if (
                        isinstance(actor_channel, str)
                        and actor_channel.strip()
                        and isinstance(actor_user_id, str)
                        and actor_user_id.strip()
                    ):
                        user_identifier = f"{actor_channel.strip()}:{actor_user_id.strip()}"
                    else:
                        user_identifier = str(record["session_id"])

                append_row(
                    ExportRow(
                        session_id=str(record["session_id"]),
                        user_identifier=user_identifier,
                        timestamp=timestamp,
                        sender=sender,
                        message=stringify_content(record["content"]),
                    )
                )
        finally:
            conn.close()

    for rows in rows_by_sheet.values():
        rows.sort(key=lambda item: item.timestamp)
    return rows_by_sheet


def column_name(index: int) -> str:
    result = ""
    current = index
    while current > 0:
        current, remainder = divmod(current - 1, 26)
        result = chr(65 + remainder) + result
    return result


def xml_cell(cell_ref: str, value: str, style_id: int | None = None) -> str:
    attrs = f'r="{cell_ref}" t="inlineStr"'
    if style_id is not None:
        attrs += f' s="{style_id}"'
    escaped = escape(value).replace("\r\n", "\n").replace("\r", "\n")
    return f"<c {attrs}><is><t xml:space=\"preserve\">{escaped}</t></is></c>"


def sheet_xml(rows: list[ExportRow]) -> str:
    all_rows: list[str] = []

    header_cells = []
    for idx, title in enumerate(HEADER, start=1):
        header_cells.append(xml_cell(f"{column_name(idx)}1", title, style_id=1))
    all_rows.append(f"<row r=\"1\">{''.join(header_cells)}</row>")

    for row_index, row in enumerate(rows, start=2):
        values = [
            row.user_identifier,
            row.timestamp.strftime("%Y-%m-%d %H:%M:%S"),
            row.sender,
            row.message,
        ]
        cells = [
            xml_cell(f"{column_name(col_idx)}{row_index}", value)
            for col_idx, value in enumerate(values, start=1)
        ]
        all_rows.append(f"<row r=\"{row_index}\">{''.join(cells)}</row>")

    dimension_end = max(1, len(rows) + 1)
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">"
        f"<dimension ref=\"A1:D{dimension_end}\"/>"
        "<sheetViews><sheetView workbookViewId=\"0\"/></sheetViews>"
        "<sheetFormatPr defaultRowHeight=\"15\"/>"
        "<cols>"
        "<col min=\"1\" max=\"1\" width=\"24\" customWidth=\"1\"/>"
        "<col min=\"2\" max=\"2\" width=\"22\" customWidth=\"1\"/>"
        "<col min=\"3\" max=\"3\" width=\"12\" customWidth=\"1\"/>"
        "<col min=\"4\" max=\"4\" width=\"100\" customWidth=\"1\"/>"
        "</cols>"
        "<sheetData>"
        f"{''.join(all_rows)}"
        "</sheetData>"
        "</worksheet>"
    )


def workbook_xml(sheet_names: list[str]) -> str:
    sheets = []
    for idx, name in enumerate(sheet_names, start=1):
        sheets.append(
            f'<sheet name="{escape(name)}" sheetId="{idx}" r:id="rId{idx}"/>'
        )
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<workbook "
        "xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" "
        "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">"
        "<sheets>"
        f"{''.join(sheets)}"
        "</sheets>"
        "</workbook>"
    )


def workbook_rels_xml(sheet_count: int) -> str:
    relationships = []
    for idx in range(1, sheet_count + 1):
        relationships.append(
            "<Relationship "
            f'Id="rId{idx}" '
            'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" '
            f'Target="worksheets/sheet{idx}.xml"/>'
        )
    relationships.append(
        "<Relationship "
        f'Id="rId{sheet_count + 1}" '
        'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" '
        'Target="styles.xml"/>'
    )
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">"
        f"{''.join(relationships)}"
        "</Relationships>"
    )


def content_types_xml(sheet_count: int) -> str:
    overrides = [
        '<Override PartName="/xl/workbook.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>',
        '<Override PartName="/xl/styles.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>',
        '<Override PartName="/docProps/core.xml" '
        'ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>',
        '<Override PartName="/docProps/app.xml" '
        'ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>',
    ]
    for idx in range(1, sheet_count + 1):
        overrides.append(
            '<Override '
            f'PartName="/xl/worksheets/sheet{idx}.xml" '
            'ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>'
        )
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">"
        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
        '<Default Extension="xml" ContentType="application/xml"/>'
        f"{''.join(overrides)}"
        "</Types>"
    )


def root_rels_xml() -> str:
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">"
        "<Relationship "
        'Id="rId1" '
        'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" '
        'Target="xl/workbook.xml"/>'
        "<Relationship "
        'Id="rId2" '
        'Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" '
        'Target="docProps/core.xml"/>'
        "<Relationship "
        'Id="rId3" '
        'Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" '
        'Target="docProps/app.xml"/>'
        "</Relationships>"
    )


def styles_xml() -> str:
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">"
        "<fonts count=\"2\">"
        "<font><sz val=\"11\"/><name val=\"Calibri\"/></font>"
        "<font><b/><sz val=\"11\"/><name val=\"Calibri\"/></font>"
        "</fonts>"
        "<fills count=\"2\">"
        "<fill><patternFill patternType=\"none\"/></fill>"
        "<fill><patternFill patternType=\"gray125\"/></fill>"
        "</fills>"
        "<borders count=\"1\"><border><left/><right/><top/><bottom/><diagonal/></border></borders>"
        "<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>"
        "<cellXfs count=\"2\">"
        "<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>"
        "<xf numFmtId=\"0\" fontId=\"1\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\"/>"
        "</cellXfs>"
        "<cellStyles count=\"1\"><cellStyle name=\"Normal\" xfId=\"0\" builtinId=\"0\"/></cellStyles>"
        "</styleSheet>"
    )


def core_props_xml(now: datetime) -> str:
    timestamp = now.astimezone(ZoneInfo("UTC")).strftime("%Y-%m-%dT%H:%M:%SZ")
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<cp:coreProperties "
        "xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" "
        "xmlns:dc=\"http://purl.org/dc/elements/1.1/\" "
        "xmlns:dcterms=\"http://purl.org/dc/terms/\" "
        "xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" "
        "xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">"
        "<dc:creator>Codex</dc:creator>"
        "<cp:lastModifiedBy>Codex</cp:lastModifiedBy>"
        f"<dcterms:created xsi:type=\"dcterms:W3CDTF\">{timestamp}</dcterms:created>"
        f"<dcterms:modified xsi:type=\"dcterms:W3CDTF\">{timestamp}</dcterms:modified>"
        "</cp:coreProperties>"
    )


def app_props_xml(sheet_names: list[str]) -> str:
    parts = [
        "<vt:lpstr>Workbook</vt:lpstr>",
        *[f"<vt:lpstr>{escape(name)}</vt:lpstr>" for name in sheet_names],
    ]
    return (
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"
        "<Properties "
        "xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\" "
        "xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\">"
        "<Application>Microsoft Excel</Application>"
        f"<TitlesOfParts><vt:vector size=\"{len(parts)}\" baseType=\"lpstr\">{''.join(parts)}</vt:vector></TitlesOfParts>"
        f"<HeadingPairs><vt:vector size=\"2\" baseType=\"variant\"><vt:variant><vt:lpstr>Worksheets</vt:lpstr></vt:variant><vt:variant><vt:i4>{len(sheet_names)}</vt:i4></vt:variant></vt:vector></HeadingPairs>"
        "</Properties>"
    )


def write_workbook(output_path: Path, ordered_sheet_names: list[str], rows_by_sheet: dict[str, list[ExportRow]]) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    now = datetime.now(SHANGHAI_TZ)

    with zipfile.ZipFile(output_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("[Content_Types].xml", content_types_xml(len(ordered_sheet_names)))
        archive.writestr("_rels/.rels", root_rels_xml())
        archive.writestr("docProps/core.xml", core_props_xml(now))
        archive.writestr("docProps/app.xml", app_props_xml(ordered_sheet_names))
        archive.writestr("xl/workbook.xml", workbook_xml(ordered_sheet_names))
        archive.writestr(
            "xl/_rels/workbook.xml.rels",
            workbook_rels_xml(len(ordered_sheet_names)),
        )
        archive.writestr("xl/styles.xml", styles_xml())
        for idx, sheet_name in enumerate(ordered_sheet_names, start=1):
            archive.writestr(
                f"xl/worksheets/sheet{idx}.xml",
                sheet_xml(rows_by_sheet.get(sheet_name, [])),
            )


def main() -> int:
    args = parse_args()
    if args.days <= 0:
        raise SystemExit("--days 必须大于 0")

    end_date = resolve_end_date(args.date)
    start_date = end_date - timedelta(days=args.days - 1)
    ordered_days = [
        (end_date - timedelta(days=offset)).strftime("%Y%m%d")
        for offset in range(args.days)
    ]

    output_path = args.output
    if output_path is None:
        output_path = Path("data/reports") / f"sessions_{end_date.strftime('%Y%m%d')}_last{args.days}days.xlsx"

    rows_by_sheet = collect_rows(
        args.sessions_dir,
        args.sqlite_db,
        start_date,
        end_date,
        args.source,
    )
    write_workbook(output_path, ordered_days, rows_by_sheet)

    total_rows = sum(len(rows_by_sheet.get(name, [])) for name in ordered_days)
    print(f"Exported {total_rows} messages to {output_path}")
    print("Sheets:", ", ".join(ordered_days))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
