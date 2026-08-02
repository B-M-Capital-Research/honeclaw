#!/usr/bin/env bash

set -euo pipefail

ENV_FILE="${1:-/etc/hone/runtime.env}"

fail() {
  printf '[FAIL] backend runtime env: %s\n' "$*" >&2
  exit 1
}

[[ -f "$ENV_FILE" ]] || fail "environment file not found: $ENV_FILE"
[[ -r "$ENV_FILE" ]] || fail "environment file is not readable: $ENV_FILE"

find_nonempty_key() {
  local keys="$1"
  awk -v accepted_keys="$keys" '
    BEGIN {
      count = split(accepted_keys, key_list, " ")
      for (key_index = 1; key_index <= count; key_index++) {
        accepted[key_list[key_index]] = 1
      }
    }
    /^[[:space:]]*(#|$)/ { next }
    {
      line = $0
      separator = index(line, "=")
      if (separator == 0) next
      key = substr(line, 1, separator - 1)
      value = substr(line, separator + 1)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", key)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
      if (!(key in accepted)) next
      if ((value ~ /^".*"$/) || (value ~ /^\047.*\047$/)) {
        value = substr(value, 2, length(value) - 2)
      }
      lowered = tolower(value)
      if (value == "" || lowered == "changeme" || lowered == "replace-me" ||
          lowered == "replace_me" || lowered == "todo" || lowered == "example" ||
          value ~ /^<.*>$/) next
      print key
      exit
    }
  ' "$ENV_FILE"
}

ACCESS_KEY_ID_NAME="$(find_nonempty_key \
  'ALIBABA_CLOUD_ACCESS_KEY_ID ALIYUN_ACCESS_KEY_ID HONE_ALIYUN_ACCESS_KEY_ID')"
[[ -n "$ACCESS_KEY_ID_NAME" ]] || fail \
  'missing non-empty Aliyun SMS AccessKey ID (ALIBABA_CLOUD_ACCESS_KEY_ID or compatibility alias)'

ACCESS_KEY_SECRET_NAME="$(find_nonempty_key \
  'ALIBABA_CLOUD_ACCESS_KEY_SECRET ALIYUN_ACCESS_KEY_SECRET HONE_ALIYUN_ACCESS_KEY_SECRET')"
[[ -n "$ACCESS_KEY_SECRET_NAME" ]] || fail \
  'missing non-empty Aliyun SMS AccessKey secret (ALIBABA_CLOUD_ACCESS_KEY_SECRET or compatibility alias)'

printf '[OK] backend runtime env: Aliyun SMS credentials present (id=%s, secret=%s)\n' \
  "$ACCESS_KEY_ID_NAME" "$ACCESS_KEY_SECRET_NAME"
