#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
# An optional private source path validates the same patch against a deployed
# proxy copy. Never copy that complete source or its credentials into the repo.
source_path="${1:-$repo_root/tests/regression/ci/fixtures/gemini-proxy-translation.original.mjs}"
if [ "$#" -gt 1 ] || [ ! -f "$source_path" ]; then
  echo "Usage: $0 [unpatched-private-server.mjs]" >&2
  exit 2
fi
umask 077
test_dir="$(mktemp -d "${TMPDIR:-/tmp}/hone-gemini-proxy-test.XXXXXX")"
trap 'rm -rf "$test_dir"' EXIT
cp "$source_path" "$test_dir/server.mjs"
if ! patch --batch --fuzz=0 -d "$test_dir" -p1 \
    < "$repo_root/scripts/patches/gemini-proxy-tool-response-names.patch" > "$test_dir/patch.log" 2>&1; then
  cat "$test_dir/patch.log" >&2
  exit 1
fi
node --check "$test_dir/server.mjs"
node "$repo_root/tests/regression/ci/test_gemini_proxy_tool_response_names.mjs" "$source_path" "$test_dir/server.mjs"
