#!/usr/bin/env bash

set -euo pipefail

usage() {
    echo "usage: ${BASH_SOURCE[0]##*/} <source-runtime-bin-dir> <new-output-dir>" >&2
}

if [[ $# -ne 2 ]]; then
    usage
    exit 2
fi

BIN_DIR="$1"
OUTPUT_DIR="$2"
REVISION="${HONE_BUILD_GIT_SHA:-}"
BUILD_TIMESTAMP="${HONE_BUILD_TIMESTAMP:-}"
BUILD_SOURCE="${HONE_BUILD_SOURCE:-}"
BUILD_PROFILE="${HONE_BUILD_PROFILE:-}"
BUILD_TARGET="${HONE_BUILD_TARGET:-}"

if [[ ! "$REVISION" =~ ^[0-9a-f]{40}$ ]]; then
    echo "runtime bundle revision must be an exact 40-character lowercase Git SHA" >&2
    exit 2
fi
if [[ ! "$BUILD_TIMESTAMP" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$ ]]; then
    echo "runtime bundle timestamp must be UTC RFC 3339 without fractional seconds" >&2
    exit 2
fi
if [[ "$BUILD_SOURCE" != "ghcr_linux_oci" ]]; then
    echo "runtime bundle source must be ghcr_linux_oci" >&2
    exit 2
fi
if [[ "$BUILD_PROFILE" != "source-runtime" ]]; then
    echo "runtime bundle Cargo profile must be source-runtime" >&2
    exit 2
fi
if [[ "$BUILD_TARGET" != "x86_64-unknown-linux-gnu" ]]; then
    echo "runtime bundle target must be x86_64-unknown-linux-gnu" >&2
    exit 2
fi
if [[ -e "$OUTPUT_DIR" ]]; then
    echo "runtime bundle output already exists: $OUTPUT_DIR" >&2
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
    if [[ ! -x "$BIN_DIR/$binary" ]]; then
        echo "required runtime binary is missing or not executable: $BIN_DIR/$binary" >&2
        exit 1
    fi
done

umask 022
mkdir -p \
    "$OUTPUT_DIR/bin" \
    "$OUTPUT_DIR/assets/skills/stock_research" \
    "$OUTPUT_DIR/tools"

for binary in "${required_binaries[@]}"; do
    install -m 0755 "$BIN_DIR/$binary" "$OUTPUT_DIR/bin/$binary"
done
install -m 0644 soul.md "$OUTPUT_DIR/assets/soul.md"
install -m 0644 \
    skills/stock_research/SKILL.md \
    "$OUTPUT_DIR/assets/skills/stock_research/SKILL.md"
install -m 0755 \
    scripts/verify_runtime_bundle.sh \
    "$OUTPUT_DIR/tools/verify_runtime_bundle.sh"

printf '%s\n' \
    'format=hone-runtime-bundle-v1' \
    "git_sha=$REVISION" \
    "build_timestamp=$BUILD_TIMESTAMP" \
    "build_source=$BUILD_SOURCE" \
    "cargo_profile=$BUILD_PROFILE" \
    "target=$BUILD_TARGET" \
    > "$OUTPUT_DIR/RELEASE_METADATA"

(
    cd "$OUTPUT_DIR"
    while IFS= read -r file; do
        sha256sum "$file"
    done < <(
        find . -type f ! -name SHA256SUMS -print \
            | sed 's#^\./##' \
            | LC_ALL=C sort
    ) > SHA256SUMS
)

bash "$OUTPUT_DIR/tools/verify_runtime_bundle.sh" "$OUTPUT_DIR" "$REVISION"
