# Gemini 3.1 Pro 财报工作流生产交接

- title: Gemini 3.1 Pro 财报工作流生产交接
- status: done
- created_at: 2026-08-05
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `crates/hone-core/src/config/agent.rs`
  - `crates/hone-channels/src/agent_session/`
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `skills/earnings-research/`
- related_docs:
  - `docs/archive/plans/gemini-earnings-workflows.md`
  - `docs/decisions.md`
  - `docs/runbooks/opencode-setup.md`
  - `docs/runbooks/backend-deployment.md`
- related_prs:
  - direct `main` commits `115db5fb` through `105ca177`; no PR or release tag

## Summary

管理员专属财报前瞻和财报分析已通过受信任的 per-turn route 使用 OpenCode ACP + OpenRouter `google/gemini-3.1-pro-preview`。普通聊天 runner/model、历史行为和非管理员边界没有变化。生产 AAOI 财报前瞻样片已完成、持久化并逐页验收。

## What Changed

- `agent.earnings_workflow` 独立拥有 runner/model；只有服务端重新确认管理员且识别结构化财报意图后才可下发覆盖。
- 专用 runner 使用独立 compiled replay 和 earnings 系统契约，不加载旧聊天任务，不走普通问答时间开头或文字降级。
- `skill_tool` 显式区分 `side_effect_status=not_started` 与 `uncertain`：仅明确的执行前拒绝可继续，已启动或状态不明的调用仍 fail closed。
- preview 审计把收入统一规范为 `USD millions`，展示到 `亿美元` 时固定 `report_scale=0.01`；renderer 拒绝 billion-based 审计和不一致展示值。
- renderer 的新闻经营影响校验会返回具体 bullet 序号和允许词，减少无效修复循环。

## Verification

- OpenRouter 直连探针：HTTP 200，精确模型且非空响应。
- 定向 runner、session、Web 管理员边界、side-effect、skill/PDF 回归均通过；技能结构校验通过。
- 生产二进制 revision：`5d26b07a32a1c2cb664f1441bbe03a3cd5e9bc23`；immutable digest `sha256:0b98e287e180a0f8a89d8a3491044421210f28ad6022fd2cb4bfc9a56f8b2ab7`。
- 生产技能 renderer 对齐 revision `910d0c95`，SHA-256 `63b5036275bb28c7c626597d77713fe18b7fec3e1e6778c49063e0f04ce0615d`。
- AAOI OpenCode session `ses_02f27ec8cffey3slgugAZzDRjF` 记录 `providerID=openrouter`、`modelID=google/gemini-3.1-pro-preview`。
- 成品 596803 bytes、4 页；收入 157/190/196 `USD millions` 对应 1.57/1.90/1.96 亿美元，结论“超出分析师预期”，含 10 条新闻、精确水印和知识星球分享页。
- 聊天 PDF 卡片点击下载成功；刷新恢复会话后卡片仍存在。

## Risks / Follow-ups

- Gemini preview 推理和严格 renderer 自修复可能持续数分钟；不要把中间校验错误发布成终稿。
- `origin.hone-claw.com` 的旧 Sunny-Ngrok 重定向仍是独立遗留风险；本次生产验证使用公开 `https://hone-claw.com` 路由。
- 用户曾在聊天中粘贴 OpenRouter 密钥，必须轮换；密钥未进入仓库、提交、测试夹具或最终样片。
- staging 时为释放空间删除的 superseded GHCR release 目录可从 immutable GHCR 恢复；当前和回滚 release 已保留。

## Next Entry Point

轮换 OpenRouter 密钥后，从管理员“财报前瞻”或“财报分析”按钮开始下一次验收。附件型财报分析应上传真实财报材料并确认报告明确列出已读/未读附件，随后重复 PDF 持久下载检查。

## Original Workflow Parity Follow-up

- 原 Dify `V2-财报前瞻` prompt 已提升为专用 skill 的首要内容契约：先做超出/低于/持平判断，再用预测桥、历史指引兑现、当前指引、最新管理层信息和公司催化剂解释结论；通用问答格式不得覆盖它。
- `1.2.3` 现在要求真实出具观点的券商、银行或研究机构，并逐家记录日期、Buy/Hold/Sell/Outperform 类评级、目标价、单季收入/EPS 预测及理由。聚合器、媒体、作者和会议不能冒充机构；机构未披露某个数字时，报告明确写“未披露”，但仍解释其评级逻辑相对管理层指引和独立预测偏保守还是偏积极。
- `1.3` 现在是八至十条无超链接自然段；至少六条且至少 60% 为公司直接新闻，必须包含上次财报/电话会和具名机构观点。客户、同行、供应链新闻合计最多三条，只有在公司关系和本季收入、利润、价格、成本、产能或交付传导均有证据时才可进入。纯股价、会议、泛行业和无公司链路的新闻会被终稿校验拒绝。
- `skill_tool` 的结构化 `script_payload` 由宿主序列化为 renderer 的单个 JSON 参数，消除了 Gemini 手写长 JSON 转义造成的失败。renderer 的 preflight 每轮只返回最多八个最高优先级修稿问题，减少长循环和无关改写。
- 生产二进制当前为 `2c2cd1db`；技能/renderer 热更新到 `105ca177`，renderer SHA-256 为 `7440cc8e67878e5f04fc1968d9fd04dbd7aa7764d473ad62bb6cddb9f45ac142`，Skill SHA-256 为 `0dc110146c0456c66a7c2ba2c7bfed733e2fc657d53520614b480979ab7ff2f1`。
- Gemini 的一次有效长推理超过旧 180 秒 step timeout，因此生产 `agent.step_timeout_seconds` 调整为 600，`overall_timeout_seconds` 保持 1200。配置回滚备份是 `/srv/honeclaw/config.yaml.bak-earnings-timeout-20260805T1840Z`；技能回滚快照保留在 `/opt/hone/skill-rollbacks/`。
- 最终 AAOI 会话 `ses_02e70e468ffevZu5Kq7yTlLAeC` 给出“超出分析师预期”：Q2 管理层营收指引中点 1.89 亿美元、机构预期 1.90 亿美元、独立预测 2.03 亿美元，独立预测高 6.8%；EPS 指引中点 -0.01 美元、机构预期 0.02 美元、独立预测 0.05 美元。
- 机构对比页包含 Rosenblatt 与 Needham 的 2026-07-29 Buy 观点并如实标注未披露目标价/单季预测；新闻页为八个自然段。成品 `AAOI_财报前瞻_机构对比_20260805.pdf` 为 A4、627276 bytes、4 页，四页均有“知识星球：巴芒科技”水印，第二页是机构对比，第三页是近期新闻，第四页是分享图和免责声明。
- 线上聊天附件 `AAOI_Earnings_Preview-e8f5a251.pdf` 已实际下载；页面刷新后精确下载按钮仍有一个，证明附件已持久化而非依赖被销毁的临时 sandbox。
