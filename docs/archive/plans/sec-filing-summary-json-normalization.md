# SEC Filing 摘要 JSON 跨渠道归一化

- title: SEC Filing 摘要 JSON 跨渠道归一化
- status: done
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - crates/hone-event-engine/src/event.rs
  - crates/hone-event-engine/src/pollers/sec_enrichment.rs
  - crates/hone-event-engine/src/renderer.rs
  - crates/hone-event-engine/src/digest/render.rs
  - crates/hone-core/src/config/tests.rs
  - config.example.yaml
  - config.yaml
- related_docs:
  - docs/handoffs/2026-07-31-sec-filing-summary-json-normalization.md
  - docs/archive/index.md

## Goal

消除 SEC filing LLM 摘要在 Discord、Telegram、飞书、iMessage、即时推送与 Digest 中暴露 `{"summary": ...}` JSON 外壳的问题，同时兼容已有数据库中已经保存的 JSON 字符串和未来 provider 返回的普通文本。

## Scope

- 在 SEC enrichment 写入 `payload.llm_summary` 前归一化纯文本、裸 JSON 和 Markdown JSON 代码块。
- 在所有即时渠道共用的 renderer 与 Digest 路径增加同一防御性归一化，保证历史/排队事件不会继续暴露 JSON 外壳。
- 移除 `filing_summary` profile 与纯文本 prompt 冲突的 `response_format: json_object`。
- 不修改或提交私有 `data/events.sqlite3`、用户配置内容或既有无关工作区改动。

## Validation

- Passed: `cargo test -p hone-event-engine --lib` (`524 passed`, `13 ignored`).
- Passed: `cargo test -p hone-core --lib` (`131 passed`), followed by the final focused `config_example_yaml_matches_current_schema` regression.
- Passed: `cargo check -p hone-web-api` with one pre-existing `dead_code` warning.
- Passed: exact changed-file `rustfmt --check`, `bash scripts/ci/check_fmt_changed.sh`, and `git diff --check`.
- `cargo fmt --all -- --check` remains blocked only by pre-existing formatting drift in untouched `crates/hone-channels/src/agent_session/artifacts.rs` and `crates/hone-channels/src/core/bot_core.rs`.
- Built `hone-cli`, `hone-console-page`, and `hone-discord`; restarted the source runtime. The effective `filing_summary` profile now has no `response_format`, Discord logged in, both local ports returned HTTP 200, and active chat count was zero.
- Live LLM news baseline was not run because this change does not touch news classification, model choice, or classifier fixtures.

## Documentation Sync

- 已写入 `docs/handoffs/2026-07-31-sec-filing-summary-json-normalization.md`。
- 已将本计划移入 `docs/archive/plans/`，从活动索引移除并更新 `docs/archive/index.md`。
- 不更新 `docs/repo-map.md`、`docs/invariants.md` 或 ADR：模块边界、真相源和长期架构不变，只修正既有摘要格式契约。

## Risks / Open Questions

- 已发送的 Discord 消息不能撤回重写；修复只影响后续渲染和仍在队列/数据库中再次读取的事件。
- 飞书协议本身使用 JSON；验收目标是用户可见文本不含摘要外壳，而不是传输 payload 不含 JSON。
- 本机 `config.yaml` 被 Git 忽略，只用于当前源码运行实例，不进入提交。
- `earnings_quality.rs`、`renderer.rs`、`router/tests.rs`、`subscription.rs` 的测试命名整理与本批用户确认的通知修复一并提交并推送到 `main`；本任务没有创建正式发布或 tag。
