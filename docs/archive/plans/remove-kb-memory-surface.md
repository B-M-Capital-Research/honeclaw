# 移除 KB 记忆入口，仅保留公司画像

- title: 移除 KB 记忆入口，仅保留公司画像
- status: done
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - `packages/app/src/app.tsx`
  - `packages/app/src/pages/layout.tsx`
  - `packages/app/src/pages/memory.tsx`
  - `packages/app/src/components/sidebar-nav.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
  - `packages/app/src/lib/persist.ts`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-web-api/src/routes/kb.rs`
  - `crates/hone-channels/src/core.rs`
  - `crates/hone-channels/src/attachments/ingest.rs`
  - `crates/hone-tools/src/lib.rs`
  - `crates/hone-tools/src/kb_search.rs`
  - `skills/kb_search/SKILL.md`
  - `skills/kb_knowledge_edit/SKILL.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/handoffs/2026-04-12-company-portrait-tracking.md`

## Goal

去掉 KB 那套用户可见记忆能力，只保留按 actor 隔离的公司画像作为长期研究记忆入口。

## Scope

- 移除前端 KB 页面、知识记忆 tab、侧边导航入口及相关 context/component/api/types
- 移除 Web API `/api/kb*` 暴露和 `kb_search` tool / skill 暴露
- 让附件持久化到 KB 的后台入口先停止工作，避免继续生成新的 KB 记忆
- 保留公司画像页、actor 空间选择和删除能力

## Validation

- `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`
- `bun run --cwd packages/app typecheck`
- `bun run --cwd packages/app test`

## Documentation Sync

- 更新 `docs/repo-map.md`
- 更新 `docs/invariants.md`
- 更新 `docs/handoffs/2026-04-12-company-portrait-tracking.md`
- 完成后从 `docs/current-plan.md` 移除，并归档到 `docs/archive/plans/`

## Risks / Open Questions

- 底层 `memory/src/kb.rs` 与旧附件落盘实现可能暂时保留为内部代码，避免这轮误伤附件 ingest 主链路
- 若后续连附件持久化也要彻底删除，需要再开一轮任务收口 `attachments/vector_store.rs` 与相关存储结构
