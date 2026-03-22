#!/usr/bin/env python3
"""Migrate legacy Hone cron job files to the current actor/channel schema."""

from __future__ import annotations

import argparse
import json
import sys
from copy import deepcopy
from pathlib import Path
from typing import Any

CANONICAL_WEB_CHANNEL = "web"
LEGACY_WEB_CHANNELS = {"", "web_test"}


def encode_component(raw: str) -> str:
    out: list[str] = []
    for byte in raw.encode():
        if chr(byte).isalnum() or byte == ord("-"):
            out.append(chr(byte))
        else:
            out.append(f"_{byte:02x}")
    return "".join(out)


def normalize_channel(channel: str) -> str:
    trimmed = channel.strip()
    if trimmed in LEGACY_WEB_CHANNELS:
        return CANONICAL_WEB_CHANNEL
    return trimmed


def storage_key(actor: dict[str, Any]) -> str:
    scope = actor.get("channel_scope") or "direct"
    return "__".join(
        [
            encode_component(actor["channel"]),
            encode_component(scope),
            encode_component(actor["user_id"]),
        ]
    )


def infer_actor(payload: dict[str, Any]) -> dict[str, Any] | None:
    raw_actor = payload.get("actor")
    if isinstance(raw_actor, dict):
        channel = normalize_channel(str(raw_actor.get("channel", "")))
        user_id = str(raw_actor.get("user_id", "")).strip()
        if channel and user_id:
            actor: dict[str, Any] = {"channel": channel, "user_id": user_id}
            scope = raw_actor.get("channel_scope")
            if isinstance(scope, str) and scope.strip() and scope.strip() != "direct":
                actor["channel_scope"] = scope.strip()
            return actor

    user_id = str(payload.get("user_id", "")).strip()
    jobs = payload.get("jobs")
    if not user_id or not isinstance(jobs, list) or not jobs:
        return None

    normalized_channels = {
        normalize_channel(str(job.get("channel", "")))
        for job in jobs
        if isinstance(job, dict)
    }
    normalized_channels.discard("")
    if len(normalized_channels) != 1:
        return None

    actor: dict[str, Any] = {"channel": normalized_channels.pop(), "user_id": user_id}
    first_scope = next(
        (
            str(job.get("channel_scope", "")).strip()
            for job in jobs
            if isinstance(job, dict) and str(job.get("channel_scope", "")).strip()
        ),
        "",
    )
    if first_scope and first_scope != "direct":
        actor["channel_scope"] = first_scope
    return actor


def migrate_payload(payload: dict[str, Any]) -> tuple[dict[str, Any], list[str]]:
    data = deepcopy(payload)
    changes: list[str] = []

    actor = infer_actor(data)
    if actor is None:
        changes.append("actor requires manual migration")
        return data, changes

    if data.get("actor") != actor:
        data["actor"] = actor
        changes.append("normalized actor")

    if data.get("user_id") != actor["user_id"]:
        data["user_id"] = actor["user_id"]
        changes.append("normalized user_id")

    jobs = data.get("jobs")
    if isinstance(jobs, list):
        changed_jobs = False
        for job in jobs:
            if not isinstance(job, dict):
                continue
            normalized_channel = normalize_channel(str(job.get("channel", "")))
            if job.get("channel") != normalized_channel:
                job["channel"] = normalized_channel
                changed_jobs = True
        if changed_jobs:
            changes.append("normalized job channels")

    return data, changes


def validate_payload(payload: dict[str, Any], path: Path) -> list[str]:
    errors: list[str] = []
    actor = payload.get("actor")
    if not isinstance(actor, dict):
        errors.append("actor is not an object")
        return [f"{path}: {err}" for err in errors]

    channel = actor.get("channel")
    user_id = actor.get("user_id")
    if not isinstance(channel, str) or not channel.strip():
        errors.append("actor.channel missing")
    elif channel in LEGACY_WEB_CHANNELS:
        errors.append("actor.channel still uses legacy web channel")
    if not isinstance(user_id, str) or not user_id.strip():
        errors.append("actor.user_id missing")

    if isinstance(channel, str) and channel.strip() and isinstance(user_id, str) and user_id.strip():
        expected_name = f"cron_jobs_{storage_key(actor)}.json"
        if path.name != expected_name:
            errors.append("filename does not match actor storage key")

    jobs = payload.get("jobs")
    if not isinstance(jobs, list):
        errors.append("jobs is not a list")
    else:
        for idx, job in enumerate(jobs):
            if not isinstance(job, dict):
                errors.append(f"jobs[{idx}] is not an object")
                continue
            job_channel = job.get("channel")
            if not isinstance(job_channel, str) or not job_channel.strip():
                errors.append(f"jobs[{idx}].channel missing")
            elif job_channel in LEGACY_WEB_CHANNELS:
                errors.append(f"jobs[{idx}].channel still uses legacy web channel")

    return [f"{path}: {err}" for err in errors]


def main() -> int:
    parser = argparse.ArgumentParser(description="Migrate Hone cron job files to current schema.")
    parser.add_argument(
        "--cron-jobs-dir",
        default="./data/cron_jobs",
        help="Directory containing cron job JSON files (default: ./data/cron_jobs)",
    )
    parser.add_argument("--write", action="store_true", help="Rewrite files in place.")
    parser.add_argument(
        "--validate-only",
        action="store_true",
        help="Validate current files without applying migration logic.",
    )
    args = parser.parse_args()

    root = Path(args.cron_jobs_dir)
    if not root.exists():
        print(f"[ERROR] cron jobs dir not found: {root}", file=sys.stderr)
        return 2

    files = sorted(root.glob("cron_jobs_*.json"))
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
            changes: list[str] = []
        else:
            target, changes = migrate_payload(payload)

        actor = target.get("actor")
        target_path = path
        if isinstance(actor, dict) and actor.get("channel") and actor.get("user_id"):
            target_path = path.with_name(f"cron_jobs_{storage_key(actor)}.json")
        errors.extend(validate_payload(target, target_path))

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
