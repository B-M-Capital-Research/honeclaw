# Open Source Copy Prep Checklist

Last updated: 2026-03-22
Status: Internal reference

> This file is a checklist for copying the current private staging repo into a standalone public repo.
> The public repo does not have to keep this file itself. If you do keep it, re-check the paths, branding, and allowlist first.

## Goal

- Clarify which files should go into the public repository
- Clarify which internal collaboration assets, runtime data, and local build outputs should be excluded
- Provide a repeatable allowlist / denylist reference for the copy process

## Recommended to Keep

- `README.md`
- `LICENSE`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `CODE_OF_CONDUCT.md`
- `Cargo.toml`
- `Cargo.lock`
- `package.json`
- `bun.lock`
- `.editorconfig`
- `.gitattributes`
- `config.example.yaml`
- `gitleaks.toml`, `.gitleaksignore`, and the matching secret-scan workflow (the current allowlist covers `config.yaml`, while `.gitleaksignore` pins the historical sample findings in `scripts/diagnose_llm.sh`, `docs/technical-spec.md`, and the deleted `tests/test_x_oauth1.py`)
- `crates/`
- `bins/`
- `agents/`
- `memory/`
- `packages/`
- `src-tauri/`
- `scripts/`
- `tests/`
- `skills/`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/adr/`
- `docs/runbooks/`
- `docs/technical-spec.md`
- `docs/landing.html`
- `docs/architecture.html`
- `soul.md`

## Recommended to Exclude

- `config.yaml`
- `data/`
- `dist/`
- `target/`
- `node_modules/`
- `src-tauri/binaries/`
- `*.log`
- `*.pid`
- `*.sqlite*`
- `*.db`
- `.env`
- `.env.*`
- `cookies.json`
- `AGENTS.md`
- `GEMINI.md`
- `docs/current-plan.md`
- `docs/current-plans/`
- `docs/handoffs/`

## Recommended Copy Steps

1. Create a clean destination directory for the public repository.
2. Copy only the items listed in "Recommended to Keep" above.
3. After copying, manually remove or replace any defaults that still point to internal infrastructure.
4. If the public repo should not keep `config.yaml`, do not add it to version control.
5. If the public repo should keep `config.yaml` as a local template, make sure it contains no real credentials before keeping the secret-scan rules.

## Post-Copy Checks

- Run `git status --short`
- Run a secret scan
- Search for internal domains, absolute paths, real tokens, real email addresses, and phone numbers
- Confirm the release bundle only contains files needed by the public repository
