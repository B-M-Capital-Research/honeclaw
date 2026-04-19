# Company Profile Optional Frontmatter

- title: Company Profile Optional Frontmatter
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: codex
- related_files:
  - `memory/src/company_profile/markdown.rs`
  - `memory/src/company_profile/storage.rs`
  - `memory/src/company_profile/transfer.rs`
  - `memory/src/company_profile/tests.rs`
- related_docs:
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/archive/index.md`

## Goal

Remove the remaining hard requirement that company profile Markdown and event Markdown must contain YAML frontmatter in order to be read, previewed, imported, or compared.

## Scope

- Relax company profile and event parsing in storage reads
- Relax bundle preview/import parsing for plain Markdown profile bundles
- Add regression coverage for plain Markdown local docs and plain Markdown transfer bundles

## Validation

- `rtk cargo test -p hone-memory company_profile -- --nocapture`
- `rtk cargo test -p hone-tools company_profile_transfer -- --nocapture`
- Bare Codex ACP + Hone MCP probe: `company_profile_transfer action=preview` succeeds on a plain-Markdown bundle that previously failed with `缺少 frontmatter`

## Documentation Sync

- Update `docs/repo-map.md` to record that company profile reads and transfer paths both tolerate plain Markdown without frontmatter
- Update `docs/invariants.md` to lock in optional frontmatter semantics for company portraits
- Add archive and handoff entries for follow-up discoverability

## Risks / Open Questions

- New writes still render structured frontmatter; this change only removes the hard read/import requirement
- `hone-cli probe + /company_profile_transfer` still has a separate Codex ACP tool-discovery issue and may claim the tool is not exposed even though bare ACP + Hone MCP execution works
