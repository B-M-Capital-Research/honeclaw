#!/usr/bin/env python3
"""Export one day of event-engine delivery evidence for calibration.

The script is read-only. It joins `delivery_log` with `events` and writes a
compact JSON and/or Markdown report that can be manually annotated before adding
representative samples to `tests/fixtures/event_engine/`.
"""

from __future__ import annotations

import argparse
import json
import sqlite3
from collections import Counter
from datetime import date, datetime, time, timedelta, timezone
from pathlib import Path
from typing import Any

try:
    from zoneinfo import ZoneInfo
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Python 3.9+ with zoneinfo support is required") from exc


DEFAULT_TZ = "Asia/Shanghai"
DEFAULT_ACTOR = "telegram::::8039067465"
DEFAULT_DB = Path("data/events.sqlite3")
DEFAULT_OUT_DIR = Path("data/exports/event-engine-calibration")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export event-engine delivery records for one local day."
    )
    parser.add_argument(
        "--db",
        type=Path,
        default=DEFAULT_DB,
        help="events.sqlite3 path, default data/events.sqlite3",
    )
    parser.add_argument(
        "--actor",
        default=DEFAULT_ACTOR,
        help=f"actor key to export, default {DEFAULT_ACTOR}",
    )
    parser.add_argument(
        "--date",
        default=None,
        help="local date YYYY-MM-DD or YYYYMMDD; default today in --timezone",
    )
    parser.add_argument(
        "--timezone",
        default=DEFAULT_TZ,
        help=f"IANA timezone for date window, default {DEFAULT_TZ}",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=DEFAULT_OUT_DIR,
        help=f"output directory, default {DEFAULT_OUT_DIR}",
    )
    parser.add_argument(
        "--format",
        choices=("json", "md", "both"),
        default="both",
        help="report format, default both",
    )
    parser.add_argument(
        "--include-body",
        action="store_true",
        help="include full sent bodies in JSON; Markdown always stores previews",
    )
    parser.add_argument(
        "--preview-chars",
        type=int,
        default=360,
        help="body preview length when --include-body is not set",
    )
    return parser.parse_args()


def resolve_local_date(raw: str | None, tz: ZoneInfo) -> date:
    if not raw:
        return datetime.now(tz).date()
    clean = raw.strip()
    if len(clean) == 8 and clean.isdigit():
        return datetime.strptime(clean, "%Y%m%d").date()
    return datetime.strptime(clean, "%Y-%m-%d").date()


def day_bounds(local_day: date, tz: ZoneInfo) -> tuple[int, int]:
    start = datetime.combine(local_day, time.min, tzinfo=tz).astimezone(timezone.utc)
    end = (datetime.combine(local_day, time.min, tzinfo=tz) + timedelta(days=1)).astimezone(
        timezone.utc
    )
    return int(start.timestamp()), int(end.timestamp())


def parse_json(raw: Any, fallback: Any) -> Any:
    if raw is None:
        return fallback
    try:
        return json.loads(raw)
    except (TypeError, json.JSONDecodeError):
        return fallback


def kind_tag(kind_json: Any) -> str:
    kind = parse_json(kind_json, {})
    if isinstance(kind, dict):
        return str(kind.get("type") or "")
    if isinstance(kind, str):
        return kind
    return ""


def normalize_body(body: str | None, include_body: bool, preview_chars: int) -> dict[str, Any]:
    if not body:
        return {"body_len": 0, "body_preview": ""}
    preview = " ".join(body.split())
    if len(preview) > preview_chars:
        preview = preview[: preview_chars - 1] + "…"
    if include_body:
        return {"body_len": len(body), "body_preview": preview, "body": body}
    return {"body_len": len(body), "body_preview": preview}


def fetch_rows(conn: sqlite3.Connection, actor: str, start_ts: int, end_ts: int) -> list[dict[str, Any]]:
    conn.row_factory = sqlite3.Row
    rows = conn.execute(
        """
        SELECT
            d.id AS delivery_id,
            d.event_id,
            d.actor,
            d.channel,
            d.severity AS delivery_severity,
            d.sent_at_ts,
            d.status,
            d.body,
            e.kind_json,
            e.severity AS event_severity,
            e.symbols_json,
            e.occurred_at_ts,
            e.title,
            e.summary,
            e.url,
            e.source,
            e.payload_json,
            e.created_at_ts
        FROM delivery_log d
        LEFT JOIN events e ON e.id = d.event_id
        WHERE d.actor = ?1
          AND d.sent_at_ts >= ?2
          AND d.sent_at_ts < ?3
        ORDER BY d.sent_at_ts ASC, d.id ASC
        """,
        (actor, start_ts, end_ts),
    ).fetchall()
    return [dict(r) for r in rows]


def event_summary(row: dict[str, Any], include_body: bool, preview_chars: int) -> dict[str, Any]:
    payload = parse_json(row.get("payload_json"), {})
    fmp = payload.get("fmp") if isinstance(payload, dict) else None
    site = fmp.get("site") if isinstance(fmp, dict) else None
    source_class = payload.get("source_class") if isinstance(payload, dict) else None
    symbols = parse_json(row.get("symbols_json"), [])
    body = normalize_body(row.get("body"), include_body, preview_chars)
    out = {
        "delivery_id": row["delivery_id"],
        "event_id": row["event_id"],
        "channel": row["channel"],
        "status": row["status"],
        "delivery_severity": row["delivery_severity"],
        "sent_at_utc": datetime.fromtimestamp(row["sent_at_ts"], timezone.utc).isoformat(),
        "kind": kind_tag(row.get("kind_json")),
        "event_severity": row.get("event_severity"),
        "symbols": symbols if isinstance(symbols, list) else [],
        "occurred_at_utc": (
            datetime.fromtimestamp(row["occurred_at_ts"], timezone.utc).isoformat()
            if row.get("occurred_at_ts") is not None
            else None
        ),
        "title": row.get("title"),
        "summary": row.get("summary"),
        "source": row.get("source"),
        "site": site,
        "source_class": source_class,
        "url": row.get("url"),
        "body_len": body["body_len"],
        "calibration_label": "",
        "calibration_note": "",
    }
    out.update(body)
    return out


def group_report(items: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    sent_immediate = [
        x for x in items if x["channel"] == "sink" and x["status"] in ("sent", "dryrun")
    ]
    sent_digests = [
        x for x in items if x["channel"] == "digest" and x["status"] in ("sent", "dryrun")
    ]
    digest_items = [
        x for x in items if x["channel"] == "digest_item" and x["status"] in ("sent", "dryrun")
    ]
    digest_omitted = [
        x for x in items if x["channel"] == "digest_item" and x["status"] == "omitted"
    ]
    queued_or_demoted = [
        x
        for x in items
        if x["channel"] == "digest" and x["status"] in ("queued", "capped", "cooled_down")
    ]
    filtered = [
        x
        for x in items
        if x["status"] in ("filtered", "no_actor", "failed") or x["channel"] == "router"
    ]
    return {
        "sent_immediate": sent_immediate,
        "sent_digests": sent_digests,
        "digest_items": digest_items,
        "digest_omitted": digest_omitted,
        "queued_or_demoted": queued_or_demoted,
        "filtered_or_failed": filtered,
    }


def write_json(path: Path, report: dict[str, Any]) -> None:
    path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def md_escape(text: Any) -> str:
    raw = "" if text is None else str(text)
    return raw.replace("|", "\\|").replace("\n", " ")


def write_markdown(path: Path, report: dict[str, Any]) -> None:
    lines = [
        f"# Event Engine Calibration {report['date']} {report['timezone']}",
        "",
        f"- actor: `{report['actor']}`",
        f"- utc_window: `{report['utc_window']['start']}` → `{report['utc_window']['end']}`",
        f"- total_delivery_rows: {report['summary']['total_delivery_rows']}",
        f"- sent_immediate: {report['summary']['sent_immediate']}",
        f"- sent_digests: {report['summary']['sent_digests']}",
        f"- digest_items: {report['summary']['digest_items']}",
        f"- digest_omitted: {report['summary']['digest_omitted']}",
        f"- queued_or_demoted: {report['summary']['queued_or_demoted']}",
        f"- filtered_or_failed: {report['summary']['filtered_or_failed']}",
        "",
        "## How To Annotate",
        "",
        "Use `calibration_label` values such as `useful`, `noise`, `should_immediate`,",
        "`should_digest`, `should_filter`, or `baseline_candidate`, then copy representative",
        "stable samples into `tests/fixtures/event_engine/`.",
        "",
    ]
    for section, title in [
        ("sent_immediate", "Sent Immediate"),
        ("sent_digests", "Sent Digest Batches"),
        ("digest_items", "Digest Items"),
        ("digest_omitted", "Digest Omitted Items"),
        ("queued_or_demoted", "Queued / Demoted"),
        ("filtered_or_failed", "Filtered / Failed"),
    ]:
        rows = report[section]
        lines.extend([f"## {title}", ""])
        if not rows:
            lines.extend(["_none_", ""])
            continue
        lines.append(
            "| sent_at_utc | status | kind | severity | symbols | source | title | body_preview | label |"
        )
        lines.append("| --- | --- | --- | --- | --- | --- | --- | --- | --- |")
        for row in rows:
            lines.append(
                "| "
                + " | ".join(
                    [
                        md_escape(row.get("sent_at_utc")),
                        md_escape(row.get("status")),
                        md_escape(row.get("kind")),
                        md_escape(row.get("delivery_severity")),
                        md_escape(",".join(row.get("symbols") or [])),
                        md_escape(row.get("source")),
                        md_escape(row.get("title")),
                        md_escape(row.get("body_preview") or ""),
                        "",
                    ]
                )
                + " |"
            )
        lines.append("")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    args = parse_args()
    tz = ZoneInfo(args.timezone)
    local_day = resolve_local_date(args.date, tz)
    start_ts, end_ts = day_bounds(local_day, tz)
    if not args.db.exists():
        raise SystemExit(f"events db not found: {args.db}")

    conn = sqlite3.connect(args.db)
    rows = fetch_rows(conn, args.actor, start_ts, end_ts)
    items = [event_summary(r, args.include_body, args.preview_chars) for r in rows]
    grouped = group_report(items)
    status_counts = Counter(x["status"] for x in items)
    channel_counts = Counter(x["channel"] for x in items)
    report = {
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "date": local_day.isoformat(),
        "timezone": args.timezone,
        "actor": args.actor,
        "utc_window": {
            "start": datetime.fromtimestamp(start_ts, timezone.utc).isoformat(),
            "end": datetime.fromtimestamp(end_ts, timezone.utc).isoformat(),
        },
        "summary": {
            "total_delivery_rows": len(items),
            "sent_immediate": len(grouped["sent_immediate"]),
            "sent_digests": len(grouped["sent_digests"]),
            "digest_items": len(grouped["digest_items"]),
            "digest_omitted": len(grouped["digest_omitted"]),
            "queued_or_demoted": len(grouped["queued_or_demoted"]),
            "filtered_or_failed": len(grouped["filtered_or_failed"]),
            "status_counts": dict(sorted(status_counts.items())),
            "channel_counts": dict(sorted(channel_counts.items())),
        },
        **grouped,
    }

    args.output_dir.mkdir(parents=True, exist_ok=True)
    stem = f"event_engine_calibration_{args.actor.replace(':', '_')}_{local_day.isoformat()}"
    outputs: list[Path] = []
    if args.format in ("json", "both"):
        path = args.output_dir / f"{stem}.json"
        write_json(path, report)
        outputs.append(path)
    if args.format in ("md", "both"):
        path = args.output_dir / f"{stem}.md"
        write_markdown(path, report)
        outputs.append(path)

    print("[PASS] event-engine calibration export written")
    for path in outputs:
        print(f"output={path}")
    print(
        "summary="
        + json.dumps(
            {
                "total": report["summary"]["total_delivery_rows"],
                "sent_immediate": report["summary"]["sent_immediate"],
                "sent_digests": report["summary"]["sent_digests"],
                "digest_items": report["summary"]["digest_items"],
                "digest_omitted": report["summary"]["digest_omitted"],
                "queued_or_demoted": report["summary"]["queued_or_demoted"],
                "filtered_or_failed": report["summary"]["filtered_or_failed"],
            },
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
