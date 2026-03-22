#!/usr/bin/env python3
"""Migrate legacy Hone session JSON files to the current schema.

Defaults to dry-run. Use --write to rewrite files in place.
"""

from __future__ import annotations

import argparse
import json
import sys
from copy import deepcopy
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

BEIJING = timezone(timedelta(hours=8))
CANONICAL_WEB_CHANNEL = "web"
LEGACY_WEB_CHANNELS = {"", "web_test"}


def beijing_now_rfc3339() -> str:
    return datetime.now(BEIJING).isoformat(timespec="seconds")


def encode_component(raw: str) -> str:
    out: list[str] = []
    for byte in raw.encode():
        if chr(byte).isalnum() or byte == ord("-"):
            out.append(chr(byte))
        else:
            out.append(f"_{byte:02x}")
    return "".join(out)


def decode_component(raw: str) -> str:
    out = bytearray()
    idx = 0
    while idx < len(raw):
        if raw[idx] == "_" and idx + 2 < len(raw):
            candidate = raw[idx + 1 : idx + 3]
            try:
                out.append(int(candidate, 16))
                idx += 3
                continue
            except ValueError:
                pass
        out.extend(raw[idx].encode())
        idx += 1
    return out.decode()


def normalize_channel(channel: str) -> str:
    trimmed = channel.strip()
    if trimmed in LEGACY_WEB_CHANNELS:
        return CANONICAL_WEB_CHANNEL
    return trimmed


def actor_storage_key(actor: dict[str, Any]) -> str:
    scope = actor.get("channel_scope") or "direct"
    return "__".join(
        [
            encode_component(actor["channel"]),
            encode_component(scope),
            encode_component(actor["user_id"]),
        ]
    )


def actor_session_id(actor: dict[str, Any]) -> str:
    return f"Actor_{actor_storage_key(actor)}"


def decode_actor_from_session_id(session_id: str) -> dict[str, Any] | None:
    if not session_id.startswith("Actor_"):
        return None
    encoded = session_id[len("Actor_") :]
    parts = encoded.split("__")
    if len(parts) != 3:
        return None
    channel = normalize_channel(decode_component(parts[0]))
    scope = decode_component(parts[1])
    user_id = decode_component(parts[2])
    if not channel or not user_id:
        return None
    actor: dict[str, Any] = {"channel": channel, "user_id": user_id}
    if scope != "direct":
        actor["channel_scope"] = scope
    return actor


def normalize_actor(
    session: dict[str, Any], path: Path
) -> tuple[dict[str, Any] | None, bool]:
    changed = False
    raw_actor = session.get("actor")
    normalized: dict[str, Any] | None = None

    if isinstance(raw_actor, dict):
        channel = normalize_channel(str(raw_actor.get("channel", "")))
        user_id = str(raw_actor.get("user_id", "")).strip()
        if channel and user_id:
            normalized = {"channel": channel, "user_id": user_id}
            scope = raw_actor.get("channel_scope")
            if isinstance(scope, str) and scope.strip() and scope.strip() != "direct":
                normalized["channel_scope"] = scope.strip()
    if normalized is None:
        source_id = session.get("id")
        if not isinstance(source_id, str) or not source_id.strip():
            source_id = path.stem
        normalized = decode_actor_from_session_id(source_id)
        if normalized is not None:
            changed = True

    if normalized is None:
        return None, changed

    if session.get("actor") != normalized:
        session["actor"] = normalized
        changed = True

    canonical_id = actor_session_id(normalized)
    if session.get("id") != canonical_id:
        session["id"] = canonical_id
        changed = True

    return normalized, changed


def choose_frozen_time(session: dict[str, Any]) -> str:
    runtime = session.get("runtime")
    if isinstance(runtime, dict):
        prompt = runtime.get("prompt")
        if isinstance(prompt, dict):
            frozen = prompt.get("frozen_time_beijing")
            if isinstance(frozen, str) and frozen.strip():
                return frozen
    for key in ("created_at", "updated_at"):
        value = session.get(key)
        if isinstance(value, str) and value.strip():
            return value
    return beijing_now_rfc3339()


def extract_summary(session: dict[str, Any]) -> tuple[dict[str, Any] | None, bool]:
    changed = False
    summary = session.get("summary")
    if isinstance(summary, dict):
        content = summary.get("content")
        updated_at = summary.get("updated_at")
        if isinstance(content, str) and content.strip():
            if not isinstance(updated_at, str) or not updated_at.strip():
                summary["updated_at"] = session.get("updated_at") or beijing_now_rfc3339()
                changed = True
            return summary, changed
        session["summary"] = None
        changed = True
        return None, changed
    if isinstance(summary, str):
        session["summary"] = {
            "content": summary,
            "updated_at": session.get("updated_at") or beijing_now_rfc3339(),
        }
        return session["summary"], True

    messages = session.get("messages")
    if not isinstance(messages, list):
        return None, changed

    migrated_messages: list[Any] = []
    summary_from_message: dict[str, Any] | None = None
    for message in messages:
        if not isinstance(message, dict):
            migrated_messages.append(message)
            continue
        metadata = message.get("metadata")
        is_summary = False
        if isinstance(metadata, dict):
            is_summary = metadata.get("is_summary") is True
        if message.get("role") == "system" and is_summary and summary_from_message is None:
            content = message.get("content")
            if isinstance(content, str) and content.strip():
                summary_from_message = {
                    "content": content,
                    "updated_at": message.get("timestamp") or session.get("updated_at") or beijing_now_rfc3339(),
                }
                changed = True
                continue
        migrated_messages.append(message)

    if changed:
        session["messages"] = migrated_messages
        session["summary"] = summary_from_message
    return summary_from_message, changed


def migrate_session(payload: dict[str, Any], path: Path) -> tuple[dict[str, Any], list[str]]:
    session = deepcopy(payload)
    changes: list[str] = []

    if not isinstance(session.get("id"), str) or not session["id"].strip():
        session["id"] = path.stem
        changes.append("set id from filename")

    if session.get("version") != 2:
        session["version"] = 2
        changes.append("set version=2")

    if not isinstance(session.get("metadata"), dict):
        session["metadata"] = {}
        changes.append("normalized metadata={}")

    runtime = session.get("runtime")
    if not isinstance(runtime, dict):
        runtime = {}
        session["runtime"] = runtime
        changes.append("added runtime")
    prompt = runtime.get("prompt")
    if not isinstance(prompt, dict):
        prompt = {}
        runtime["prompt"] = prompt
        changes.append("added runtime.prompt")
    frozen = prompt.get("frozen_time_beijing")
    if not isinstance(frozen, str) or not frozen.strip():
        prompt["frozen_time_beijing"] = choose_frozen_time(session)
        changes.append("set runtime.prompt.frozen_time_beijing")

    summary, summary_changed = extract_summary(session)
    if summary_changed:
        changes.append("normalized summary")
    if summary is None and session.get("summary") is not None:
        session["summary"] = None
        changes.append("cleared invalid summary")

    actor, actor_changed = normalize_actor(session, path)
    if actor_changed:
        changes.append("normalized actor/session_id")
    if actor is None:
        changes.append("actor requires manual migration")

    return session, changes


def validate_session(session: dict[str, Any], path: Path) -> list[str]:
    errors: list[str] = []
    if session.get("version") != 2:
        errors.append("version != 2")
    if not isinstance(session.get("id"), str) or not session["id"].strip():
        errors.append("missing id")
    elif path.stem != session["id"]:
        errors.append("filename does not match session id")
    if not isinstance(session.get("messages"), list):
        errors.append("messages is not a list")
    metadata = session.get("metadata")
    if not isinstance(metadata, dict):
        errors.append("metadata is not an object")
    runtime = session.get("runtime")
    if not isinstance(runtime, dict):
        errors.append("runtime is not an object")
    else:
        prompt = runtime.get("prompt")
        if not isinstance(prompt, dict):
            errors.append("runtime.prompt is not an object")
        else:
            frozen = prompt.get("frozen_time_beijing")
            if not isinstance(frozen, str) or not frozen.strip():
                errors.append("runtime.prompt.frozen_time_beijing missing")
    summary = session.get("summary")
    if summary is not None:
        if not isinstance(summary, dict):
            errors.append("summary is not an object")
        else:
            if not isinstance(summary.get("content"), str):
                errors.append("summary.content missing")
            if not isinstance(summary.get("updated_at"), str):
                errors.append("summary.updated_at missing")
    actor = session.get("actor")
    if isinstance(actor, dict):
        channel = actor.get("channel")
        user_id = actor.get("user_id")
        if not isinstance(channel, str) or not channel.strip():
            errors.append("actor.channel missing")
        elif channel in LEGACY_WEB_CHANNELS:
            errors.append("actor.channel still uses legacy web channel")
        if not isinstance(user_id, str) or not user_id.strip():
            errors.append("actor.user_id missing")
        if (
            isinstance(channel, str)
            and channel.strip()
            and isinstance(user_id, str)
            and user_id.strip()
            and session.get("id") != actor_session_id(
                {
                    "channel": normalize_channel(channel),
                    "user_id": user_id.strip(),
                    **(
                        {"channel_scope": actor.get("channel_scope").strip()}
                        if isinstance(actor.get("channel_scope"), str)
                        and actor.get("channel_scope").strip()
                        and actor.get("channel_scope").strip() != "direct"
                        else {}
                    ),
                }
            )
        ):
            errors.append("session id does not match actor identity")
    if errors:
        return [f"{path}: {err}" for err in errors]
    return []


def main() -> int:
    parser = argparse.ArgumentParser(description="Migrate Hone session JSON files to v2.")
    parser.add_argument(
        "--sessions-dir",
        default="./data/sessions",
        help="Directory containing session JSON files (default: ./data/sessions)",
    )
    parser.add_argument(
        "--write",
        action="store_true",
        help="Rewrite files in place. Without this flag the script only reports planned changes.",
    )
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Validate current files without applying any migration logic.",
    )
    args = parser.parse_args()

    sessions_dir = Path(args.sessions_dir)
    if not sessions_dir.exists():
        print(f"[ERROR] sessions dir not found: {sessions_dir}", file=sys.stderr)
        return 2

    files = sorted(sessions_dir.glob("*.json"))
    migrated = 0
    untouched = 0
    errors: list[str] = []

    for path in files:
        try:
            payload = json.loads(path.read_text())
        except Exception as exc:
            errors.append(f"{path}: failed to parse JSON: {exc}")
            continue
        if not isinstance(payload, dict):
            errors.append(f"{path}: top-level JSON must be an object")
            continue

        if args.validate_only:
            target = payload
            changes = []
        else:
            target, changes = migrate_session(payload, path)
        target_path = path
        session_id = target.get("id")
        if isinstance(session_id, str) and session_id.strip():
            target_path = path.with_name(f"{session_id}.json")
        errors.extend(validate_session(target, target_path))
        if args.validate_only:
            untouched += 1
            continue
        if not changes:
            untouched += 1
            continue
        migrated += 1
        print(f"[MIGRATE] {path}")
        for change in changes:
            print(f"  - {change}")
        if args.write:
            if target_path != path and target_path.exists():
                errors.append(f"{target_path}: target path already exists")
                migrated -= 1
                continue
            target_path.write_text(json.dumps(target, ensure_ascii=False, indent=2) + "\n")
            if target_path != path:
                path.unlink()

    mode = "write" if args.write else "dry-run"
    if args.validate_only:
        mode = "validate-only"
    print(
        f"[SUMMARY] mode={mode} files={len(files)} migrated={migrated} untouched={untouched} errors={len(errors)}"
    )
    for err in errors:
        print(f"[ERROR] {err}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
