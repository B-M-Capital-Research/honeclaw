#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/hone-fmt-regression.XXXXXX")"
trap 'rm -rf -- "$scratch"' EXIT
cd "$scratch"
git -c init.defaultBranch=main init -q
git config user.name 'HONE Regression'
git config user.email 'regression@example.invalid'
git config core.hooksPath /dev/null
printf 'mod child;\n' > lib.rs
printf 'fn untouched( ){ }\n' > child.rs
git add lib.rs child.rs
git commit -qm 'fixture base'
base="$(git rev-parse HEAD)"
printf '// Changed parent; historical child formatting is out of scope.\nmod child;\n' > lib.rs
git add lib.rs
git commit -qm 'change parent'
GITHUB_EVENT_NAME=push GITHUB_EVENT_BEFORE="$base" bash "$repo_root/scripts/ci/check_fmt_changed.sh"
echo 'changed-file rustfmt does not traverse unchanged child modules'
