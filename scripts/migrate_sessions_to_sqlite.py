#!/usr/bin/env python3
"""Mirror Hone session JSON files into a standalone SQLite database.

This script is intentionally out-of-band: it does not change the runtime read/write
path and only maintains a shadow SQLite mirror for inspection and future cutover.

Defaults to dry-run. Use --write to apply changes.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import sys
from copy import deepcopy
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


DEFAULT_DB_PATH = "./data/sessions.sqlite3"
DEFAULT_SESSIONS_DIR = "./data/sessions"


class MigrationError(RuntimeError):
    pass


class UnstableSourceError(MigrationError):
    pass


@dataclass
class SessionSource:
    path: Path
    raw_bytes: bytes
    source_mtime_ns: int
    source_size: int
    content_sha256: str


def utc_now_rfc3339() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def json_dumps(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True)


def stable_read(path: Path) -> SessionSource:
    before = path.stat()
    raw = path.read_bytes()
    after = path.stat()
    if before.st_mtime_ns != after.st_mtime_ns or before.st_size != after.st_size:
        raise UnstableSourceError(f"{path}: file changed while being read")
    return SessionSource(
        path=path,
        raw_bytes=raw,
        source_mtime_ns=after.st_mtime_ns,
        source_size=after.st_size,
        content_sha256=hashlib.sha256(raw).hexdigest(),
    )


def preview_text(value: Any, limit: int = 160) -> str | None:
    if not isinstance(value, str):
        return None
    collapsed = " ".join(value.split()).strip()
    if not collapsed:
        return None
    if len(collapsed) <= limit:
        return collapsed
    return collapsed[: limit - 1] + "…"


def ensure_schema(conn: sqlite3.Connection) -> None:
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA synchronous = NORMAL")
    conn.execute("PRAGMA busy_timeout = 5000")
    conn.execute("PRAGMA foreign_keys = ON")
    conn.executescript(
        """
        CREATE TABLE IF NOT EXISTS migration_runs (
            run_id INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            mode TEXT NOT NULL,
            sessions_scanned INTEGER NOT NULL DEFAULT 0,
            sessions_imported INTEGER NOT NULL DEFAULT 0,
            sessions_skipped INTEGER NOT NULL DEFAULT 0,
            sessions_failed INTEGER NOT NULL DEFAULT 0,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS sessions (
            session_id TEXT PRIMARY KEY,
            source_path TEXT NOT NULL UNIQUE,
            version INTEGER,
            actor_channel TEXT,
            actor_user_id TEXT,
            actor_channel_scope TEXT,
            session_channel TEXT,
            session_kind TEXT,
            session_user_id TEXT,
            session_channel_scope TEXT,
            created_at TEXT,
            updated_at TEXT,
            frozen_time_beijing TEXT,
            message_count INTEGER NOT NULL DEFAULT 0,
            last_message_at TEXT,
            last_message_role TEXT,
            last_message_preview TEXT,
            actor_json TEXT,
            session_identity_json TEXT,
            runtime_json TEXT,
            summary_json TEXT,
            metadata_json TEXT,
            source_json TEXT NOT NULL,
            normalized_json TEXT NOT NULL,
            source_mtime_ns INTEGER NOT NULL,
            source_size INTEGER NOT NULL,
            content_sha256 TEXT NOT NULL,
            imported_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_updated_at
            ON sessions(updated_at DESC);
        CREATE INDEX IF NOT EXISTS idx_sessions_actor
            ON sessions(actor_channel, actor_user_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_scope
            ON sessions(session_channel, session_kind, session_channel_scope);
        CREATE INDEX IF NOT EXISTS idx_sessions_sha
            ON sessions(content_sha256);

        CREATE TABLE IF NOT EXISTS session_metadata (
            session_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value_json TEXT NOT NULL,
            imported_at TEXT NOT NULL,
            PRIMARY KEY(session_id, key),
            FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS session_messages (
            session_id TEXT NOT NULL,
            ordinal INTEGER NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT,
            metadata_json TEXT,
            message_id TEXT,
            tool_name TEXT,
            tool_call_id TEXT,
            channel TEXT,
            open_id TEXT,
            mobile TEXT,
            chat_id TEXT,
            chat_type TEXT,
            message_type TEXT,
            content_sha256 TEXT NOT NULL,
            imported_at TEXT NOT NULL,
            PRIMARY KEY(session_id, ordinal),
            FOREIGN KEY(session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_session_messages_session_ts
            ON session_messages(session_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_session_messages_tool
            ON session_messages(tool_name, tool_call_id);
        CREATE INDEX IF NOT EXISTS idx_session_messages_message_id
            ON session_messages(message_id);
        """
    )


def open_db(db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    ensure_schema(conn)
    return conn


def choose_frozen_time(payload: dict[str, Any]) -> str | None:
    runtime = payload.get("runtime")
    if isinstance(runtime, dict):
        prompt = runtime.get("prompt")
        if isinstance(prompt, dict):
            frozen = prompt.get("frozen_time_beijing")
            if isinstance(frozen, str) and frozen.strip():
                return frozen.strip()
    for key in ("created_at", "updated_at"):
        value = payload.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    return None


def hydrate_payload(payload: dict[str, Any], path: Path) -> tuple[dict[str, Any], list[str]]:
    hydrated = deepcopy(payload)
    changes: list[str] = []

    session_id = hydrated.get("id")
    if not isinstance(session_id, str) or not session_id.strip():
        hydrated["id"] = path.stem
        changes.append("set id from filename")

    if not isinstance(hydrated.get("messages"), list):
        raise MigrationError(f"{path}: messages is not a list")

    if not isinstance(hydrated.get("metadata"), dict):
        hydrated["metadata"] = {}
        changes.append("normalized metadata={}")

    if "summary" not in hydrated:
        hydrated["summary"] = None
        changes.append("added summary=null")

    if not isinstance(hydrated.get("runtime"), dict):
        hydrated["runtime"] = {}
        changes.append("added runtime={}")

    runtime = hydrated["runtime"]
    if not isinstance(runtime.get("prompt"), dict):
        runtime["prompt"] = {}
        changes.append("added runtime.prompt={}")

    frozen = runtime["prompt"].get("frozen_time_beijing")
    if not isinstance(frozen, str) or not frozen.strip():
        derived_frozen = choose_frozen_time(hydrated)
        if derived_frozen:
            runtime["prompt"]["frozen_time_beijing"] = derived_frozen
            changes.append("derived runtime.prompt.frozen_time_beijing")

    return hydrated, changes


def get_existing_row(
    conn: sqlite3.Connection, session_id: str, source_path: str
) -> sqlite3.Row | None:
    row = conn.execute(
        """
        SELECT *
        FROM sessions
        WHERE session_id = ? OR source_path = ?
        ORDER BY session_id = ? DESC
        LIMIT 1
        """,
        (session_id, source_path, session_id),
    ).fetchone()
    return row


def import_session(
    conn: sqlite3.Connection,
    source: SessionSource,
    payload: dict[str, Any],
    hydrated: dict[str, Any],
) -> str:
    session_id = str(hydrated["id"])
    source_path = str(source.path.resolve())
    imported_at = utc_now_rfc3339()

    actor = hydrated.get("actor") if isinstance(hydrated.get("actor"), dict) else None
    session_identity = (
        hydrated.get("session_identity")
        if isinstance(hydrated.get("session_identity"), dict)
        else None
    )
    runtime = hydrated.get("runtime") if isinstance(hydrated.get("runtime"), dict) else None
    metadata = hydrated.get("metadata") if isinstance(hydrated.get("metadata"), dict) else {}
    summary_value = hydrated.get("summary")
    messages = hydrated.get("messages") or []
    frozen_time = None
    if runtime:
        prompt = runtime.get("prompt")
        if isinstance(prompt, dict):
            value = prompt.get("frozen_time_beijing")
            if isinstance(value, str) and value.strip():
                frozen_time = value.strip()

    last_message = messages[-1] if messages else None
    last_message_at = None
    last_message_role = None
    last_message_preview = None
    if isinstance(last_message, dict):
        timestamp = last_message.get("timestamp")
        if isinstance(timestamp, str) and timestamp.strip():
            last_message_at = timestamp.strip()
        role = last_message.get("role")
        if isinstance(role, str) and role.strip():
            last_message_role = role.strip()
        last_message_preview = preview_text(last_message.get("content"))

    with conn:
        conn.execute("DELETE FROM session_metadata WHERE session_id = ?", (session_id,))
        conn.execute("DELETE FROM session_messages WHERE session_id = ?", (session_id,))
        conn.execute(
            "DELETE FROM sessions WHERE session_id = ? OR source_path = ?",
            (session_id, source_path),
        )
        conn.execute(
            """
            INSERT INTO sessions (
                session_id,
                source_path,
                version,
                actor_channel,
                actor_user_id,
                actor_channel_scope,
                session_channel,
                session_kind,
                session_user_id,
                session_channel_scope,
                created_at,
                updated_at,
                frozen_time_beijing,
                message_count,
                last_message_at,
                last_message_role,
                last_message_preview,
                actor_json,
                session_identity_json,
                runtime_json,
                summary_json,
                metadata_json,
                source_json,
                normalized_json,
                source_mtime_ns,
                source_size,
                content_sha256,
                imported_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                session_id,
                source_path,
                hydrated.get("version"),
                actor.get("channel") if actor else None,
                actor.get("user_id") if actor else None,
                actor.get("channel_scope") if actor else None,
                session_identity.get("channel") if session_identity else None,
                session_identity.get("kind") if session_identity else None,
                session_identity.get("user_id") if session_identity else None,
                session_identity.get("channel_scope") if session_identity else None,
                hydrated.get("created_at"),
                hydrated.get("updated_at"),
                frozen_time,
                len(messages),
                last_message_at,
                last_message_role,
                last_message_preview,
                json_dumps(actor) if actor is not None else None,
                json_dumps(session_identity) if session_identity is not None else None,
                json_dumps(runtime) if runtime is not None else None,
                json_dumps(summary_value) if summary_value is not None else None,
                json_dumps(metadata),
                source.raw_bytes.decode("utf-8"),
                json_dumps(hydrated),
                source.source_mtime_ns,
                source.source_size,
                source.content_sha256,
                imported_at,
            ),
        )
        if metadata:
            conn.executemany(
                """
                INSERT INTO session_metadata (session_id, key, value_json, imported_at)
                VALUES (?, ?, ?, ?)
                """,
                [
                    (session_id, key, json_dumps(value), imported_at)
                    for key, value in sorted(metadata.items())
                ],
            )
        conn.executemany(
            """
            INSERT INTO session_messages (
                session_id,
                ordinal,
                role,
                content,
                timestamp,
                metadata_json,
                message_id,
                tool_name,
                tool_call_id,
                channel,
                open_id,
                mobile,
                chat_id,
                chat_type,
                message_type,
                content_sha256,
                imported_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            [
                (
                    session_id,
                    ordinal,
                    str(message.get("role", "")),
                    str(message.get("content", "")),
                    message.get("timestamp"),
                    (
                        json_dumps(message.get("metadata"))
                        if isinstance(message.get("metadata"), dict)
                        else None
                    ),
                    message.get("metadata", {}).get("message_id")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("tool_name")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("tool_call_id")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("channel")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("open_id")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("mobile")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("chat_id")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("chat_type")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    message.get("metadata", {}).get("message_type")
                    if isinstance(message.get("metadata"), dict)
                    else None,
                    hashlib.sha256(str(message.get("content", "")).encode("utf-8")).hexdigest(),
                    imported_at,
                )
                for ordinal, message in enumerate(messages)
                if isinstance(message, dict)
            ],
        )
    return session_id


def record_run_start(conn: sqlite3.Connection, mode: str, notes: str | None) -> int:
    with conn:
        cursor = conn.execute(
            """
            INSERT INTO migration_runs (started_at, mode, notes)
            VALUES (?, ?, ?)
            """,
            (utc_now_rfc3339(), mode, notes),
        )
    return int(cursor.lastrowid)


def record_run_end(
    conn: sqlite3.Connection,
    run_id: int,
    scanned: int,
    imported: int,
    skipped: int,
    failed: int,
) -> None:
    with conn:
        conn.execute(
            """
            UPDATE migration_runs
            SET completed_at = ?,
                sessions_scanned = ?,
                sessions_imported = ?,
                sessions_skipped = ?,
                sessions_failed = ?
            WHERE run_id = ?
            """,
            (utc_now_rfc3339(), scanned, imported, skipped, failed, run_id),
        )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Mirror Hone session JSON files into a standalone SQLite database."
    )
    parser.add_argument(
        "--sessions-dir",
        default=DEFAULT_SESSIONS_DIR,
        help=f"Directory containing session JSON files (default: {DEFAULT_SESSIONS_DIR})",
    )
    parser.add_argument(
        "--db-path",
        default=DEFAULT_DB_PATH,
        help=f"Target SQLite database path (default: {DEFAULT_DB_PATH})",
    )
    parser.add_argument(
        "--write",
        action="store_true",
        help="Write to SQLite. Without this flag the script only reports what would change.",
    )
    parser.add_argument(
        "--notes",
        default=None,
        help="Optional note stored in migration_runs for this execution.",
    )
    args = parser.parse_args()

    sessions_dir = Path(args.sessions_dir)
    if not sessions_dir.exists():
        print(f"[ERROR] sessions dir not found: {sessions_dir}", file=sys.stderr)
        return 2

    db_path = Path(args.db_path)
    conn = open_db(db_path)
    mode = "write" if args.write else "dry-run"
    run_id = record_run_start(conn, mode, args.notes)

    scanned = imported = skipped = failed = 0
    for path in sorted(sessions_dir.glob("*.json")):
        scanned += 1
        try:
            source = stable_read(path)
            payload = json.loads(source.raw_bytes.decode("utf-8"))
            if not isinstance(payload, dict):
                raise MigrationError(f"{path}: top-level JSON must be an object")
            hydrated, changes = hydrate_payload(payload, path)
            session_id = str(hydrated["id"])
            existing = get_existing_row(conn, session_id, str(path.resolve()))
            if (
                existing is not None
                and existing["content_sha256"] == source.content_sha256
                and existing["source_path"] == str(path.resolve())
            ):
                skipped += 1
                print(f"[SKIP] {path} session_id={session_id} reason=unchanged")
                continue

            if not args.write:
                imported += 1
                detail = ", ".join(changes) if changes else "content changed"
                print(f"[PLAN] {path} session_id={session_id} changes={detail}")
                continue

            imported_session_id = import_session(conn, source, payload, hydrated)
            imported += 1
            detail = ", ".join(changes) if changes else "imported"
            print(f"[IMPORT] {path} session_id={imported_session_id} changes={detail}")
        except Exception as exc:
            failed += 1
            print(f"[ERROR] {path}: {exc}", file=sys.stderr)

    record_run_end(conn, run_id, scanned, imported, skipped, failed)
    print(
        f"[SUMMARY] mode={mode} db={db_path} scanned={scanned} imported={imported} skipped={skipped} failed={failed}"
    )
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
