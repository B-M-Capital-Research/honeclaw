#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[FAIL] missing command: $1" >&2
    exit 1
  fi
}

require_env() {
  if [[ -z "${!1:-}" ]]; then
    echo "[FAIL] missing env: $1" >&2
    exit 1
  fi
}

require_any_env() {
  local name
  for name in "$@"; do
    if [[ -n "${!name:-}" ]]; then
      return 0
    fi
  done
  echo "[FAIL] missing one of env: $*" >&2
  exit 1
}

require_aliyun_credentials() {
  require_any_env \
    ALIBABA_CLOUD_ACCESS_KEY_ID \
    ALIYUN_ACCESS_KEY_ID \
    HONE_ALIYUN_ACCESS_KEY_ID
  require_any_env \
    ALIBABA_CLOUD_ACCESS_KEY_SECRET \
    ALIYUN_ACCESS_KEY_SECRET \
    HONE_ALIYUN_ACCESS_KEY_SECRET
}

require_captcha_env() {
  require_aliyun_credentials
  require_env HONE_ALIYUN_CAPTCHA_PREFIX
  require_env HONE_ALIYUN_CAPTCHA_SCENE_ID
}

require_sms_env() {
  require_aliyun_credentials
  require_env HONE_ALIYUN_SMS_LIVE_PHONE
}

run_captcha_smoke() {
  require_captcha_env

  echo "==> hone-web-api::aliyun_captcha::tests::live_probe_smoke"
  cargo test -p hone-web-api --lib aliyun_captcha::tests::live_probe_smoke -- --ignored --nocapture
}

run_sms_smoke() {
  require_sms_env

  echo "==> hone-web-api::aliyun_sms::tests::live_send_verify_code_smoke"
  cargo test -p hone-web-api --lib aliyun_sms::tests::live_send_verify_code_smoke -- --ignored --nocapture
}

if [[ "${RUN_ALIYUN_LIVE_SMOKES:-0}" != "1" ]]; then
  echo "[INFO] skip Aliyun live smokes; set RUN_ALIYUN_LIVE_SMOKES=1 to run"
  exit 0
fi

require_cmd cargo

case "${ALIYUN_LIVE_SCOPE:-all}" in
  captcha)
    run_captcha_smoke
    ;;
  sms)
    run_sms_smoke
    ;;
  all)
    require_captcha_env
    require_sms_env
    run_captcha_smoke
    run_sms_smoke
    ;;
  *)
    echo "[FAIL] ALIYUN_LIVE_SCOPE must be one of: all, captcha, sms" >&2
    exit 1
    ;;
esac
