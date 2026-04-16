# 对话额度改为可配置并支持无限制

- title: 对话额度改为可配置并支持无限制
- status: archived
- created_at: 2026-04-17
- updated_at: 2026-04-17
- owner: codex
- related_files:
  - `docs/current-plan.md`
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-channels/src/agent_session.rs`
  - `config.example.yaml`
  - `config.yaml`
  - `docs/repo-map.md`
- related_docs:
  - `docs/archive/index.md`

## Goal

把每日用户对话额度从硬编码常量改为配置项，并支持通过配置关闭该限制。

## Scope

- 新增 `agent.daily_conversation_limit` 配置字段。
- 保持默认行为与当前一致。
- 约定 `0` 表示不限制每日用户对话数。
- 更新本地 `config.yaml` 使当前运行环境不再限制每日用户对话数。
- 补回归测试并同步最小必要文档。

## Validation

- `cargo test -p hone-core`
- `cargo test -p hone-channels run_success_commits_daily_conversation_quota -- --nocapture`
- `cargo test -p hone-channels run_rejects_over_daily_limit_without_persisting_user_message -- --nocapture`
- `cargo test -p hone-channels run_zero_daily_conversation_limit_bypasses_quota -- --nocapture`
- `cargo run -q -p hone-cli -- config validate`

## Documentation Sync

- 更新 `config.example.yaml` 说明新的配置项和 `0=unlimited` 语义。
- 更新 `docs/repo-map.md` 中关于 quota 的描述。
- 完成后从 `docs/current-plan.md` 移出，并归档到 `docs/archive/plans/`，同时补 `docs/archive/index.md`。

## Risks / Open Questions

- Archived. Future quota work should preserve the current semantics: `0 = unlimited`, admin actors still bypass quota, and scheduled tasks still do not consume user conversation quota.
