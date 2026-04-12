# 公司画像与长期基本面追踪

- title: 公司画像与长期基本面追踪
- status: archived
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - `memory/src/company_profile.rs`
  - `crates/hone-tools/src/company_profile.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `packages/app/src/context/company-profiles.tsx`
  - `packages/app/src/components/company-profile-*.tsx`
  - `skills/company_portrait/SKILL.md`
- related_docs:
  - `docs/archive/index.md`
  - `docs/handoffs/2026-04-12-company-portrait-tracking.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`

## Goal

为 Hone 增加 Markdown 形式的公司画像与事件时间线，让研究过程能够逐步沉淀长期结论，并在后续财报、公告、管理层变化、关键指标变化时持续追加。

## Scope

- 新增 `data/company_profiles/<profile_id>/profile.md + events/*.md` 存储模型
- 新增 `company_profile` 工具与 Web API
- 在记忆页新增“公司画像”列表与详情视图
- 从知识记忆表增加画像打开入口，并为未建档公司展示“通过 agent 建立”的只读提示
- 新增 `company_portrait` skill，并给 prompt 增加轻量画像策略

## Validation

- `cargo test -p hone-memory company_profile -- --nocapture`
- `cargo test -p hone-tools company_profile -- --nocapture`
- `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`
- `bun run --cwd packages/app typecheck`
- `bun run --cwd packages/app test`

## Documentation Sync

- 已更新 `docs/repo-map.md`、`docs/invariants.md`
- 已新增 handoff：`docs/handoffs/2026-04-12-company-portrait-tracking.md`
- 已从活跃计划索引移出，并归档到 `docs/archive/plans/`

## Risks / Open Questions

- V1 前端收口为只读展示与彻底删除；建档、section 更新、事件追加与追踪设置统一通过 agent / tool 路径完成
- agent 侧“自动检测研究新公司并提示建档”当前通过全局 prompt + skill 约束落地，后续可能还需要更强的显式 runner 行为
