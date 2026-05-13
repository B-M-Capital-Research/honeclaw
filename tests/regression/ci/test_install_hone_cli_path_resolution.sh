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
INCOMPLETE_ARCHIVE_PATH="$TMP_ROOT/incomplete-$ASSET_NAME"
UNSAFE_ARCHIVE_PATH="$TMP_ROOT/unsafe-$ASSET_NAME"
MULTI_ROOT_ARCHIVE_PATH="$TMP_ROOT/multi-root-$ASSET_NAME"

mkdir -p \
  "$TMP_ROOT/$BUNDLE_NAME/bin" \
  "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/web" \
  "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/web-public"
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
cat > "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/web/index.html" <<'EOF'
<!DOCTYPE html>
EOF
cat > "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/web-public/index.html" <<'EOF'
<!DOCTYPE html>
EOF
tar -czf "$ARCHIVE_PATH" -C "$TMP_ROOT" "$BUNDLE_NAME"

BROKEN_BUNDLE_NAME="${BUNDLE_NAME}-broken"
mkdir -p \
  "$TMP_ROOT/$BROKEN_BUNDLE_NAME/bin" \
  "$TMP_ROOT/$BROKEN_BUNDLE_NAME/share/honeclaw/web"
cp "$TMP_ROOT/$BUNDLE_NAME/bin/hone-cli" "$TMP_ROOT/$BROKEN_BUNDLE_NAME/bin/hone-cli"
cp "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/config.example.yaml" "$TMP_ROOT/$BROKEN_BUNDLE_NAME/share/honeclaw/config.example.yaml"
cp "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/soul.md" "$TMP_ROOT/$BROKEN_BUNDLE_NAME/share/honeclaw/soul.md"
cp "$TMP_ROOT/$BUNDLE_NAME/share/honeclaw/web/index.html" "$TMP_ROOT/$BROKEN_BUNDLE_NAME/share/honeclaw/web/index.html"
tar -czf "$INCOMPLETE_ARCHIVE_PATH" -C "$TMP_ROOT" "$BROKEN_BUNDLE_NAME"

HONE_UNSAFE_ARCHIVE_PATH="$UNSAFE_ARCHIVE_PATH" HONE_UNSAFE_BUNDLE_NAME="$BUNDLE_NAME" python3 - <<'PY'
import io
import os
import tarfile
from pathlib import Path

archive_path = Path(os.environ["HONE_UNSAFE_ARCHIVE_PATH"])
bundle_name = os.environ["HONE_UNSAFE_BUNDLE_NAME"]
with tarfile.open(archive_path, "w:gz") as archive:
    safe = tarfile.TarInfo(f"{bundle_name}/bin/hone-cli")
    safe.mode = 0o755
    safe_payload = b"#!/usr/bin/env bash\n"
    safe.size = len(safe_payload)
    archive.addfile(safe, fileobj=io.BytesIO(safe_payload))

    unsafe = tarfile.TarInfo("../outside-honeclaw-install")
    unsafe_payload = b"unsafe\n"
    unsafe.size = len(unsafe_payload)
    archive.addfile(unsafe, fileobj=io.BytesIO(unsafe_payload))
PY

HONE_MULTI_ROOT_ARCHIVE_PATH="$MULTI_ROOT_ARCHIVE_PATH" HONE_MULTI_ROOT_BUNDLE_NAME="$BUNDLE_NAME" python3 - <<'PY'
import io
import os
import tarfile
from pathlib import Path

archive_path = Path(os.environ["HONE_MULTI_ROOT_ARCHIVE_PATH"])
bundle_name = os.environ["HONE_MULTI_ROOT_BUNDLE_NAME"]
with tarfile.open(archive_path, "w:gz") as archive:
    first = tarfile.TarInfo(f"{bundle_name}/bin/hone-cli")
    first_payload = b"#!/usr/bin/env bash\n"
    first.size = len(first_payload)
    archive.addfile(first, fileobj=io.BytesIO(first_payload))

    second = tarfile.TarInfo("unexpected-root/README.md")
    second_payload = b"unexpected\n"
    second.size = len(second_payload)
    archive.addfile(second, fileobj=io.BytesIO(second_payload))
PY

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

  if [[ "$output" != *'hone-cli web user-ui'* ]]; then
    echo "[FAIL] installer next steps should include the public Web UI command" >&2
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

run_explicit_bin_dir_failure_case() {
  local home_dir="$TMP_ROOT/home-bad-bin"
  local blocked_parent="$TMP_ROOT/not-a-directory"
  touch "$blocked_parent"
  mkdir -p "$home_dir"

  local output
  if output="$(
    env \
      HOME="$home_dir" \
      PATH="$TOOLS_DIR:/usr/bin:/bin" \
      HONE_BIN_DIR="$blocked_parent/hone-bin" \
      HONE_RUN_ONBOARD=0 \
      HONE_GITHUB_REPO="example/honeclaw" \
      MOCK_ARCHIVE_PATH="$ARCHIVE_PATH" \
      bash "$INSTALL_SCRIPT" 2>&1
  )"; then
    echo "[FAIL] installer succeeded despite an uncreatable HONE_BIN_DIR" >&2
    exit 1
  fi

  if [[ "$output" != *"failed to create wrapper bin dir: $blocked_parent/hone-bin"* ]]; then
    echo "[FAIL] installer did not explain the uncreatable HONE_BIN_DIR" >&2
    exit 1
  fi

  if [[ "$output" != *"set HONE_BIN_DIR to a writable directory"* ]]; then
    echo "[FAIL] installer did not provide HONE_BIN_DIR recovery guidance" >&2
    exit 1
  fi
}

run_missing_bundle_path_case() {
  local home_dir="$TMP_ROOT/home-incomplete-bundle"
  mkdir -p "$home_dir"

  local output
  if output="$(
    env \
      HOME="$home_dir" \
      PATH="$TOOLS_DIR:/usr/bin:/bin" \
      HONE_RUN_ONBOARD=0 \
      HONE_GITHUB_REPO="example/honeclaw" \
      MOCK_ARCHIVE_PATH="$INCOMPLETE_ARCHIVE_PATH" \
      bash "$INSTALL_SCRIPT" 2>&1
  )"; then
    echo "[FAIL] installer accepted an incomplete release bundle" >&2
    exit 1
  fi

  if [[ "$output" != *"release asset is missing required bundle path: share/honeclaw/web-public/index.html"* ]]; then
    echo "[FAIL] installer did not explain the missing public Web bundle path" >&2
    exit 1
  fi
}

run_unsafe_archive_layout_case() {
  local home_dir="$TMP_ROOT/home-unsafe-bundle"
  mkdir -p "$home_dir"

  local output
  if output="$(
    env \
      HOME="$home_dir" \
      PATH="$TOOLS_DIR:/usr/bin:/bin" \
      HONE_RUN_ONBOARD=0 \
      HONE_GITHUB_REPO="example/honeclaw" \
      MOCK_ARCHIVE_PATH="$UNSAFE_ARCHIVE_PATH" \
      bash "$INSTALL_SCRIPT" 2>&1
  )"; then
    echo "[FAIL] installer accepted an archive with unsafe paths" >&2
    exit 1
  fi

  if [[ "$output" != *"release asset contains unsafe archive path: ../outside-honeclaw-install"* ]]; then
    echo "[FAIL] installer did not explain the unsafe archive path" >&2
    echo "$output" >&2
    exit 1
  fi
}

run_multi_root_archive_layout_case() {
  local home_dir="$TMP_ROOT/home-multi-root-bundle"
  mkdir -p "$home_dir"

  local output
  if output="$(
    env \
      HOME="$home_dir" \
      PATH="$TOOLS_DIR:/usr/bin:/bin" \
      HONE_RUN_ONBOARD=0 \
      HONE_GITHUB_REPO="example/honeclaw" \
      MOCK_ARCHIVE_PATH="$MULTI_ROOT_ARCHIVE_PATH" \
      bash "$INSTALL_SCRIPT" 2>&1
  )"; then
    echo "[FAIL] installer accepted an archive with multiple top-level roots" >&2
    exit 1
  fi

  if [[ "$output" != *"release asset must contain exactly one top-level bundle directory"* ]]; then
    echo "[FAIL] installer did not explain the multi-root archive layout" >&2
    echo "$output" >&2
    exit 1
  fi
}

run_missing_current_binary_case() {
  local home_dir="$TMP_ROOT/home-missing-current-binary"
  local path_bin="$home_dir/custom-bin"
  mkdir -p "$path_bin"

  env \
    HOME="$home_dir" \
    PATH="$TOOLS_DIR:$path_bin:/usr/bin:/bin" \
    HONE_RUN_ONBOARD=0 \
    HONE_GITHUB_REPO="example/honeclaw" \
    MOCK_ARCHIVE_PATH="$ARCHIVE_PATH" \
    bash "$INSTALL_SCRIPT" >/dev/null

  rm "$home_dir/.honeclaw/current/bin/hone-cli"

  local output
  if output="$(
    env HOME="$home_dir" PATH="$path_bin:$TOOLS_DIR:/usr/bin:/bin" hone-cli smoke 2>&1
  )"; then
    echo "[FAIL] installed wrapper succeeded despite a missing current hone-cli binary" >&2
    exit 1
  fi

  if [[ "$output" != *"installed Hone CLI binary is missing: $home_dir/.honeclaw/current/bin/hone-cli"* ]]; then
    echo "[FAIL] installed wrapper did not explain the missing current hone-cli binary" >&2
    echo "$output" >&2
    exit 1
  fi

  if [[ "$output" != *"rerun the Hone installer"* ]]; then
    echo "[FAIL] installed wrapper did not provide reinstall recovery guidance" >&2
    echo "$output" >&2
    exit 1
  fi
}

run_path_hit_case

run_fallback_case "/bin/zsh" "$TMP_ROOT/home-fallback-zsh" "$TMP_ROOT/home-fallback-zsh/.zshrc"
mkdir -p "$TMP_ROOT/home-fallback-bash"
touch "$TMP_ROOT/home-fallback-bash/.bashrc"
run_fallback_case "/bin/bash" "$TMP_ROOT/home-fallback-bash" "$TMP_ROOT/home-fallback-bash/.bashrc"
run_explicit_bin_dir_failure_case
run_missing_bundle_path_case
run_unsafe_archive_layout_case
run_multi_root_archive_layout_case
run_missing_current_binary_case

echo "[PASS] hone-cli install path resolution regression passed"
