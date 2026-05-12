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

run_missing_homebrew_formula_value_case
run_homebrew_formula_generation_case
run_changed_fmt_bash3_compatible_case
run_script_self_path_quality_case

echo "[PASS] ops script argument quality regression passed"
