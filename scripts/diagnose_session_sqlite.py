#!/usr/bin/env python3
"""Inspect the standalone Hone session SQLite mirror."""

from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from pathlib import Path


DEFAULT_DB_PATH = "./data/sessions.sqlite3"


def open_db(db_path: Path) -> sqlite3.Connection:
    if not db_path.exists():
        raise SystemExit(f"[ERROR] db not found: {db_path}")
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA query_only = ON")
    return conn


def print_json(rows: list[sqlite3.Row]) -> None:
    print(json.dumps([dict(row) for row in rows], ensure_ascii=False, indent=2))


def cmd_summary(conn: sqlite3.Connection, as_json: bool) -> None:
    summary = conn.execute(
        """
        SELECT
            (SELECT COUNT(*) FROM sessions) AS sessions,
            (SELECT COUNT(*) FROM session_messages) AS messages,
            (SELECT COUNT(*) FROM session_metadata) AS metadata_rows,
            (SELECT COUNT(*) FROM migration_runs) AS migration_runs,
            (SELECT MAX(updated_at) FROM sessions) AS latest_session_update,
            (SELECT MAX(imported_at) FROM sessions) AS latest_imported_at
        """
    ).fetchone()
    if as_json:
        print(json.dumps(dict(summary), ensure_ascii=False, indent=2))
        return
    for key in summary.keys():
        print(f"{key}: {summary[key]}")


def cmd_sessions(
    conn: sqlite3.Connection,
    limit: int,
    channel: str | None,
    contains: str | None,
    as_json: bool,
) -> None:
    clauses: list[str] = []
    params: list[object] = []
    if channel:
        clauses.append("(actor_channel = ? OR session_channel = ?)")
        params.extend([channel, channel])
    if contains:
        clauses.append("(session_id LIKE ? OR last_message_preview LIKE ?)")
        needle = f"%{contains}%"
        params.extend([needle, needle])
    where = f"WHERE {' AND '.join(clauses)}" if clauses else ""
    rows = conn.execute(
        f"""
        SELECT
            session_id,
            actor_channel,
            actor_user_id,
            session_channel,
            session_kind,
            session_channel_scope,
            updated_at,
            message_count,
            last_message_role,
            last_message_preview
        FROM sessions
        {where}
        ORDER BY COALESCE(updated_at, imported_at) DESC
        LIMIT ?
        """,
        [*params, limit],
    ).fetchall()
    if as_json:
        print_json(rows)
        return
    for row in rows:
        print(
            f"{row['session_id']} | actor={row['actor_channel']}:{row['actor_user_id']} | "
            f"scope={row['session_channel']}:{row['session_kind']}:{row['session_channel_scope']} | "
            f"updated_at={row['updated_at']} | messages={row['message_count']} | "
            f"last={row['last_message_role']} | preview={row['last_message_preview']}"
        )


def cmd_messages(
    conn: sqlite3.Connection,
    session_id: str,
    limit: int | None,
    as_json: bool,
) -> None:
    sql = """
        SELECT
            ordinal,
            role,
            timestamp,
            content,
            tool_name,
            tool_call_id,
            channel,
            open_id,
            mobile,
            chat_id,
            chat_type,
            message_type,
            metadata_json
        FROM session_messages
        WHERE session_id = ?
        ORDER BY ordinal ASC
    """
    params: list[object] = [session_id]
    if limit is not None:
        sql += " LIMIT ?"
        params.append(limit)
    rows = conn.execute(sql, params).fetchall()
    if as_json:
        print_json(rows)
        return
    for row in rows:
        print(
            f"[{row['ordinal']:04d}] {row['timestamp']} {row['role']} "
            f"tool={row['tool_name'] or '-'} msg_id={row['tool_call_id'] or row['chat_id'] or '-'}"
        )
        print(row["content"])
        if row["metadata_json"]:
            print(f"metadata={row['metadata_json']}")
        print("---")


def main() -> int:
    parser = argparse.ArgumentParser(description="Inspect Hone session SQLite mirror.")
    parser.add_argument(
        "--db-path",
        default=DEFAULT_DB_PATH,
        help=f"SQLite database path (default: {DEFAULT_DB_PATH})",
    )
    parser.add_argument("--json", action="store_true", help="Print JSON instead of text.")
    subparsers = parser.add_subparsers(dest="command")

    subparsers.add_parser("summary", help="Show table counts and latest import time.")

    sessions_parser = subparsers.add_parser("sessions", help="List imported sessions.")
    sessions_parser.add_argument("--limit", type=int, default=20)
    sessions_parser.add_argument("--channel", default=None)
    sessions_parser.add_argument("--contains", default=None)

    messages_parser = subparsers.add_parser("messages", help="Show messages for one session.")
    messages_parser.add_argument("--session-id", required=True)
    messages_parser.add_argument("--limit", type=int, default=None)

    args = parser.parse_args()
    command = args.command or "summary"
    conn = open_db(Path(args.db_path))

    if command == "summary":
        cmd_summary(conn, args.json)
    elif command == "sessions":
        cmd_sessions(conn, args.limit, args.channel, args.contains, args.json)
    elif command == "messages":
        cmd_messages(conn, args.session_id, args.limit, args.json)
    else:
        print(f"[ERROR] unsupported command: {command}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
