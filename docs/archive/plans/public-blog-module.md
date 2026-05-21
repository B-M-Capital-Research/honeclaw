# Public Blog Module

- title: Public Blog Module
- status: done
- created_at: 2026-05-21
- updated_at: 2026-05-21
- owner: Codex
- related_files:
  - `packages/app/src/app.tsx`
  - `packages/app/src/pages/public-blog.tsx`
  - `packages/app/src/pages/public-blog-post.tsx`
  - `packages/app/src/lib/public-blog.ts`
  - `packages/app/src/lib/public-content.ts`
  - `packages/app/public/blog/`
- related_docs:
  - `docs/repo-map.md`
  - `docs/archive/index.md`

## Goal

Add a bilingual public Blog module for hone-claw.com, including a Blog index, article detail route, homepage entry point, navigation entry, and the first Chinese/English article about why Hone uses Rust.

## Scope

- Add public routes `/blog` and `/blog/:slug`.
- Migrate the provided root Markdown article into tracked bilingual site content.
- Download and serve the article's Chinese and English first images as local public assets.
- Remove the temporary untracked source file and unrelated local untracked directories after migration.

## Validation

- `bun --filter @hone-financial/app test` — passed
- `bun --filter @hone-financial/app typecheck` — passed after running `bun install` to restore missing local `qrcode` dependency symlinks; no lockfile changes
- `HONE_APP_OUT_DIR=dist-public HONE_APP_SURFACE=public bun --filter @hone-financial/app build` — passed

## Documentation Sync

- Updated `docs/repo-map.md` for the new public routes/content module.
- Archived this plan to `docs/archive/plans/public-blog-module.md`.
- Updated `docs/archive/index.md` with the completed task entry.
- Added `docs/handoffs/2026-05-21-public-blog-module.md`.

## Risks / Open Questions

- Feishu source image links may expire; local public copies are required before deleting the source Markdown.
- The English article should preserve the original argument and structure while reading naturally for English readers.
