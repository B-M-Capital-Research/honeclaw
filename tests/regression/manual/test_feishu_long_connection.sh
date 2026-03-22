#!/usr/bin/env bash

# Feishu long-connection manual script.
# Args:
#   $1 run_seconds (optional, default: 60)
#   $2 app_id (required by request)
#   $3 app_secret (required by request)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

RUN_SECONDS="${1:-60}"
APP_ID="${2:-}"
APP_SECRET="${3:-}"

if [[ -z "$APP_ID" || -z "$APP_SECRET" ]]; then
  echo "[SKIP] Missing credentials. Usage: bash tests/regression/manual/test_feishu_long_connection.sh <run_seconds> <app_id> <app_secret>"
  exit 0
fi

if ! [[ "$RUN_SECONDS" =~ ^[0-9]+$ ]] || [[ "$RUN_SECONDS" -le 0 ]]; then
  echo "[FAIL] run_seconds must be a positive integer, got: $RUN_SECONDS" >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "[FAIL] node command not found in PATH" >&2
  exit 1
fi

if ! node --input-type=module -e "import('@larksuiteoapi/node-sdk').then(() => process.exit(0)).catch(() => process.exit(1))"; then
  echo "[FAIL] Missing dependency @larksuiteoapi/node-sdk. Run: bun install" >&2
  exit 1
fi

echo "[INFO] Starting Feishu long connection for ${RUN_SECONDS}s..."
echo "[INFO] app_id=$APP_ID"

node --input-type=module - "$APP_ID" "$APP_SECRET" "$RUN_SECONDS" <<'EOF'
import * as Lark from "@larksuiteoapi/node-sdk";

const [, , appId, appSecret, runSecondsRaw] = process.argv;
const runSeconds = Number(runSecondsRaw);

const wsClient = new Lark.WSClient({
  appId,
  appSecret,
  loggerLevel: Lark.LoggerLevel.info,
});

const eventDispatcher = new Lark.EventDispatcher({}).register({
  "im.message.receive_v1": async (data) => {
    const message = data?.message ?? {};
    const sender = data?.sender?.sender_id?.open_id ?? "unknown";
    let text = "";
    try {
      const parsed = JSON.parse(message.content ?? "{}");
      text = parsed.text ?? message.content ?? "";
    } catch {
      text = message.content ?? "";
    }

    console.log(
      `[EVENT] ts=${new Date().toISOString()} message_id=${message.message_id ?? "unknown"} sender=${sender} chat_id=${message.chat_id ?? "unknown"} text=${text}`
    );

    return {};
  },
});

let finished = false;
const exitSafely = (reason) => {
  if (finished) return;
  finished = true;
  console.log(`[INFO] Feishu long connection stopped: ${reason}`);
  process.exit(0);
};

process.on("SIGINT", () => exitSafely("SIGINT"));
process.on("SIGTERM", () => exitSafely("SIGTERM"));

console.log(`[INFO] WS client connecting, will auto-stop after ${runSeconds}s...`);
wsClient.start({ eventDispatcher });

setTimeout(() => {
  exitSafely("timeout");
}, runSeconds * 1000);
EOF
