# Company Profile 模块拆分

- title: Company Profile 模块拆分
- status: done
- created_at: 2026-04-19
- updated_at: 2026-04-19
- owner: codex
- related_files:
  - `memory/src/company_profile/mod.rs`
  - `memory/src/company_profile/types.rs`
  - `memory/src/company_profile/markdown.rs`
  - `memory/src/company_profile/storage.rs`
  - `memory/src/company_profile/transfer.rs`
  - `memory/src/company_profile/tests.rs`
  - `memory/src/lib.rs`
  - `docs/repo-map.md`
  - `docs/archive/index.md`
  - `docs/handoffs/2026-04-19-company-profile-transfer.md`
- related_docs:
  - `docs/repo-map.md`
  - `docs/archive/index.md`
  - `docs/handoffs/2026-04-19-company-profile-transfer.md`

## Goal

把 `memory/src/company_profile.rs` 从“类型 + Markdown + transfer zip + 存储实现 + tests” 的超大单文件，拆成可维护的内聚模块，同时不改变 `hone-memory` 对外导出和 company profile 现有行为。

## Scope

- 按职责拆分 `company_profile` 模块
- 保持 `memory/src/lib.rs` 的 re-export 稳定
- 保持已有 company profile transfer / raw listing / CRUD / tests 语义不变
- 补充必要的 repo-map / handoff / archive 同步

## Validation

- `cargo fmt --all`
- `cargo test -p hone-memory company_profile`
- `cargo test -p hone-web-api`
- `cargo check -p hone-memory -p hone-web-api -p hone-channels`

## Documentation Sync

- 已更新 `docs/repo-map.md`
- 已把本计划归档到 `docs/archive/plans/company-profile-module-split.md`
- 已更新 `docs/archive/index.md`
- 已追加 `docs/handoffs/2026-04-19-company-profile-transfer.md`

## Risks / Open Questions

- 模块拆分没有改变现有 bundle contract；如果后续要继续扩展导入导出能力，优先在 `transfer.rs` 内增量演进，避免重新把类型、存储与 zip 逻辑揉回一个文件
- legacy plain Markdown transfer 兼容路径仍然依赖“最小 metadata 推断”，后续若要提高准确度，应该补显式迁移而不是在 refactor 中隐式改变语义
