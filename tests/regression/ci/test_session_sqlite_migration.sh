#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

TMP_DIR="$(mktemp -d)"
SESSIONS_DIR="$TMP_DIR/sessions"
DB_PATH="$TMP_DIR/sessions.sqlite3"
mkdir -p "$SESSIONS_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT

cat > "$SESSIONS_DIR/Actor_feishu__direct__alice.json" <<'JSON'
{
  "id": "Actor_feishu__direct__alice",
  "actor": {
    "channel": "feishu",
    "user_id": "alice"
  },
  "created_at": "2026-03-25T09:00:00+08:00",
  "updated_at": "2026-03-25T09:00:00+08:00",
  "messages": [
    {
      "role": "user",
      "content": "hello",
      "timestamp": "2026-03-25T09:00:00+08:00",
      "metadata": {
        "channel": "feishu",
        "open_id": "ou_alice"
      }
    }
  ],
  "metadata": {
    "source": "test"
  },
  "summary": null
}
JSON

cat > "$SESSIONS_DIR/legacy_web_session.json" <<'JSON'
{
  "id": "legacy_web_session",
  "actor": {
    "channel": "",
    "user_id": "bob"
  },
  "created_at": "2026-03-24T10:00:00+08:00",
  "updated_at": "2026-03-24T10:00:00+08:00",
  "messages": [
    {
      "role": "user",
      "content": "legacy hello",
      "timestamp": "2026-03-24T10:00:00+08:00",
      "metadata": null
    },
    {
      "role": "assistant",
      "content": "legacy world",
      "timestamp": "2026-03-24T10:01:00+08:00",
      "metadata": null
    }
  ],
  "metadata": {},
  "summary": null
}
JSON

python3 scripts/migrate_sessions_to_sqlite.py \
  --sessions-dir "$SESSIONS_DIR" \
  --db-path "$DB_PATH" >/tmp/hone_session_sqlite_dry_run.log

python3 scripts/migrate_sessions_to_sqlite.py \
  --sessions-dir "$SESSIONS_DIR" \
  --db-path "$DB_PATH" \
  --write >/tmp/hone_session_sqlite_write1.log

python3 scripts/migrate_sessions_to_sqlite.py \
  --sessions-dir "$SESSIONS_DIR" \
  --db-path "$DB_PATH" \
  --write >/tmp/hone_session_sqlite_write2.log

python3 - <<'PY' "$SESSIONS_DIR/Actor_feishu__direct__alice.json"
from pathlib import Path
import json
path = Path(__import__("sys").argv[1])
payload = json.loads(path.read_text())
payload["messages"].append(
    {
        "role": "assistant",
        "content": "second message",
        "timestamp": "2026-03-25T09:05:00+08:00",
        "metadata": {"tool_name": "demo_tool", "tool_call_id": "call_1"},
    }
)
payload["updated_at"] = "2026-03-25T09:05:00+08:00"
path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n")
PY

python3 scripts/migrate_sessions_to_sqlite.py \
  --sessions-dir "$SESSIONS_DIR" \
  --db-path "$DB_PATH" \
  --write >/tmp/hone_session_sqlite_write3.log

python3 - <<'PY' "$DB_PATH"
import sqlite3
import sys

conn = sqlite3.connect(sys.argv[1])
sessions = conn.execute("SELECT COUNT(*) FROM sessions").fetchone()[0]
messages = conn.execute("SELECT COUNT(*) FROM session_messages").fetchone()[0]
alice_count = conn.execute(
    "SELECT message_count FROM sessions WHERE session_id = ?",
    ("Actor_feishu__direct__alice",),
).fetchone()[0]
legacy_exists = conn.execute(
    "SELECT COUNT(*) FROM sessions WHERE session_id = ?",
    ("legacy_web_session",),
).fetchone()[0]
tool_name = conn.execute(
    "SELECT tool_name FROM session_messages WHERE session_id = ? AND ordinal = 1",
    ("Actor_feishu__direct__alice",),
).fetchone()[0]
assert sessions == 2, sessions
assert messages == 4, messages
assert alice_count == 2, alice_count
assert legacy_exists == 1, legacy_exists
assert tool_name == "demo_tool", tool_name
PY

python3 scripts/diagnose_session_sqlite.py \
  --db-path "$DB_PATH" summary >/tmp/hone_session_sqlite_summary.log
python3 scripts/diagnose_session_sqlite.py \
  --db-path "$DB_PATH" sessions --limit 5 >/tmp/hone_session_sqlite_sessions.log
python3 scripts/diagnose_session_sqlite.py \
  --db-path "$DB_PATH" messages --session-id Actor_feishu__direct__alice >/tmp/hone_session_sqlite_messages.log

grep -q "skipped=2" /tmp/hone_session_sqlite_write2.log
grep -q "imported=1" /tmp/hone_session_sqlite_write3.log
grep -q "sessions: 2" /tmp/hone_session_sqlite_summary.log
grep -q "Actor_feishu__direct__alice" /tmp/hone_session_sqlite_sessions.log
grep -q "second message" /tmp/hone_session_sqlite_messages.log

echo "session sqlite migration regression passed"
