- title: 投研回答校验后提交与资产路由源码恢复
- status: archived
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/agent_session/emitter.rs
  - crates/hone-channels/src/agent_session/tests.rs
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-web-api/src/routes/chat.rs
  - crates/hone-tools/src/data_fetch.rs
  - tests/regression/ci/test_finance_automation_contracts.sh
- related_docs:
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/handoffs/2026-07-16-entity-first-investment-pipeline.md

## Goal

投研回答在实体、证据和完整性校验全部通过之前不向任何客户端暴露草稿；正常结束只提交一次最终答案，不再通过 `StreamReset` 清屏后生成第二轮。同时恢复 2026-07-16 19:01 被自动巡检误删、但仍存在于运行二进制中的 equity / fund / crypto 资产感知路由，避免重启回退 INTL 修复。

## Scope

- 在 session 层引入投研受保护回答的延迟提交边界，屏蔽候选轮次的 delta、reset、thought 和临时 error。
- 校验、最终清洗和附件处理完成后，只提交一次 canonical response。
- 删除 Web 失败路径重复发送的 `run_finished`。
- 恢复资产类型识别、按资产取证和价格一致性门禁；扩展 Forward PE 等常见估值表达识别。
- 增加 runner 真实发流事件、SSE 终止事件和 RMBS / INTL 的回归证明。

## Validation

- `cargo test -p hone-channels --lib -- --test-threads=1`：510 passed。
- `cargo test -p hone-tools data_fetch --lib`：24 passed。
- `cargo test -p hone-web-api routes::chat::tests --lib`：3 passed。
- `bash tests/regression/ci/test_finance_automation_contracts.sh`：17 success。
- CLI、Web、Discord、Feishu 构建并完整重启；8077/8088、Postgres、S3 与子进程健康。
- RMBS 与 INTL 真实 SSE 均为一次正文、0 reset、0 error、一次 `success=true` 终止，并通过九节与资产路由检查。

## Documentation Sync

- 长期一次提交约束已同步到 `docs/invariants.md`、`docs/decisions.md` 和 `docs/repo-map.md`。
- 验证与恢复根因追加到同日实体优先 handoff；本计划已移出活跃索引并登记到 archive index。

## Risks / Open Questions

- 投研回答改为终审后一次显示，牺牲逐 token 可见性以保证不泄露草稿；普通请求保持原生流式。
- 自动 bug patrol 的越界 `git restore` 属于独立自动化治理风险；本次已恢复源码，但仍需在该自动化任务自身范围内限制工作区写操作。
