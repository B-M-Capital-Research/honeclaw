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
- 财报入口只把当前结构化工作流、当前附件和本轮证据交给专用 runner；同一聊天中的旧任务仍留在 UI/持久化历史，但不进入本轮可执行上下文。
- 财报入口使用受信任的独立系统 profile，跳过普通 Interactive 投研预加载、首行时间和通用答案模板；skill 的 Workflow 格式与 PDF 成功门禁是唯一终稿契约。
- 增加配置、路由、执行器和上下文策略回归测试；更新长期架构与运维文档。
- 构建、推送并部署精确 revision；在零活跃会话窗口切换，完成健康检查、AAOI 真机样片和可下载验证。

## Validation

- OpenRouter 直连探针精确命中 `google/gemini-3.1-pro-preview` 并获得非空正文。
- 定向 Rust 测试覆盖：默认配置、普通聊天无覆盖、管理员两个财报入口精确覆盖、非管理员拒绝、全局 Codex 下受信任 OpenCode 路由、历史隔离及独立系统 profile。
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
- 生产验收发现 OpenCode 1.18.13 将 `initialize.agentInfo.name` 从旧夹具中的 `opencode` 改为 `OpenCode`；适配器只接受这两个已观测身份并继续拒绝其它大小写变体，避免无界放宽身份检查。
- 首次 AAOI 真机运行证明 Gemini/OpenRouter 工具续写可用，同时暴露出旧附件验收指令被 fresh replay 重新激活；专用工作流必须隔离 prior history，不能依赖模型自行判断历史任务已结束。
- 第二次 AAOI 真机运行证明历史隔离生效，但普通投研系统契约仍强制了首行时间，并允许模型在首次 renderer 校验失败后以文字降级；专用 profile 必须同时移除通用投研契约并把可修正的 renderer 错误设为强制重试。
- 第三次 AAOI 真机运行已进入强制 renderer 修复循环；模型对抽象的 `neutral tolerance` 错误连续使用未被允许的同义词。renderer 错误必须给出可复制的字面量，skill 也必须指定统一的 `中性带`，否则严格校验会退化为无效重试。
- 部署重启必须等待连续两次 active chat 为 0，避免中断用户任务。
