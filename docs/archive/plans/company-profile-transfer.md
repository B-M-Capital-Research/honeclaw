# 公司画像包导入导出与傻瓜式导入流

- title: 公司画像包导入导出与傻瓜式导入流
- status: archived
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: codex
- related_files:
  - `memory/src/company_profile.rs`
  - `memory/src/lib.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `crates/hone-web-api/src/routes/meta.rs`
- `packages/app/src/context/company-profiles.tsx`
- `packages/app/src/components/company-profile-list.tsx`
- `packages/app/src/components/company-profile-detail.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/company-profile-transfer.ts`
  - `packages/app/playwright.config.ts`
  - `packages/app/e2e/company-profile-transfer.spec.ts`
- related_docs:
  - `docs/invariants.md`
  - `docs/repo-map.md`
  - `docs/handoffs/2026-04-19-company-profile-transfer.md`
  - `docs/archive/index.md`

## Goal

为 actor 私有公司画像提供可分享、可恢复的 zip 画像包导入导出能力，并把交互收敛到现有 Memory 页面，做到“选择目标空间 -> 选择画像包 -> 自动扫描 -> 仅在冲突时做最少判断 -> 完成”。

## Scope

- 在 `hone-memory` 增加公司画像包 `export / preview / apply` 能力，载体固定为 `company-profile-bundle-v1.zip`
- 在 `hone-web-api` 增加 `/api/company-profiles/export`、`/api/company-profiles/import/preview`、`/api/company-profiles/import/apply`
- 把 Memory 页面左侧升级为“目标用户空间”选择器，并在右侧补齐导出、上传、冲突审阅、导入完成与备份下载
- 同步长期文档，明确 UI 允许的公司画像 bundle transfer 例外

## Validation

- `cargo test -p hone-memory company_profile`
- `cargo test -p hone-web-api`
- `bun run test:web`
- `bun run typecheck:web`
- `bun run build:web`
- `bun run --cwd packages/app test:e2e`
- `cargo check -p hone-memory -p hone-web-api -p hone-channels`

## Documentation Sync

- 已更新 `docs/invariants.md`
- 已更新 `docs/repo-map.md`
- 已新增 `docs/handoffs/2026-04-19-company-profile-transfer.md`
- 已更新 `docs/archive/index.md`

## Risks / Open Questions

- 首版冲突处理仍是整家公司目录级 `skip / replace`，不支持 section 级 merge
- 左侧“最近会话用户”仍只来自已有 session，彻底无历史的新目标仍需手动指定
- 替换前备份目前保留在前端本地 blob，不写回服务端或 actor 沙箱
- legacy plain Markdown 画像虽然已能导出/备份，但 transfer metadata 仍是 best-effort 推断，不等同于补齐了完整结构化历史数据
