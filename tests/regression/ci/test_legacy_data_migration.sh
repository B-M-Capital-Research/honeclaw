#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

SESSIONS_DIR="$TMP_DIR/sessions"
CRON_DIR="$TMP_DIR/cron_jobs"
SKILLS_DIR="$TMP_DIR/skills"
mkdir -p "$SESSIONS_DIR" "$CRON_DIR" "$SKILLS_DIR"

cat > "$SESSIONS_DIR/legacy_web_session.json" <<'JSON'
{
  "version": 1,
  "id": "Actor_web_5ftest__direct__bob",
  "created_at": "2026-03-17T10:00:00+08:00",
  "updated_at": "2026-03-17T11:00:00+08:00",
  "messages": [],
  "metadata": {}
}
JSON

cat > "$CRON_DIR/cron_jobs_web_5ftest__direct__bob.json" <<'JSON'
{
  "user_id": "bob",
  "jobs": [
    {
      "id": "j_demo",
      "name": "legacy web task",
      "schedule": { "hour": 9, "minute": 30, "repeat": "daily" },
      "task_prompt": "ping",
      "push": { "type": "analysis" },
      "enabled": true,
      "channel": "web_test",
      "channel_target": "bob",
      "created_at": "2026-03-17T10:00:00+08:00",
      "last_run_at": null
    }
  ],
  "pending_updates": []
}
JSON

cat > "$SKILLS_DIR/portfolio_management.yaml" <<'YAML'
name: "仓位管理"
description: "管理仓位"
aliases:
  - 持仓
tools:
  - portfolio_tool
prompt: |
  请读取用户仓位并给出建议。
YAML

python3 scripts/migrate_legacy_data.py \
  --sessions-dir "$SESSIONS_DIR" \
  --cron-jobs-dir "$CRON_DIR" \
  --skills-dir "$SKILLS_DIR" >/tmp/hone_legacy_migrate_dry_run.log

python3 scripts/migrate_legacy_data.py \
  --sessions-dir "$SESSIONS_DIR" \
  --cron-jobs-dir "$CRON_DIR" \
  --skills-dir "$SKILLS_DIR" \
  --write >/tmp/hone_legacy_migrate_write.log

python3 scripts/migrate_legacy_data.py \
  --sessions-dir "$SESSIONS_DIR" \
  --cron-jobs-dir "$CRON_DIR" \
  --skills-dir "$SKILLS_DIR" \
  --validate-only >/tmp/hone_legacy_migrate_validate.log

python3 - <<'PY' "$SESSIONS_DIR" "$CRON_DIR" "$SKILLS_DIR"
import json, sys
from pathlib import Path

sessions = Path(sys.argv[1])
cron_dir = Path(sys.argv[2])
skills = Path(sys.argv[3])

session = json.loads((sessions / "Actor_web__direct__bob.json").read_text())
assert session["actor"]["channel"] == "web"
assert not (sessions / "legacy_web_session.json").exists()

cron = json.loads((cron_dir / "cron_jobs_web__direct__bob.json").read_text())
assert cron["actor"]["channel"] == "web"
assert cron["jobs"][0]["channel"] == "web"
assert not (cron_dir / "cron_jobs_web_5ftest__direct__bob.json").exists()

skill_md = (skills / "portfolio_management" / "SKILL.md").read_text()
assert 'name: "仓位管理"' in skill_md
assert "请读取用户仓位并给出建议。" in skill_md
assert not (skills / "portfolio_management.yaml").exists()
PY

echo "legacy data migration regression passed"
