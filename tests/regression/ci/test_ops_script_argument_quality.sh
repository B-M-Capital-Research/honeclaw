#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hone-ops-script-quality.XXXXXX")"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"

  if [[ "$haystack" != *"$needle"* ]]; then
    echo "[FAIL] $label" >&2
    echo "expected snippet: $needle" >&2
    echo "actual output:" >&2
    printf '%s\n' "$haystack" >&2
    exit 1
  fi
}

run_missing_homebrew_formula_value_case() {
  local output
  if output="$(bash "$ROOT_DIR/scripts/update_homebrew_formula.sh" --version 2>&1)"; then
    echo "[FAIL] update_homebrew_formula accepted --version without a value" >&2
    exit 1
  fi

  assert_contains "$output" "missing value for --version" "missing --version value should be explicit"
  assert_contains "$output" "scripts/update_homebrew_formula.sh" "usage should name the script"
}

run_invalid_homebrew_formula_sha_case() {
  local output
  if output="$(
    bash "$ROOT_DIR/scripts/update_homebrew_formula.sh" \
      --version 1.2.3 \
      --darwin-aarch64-sha not-a-sha \
      --darwin-x86_64-sha "$(printf 'b%.0s' {1..64})" \
      --linux-x86_64-sha "$(printf 'c%.0s' {1..64})" \
      2>&1
  )"; then
    echo "[FAIL] update_homebrew_formula accepted an invalid sha256" >&2
    exit 1
  fi

  assert_contains "$output" "invalid sha256 for --darwin-aarch64-sha" "invalid checksum should name the bad flag"
  assert_contains "$output" "scripts/update_homebrew_formula.sh" "invalid checksum should include usage"
}

run_homebrew_formula_generation_case() {
  local output_path="$TMP_ROOT/honeclaw.rb"

  bash "$ROOT_DIR/scripts/update_homebrew_formula.sh" \
    --version 1.2.3 \
    --darwin-aarch64-sha "$(printf 'a%.0s' {1..64})" \
    --darwin-x86_64-sha "$(printf 'b%.0s' {1..64})" \
    --linux-x86_64-sha "$(printf 'c%.0s' {1..64})" \
    --output "$output_path"

  if [[ ! -s "$output_path" ]]; then
    echo "[FAIL] update_homebrew_formula did not write the output formula" >&2
    exit 1
  fi

  assert_contains "$(cat "$output_path")" 'version "1.2.3"' "formula should include requested version"
  assert_contains "$(cat "$output_path")" "hone-cli web user-ui" "formula caveats should include the public Web UI command"
  assert_contains "$(cat "$output_path")" 'mkdir -p "$HONE_HOME"' "formula wrapper should create HONE_HOME before seeding config files"
  assert_contains "$(cat "$output_path")" "installed Hone CLI binary is missing:" "formula wrapper should explain missing bundled CLI binaries"
}

run_homebrew_formula_version_normalization_case() {
  local output_path="$TMP_ROOT/honeclaw-versioned.rb"

  bash "$ROOT_DIR/scripts/update_homebrew_formula.sh" \
    --version v1.2.3 \
    --darwin-aarch64-sha "$(printf 'd%.0s' {1..64})" \
    --darwin-x86_64-sha "$(printf 'e%.0s' {1..64})" \
    --linux-x86_64-sha "$(printf 'f%.0s' {1..64})" \
    --output "$output_path"

  assert_contains "$(cat "$output_path")" 'version "1.2.3"' "formula should normalize a leading v in --version"
  assert_contains "$(cat "$output_path")" "/releases/download/v1.2.3/honeclaw-darwin-aarch64.tar.gz" "formula URL should not duplicate the v prefix"
}

run_changed_fmt_bash3_compatible_case() {
  local repo_dir="$TMP_ROOT/fmt-repo"
  local tools_dir="$TMP_ROOT/tools"
  local rustfmt_log="$TMP_ROOT/rustfmt-args.log"

  mkdir -p "$repo_dir" "$tools_dir"
  cat > "$tools_dir/rustfmt" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" > "$HONE_TEST_RUSTFMT_LOG"
EOF
  chmod +x "$tools_dir/rustfmt"

  (
    cd "$repo_dir"
    git init -q
    git config user.email "patrol@example.invalid"
    git config user.name "Code Patrol"
    printf '# fixture\n' > README.md
    git add README.md
    git commit -q -m initial
    mkdir -p src
    printf 'fn main() {}\n' > src/main.rs
    git add src/main.rs
    git commit -q -m add-rust-file
    printf 'fn main() { println!("hi"); }\n' > src/main.rs

    env \
      PATH="$tools_dir:$PATH" \
      HONE_TEST_RUSTFMT_LOG="$rustfmt_log" \
      bash "$ROOT_DIR/scripts/ci/check_fmt_changed.sh"
  )

  if [[ ! -s "$rustfmt_log" ]]; then
    echo "[FAIL] check_fmt_changed did not invoke rustfmt for changed Rust files" >&2
    exit 1
  fi

  assert_contains "$(cat "$rustfmt_log")" "src/main.rs" "rustfmt args should include changed Rust file"
}

run_script_self_path_quality_case() {
  local matches=""
  local script

  while IFS= read -r script; do
    while IFS= read -r match; do
      matches="${matches}${script}:${match}"$'\n'
    done < <(grep -n 'dirname "[$]0"' "$script" || true)
    while IFS= read -r match; do
      matches="${matches}${script}:${match}"$'\n'
    done < <(grep -n 'usage: [$]0' "$script" || true)
  done < <(find "$ROOT_DIR/scripts" "$ROOT_DIR/tests/regression" -type f -name '*.sh' | sort)

  if [[ -n "$matches" ]]; then
    echo "[FAIL] scripts should use BASH_SOURCE[0] or stable usage names instead of \$0" >&2
    printf '%s' "$matches" >&2
    exit 1
  fi
}

run_gitleaks_archive_layout_case() {
  local repo_dir="$TMP_ROOT/gitleaks-repo"
  local tools_dir="$TMP_ROOT/gitleaks-tools"
  local fixture_dir="$TMP_ROOT/gitleaks-fixture"
  local archive_path="$TMP_ROOT/gitleaks_0.0.0_darwin_arm64.tar.gz"

  mkdir -p "$repo_dir" "$tools_dir" "$fixture_dir"
  printf 'not the binary\n' > "$fixture_dir/README.md"
  tar -czf "$archive_path" -C "$fixture_dir" README.md

  cat > "$tools_dir/curl" <<'EOF'
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

cp "$HONE_TEST_GITLEAKS_ARCHIVE" "$output"
EOF
  chmod +x "$tools_dir/curl"

  (
    cd "$repo_dir"
    git init -q
    git config user.email "patrol@example.invalid"
    git config user.name "Code Patrol"

    local output
    if output="$(
      env \
        PATH="$tools_dir:/usr/bin:/bin" \
        GITLEAKS_VERSION=0.0.0 \
        HONE_TEST_GITLEAKS_ARCHIVE="$archive_path" \
        bash "$ROOT_DIR/scripts/install_gitleaks.sh" 2>&1
    )"; then
      echo "[FAIL] install_gitleaks accepted an archive without a gitleaks binary" >&2
      exit 1
    fi

    assert_contains "$output" "downloaded gitleaks archive did not contain expected binary: gitleaks" "gitleaks archive layout failure should be explicit"
    assert_contains "$output" "archive: https://github.com/gitleaks/gitleaks/releases/download/v0.0.0/gitleaks_0.0.0_" "gitleaks archive layout failure should include the source archive"
  )
}

run_gitleaks_unsafe_archive_path_case() {
  local repo_dir="$TMP_ROOT/gitleaks-unsafe-repo"
  local tools_dir="$TMP_ROOT/gitleaks-unsafe-tools"
  local archive_path="$TMP_ROOT/gitleaks_0.0.1_darwin_arm64.tar.gz"

  mkdir -p "$repo_dir" "$tools_dir"
  HONE_TEST_UNSAFE_GITLEAKS_ARCHIVE="$archive_path" python3 - <<'PY'
import io
import os
import tarfile
from pathlib import Path

archive_path = Path(os.environ["HONE_TEST_UNSAFE_GITLEAKS_ARCHIVE"])
with tarfile.open(archive_path, "w:gz") as archive:
    safe = tarfile.TarInfo("gitleaks")
    safe.mode = 0o755
    safe_payload = b"#!/usr/bin/env bash\n"
    safe.size = len(safe_payload)
    archive.addfile(safe, fileobj=io.BytesIO(safe_payload))

    unsafe = tarfile.TarInfo("../outside-gitleaks-install")
    unsafe_payload = b"unsafe\n"
    unsafe.size = len(unsafe_payload)
    archive.addfile(unsafe, fileobj=io.BytesIO(unsafe_payload))

    trailing_parent = tarfile.TarInfo("nested/..")
    trailing_parent_payload = b"unsafe\n"
    trailing_parent.size = len(trailing_parent_payload)
    archive.addfile(trailing_parent, fileobj=io.BytesIO(trailing_parent_payload))
PY

  cat > "$tools_dir/curl" <<'EOF'
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

cp "$HONE_TEST_GITLEAKS_ARCHIVE" "$output"
EOF
  chmod +x "$tools_dir/curl"

  (
    cd "$repo_dir"
    git init -q
    git config user.email "patrol@example.invalid"
    git config user.name "Code Patrol"

    local output
    if output="$(
      env \
        PATH="$tools_dir:/usr/bin:/bin" \
        GITLEAKS_VERSION=0.0.1 \
        HONE_TEST_GITLEAKS_ARCHIVE="$archive_path" \
        bash "$ROOT_DIR/scripts/install_gitleaks.sh" 2>&1
    )"; then
      echo "[FAIL] install_gitleaks accepted an archive with unsafe paths" >&2
      exit 1
    fi

    assert_contains "$output" "gitleaks archive contains unsafe path: ../outside-gitleaks-install" "gitleaks installer should reject unsafe archive paths"
    assert_contains "$output" "archive: https://github.com/gitleaks/gitleaks/releases/download/v0.0.1/gitleaks_0.0.1_" "gitleaks unsafe-path failure should include the source archive"
  )
}

run_gitleaks_existing_symlink_case() {
  local repo_dir="$TMP_ROOT/gitleaks-symlink-repo"
  local tools_dir="$TMP_ROOT/gitleaks-symlink-tools"
  local fixture_dir="$TMP_ROOT/gitleaks-symlink-fixture"
  local archive_path="$TMP_ROOT/gitleaks_0.0.2_darwin_arm64.tar.gz"

  mkdir -p "$repo_dir/.git-tools/gitleaks/0.0.2" "$repo_dir/.githooks" "$tools_dir" "$fixture_dir"
  touch "$repo_dir/.githooks/pre-commit" "$repo_dir/.githooks/pre-push"
  ln -s /bin/sh "$repo_dir/.git-tools/gitleaks/0.0.2/gitleaks"
  cat > "$fixture_dir/gitleaks" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "mock gitleaks"
EOF
  chmod +x "$fixture_dir/gitleaks"
  tar -czf "$archive_path" -C "$fixture_dir" gitleaks

  cat > "$tools_dir/curl" <<'EOF'
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

cp "$HONE_TEST_GITLEAKS_ARCHIVE" "$output"
EOF
  chmod +x "$tools_dir/curl"

  (
    cd "$repo_dir"
    git init -q
    git config user.email "patrol@example.invalid"
    git config user.name "Code Patrol"

    env \
      PATH="$tools_dir:/usr/bin:/bin" \
      GITLEAKS_VERSION=0.0.2 \
      HONE_TEST_GITLEAKS_ARCHIVE="$archive_path" \
      bash "$ROOT_DIR/scripts/install_gitleaks.sh" >/dev/null

    if [[ -L "$repo_dir/.git-tools/gitleaks/0.0.2/gitleaks" ]]; then
      echo "[FAIL] install_gitleaks kept an existing symlinked binary" >&2
      exit 1
    fi
    if [[ "$("$repo_dir/.git-tools/gitleaks/0.0.2/gitleaks")" != "mock gitleaks" ]]; then
      echo "[FAIL] install_gitleaks did not replace the symlink with the downloaded binary" >&2
      exit 1
    fi
  )
}

run_gitleaks_outside_repo_case() {
  local outside_dir="$TMP_ROOT/outside-git-repo"
  mkdir -p "$outside_dir"

  local output
  if output="$(
    cd "$outside_dir"
    bash "$ROOT_DIR/scripts/install_gitleaks.sh" 2>&1
  )"; then
    echo "[FAIL] install_gitleaks succeeded outside a git checkout" >&2
    exit 1
  fi

  assert_contains "$output" "install_gitleaks.sh must be run from inside a git checkout" "gitleaks installer should explain the git checkout requirement"
}

run_build_desktop_home_bun_case() {
  local home_dir="$TMP_ROOT/build-desktop-home"
  local bun_log="$TMP_ROOT/build-desktop-bun.log"
  local bunx_log="$TMP_ROOT/build-desktop-bunx.log"

  mkdir -p "$home_dir/.bun/bin"
  cat > "$home_dir/.bun/bin/bun" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$HONE_TEST_BUILD_DESKTOP_BUN_LOG"
EOF
  cat > "$home_dir/.bun/bin/bunx" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$HONE_TEST_BUILD_DESKTOP_BUNX_LOG"
EOF
  chmod +x "$home_dir/.bun/bin/bun" "$home_dir/.bun/bin/bunx"

  env \
    HOME="$home_dir" \
    PATH="/usr/bin:/bin" \
    HONE_TEST_BUILD_DESKTOP_BUN_LOG="$bun_log" \
    HONE_TEST_BUILD_DESKTOP_BUNX_LOG="$bunx_log" \
    bash "$ROOT_DIR/scripts/build_desktop.sh" >/dev/null

  assert_contains "$(cat "$bun_log")" "install" "build_desktop should run bun install through the home Bun fallback"
  assert_contains "$(cat "$bun_log")" "scripts/prepare_tauri_sidecar.mjs release" "build_desktop should prepare release sidecars"
  assert_contains "$(cat "$bunx_log")" "tauri build --config bins/hone-desktop/tauri.generated.conf.json" "build_desktop should invoke the Tauri build command"
}

run_missing_homebrew_formula_value_case
run_invalid_homebrew_formula_sha_case
run_homebrew_formula_generation_case
run_homebrew_formula_version_normalization_case
run_changed_fmt_bash3_compatible_case
run_script_self_path_quality_case
run_gitleaks_archive_layout_case
run_gitleaks_unsafe_archive_path_case
run_gitleaks_existing_symlink_case
run_gitleaks_outside_repo_case
run_build_desktop_home_bun_case

echo "[PASS] ops script argument quality regression passed"
