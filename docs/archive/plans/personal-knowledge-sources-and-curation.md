# Personal Knowledge Sources and Curation

- title: HONE 个人外部知识源与社区投稿采纳
- status: archived
- created_at: 2026-08-11
- updated_at: 2026-08-11
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/research_library.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `packages/app/src/pages/public-research-library.tsx`
  - `packages/app/src/pages/public-me.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/repo-map.md`
  - `docs/decisions.md#d-2026-08-11-12-promote-external-research-through-an-explicit-trust-ladder`
  - `docs/handoffs/2026-08-11-personal-knowledge-sources-and-curation.md`

## Goal

把现有统一研究资料库收口为“我的知识源”，让每个用户安全导入自己的知识星球或 iMA 材料，同时建立社区投稿、管理员核验、官方资料采纳的单向信任升级流程。

## Delivered Scope

- `/me` 入口和“我的知识源”页面。
- 结构化知识星球/iMA 只读导入状态与真实能力披露。
- `personal → community_candidate → hone_global` 三域隔离、投稿、审核、驳回与审批复制。
- 候选资料在审批前不进入任何检索或每日产品。
- 自动化契约、完整 Web 验证、生产构建和真实浏览器验收。

## Validation

- Web API 278 passed / 2 ignored; final focused research-library 3/3.
- Web 445/445; typecheck and public build passed.
- CI-safe curation contract passed.
- Desktop/mobile authenticated browser acceptance passed.

## Documentation Sync

- `docs/repo-map.md`, `docs/decisions.md`, handoff and archive index updated.

## Risks / Open Questions

- Production storage migration and official multi-tenant OAuth remain prerequisites for automatic sync.
- Forum UGC remains a separate future slice.
