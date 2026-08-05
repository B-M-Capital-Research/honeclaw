# Gemini 3.1 Pro 财报工作流路由与 AAOI 样片

- title: Gemini 3.1 Pro 财报工作流路由与 AAOI 样片
- status: in_progress
- created_at: 2026-08-05
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-channels/src/agent_session/`
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-channels/src/core/bot_core.rs`
  - `crates/hone-web-api/src/routes/chat.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `config.example.yaml`
  - `skills/earnings-research/`
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/runbooks/production-deployment.md`

## Goal

让管理员专属的“财报前瞻”和“财报分析”统一通过 OpenCode ACP 调用 OpenRouter 上的 `google/gemini-3.1-pro-preview`，不改变普通聊天的 runner/model，并在生产环境完成一份 AAOI 财报前瞻可分享 PDF 样片。

## Scope

- 增加配置拥有的财报工作流 runner/model 路由；密钥只保存在运行时配置，不进入仓库。
- 从已完成服务端管理员复核的结构化财报入口传递受信任执行覆盖，防止普通聊天或伪造消息越权切换宿主 runner。
- 对齐 OpenCode 的 fresh-session replay 上下文所有权，继续强制加载 `earnings-research` skill，并保留证据核验、PDF 水印与 OSS 持久化链路。
- 增加配置、路由、执行器和上下文策略回归测试；更新长期架构与运维文档。
- 构建、推送并部署精确 revision；在零活跃会话窗口切换，完成健康检查、AAOI 真机样片和可下载验证。

## Validation

- OpenRouter 直连探针精确命中 `google/gemini-3.1-pro-preview` 并获得非空正文。
- 定向 Rust 测试覆盖：默认配置、普通聊天无覆盖、管理员两个财报入口精确覆盖、非管理员拒绝、全局 Codex 下受信任 OpenCode 路由及上下文策略。
- `skill-creator` 的 `quick_validate.py` 校验 `skills/earnings-research`。
- 仓库门禁：changed rustfmt、workspace check/test、Web test、Worker typecheck/test、CI regression。
- 生产验证：精确 revision/digest、服务健康、云/PG/OSS 健康、日志确认 OpenCode + Gemini 3.1 Pro、AAOI 报告/PDF 下载与水印检查。

## Documentation Sync

- 更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md` 和生产部署 runbook，明确工作流专用模型路由及密钥配置边界。
- 完成后新增或更新 handoff，把计划移入 `docs/archive/plans/`，更新 `docs/archive/index.md` 并从 `docs/current-plan.md` 移除。

## Risks / Open Questions

- OpenCode ACP 与 Codex ACP 的上下文所有权不同；若仍按全局 Codex 判断，会丢失本轮编译后的技能和历史上下文。
- OpenRouter 密钥不能出现在 Git、命令参数、日志或样片中；生产变更必须原子写入并保留权限受限的可回滚备份。
- Gemini 3.1 Pro preview 可能消耗较多推理 token；生产 overall timeout 需保持覆盖完整研究与 PDF 生成。
- 部署重启必须等待连续两次 active chat 为 0，避免中断用户任务。
