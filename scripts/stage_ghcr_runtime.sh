#!/usr/bin/env bash

set -euo pipefail

usage() {
    cat >&2 <<'EOF'
usage: stage_ghcr_runtime.sh --image <immutable-image-ref> --revision <git-sha> [--release-root <dir>]

Pulls a Linux runtime bundle with crane, verifies every file and the embedded
revision, and stages it under <release-root>. It does not change current or
restart any service.
EOF
}

IMAGE_REF=""
REVISION=""
RELEASE_ROOT="/opt/hone/releases"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --image)
            IMAGE_REF="${2:-}"
            shift 2
            ;;
        --revision)
            REVISION="${2:-}"
            shift 2
            ;;
        --release-root)
            RELEASE_ROOT="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage
            exit 2
            ;;
    esac
done

if [[ -z "$IMAGE_REF" || -z "$REVISION" ]]; then
    usage
    exit 2
fi
if [[ ! "$REVISION" =~ ^[0-9a-f]{40}$ ]]; then
    echo "revision must be an exact 40-character lowercase Git SHA" >&2
    exit 2
fi
if [[ ! "$IMAGE_REF" =~ ^ghcr\.io/[a-z0-9._/-]+@sha256:[0-9a-f]{64}$ ]]; then
    echo "runtime image must be an exact ghcr.io digest reference" >&2
    exit 2
fi
if [[ ! -d "$RELEASE_ROOT" ]]; then
    echo "release root does not exist: $RELEASE_ROOT" >&2
    exit 2
fi
if ! command -v crane >/dev/null 2>&1; then
    echo "crane is required; install the pinned version from the deployment runbook" >&2
    exit 2
fi

RELEASE_DIR="$RELEASE_ROOT/${REVISION}-ghcr-runtime"
if [[ -d "$RELEASE_DIR" ]]; then
    bash "$RELEASE_DIR/tools/verify_runtime_bundle.sh" "$RELEASE_DIR" "$REVISION"
    printf 'STAGED_RELEASE=%s\n' "$RELEASE_DIR"
    exit 0
fi
if [[ -e "$RELEASE_DIR" ]]; then
    echo "release target exists but is not a directory: $RELEASE_DIR" >&2
    exit 1
fi

STAGING_DIR="$(mktemp -d "$RELEASE_ROOT/.ghcr-stage.XXXXXX")"
cleanup() {
    case "$STAGING_DIR" in
        "$RELEASE_ROOT"/.ghcr-stage.*)
            rm -rf -- "$STAGING_DIR"
            ;;
    esac
}
trap cleanup EXIT

crane export --platform linux/amd64 "$IMAGE_REF" - | tar -xf - -C "$STAGING_DIR"

if [[ ! -d "$STAGING_DIR/release" ]]; then
    echo "OCI image did not contain /release" >&2
    exit 1
fi
bash \
    "$STAGING_DIR/release/tools/verify_runtime_bundle.sh" \
    "$STAGING_DIR/release" \
    "$REVISION"

chown -R root:root "$STAGING_DIR/release"
mv "$STAGING_DIR/release" "$RELEASE_DIR"

printf 'STAGED_RELEASE=%s\n' "$RELEASE_DIR"
