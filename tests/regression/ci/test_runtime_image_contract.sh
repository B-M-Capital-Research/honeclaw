#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

fail() {
    echo "[FAIL] $*" >&2
    exit 1
}

WORKFLOW=.github/workflows/runtime-image.yml
DOCKERFILE=deploy/runtime/Dockerfile
STAGE_SCRIPT=scripts/stage_ghcr_runtime.sh

# Runtime images must not silently ship an unoptimized dependency graph again.
# Keep dev's overflow/debug assertions while optimizing production CPU work.
runtime_profile="$(awk '/^\[profile.source-runtime\]/{inside=1;next} /^\[/{inside=0} inside{print}' Cargo.toml)"
grep -Eq '^opt-level[[:space:]]*=[[:space:]]*3$' <<<"$runtime_profile" \
    || fail "source-runtime must optimize production code and dependencies"
grep -Eq '^inherits[[:space:]]*=[[:space:]]*"dev"$' <<<"$runtime_profile" \
    || fail "source-runtime must retain existing overflow/debug assertion semantics"

grep -Fq 'packages: read' "$WORKFLOW" \
    || fail "runtime export must use job-scoped read-only registry access"
grep -Fq '[[ "$EXPORT_REVISION" =~ ^[0-9a-f]{40}$ ]]' "$WORKFLOW" \
    || fail "runtime export must validate an exact source revision"
grep -Fq 'bash runtime-release/tools/verify_runtime_bundle.sh runtime-release "$EXPORT_REVISION"' "$WORKFLOW" \
    || fail "runtime export must verify payload and revision before upload"
grep -Fq 'sha256sum runtime-bundle.tar.gz > runtime-bundle.tar.gz.sha256' "$WORKFLOW" \
    || fail "runtime export must retain a transfer checksum"

grep -Fq 'platforms: linux/amd64' "$WORKFLOW" \
    || fail "runtime workflow must publish only linux/amd64"
grep -Fq 'packages: write' "$WORKFLOW" \
    || fail "runtime workflow must scope GITHUB_TOKEN for GHCR publication"
grep -Fq 'ghcr.io/b-m-capital-research/honeclaw-runtime' "$WORKFLOW" \
    || fail "runtime workflow must use the canonical GHCR image"
grep -Fq 'cache-to: type=gha,mode=max,scope=hone-runtime-linux-amd64' "$WORKFLOW" \
    || fail "runtime workflow must preserve the Linux BuildKit cache"
grep -Fq 'org.opencontainers.image.source="https://github.com/B-M-Capital-Research/honeclaw"' "$DOCKERFILE" \
    || fail "runtime image must link itself to the source repository before first publish"
grep -Fq 'FROM --platform=linux/amd64 rust:1.95.0-bookworm@sha256:4c2fd73ef19c5ef9d54bee03b06b2839a392604fbfcd578ed948b71b37c1d7fb AS chef' "$DOCKERFILE" \
    || fail "runtime binaries must be built inside the pinned Debian Linux image"
grep -Fq 'cargo install --locked --version 0.1.77 cargo-chef' "$DOCKERFILE" \
    || fail "runtime builder must pin cargo-chef"
grep -Fq 'cargo chef prepare --recipe-path recipe.json' "$DOCKERFILE" \
    || fail "runtime builder must create a dependency-only cache recipe"
grep -Fq 'cargo chef cook --locked --profile source-runtime' "$DOCKERFILE" \
    || fail "runtime builder must export reusable dependency layers"
grep -Fq 'FROM scratch' "$DOCKERFILE" \
    || fail "runtime artifact image must not add an unrelated runtime filesystem"
grep -Fq 'runtime image must be an exact ghcr.io digest reference' "$STAGE_SCRIPT" \
    || fail "runtime staging must reject mutable GHCR tags"
grep -Fq 'crane export --platform linux/amd64 "$IMAGE_REF" -' "$STAGE_SCRIPT" \
    || fail "runtime staging must select the Linux manifest instead of an attestation"

tmp_dir="$(mktemp -d)"
cleanup() {
    rm -rf -- "$tmp_dir"
}
trap cleanup EXIT

mkdir -p "$tmp_dir/bin"
for binary in hone-cli hone-console-page hone-discord hone-feishu hone-mcp hone-telegram; do
    printf '#!/usr/bin/env sh\nexit 0\n' > "$tmp_dir/bin/$binary"
    chmod 0755 "$tmp_dir/bin/$binary"
done

revision=0123456789abcdef0123456789abcdef01234567
HONE_BUILD_GIT_SHA="$revision" \
HONE_BUILD_TIMESTAMP=2026-08-04T00:00:00Z \
HONE_BUILD_SOURCE=ghcr_linux_oci \
HONE_BUILD_PROFILE=source-runtime \
HONE_BUILD_TARGET=x86_64-unknown-linux-gnu \
bash scripts/package_runtime_bundle.sh "$tmp_dir/bin" "$tmp_dir/bundle"

bash scripts/verify_runtime_bundle.sh "$tmp_dir/bundle" "$revision"

printf 'tampered\n' >> "$tmp_dir/bundle/assets/soul.md"
if bash scripts/verify_runtime_bundle.sh "$tmp_dir/bundle" "$revision" >/dev/null 2>&1; then
    fail "runtime bundle verification accepted a tampered payload"
fi

echo "[PASS] Linux GHCR runtime image contract"
