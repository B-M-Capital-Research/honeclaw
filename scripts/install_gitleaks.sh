#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel)"
VERSION="${GITLEAKS_VERSION:-8.30.1}"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH_RAW="$(uname -m)"

case "$ARCH_RAW" in
  x86_64|amd64)
    ARCH="x64"
    ;;
  arm64|aarch64)
    ARCH="arm64"
    ;;
  *)
    echo "unsupported architecture: $ARCH_RAW" >&2
    exit 1
    ;;
esac

case "$OS" in
  darwin|linux)
    ;;
  *)
    echo "unsupported OS: $OS" >&2
    exit 1
    ;;
esac

INSTALL_DIR="$ROOT_DIR/.git-tools/gitleaks/$VERSION"
CURRENT_LINK="$ROOT_DIR/.git-tools/gitleaks/current"
BIN_PATH="$INSTALL_DIR/gitleaks"
ARCHIVE_NAME="gitleaks_${VERSION}_${OS}_${ARCH}.tar.gz"
DOWNLOAD_URL="https://github.com/gitleaks/gitleaks/releases/download/v${VERSION}/${ARCHIVE_NAME}"

mkdir -p "$INSTALL_DIR"

if [ ! -x "$BIN_PATH" ]; then
  TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/gitleaks-install.XXXXXX")"
  trap 'rm -rf "$TMP_DIR"' EXIT
  ARCHIVE_PATH="$TMP_DIR/$ARCHIVE_NAME"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
  else
    python3 - <<PY
import urllib.request
urllib.request.urlretrieve("${DOWNLOAD_URL}", "${ARCHIVE_PATH}")
PY
  fi

  tar -xzf "$ARCHIVE_PATH" -C "$INSTALL_DIR"
  chmod +x "$BIN_PATH"
fi

ln -sfn "$INSTALL_DIR" "$CURRENT_LINK"
chmod +x "$ROOT_DIR/.githooks/pre-push"
git config core.hooksPath .githooks

cat <<EOF
Installed gitleaks $VERSION to $BIN_PATH
Configured local core.hooksPath=.githooks
pre-push secret scan and Rust rustfmt gate are now enabled for this clone
EOF
