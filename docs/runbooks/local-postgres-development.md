# Runbook: Local Docker PostgreSQL

Last updated: 2026-08-16

Use this runbook to start the PostgreSQL 16 development database required by
Hone's cloud storage commands. It listens only on `127.0.0.1:5433`, so it does
not conflict with a PostgreSQL server using the default host port `5432`.

## Prerequisites

- Docker Desktop or another Docker engine with Compose v2 is running.
- Run commands from this repository checkout. The helper script also works
  when invoked by absolute path from another directory.

The development-only connection settings are:

```text
host=127.0.0.1
port=5433
user=honeclaw
password=honeclaw_dev
database=honeclaw
sslmode=disable
```

Do not reuse this password outside local development.

## Start PostgreSQL

```bash
bash scripts/dev_pg.sh up
```

The equivalent direct Compose command is:

```bash
docker compose -f docker-compose.dev.yml up -d
```

Compose waits on the configured `pg_isready` health check when the helper is
used. Database files persist in the named `honeclaw_pg_dev_data` volume.

## Configure Hone And Create The Schema

Export the existing `PostgresConfig` environment variables. No additional Hone
configuration fields are needed:

```bash
export HONE_POSTGRES_HOST=127.0.0.1
export HONE_POSTGRES_PORT=5433
export HONE_POSTGRES_USER=honeclaw
export HONE_POSTGRES_PASSWORD=honeclaw_dev
export HONE_POSTGRES_DATABASE=honeclaw
```

If this checkout has no local `config.yaml`, point the CLI at the tracked
development template:

```bash
export HONE_USER_CONFIG_PATH="$PWD/config.example.yaml"
```

Check PostgreSQL health and create any missing tables:

```bash
cargo run -p hone-cli -- cloud doctor --ensure-schema --json
```

On an empty database, `schema_ensured` should be `true`, and
`postgres_health.ok` should be `true`.

## Inspect The Database

Open an interactive shell:

```bash
bash scripts/dev_pg.sh psql
```

List tables from inside `psql`:

```text
\dt
```

Pass ordinary `psql` arguments through the helper for a one-shot query:

```bash
bash scripts/dev_pg.sh psql -c '\dt'
bash scripts/dev_pg.sh psql -Atc \
  "SELECT count(*) FROM pg_tables WHERE schemaname = 'public';"
```

At revision `62d0c889`, `CloudPgRuntime::ensure_schema()` creates 26 public
tables in an empty database.

## Stop Or Reset

Stop the container while preserving the named data volume:

```bash
bash scripts/dev_pg.sh down
```

Delete all local development database data and start again:

```bash
bash scripts/dev_pg.sh reset
```

`reset` only runs in an interactive terminal. It first asks whether to
continue, then requires typing `RESET` before removing the named volume. This
operation cannot recover data from that volume.
