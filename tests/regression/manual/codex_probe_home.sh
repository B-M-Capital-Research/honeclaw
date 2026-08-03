#!/usr/bin/env bash

# Keep real Codex acceptance probes out of the user's primary Codex task
# index. The probe gets an isolated CODEX_HOME and reuses only the existing
# authentication file through a symlink; sessions, logs, cache, and state are
# created below the caller-owned temporary directory and removed on cleanup.
hone_prepare_isolated_codex_home() {
  local probe_parent="$1"
  local source_home="${HONE_CODEX_PROBE_SOURCE_HOME:-${CODEX_HOME:-${HOME}/.codex}}"
  local source_auth="${HONE_CODEX_PROBE_AUTH_FILE:-$source_home/auth.json}"
  local isolated_home="$probe_parent/codex-home"

  if [[ ! -f "$source_auth" ]]; then
    echo "[FAIL] Codex probe auth is unavailable; set HONE_CODEX_PROBE_AUTH_FILE" >&2
    return 1
  fi
  mkdir -p "$isolated_home"
  ln -s "$source_auth" "$isolated_home/auth.json"
  export CODEX_HOME="$isolated_home"
  echo "[INFO] Codex probe state is isolated from the primary task index"
}
