#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "usage: ${BASH_SOURCE[0]##*/} <bundle-dir> [expected-git-sha]" >&2
    exit 2
fi

BUNDLE_DIR="$1"
EXPECTED_REVISION="${2:-}"

if [[ ! -d "$BUNDLE_DIR" ]]; then
    echo "runtime bundle directory does not exist: $BUNDLE_DIR" >&2
    exit 2
fi
if [[ -n "$EXPECTED_REVISION" && ! "$EXPECTED_REVISION" =~ ^[0-9a-f]{40}$ ]]; then
    echo "expected revision must be an exact 40-character lowercase Git SHA" >&2
    exit 2
fi

required_binaries=(
    hone-cli
    hone-console-page
    hone-discord
    hone-feishu
    hone-mcp
    hone-telegram
)

for binary in "${required_binaries[@]}"; do
    if [[ ! -x "$BUNDLE_DIR/bin/$binary" ]]; then
        echo "required runtime binary is missing or not executable: bin/$binary" >&2
        exit 1
    fi
done

required_files=(
    RELEASE_METADATA
    SHA256SUMS
    assets/soul.md
    assets/skills/stock_research/SKILL.md
    tools/verify_runtime_bundle.sh
)

for file in "${required_files[@]}"; do
    if [[ ! -f "$BUNDLE_DIR/$file" ]]; then
        echo "required runtime bundle file is missing: $file" >&2
        exit 1
    fi
done

if find "$BUNDLE_DIR" -type l -print -quit | grep -q .; then
    echo "runtime bundle must not contain symbolic links" >&2
    exit 1
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf -- "$TMP_DIR"
}
trap cleanup EXIT

(
    cd "$BUNDLE_DIR"
    find . -type f ! -name SHA256SUMS -print \
        | sed 's#^\./##' \
        | LC_ALL=C sort \
        > "$TMP_DIR/actual-files"

    awk '
        length($1) != 64 || $1 !~ /^[0-9a-f]+$/ || NF != 2 { invalid = 1 }
        { print $2 }
        END { if (invalid) exit 1 }
    ' SHA256SUMS \
        | LC_ALL=C sort \
        > "$TMP_DIR/manifest-files"

    if ! cmp -s "$TMP_DIR/actual-files" "$TMP_DIR/manifest-files"; then
        echo "runtime bundle file list does not exactly match SHA256SUMS" >&2
        diff -u "$TMP_DIR/manifest-files" "$TMP_DIR/actual-files" >&2 || true
        exit 1
    fi

    sha256sum -c --strict SHA256SUMS >/dev/null
)

if [[ "$(wc -l < "$BUNDLE_DIR/RELEASE_METADATA" | tr -d ' ')" != "6" ]]; then
    echo "runtime bundle metadata must contain exactly six fields" >&2
    exit 1
fi
if cut -d= -f1 "$BUNDLE_DIR/RELEASE_METADATA" | LC_ALL=C sort | uniq -d | grep -q .; then
    echo "runtime bundle metadata contains duplicate fields" >&2
    exit 1
fi

grep -Fxq 'format=hone-runtime-bundle-v1' "$BUNDLE_DIR/RELEASE_METADATA"
grep -Eq '^git_sha=[0-9a-f]{40}$' "$BUNDLE_DIR/RELEASE_METADATA"
grep -Eq '^build_timestamp=[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$' "$BUNDLE_DIR/RELEASE_METADATA"
grep -Fxq 'build_source=ghcr_linux_oci' "$BUNDLE_DIR/RELEASE_METADATA"
grep -Fxq 'cargo_profile=source-runtime' "$BUNDLE_DIR/RELEASE_METADATA"
grep -Fxq 'target=x86_64-unknown-linux-gnu' "$BUNDLE_DIR/RELEASE_METADATA"

if [[ -n "$EXPECTED_REVISION" ]]; then
    grep -Fxq "git_sha=$EXPECTED_REVISION" "$BUNDLE_DIR/RELEASE_METADATA"
fi

echo "[PASS] verified HONE runtime bundle${EXPECTED_REVISION:+ for $EXPECTED_REVISION}"
