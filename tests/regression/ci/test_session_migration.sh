#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

cat > "$TMP_DIR/legacy_summary_string.json" <<'JSON'
{
  "id": "legacy-summary-string",
  "created_at": "2026-03-17T10:00:00+08:00",
  "updated_at": "2026-03-17T11:00:00+08:00",
  "messages": [],
  "metadata": {},
  "summary": "old summary"
}
JSON

cat > "$TMP_DIR/legacy_summary_message.json" <<'JSON'
{
  "id": "legacy-summary-message",
  "created_at": "2026-03-17T10:00:00+08:00",
  "updated_at": "2026-03-17T11:00:00+08:00",
  "messages": [
    {
      "role": "system",
      "content": "summary from message",
      "timestamp": "2026-03-17T10:30:00+08:00",
      "metadata": {"is_summary": true}
    },
    {
      "role": "user",
      "content": "hello",
      "timestamp": "2026-03-17T10:31:00+08:00"
    }
  ]
}
JSON

cat > "$TMP_DIR/Actor_web_5ftest__direct__alice.json" <<'JSON'
{
  "version": 1,
  "id": "Actor_web_5ftest__direct__alice",
  "created_at": "2026-03-17T10:00:00+08:00",
  "updated_at": "2026-03-17T11:00:00+08:00",
  "messages": [],
  "metadata": {}
}
JSON

python3 scripts/migrate_sessions.py --sessions-dir "$TMP_DIR" >/tmp/hone_migrate_dry_run.log
python3 scripts/migrate_sessions.py --sessions-dir "$TMP_DIR" --write >/tmp/hone_migrate_write.log
python3 scripts/migrate_sessions.py --sessions-dir "$TMP_DIR" --validate-only >/tmp/hone_migrate_validate.log

python3 - <<'PY' "$TMP_DIR"
import json, sys
from pathlib import Path
root = Path(sys.argv[1])
summary_string = json.loads((root / 'legacy-summary-string.json').read_text())
assert summary_string['version'] == 2
assert summary_string['summary']['content'] == 'old summary'
assert summary_string['runtime']['prompt']['frozen_time_beijing'] == '2026-03-17T10:00:00+08:00'
assert not (root / 'legacy_summary_string.json').exists()

summary_message = json.loads((root / 'legacy-summary-message.json').read_text())
assert summary_message['version'] == 2
assert summary_message['summary']['content'] == 'summary from message'
assert len(summary_message['messages']) == 1
assert summary_message['messages'][0]['role'] == 'user'
assert summary_message['runtime']['prompt']['frozen_time_beijing'] == '2026-03-17T10:00:00+08:00'
assert not (root / 'legacy_summary_message.json').exists()

web_session = json.loads((root / 'Actor_web__direct__alice.json').read_text())
assert web_session['version'] == 2
assert web_session['id'] == 'Actor_web__direct__alice'
assert web_session['actor']['channel'] == 'web'
assert web_session['actor']['user_id'] == 'alice'
assert not (root / 'Actor_web_5ftest__direct__alice.json').exists()
PY

echo "session migration regression passed"
