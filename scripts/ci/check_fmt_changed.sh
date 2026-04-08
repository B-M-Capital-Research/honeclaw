#!/usr/bin/env bash

set -euo pipefail

# Check rustfmt only for changed Rust files in the current CI range.
# This avoids blocking on historical formatting debt while enforcing style on new changes.

BASE_REF=""
DIFF_RANGE=""

if [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]] && [[ -n "${GITHUB_BASE_REF:-}" ]]; then
  git fetch --no-tags --depth=64 origin "${GITHUB_BASE_REF}"
  if ! BASE_REF="$(git merge-base HEAD "origin/${GITHUB_BASE_REF}" 2>/dev/null)"; then
    git fetch --no-tags origin "${GITHUB_BASE_REF}"
    if ! BASE_REF="$(git merge-base HEAD "origin/${GITHUB_BASE_REF}" 2>/dev/null)"; then
      DIFF_RANGE="origin/${GITHUB_BASE_REF}..HEAD"
    fi
  fi
elif [[ -n "${GITHUB_EVENT_BEFORE:-}" ]] && [[ "${GITHUB_EVENT_BEFORE}" != "0000000000000000000000000000000000000000" ]]; then
  BASE_REF="${GITHUB_EVENT_BEFORE}"
elif git rev-parse HEAD^ >/dev/null 2>&1; then
  BASE_REF="$(git rev-parse HEAD^)"
else
  echo "[INFO] unable to determine base ref; skip rustfmt changed-file check"
  exit 0
fi

if [[ -z "${DIFF_RANGE}" ]]; then
  DIFF_RANGE="${BASE_REF}...HEAD"
fi

mapfile -t rs_files < <(git diff --name-only "${DIFF_RANGE}" -- '*.rs')

if [[ ${#rs_files[@]} -eq 0 ]]; then
  echo "[INFO] no changed Rust files; skip rustfmt check"
  exit 0
fi

echo "[INFO] rustfmt --check on changed files:"
printf ' - %s\n' "${rs_files[@]}"
rustfmt --edition 2024 --check "${rs_files[@]}"
