# 公司画像 Skill 框架对齐专业投研档案

- title: 公司画像 Skill 框架对齐专业投研档案
- status: archived
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - `skills/company_portrait/SKILL.md`
  - `skills/company_portrait/references/profile-framework.md`
  - `skills/company_portrait/references/event-template.md`
  - `skills/company_portrait/references/research-trail.md`
  - `memory/src/company_profile.rs`
  - `crates/hone-tools/src/company_profile.rs`
  - `crates/hone-channels/src/prompt.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `crates/hone-web-api/src/types.rs`
- related_docs:
  - `docs/current-plan.md`
  - `docs/handoffs/2026-04-12-company-portrait-tracking.md`
  - `docs/archive/index.md`

## Goal

把公司画像相关 skill 和文档型画像模板升级为更接近专业投研机构的研究档案框架，在不引入重 schema 的前提下，保留“当前结论、为什么成立、什么会证伪、当时研究路径与来源”。

## Scope

- 更新 `company_portrait` skill，采用更符合 Codex 的“简主文档 + references”结构
- 调整画像默认 section，使其更贴近 Thesis / 关键经营指标 / 风险台账框架
- 扩展事件文档内容，补上 why / evidence / research trail 的开放式记录
- 更新 prompt 与相关文档描述

## Validation

- `cargo fmt --all`
- `cargo test -p hone-memory company_profile -- --nocapture`
- `cargo test -p hone-tools company_profile -- --nocapture`
- `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`

## Documentation Sync

- 已更新 `docs/current-plan.md`
- 已更新 `docs/handoffs/2026-04-12-company-portrait-tracking.md`
- 已更新 `docs/archive/index.md`
- 已从活跃计划索引移出，并归档到 `docs/archive/plans/`

## Risks / Open Questions

- 当前运行时尚未落独立 `research_notes` 存储层，因此“研究路径”先体现在事件文档中，避免 skill 指导超前于实现
- 画像默认 section 已向 Thesis / 风险台账对齐，但既有 profile 仍可能保留旧 section 命名；当前实现保持兼容，不主动批量迁移
