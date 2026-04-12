#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
INSTALL_SCRIPT="$ROOT_DIR/scripts/install_hone_cli.sh"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hone-install-path.XXXXXX")"
TOOLS_DIR="$TMP_ROOT/tools"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

mkdir -p "$TOOLS_DIR"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH_RAW="$(uname -m)"

case "$OS" in
  darwin) OS_SLUG="darwin" ;;
  linux) OS_SLUG="linux" ;;
  *)
    echo "[FAIL] unsupported test OS: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH_RAW" in
  arm64|aarch64) ARCH_SLUG="aarch64" ;;
  x86_64|amd64) ARCH_SLUG="x86_64" ;;
  *)
    echo "[FAIL] unsupported test architecture: $ARCH_RAW" >&2
    exit 1
    ;;
esac

ASSET_NAME="honeclaw-${OS_SLUG}-${ARCH_SLUG}.tar.gz"
BUNDLE_NAME="${ASSET_NAME%.tar.gz}"
ARCHIVE_PATH="$TMP_ROOT/$ASSET_NAME"

mkdir -p "$TMP_ROOT/$BUNDLE_NAME/bin" "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw"
cat > "$TMP_ROOT/$BUNDLE_NAME/bin/hone-cli" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "mock hone-cli $*"
EOF
chmod +x "$TMP_ROOT/$BUNDLE_NAME/bin/hone-cli"
cat > "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/config.example.yaml" <<'EOF'
agent:
  runner: mock
EOF
cat > "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/soul.md" <<'EOF'
# mock soul
EOF
tar -czf "$ARCHIVE_PATH" -C "$TMP_ROOT" "$BUNDLE_NAME"

cat > "$TOOLS_DIR/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

output=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    -o)
      output="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [[ -z "$output" ]]; then
  echo "missing curl -o target" >&2
  exit 1
fi

cp "$MOCK_ARCHIVE_PATH" "$output"
EOF
chmod +x "$TOOLS_DIR/curl"

run_path_hit_case() {
  local home_dir="$TMP_ROOT/home-path-hit"
  local path_bin="$home_dir/custom-bin"
  mkdir -p "$path_bin"

  local output
  output="$(
    env \
      HOME="$home_dir" \
      PATH="$TOOLS_DIR:$path_bin:/usr/bin:/bin" \
      HONE_RUN_ONBOARD=0 \
      HONE_GITHUB_REPO="example/honeclaw" \
      MOCK_ARCHIVE_PATH="$ARCHIVE_PATH" \
      bash "$INSTALL_SCRIPT"
  )"

  if [[ ! -x "$path_bin/hone-cli" ]]; then
    echo "[FAIL] installer did not place wrapper into writable PATH dir" >&2
    exit 1
  fi

  local resolved
  resolved="$(
    env HOME="$home_dir" PATH="$TOOLS_DIR:$path_bin:/usr/bin:/bin" bash -c 'command -v hone-cli'
  )"
  if [[ "$resolved" != "$path_bin/hone-cli" ]]; then
    echo "[FAIL] command -v hone-cli resolved to unexpected path: $resolved" >&2
    exit 1
  fi

  local cli_output
  cli_output="$(
    env HOME="$home_dir" PATH="$TOOLS_DIR:$path_bin:/usr/bin:/bin" hone-cli smoke
  )"
  if [[ "$cli_output" != "mock hone-cli smoke" ]]; then
    echo "[FAIL] installed wrapper did not launch bundled hone-cli: $cli_output" >&2
    exit 1
  fi

  if [[ "$output" == *'Current shell PATH does not include'* ]]; then
    echo "[FAIL] installer printed fallback PATH warning despite using a PATH dir" >&2
    exit 1
  fi
}

run_fallback_case() {
  local shell_path="$1"
  local home_dir="$2"
  local expected_rc="$3"
  local expected_shell_name
  expected_shell_name="$(basename "$shell_path")"
  mkdir -p "$home_dir"

  local output
  output="$(
    env \
      HOME="$home_dir" \
      SHELL="$shell_path" \
      PATH="$TOOLS_DIR:/usr/bin:/bin" \
      HONE_RUN_ONBOARD=0 \
      HONE_GITHUB_REPO="example/honeclaw" \
      MOCK_ARCHIVE_PATH="$ARCHIVE_PATH" \
      bash "$INSTALL_SCRIPT"
  )"

  local fallback_bin="$home_dir/.local/bin/hone-cli"
  if [[ ! -x "$fallback_bin" ]]; then
    echo "[FAIL] installer did not fall back to ~/.local/bin" >&2
    exit 1
  fi

  if [[ "$output" != *'Current shell PATH does not include'* ]]; then
    echo "[FAIL] installer did not explain missing PATH export for fallback bin dir" >&2
    exit 1
  fi

  if [[ "$output" != *"Detected login shell: $expected_shell_name"* ]]; then
    echo "[FAIL] installer did not print detected shell-specific guidance" >&2
    exit 1
  fi

  if [[ "$output" != *"printf '\\nexport PATH=\"$home_dir/.local/bin:\$PATH\"\\n' >> \"$expected_rc\""* ]]; then
    echo "[FAIL] installer did not print the expected persistent PATH command" >&2
    exit 1
  fi

  if [[ "$output" != *"source \"$expected_rc\""* ]]; then
    echo "[FAIL] installer did not print the expected shell reload command" >&2
    exit 1
  fi

  local cli_output
  cli_output="$(
    env HOME="$home_dir" PATH="$home_dir/.local/bin:$TOOLS_DIR:/usr/bin:/bin" hone-cli smoke
  )"
  if [[ "$cli_output" != "mock hone-cli smoke" ]]; then
    echo "[FAIL] fallback wrapper did not launch bundled hone-cli: $cli_output" >&2
    exit 1
  fi
}

run_path_hit_case

run_fallback_case "/bin/zsh" "$TMP_ROOT/home-fallback-zsh" "$TMP_ROOT/home-fallback-zsh/.zshrc"
mkdir -p "$TMP_ROOT/home-fallback-bash"
touch "$TMP_ROOT/home-fallback-bash/.bashrc"
run_fallback_case "/bin/bash" "$TMP_ROOT/home-fallback-bash" "$TMP_ROOT/home-fallback-bash/.bashrc"

echo "[PASS] hone-cli install path resolution regression passed"
