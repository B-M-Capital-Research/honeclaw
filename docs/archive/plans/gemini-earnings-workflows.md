# Gemini 3.1 Pro 财报工作流路由与 AAOI 样片

- title: Gemini 3.1 Pro 财报工作流路由与 AAOI 样片
- status: archived
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
  - `docs/runbooks/backend-deployment.md`

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

## Outcome

- 管理员专属“财报前瞻”和“财报分析”已在生产环境固定路由到 OpenCode ACP + OpenRouter `google/gemini-3.1-pro-preview`；普通聊天继续使用全局 runner/model。
- 精确运行时 revision `5d26b07a32a1c2cb664f1441bbe03a3cd5e9bc23` 已部署，生产技能脚本同步到 `910d0c95`；服务、PostgreSQL、OSS 和公开入口健康。
- AAOI 财报前瞻真实运行使用 `providerID=openrouter`、`modelID=google/gemini-3.1-pro-preview`，生成 4 页 PDF。收入审计统一为 `USD millions`：锚点 157、共识 190、独立预测 196，展示为 1.57、1.90、1.96 亿美元；报告结论为超出分析师预期。
- PDF 包含 10 条近期新闻、精确水印“知识星球：巴芒科技”和知识星球分享图；聊天下载卡片点击成功，刷新恢复后仍可下载。
- renderer 的新闻经营影响错误会精确指出第几条和允许词，避免模型在严格修复循环中反复修改错误对象。

## Final Verification

- `cargo test -p hone-tools skill_tool --lib`：11 passed。
- `cargo test -p hone-channels side_effect_status --lib`：1 passed；`cargo test -p hone-channels tool_trace --lib`：9 passed。
- `cargo test -p hone-core tool_effect --lib`：2 passed。
- `bash tests/regression/ci/test_earnings_research_pdf_markdown.sh` 与技能校验通过。
- AAOI 成品 596803 bytes、4 页；逐页 PNG 和文本抽取检查通过，错误的 10 倍收入展示不存在。
- 生产附件在页面刷新后恢复，下载按钮保持可用。

## Risks / Open Questions

- OpenCode ACP 与 Codex ACP 的上下文所有权不同；若仍按全局 Codex 判断，会丢失本轮编译后的技能和历史上下文。
- OpenRouter 密钥不能出现在 Git、命令参数、日志或样片中；生产变更必须原子写入并保留权限受限的可回滚备份。
- Gemini 3.1 Pro preview 可能消耗较多推理 token；生产 overall timeout 需保持覆盖完整研究与 PDF 生成。
- 生产验收发现 OpenCode 1.18.13 将 `initialize.agentInfo.name` 从旧夹具中的 `opencode` 改为 `OpenCode`；适配器只接受这两个已观测身份并继续拒绝其它大小写变体，避免无界放宽身份检查。
- 首次 AAOI 真机运行证明 Gemini/OpenRouter 工具续写可用，同时暴露出旧附件验收指令被 fresh replay 重新激活；专用工作流必须隔离 prior history，不能依赖模型自行判断历史任务已结束。
- 第二次 AAOI 真机运行证明历史隔离生效，但普通投研系统契约仍强制了首行时间，并允许模型在首次 renderer 校验失败后以文字降级；专用 profile 必须同时移除通用投研契约并把可修正的 renderer 错误设为强制重试。
- 第三次 AAOI 真机运行已进入强制 renderer 修复循环；模型对抽象的 `neutral tolerance` 错误连续使用未被允许的同义词。renderer 错误必须给出可复制的字面量，skill 也必须指定统一的 `中性带`，否则严格校验会退化为无效重试。
- 后续真机校验显示报告已正确包含机构预期、独立预测、百分比和中性带，但使用了正常中文财务同义词 `营收`。校验器接受 `收入/营收` 两种 Workflow 常用表达，并在字段确实缺失时返回逐项字面量，避免把文风差异误判为证据缺失。
- 同一真机校验继续暴露出审计展示值不一致时的抽象报错无法指导模型修稿。校验器仍严格拒绝不一致数据，但错误会列出缺失的 `report_anchor`、`report_consensus`、`report_forecast`、`report_tolerance` 精确展示字符串，供模型按审计对象逐项修复。
- 第四次真机运行已由 Gemini 生成合格 PDF，但 Gemini 曾有一次漏传 `skill_name` 的调用在后续成功调用之后仍被全局副作用保护器判作“状态不确定”，导致终稿和附件未持久化。副作用分类现在把缺少非空技能目标的 `execute_script` 视为前置校验阶段、不可产生副作用；有明确技能目标的脚本执行仍保持持久副作用保护。
- 第五次真机运行进一步证明“有技能目标”仍不足以区分前置拒绝和执行后失败：对象参数缺少声明顺序也会在脚本启动前失败。`skill_tool` 现在显式返回 `side_effect_status=not_started/uncertain`，ACP 保留该字段，副作用门禁只放行明确的 `not_started`；进程启动、超时、非零退出、输出或 artifact 校验失败继续按不确定副作用处理。
- 第六次真机运行首次完整持久化了终稿与 PDF，却在逐页验收中发现模型把 `$157 million` 错写为 `1.57 USD billions`，继而展示成 `15.7 亿美元`。财报前瞻审计现在统一把收入规范化为 `USD millions`，并固定以 `report_scale=0.01` 输出 `亿美元`；renderer 会拒绝 billion-based 原始审计，即使其内部算术自洽，避免数量级错误穿过结构校验。
- 部署重启必须等待连续两次 active chat 为 0，避免中断用户任务。
