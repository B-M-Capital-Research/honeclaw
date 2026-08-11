# Community Discussion Forum

- title: HONE 社区讨论区第一版
- status: completed
- created_at: 2026-08-11
- completed_at: 2026-08-11
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/community_forum.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
  - `packages/app/src/components/community-forum.tsx`
  - `packages/app/src/components/community-forum.css`
  - `packages/app/src/pages/public-community.tsx`
- related_docs:
  - `docs/decisions.md#d-2026-08-11-14-keep-member-discussion-outside-the-research-authority`
  - `docs/repo-map.md`
  - `docs/handoffs/2026-08-11-community-discussion-forum.md`

## Goal

在现有只读官方社区旁增加登录用户可参与的讨论区，让用户能够发帖、评论、点赞、举报和分享一份受限附件，同时保持论坛内容与 HONE 官方研究、Agent 检索和每日产品完全隔离。

## Delivered

- 登录会员可发帖、评论、点赞、举报和删除自己的内容；管理员可隐藏或恢复。
- 三个不同用户举报后自动进入 `pending_review`，普通用户不再看到。
- 作者身份使用域隔离 SHA-256 化名，不投射手机号、邮箱或内部 user id。
- 单帖可带一份 10 MB 内的 PDF、UTF-8 Markdown/文本、PNG、JPEG 或 WebP；MIME、扩展名和文件头联合验证，下载前复核 SHA-256。
- `/community` 明确分为“官方动态”和“讨论区”；讨论区链接回“我的知识源”采纳流程。
- CI-safe 边界回归确保论坛不进入提示词、评级、红绿灯、关键事件链、持仓新闻或研究资料检索。

## Validation

- Forum Rust 7/7; full Web API 286 passed, 2 ignored.
- Focused Web 10/10; full Web 451/451.
- TypeScript, Rust formatting/diff check, public production build and research-boundary regression passed.
- Authenticated desktop and 390×844 browser acceptance covered post, alias, tags, like, comment, empty-state cleanup and zero horizontal overflow.

## Remaining Operational Work

Before production, move rows to PostgreSQL and files to object storage, then add retention/deletion, moderator audit queue, abuse telemetry and operational backup. Do not enable forum content as an investment evidence source through that migration.
