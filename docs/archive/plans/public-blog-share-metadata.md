# Public Blog Share Metadata

- title: Public Blog Share Metadata
- status: done
- created_at: 2026-05-21
- updated_at: 2026-05-21
- owner: Codex
- related_files:
  - `packages/app/src/pages/public-blog-post.tsx`
  - `packages/app/src/lib/public-blog.ts`
  - `packages/app/public/_worker.js`
  - `packages/app/src/pages/public-site.css`
  - `README.md`
  - `README_EN.md`
  - `README_ZH.md`
- related_docs:
  - `docs/archive/plans/public-blog-module.md`
  - `docs/repo-map.md`

## Goal

Improve Blog article readability and share previews by showing both Chinese and English titles on article pages, adding a cross-language navigation card, and serving article-specific title/description/OG/Twitter metadata to social crawlers.

## Scope

- Add bilingual title fields and alternate-language lookup helpers to Blog content metadata.
- Add article-page cross-language card.
- Add runtime meta tags and Cloudflare Worker HTML metadata injection for `/blog/why-hone-uses-rust`.
- Add README top navigation Blog links and Rust-stack Blog references in the matching README language.

## Validation

- `bun --filter @hone-financial/app test` — passed
- `bun --filter @hone-financial/app typecheck` — passed
- `HONE_APP_OUT_DIR=dist-public HONE_APP_SURFACE=public bun --filter @hone-financial/app build` — passed

## Documentation Sync

- Archived this plan to `docs/archive/plans/public-blog-share-metadata.md`.
- Updated `docs/archive/index.md` and the existing Blog handoff with the metadata crawler note.

## Risks / Open Questions

- Social platforms cache previews aggressively; after deployment, cached old previews may require platform-specific rescrape/debug tools.
