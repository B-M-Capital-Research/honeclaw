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
  - `docs/archive/plans/company-profile-optional-frontmatter.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/archive/index.md`
- related_prs:
  - N/A

## Summary

公司画像与事件现在都把 frontmatter 当成可选增强信息，而不是硬前置条件。缺少 frontmatter 的本地 `profile.md` / `events/*.md` 可以继续被读取、列出、匹配和展示；缺少 frontmatter 的画像包也可以继续 preview / import，不会再因为 `缺少 frontmatter` 直接失败。

## What Changed

- `memory/src/company_profile/markdown.rs` 新增宽松解析 helper：有 frontmatter 时继续读取结构化 metadata；没有时从标题、文件名、mtime 和 bundle manifest 的 `updated_at` 推断最小 metadata
- `memory/src/company_profile/storage.rs` 的 `get_profile`、`list_profiles`、`find_profile_id`、事件读取与“已有事件直接返回”路径都改成宽松解析
- `memory/src/company_profile/transfer.rs` 的 bundle 解析改成宽松模式，plain Markdown bundle 现在也能 preview / apply
- `memory/src/company_profile/tests.rs` 新增 plain Markdown 本地画像读取和 plain Markdown bundle preview 回归

## Verification

- `cargo test -p hone-memory company_profile -- --nocapture`

## Risks / Follow-ups

- 这次没有移除新写入时生成的 frontmatter；只是保证没有它也能读/导/比对

## Next Entry Point

- `memory/src/company_profile/markdown.rs`
- `memory/src/company_profile/storage.rs`
- `memory/src/company_profile/transfer.rs`
