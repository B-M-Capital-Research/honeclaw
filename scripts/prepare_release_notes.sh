#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <tag> [output-path]" >&2
  exit 1
fi

TAG="$1"
OUTPUT_PATH="${2:-}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NOTES_TEMPLATE="$ROOT_DIR/docs/releases/${TAG}.md"

if [[ ! -f "$NOTES_TEMPLATE" ]]; then
  echo "missing release notes file: docs/releases/${TAG}.md" >&2
  echo "create it from docs/templates/release-notes.md and focus on user-facing changes." >&2
  exit 1
fi

PREVIOUS_TAG="$(
  git -C "$ROOT_DIR" tag --sort=-v:refname \
    | awk -v current="$TAG" '$0 != current { print; exit }'
)"

REPO_SLUG="${GITHUB_REPOSITORY:-B-M-Capital-Research/honeclaw}"
if [[ -n "$PREVIOUS_TAG" ]]; then
  COMPARE_URL="https://github.com/${REPO_SLUG}/compare/${PREVIOUS_TAG}...${TAG}"
else
  COMPARE_URL="https://github.com/${REPO_SLUG}/releases/tag/${TAG}"
fi

RELEASE_DATE="$(date -u '+%Y-%m-%d')"

python3 - "$NOTES_TEMPLATE" "$TAG" "$PREVIOUS_TAG" "$COMPARE_URL" "$RELEASE_DATE" <<'PY' > "${OUTPUT_PATH:-/dev/stdout}"
import sys
from pathlib import Path

template_path, tag, previous_tag, compare_url, release_date = sys.argv[1:]
content = Path(template_path).read_text()

replacements = {
    "{{TAG}}": tag,
    "{{PREVIOUS_TAG}}": previous_tag,
    "{{COMPARE_URL}}": compare_url,
    "{{RELEASE_DATE_UTC}}": release_date,
}

for needle, value in replacements.items():
    content = content.replace(needle, value)

sys.stdout.write(content)
if not content.endswith("\n"):
    sys.stdout.write("\n")
PY
