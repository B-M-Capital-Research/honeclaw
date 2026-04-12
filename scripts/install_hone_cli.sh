#!/usr/bin/env bash

set -euo pipefail

REPO="${HONE_GITHUB_REPO:-B-M-Capital-Research/honeclaw}"
VERSION="${HONE_VERSION:-latest}"
INSTALL_ROOT="${HONE_INSTALL_DIR:-$HOME/.honeclaw}"
BIN_DIR="${HONE_BIN_DIR:-$HOME/.local/bin}"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/honeclaw-install.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH_RAW="$(uname -m)"

case "$OS" in
  darwin) OS_SLUG="darwin" ;;
  linux) OS_SLUG="linux" ;;
  *)
    echo "unsupported OS: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH_RAW" in
  arm64|aarch64) ARCH_SLUG="aarch64" ;;
  x86_64|amd64) ARCH_SLUG="x86_64" ;;
  *)
    echo "unsupported architecture: $ARCH_RAW" >&2
    exit 1
    ;;
esac

ASSET_NAME="honeclaw-${OS_SLUG}-${ARCH_SLUG}.tar.gz"
if [[ "$VERSION" == "latest" ]]; then
  DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
else
  TAG="$VERSION"
  if [[ "$TAG" != v* ]]; then
    TAG="v${TAG}"
  fi
  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET_NAME}"
fi

ARCHIVE_PATH="$TMP_DIR/$ASSET_NAME"

download_file() {
  if command -v curl >/dev/null 2>&1; then
    if ! curl --retry 3 --retry-delay 1 -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"; then
      echo "failed to download release asset: $DOWNLOAD_URL" >&2
      echo "ensure the requested Hone release exists and includes $ASSET_NAME" >&2
      exit 1
    fi
  elif command -v python3 >/dev/null 2>&1; then
    if ! python3 - <<PY
import urllib.request
urllib.request.urlretrieve("${DOWNLOAD_URL}", "${ARCHIVE_PATH}")
PY
    then
      echo "failed to download release asset: $DOWNLOAD_URL" >&2
      echo "ensure the requested Hone release exists and includes $ASSET_NAME" >&2
      exit 1
    fi
  else
    echo "curl or python3 is required to download ${DOWNLOAD_URL}" >&2
    exit 1
  fi
}

download_file

TOP_DIR="$(tar -tzf "$ARCHIVE_PATH" | head -1 | cut -d/ -f1)"
if [[ -z "$TOP_DIR" ]]; then
  echo "failed to inspect archive layout: $ARCHIVE_PATH" >&2
  exit 1
fi

RELEASES_DIR="$INSTALL_ROOT/releases"
DEST_DIR="$RELEASES_DIR/$TOP_DIR"
mkdir -p "$RELEASES_DIR" "$BIN_DIR"
rm -rf "$DEST_DIR"
tar -xzf "$ARCHIVE_PATH" -C "$RELEASES_DIR"

CURRENT_LINK="$INSTALL_ROOT/current"
ln -sfn "$DEST_DIR" "$CURRENT_LINK"

if [[ ! -f "$INSTALL_ROOT/config.yaml" ]]; then
  cp "$DEST_DIR/share/honeclaw/config.example.yaml" "$INSTALL_ROOT/config.yaml"
fi
if [[ ! -f "$INSTALL_ROOT/soul.md" ]]; then
  cp "$DEST_DIR/share/honeclaw/soul.md" "$INSTALL_ROOT/soul.md"
fi
mkdir -p "$INSTALL_ROOT/data/runtime"

WRAPPER_PATH="$BIN_DIR/hone-cli"
cat > "$WRAPPER_PATH" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

HONE_HOME="${HONE_HOME:-$HOME/.honeclaw}"
CURRENT_ROOT="$HONE_HOME/current"

export HONE_HOME
export HONE_INSTALL_ROOT="${HONE_INSTALL_ROOT:-$CURRENT_ROOT}"
export HONE_USER_CONFIG_PATH="${HONE_USER_CONFIG_PATH:-$HONE_HOME/config.yaml}"
export HONE_DATA_DIR="${HONE_DATA_DIR:-$HONE_HOME/data}"
export HONE_SKILLS_DIR="${HONE_SKILLS_DIR:-$CURRENT_ROOT/share/honeclaw/skills}"

exec "$CURRENT_ROOT/bin/hone-cli" "$@"
EOF
chmod +x "$WRAPPER_PATH"

run_onboard=0
case "${HONE_RUN_ONBOARD:-ask}" in
  1|true|TRUE|yes|YES|on|ON)
    run_onboard=1
    ;;
  0|false|FALSE|no|NO|off|OFF)
    run_onboard=0
    ;;
  *)
    if [[ -t 0 && -t 1 ]]; then
      read -r -p "Run guided setup now? [Y/n] " response
      case "${response:-Y}" in
        n|N|no|NO) run_onboard=0 ;;
        *) run_onboard=1 ;;
      esac
    fi
    ;;
esac

if [[ "$run_onboard" == "1" ]]; then
  if ! HONE_HOME="$INSTALL_ROOT" "$WRAPPER_PATH" onboard; then
    echo "Guided setup exited before completion. You can rerun it later with: hone-cli onboard" >&2
  fi
fi

cat <<EOF
Installed Hone CLI bundle to $DEST_DIR
Wrapper: $WRAPPER_PATH
Config: $INSTALL_ROOT/config.yaml
Data dir: $INSTALL_ROOT/data

If "$BIN_DIR" is not in PATH, add:
  export PATH="$BIN_DIR:\$PATH"

Next steps:
  hone-cli doctor
  hone-cli onboard
  hone-cli configure --section agent --section channels --section providers
  hone-cli start
EOF
