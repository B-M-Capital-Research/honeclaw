#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.dev.yml"
COMPOSE=(docker compose --project-directory "$PROJECT_ROOT" -f "$COMPOSE_FILE")

usage() {
  cat <<'EOF'
Usage: bash scripts/dev_pg.sh <command> [args]

Commands:
  up              Start local PostgreSQL and wait until it is healthy
  down            Stop local PostgreSQL without deleting its data volume
  reset           Delete the data volume after two confirmations, then restart
  psql [args...]  Open psql in the running PostgreSQL container
EOF
}

fail() {
  printf '[dev-pg] error: %s\n' "$*" >&2
  exit 1
}

require_docker() {
  command -v docker >/dev/null 2>&1 || fail "docker is not installed or not in PATH"
  docker info >/dev/null 2>&1 || fail "Docker is not running or is not accessible"
}

reset_postgres() {
  [[ -t 0 ]] || fail "reset requires an interactive terminal"

  printf '%s\n' \
    'This permanently deletes the local PostgreSQL development volume and all data in it.'
  read -r -p 'Continue? [y/N] ' first_confirmation
  case "$first_confirmation" in
    y | Y | yes | YES) ;;
    *)
      printf '%s\n' '[dev-pg] reset cancelled'
      return 0
      ;;
  esac

  read -r -p 'Type RESET to confirm deletion: ' second_confirmation
  if [[ "$second_confirmation" != "RESET" ]]; then
    printf '%s\n' '[dev-pg] reset cancelled'
    return 0
  fi

  "${COMPOSE[@]}" down --volumes --remove-orphans
  "${COMPOSE[@]}" up -d --wait
}

COMMAND="${1:-}"
if [[ $# -gt 0 ]]; then
  shift
fi

case "$COMMAND" in
  up)
    require_docker
    "${COMPOSE[@]}" up -d --wait
    ;;
  down)
    require_docker
    "${COMPOSE[@]}" down --remove-orphans
    ;;
  reset)
    require_docker
    reset_postgres
    ;;
  psql)
    require_docker
    exec "${COMPOSE[@]}" exec postgres sh -c \
      'exec psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" "$@"' sh "$@"
    ;;
  -h | --help | help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
