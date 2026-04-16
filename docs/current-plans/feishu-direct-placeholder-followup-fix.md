# Plan

- title: Feishu 直聊 placeholder 假启动与 release runner 生效链路修复
- status: in_progress
- created_at: 2026-04-16 14:12 CST
- updated_at: 2026-04-16 14:49 CST
- owner: Codex
- related_files:
  - bins/hone-feishu/src/handler.rs
  - bins/hone-desktop/src/sidecar/runtime_env.rs
  - bins/hone-feishu/src/types.rs
  - docs/runbooks/desktop-release-app-runtime.md
  - docs/bugs/README.md
  - docs/bugs/desktop_release_runner_legacy_config_source.md
  - docs/bugs/feishu_direct_placeholder_without_agent_run.md
- related_docs:
  - docs/current-plan.md
  - docs/bugs/README.md
  - docs/bugs/desktop_release_runner_legacy_config_source.md
  - docs/bugs/feishu_direct_placeholder_without_agent_run.md

## Goal

修复 Feishu 直聊消息在发送 placeholder 后未进入主链路的问题，恢复受影响用户可用性；同时收口 release app / runtime 仍读取 legacy `config_runtime.yaml` 导致 runner 改完不立即生效的问题，并完成当前运行服务切换到 `codex_acp` 的验证。

## Scope

- 排查 `process_incoming_message` 在 placeholder 前后和 `session.run()` 之前的静默中断点
- 修复 Feishu 文本/空输入/异常兜底路径，避免“placeholder 假启动”
- 将 `+8613871396421` 对应 Feishu 身份加入当前运行配置管理员名单
- 修复 desktop release 运行态误把 legacy `data/runtime/config_runtime.yaml` 当作 steady-state 配置源的问题
- 更新 release runbook，明确 canonical `config.yaml` 与 `effective-config.yaml` 的使用边界
- 更新相关 bug 文档与导航页

## Validation

- 检查 Feishu 运行日志，确认新消息不再只停留在 `reply.placeholder`
- 至少验证出现 `session.persist_user` / `recv` / `agent.prepare` / `agent.run` 或显式失败兜底
- 检查当前运行配置中管理员项已生效
- 验证 desktop runtime 在 legacy `config_runtime.yaml` override 下仍会回退到 canonical `config.yaml`
- 验证当前 live `hone-feishu` 启动日志中的 `dialog.engine` 已切到 `codex_acp`
- 已完成：
  - `cargo test -p hone-feishu actionable_user_input_detects_empty_payload -- --nocapture`
  - `cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture`
  - `cargo build --release -p hone-feishu`
  - 重启当前 `hone-release` 进程组并确认 Feishu 渠道重新连上 stream
- 待补：
  - 下一条真实 Feishu 用户消息的端到端验证

## Documentation Sync

- 更新 `docs/current-plan.md` 活跃索引
- 视修复结果更新 `docs/bugs/README.md`、`docs/bugs/feishu_direct_placeholder_without_agent_run.md` 与新的 runner 生效链路缺陷文档
- 同步更新 `docs/runbooks/desktop-release-app-runtime.md`
- 完成后将计划移出活跃索引并按需要归档

## Risks / Open Questions

- 最新“喂喂喂”“1”两条消息未成功落库，现有证据只能定位到 placeholder 后静默中断，仍需通过代码路径与新日志进一步缩小范围
- release app 当前运行方式混用了 legacy `config_runtime.yaml` 与 canonical/effective config；若 live service 不是由 desktop bundled sidecar 拉起，还需额外校准外部 supervisor 的启动环境
