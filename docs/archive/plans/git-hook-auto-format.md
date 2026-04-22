# Git Hook Auto Format

- title: Git Hook Auto Format
- status: done
- created_at: 2026-04-22
- updated_at: 2026-04-22
- owner: Codex
- related_files:
  - `.githooks/pre-commit`
  - `.githooks/pre-push`
  - `scripts/install_gitleaks.sh`
  - `AGENTS.md`
- related_docs:
  - `docs/archive/index.md`

## Goal

Reduce push-time rustfmt failures by formatting staged Rust files during commit, while keeping pre-push as the final gate for already-created commits.

## Scope

- Add a local `pre-commit` hook that runs `rustfmt` on staged Rust files and restages the formatted files.
- Avoid auto-formatting partially staged Rust files, because that could accidentally include unstaged user edits in the commit.
- Keep the existing `pre-push` rustfmt and gitleaks checks as a safety net.
- Update hook installation output and repository collaboration rules.

## Validation

- Validate the hook formats and restages a deliberately unformatted staged Rust file.
- Validate no worktree test changes remain after the hook test.
- Validate shell syntax for the new hook and installer script.

## Documentation Sync

- Update `AGENTS.md` to describe commit-time auto-formatting and the remaining push-time fallback.
- Add this archived plan and index it from `docs/archive/index.md`.

## Risks / Open Questions

- Partially staged Rust files still require manual staging or stashing before commit; this is intentional to avoid committing unintended edits.
