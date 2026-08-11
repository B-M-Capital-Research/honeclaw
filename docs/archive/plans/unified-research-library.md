# Unified Research Library

- title: HONE 统一研究资料库与下游引用
- status: done_locally
- created_at: 2026-08-11
- updated_at: 2026-08-11
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/research_library.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/key_event_chain.rs`
  - `crates/hone-web-api/src/routes/portfolio_news.rs`
  - `packages/app/src/pages/public-research-library.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/repo-map.md`
  - `docs/decisions.md`
  - `docs/handoffs/2026-08-11-unified-research-library.md`

## Goal

建立一个可每日持续上传的统一研究资料库，让 HONE 问答、关键事件链和持仓新闻使用同一份有来源、有日期、可去重、可撤销授权的材料；外部平台只通过官方授权连接器、官方导出或用户主动上传接入。

## Delivered Scope

- actor 隔离的个人资料与仅管理员可写的 HONE 全局资料。
- PDF、TXT、Markdown、CSV、JSON、DOCX 等常见研究文件的持久化、元数据、内容哈希去重、用途标签和安全下载。
- 独立 `/research-library` 页面，支持上传、筛选、用途设置和删除。
- 问答自动注入获授权且与问题相关的资料摘要；关键事件链与持仓新闻读取相应用途的研究材料。
- 知识星球和 IMA 暂不模拟登录或抓取私人内容；首版支持官方导出/用户上传，并为后续官方 Skill/API 授权连接器保留来源类型。

## Validation

- `cargo test -p hone-web-api --lib --no-fail-fast`: 262 passed, 2 ignored.
- `bun run typecheck:web`: passed.
- `bun run test:web`: 441 passed.
- `bun run build:web:public`: passed.
- `cargo fmt --all -- --check`: passed.
- Authenticated desktop/mobile browser acceptance and real multipart API upload/list smoke passed.

## Follow-up

- Production multi-instance deployment must first move manifests to PostgreSQL and file bytes to object storage.
- Add Knowledge Planet/IMA connectors only through documented official authorization; do not automate signed-in scraping.
- Add full-text/vector retrieval only after document parsing, deletion, actor isolation and citation semantics remain enforceable end-to-end.
