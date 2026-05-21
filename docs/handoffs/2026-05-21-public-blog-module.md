# Public Blog Module

- title: Public Blog Module
- status: done
- created_at: 2026-05-21
- updated_at: 2026-05-21
- owner: Codex
- related_files:
  - `packages/app/src/lib/public-blog.ts`
  - `packages/app/src/pages/public-blog.tsx`
  - `packages/app/src/pages/public-blog-post.tsx`
  - `packages/app/src/content/blog/`
  - `packages/app/public/blog/`
- related_docs:
  - `docs/archive/plans/public-blog-module.md`
  - `docs/repo-map.md`
- related_prs: N/A

## Summary

Added a static bilingual Blog module to the public Hone site, including `/blog`, `/blog/:slug`, navigation and homepage entry points, and the first Rust article in Chinese and English.

## What Changed

- Migrated the provided Chinese Markdown article into tracked Blog content and added a faithful English version.
- Downloaded the first Chinese and English article images from the provided Feishu links into `packages/app/public/blog/` so the site is not dependent on expiring external image URLs.
- Reused the existing Markdown renderer and public locale state so language switching updates the Blog index and article content.

## Verification

- `bun --filter @hone-financial/app test` passed.
- `bun --filter @hone-financial/app typecheck` passed after `bun install` restored missing local `qrcode` dependency symlinks; no lockfile changes were produced.
- `HONE_APP_OUT_DIR=dist-public HONE_APP_SURFACE=public bun --filter @hone-financial/app build` passed.

## Risks / Follow-ups

- Only the first article image is localized and stored locally. Other images from the source article were intentionally not imported because only the first bilingual image pair was required.
- Future Blog posts should be added through `packages/app/src/lib/public-blog.ts` and colocated Markdown files, keeping slug parity between `zh` and `en`.

## Next Entry Point

Start with `packages/app/src/lib/public-blog.ts` for content metadata, then `packages/app/src/pages/public-blog.tsx` and `packages/app/src/pages/public-blog-post.tsx` for layout behavior.
