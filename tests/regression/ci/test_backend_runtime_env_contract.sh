#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CHECK_SCRIPT="$REPO_ROOT/scripts/check_backend_runtime_env.sh"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT

expect_failure() {
  local label="$1"
  local env_file="$2"
  if output="$(bash "$CHECK_SCRIPT" "$env_file" 2>&1)"; then
    printf '[FAIL] %s unexpectedly passed\n' "$label" >&2
    exit 1
  fi
  [[ "$output" == *'[FAIL] backend runtime env:'* ]] || {
    printf '[FAIL] %s did not return the safe failure prefix\n' "$label" >&2
    exit 1
  }
}

expect_success() {
  local label="$1"
  local env_file="$2"
  local forbidden_value="$3"
  local output
  output="$(bash "$CHECK_SCRIPT" "$env_file")"
  [[ "$output" == *'[OK] backend runtime env:'* ]] || {
    printf '[FAIL] %s did not return the success marker\n' "$label" >&2
    exit 1
  }
  [[ "$output" != *"$forbidden_value"* ]] || {
    printf '[FAIL] %s leaked a credential value\n' "$label" >&2
    exit 1
  }
}

expect_failure missing-file "$TMP_ROOT/missing.env"

EMPTY_FILE="$TMP_ROOT/empty.env"
printf '# no credentials\n' > "$EMPTY_FILE"
expect_failure empty-file "$EMPTY_FILE"

ID_ONLY_FILE="$TMP_ROOT/id-only.env"
printf 'ALIBABA_CLOUD_ACCESS_KEY_ID=test-id-only\n' > "$ID_ONLY_FILE"
expect_failure id-only "$ID_ONLY_FILE"

PLACEHOLDER_FILE="$TMP_ROOT/placeholder.env"
printf 'ALIBABA_CLOUD_ACCESS_KEY_ID=<access-key-id>\nALIBABA_CLOUD_ACCESS_KEY_SECRET=changeme\n' > "$PLACEHOLDER_FILE"
expect_failure placeholder "$PLACEHOLDER_FILE"

CANONICAL_FILE="$TMP_ROOT/canonical.env"
printf 'ALIBABA_CLOUD_ACCESS_KEY_ID="canonical-id"\nALIBABA_CLOUD_ACCESS_KEY_SECRET="canonical-secret"\n' > "$CANONICAL_FILE"
expect_success canonical "$CANONICAL_FILE" canonical-secret

COMPAT_FILE="$TMP_ROOT/compat.env"
printf 'HONE_ALIYUN_ACCESS_KEY_ID=compat-id\nHONE_ALIYUN_ACCESS_KEY_SECRET=compat-secret\n' > "$COMPAT_FILE"
expect_success compatibility-alias "$COMPAT_FILE" compat-secret

MIXED_FILE="$TMP_ROOT/mixed.env"
printf 'ALIYUN_ACCESS_KEY_ID=mixed-id\nALIBABA_CLOUD_ACCESS_KEY_SECRET=mixed-secret\n' > "$MIXED_FILE"
expect_success mixed-aliases "$MIXED_FILE" mixed-secret

printf '[OK] backend runtime env contract regression passed\n'
