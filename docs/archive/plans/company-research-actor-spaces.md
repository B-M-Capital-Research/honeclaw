# 公司研究迁移到按 Actor 隔离的用户空间

- title: 公司研究迁移到按 Actor 隔离的用户空间
- status: done
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - `memory/src/company_profile.rs`
  - `crates/hone-channels/src/core.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-channels/src/sandbox.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `crates/hone-core/src/config/server.rs`
  - `config.example.yaml`
  - `packages/app/src/context/company-profiles.tsx`
  - `packages/app/src/components/company-profile-list.tsx`
  - `packages/app/src/components/company-profile-detail.tsx`
  - `packages/app/src/components/kb-stock-table.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/persist.ts`
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/handoffs/2026-04-12-company-portrait-tracking.md`

## Goal

取消公司画像的公共空间假设，把公司研究资产迁移为按 actor（`channel + channel_scope + user_id`）隔离的用户空间，并让 agent 通过 runner 原生文件读写直接维护这些文档。

## Scope

- 调整公司画像存储层，使 profile / events 落到 actor sandbox 下的 `company_profiles/`
- 移除专用 `company_profile` mutation tool 与对应公共 mutation API
- 调整 Web API 与前端，让画像读取与删除都显式基于 actor
- 保持页面层只读展示 + 删除，不恢复页面写入

## Validation

- `cargo test -p hone-memory company_profile -- --nocapture`
- `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`
- `bun run --cwd packages/app typecheck`

## Documentation Sync

- 更新 `docs/repo-map.md`
- 更新 `docs/invariants.md`
- 更新 `docs/handoffs/2026-04-12-company-portrait-tracking.md`
- 完成后从 `docs/current-plan.md` 移除，并归档到 `docs/archive/plans/`

## Risks / Open Questions

- `kb-stock-table` 当前不是严格 actor 视图，前端需要决定如何与当前选中的画像空间联动
- 当前方案已假设主要研究 runner 具备 actor sandbox 下的原生文件读写能力；若未来重新启用无文件工具链 runner，需要单独设计 fallback
