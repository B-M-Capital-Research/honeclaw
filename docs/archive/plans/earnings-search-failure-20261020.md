# 财报前瞻失败与资料新鲜度排查

- title: 财报前瞻失败与资料新鲜度排查
- status: archived（2026-10-21 已上线并完成恢复验收）
- created_at: 2026-10-20
- updated_at: 2026-10-21
- owner: Codex
- related_files:
  - skills/earnings-research/
  - crates/hone-channels/src/earnings_materials.rs
  - crates/hone-channels/src/agent_session/
  - crates/hone-tools/src/web_search.rs
  - scripts/diagnose_fmp_tavily.sh
- related_docs:
  - docs/runbooks/backend-deployment.md
  - docs/runbooks/opencode-setup.md
  - docs/handoffs/2026-09-13-earnings-workflow-refactor.md

以下初始 Scope/Validation/Risks 保留 10 月 20 日诊断阶段；后续用户授权恢复范围与结果见 2026-10-21 节。

## Goal

同步最新代码，定位近期用户财报前瞻失败的实际原因，并核对成功报告缺少最新资料是否与 Tavily 凭证、配额或检索流程有关。

## Scope

- [x] 安全同步 origin/main，保留原有本地提交及未提交文件。
- [x] 只读核对线上版本、近期失败日志及实际工作流/搜索调用。
- [x] 检查有效配置与供应商连通性，仅输出脱敏状态及必要证据。
- [x] 对比历史成功报告的检索行为、材料日期与当前实现：TSLA/BE 最新新闻参数正确，但搜索全部失败且未成功读取公开原文；不做无依据的全文事实结论。
- [x] 根据证据确定修复范围：OpenRouter 余额不足、Gemini 搜索请求 404、Tavily 套餐额度耗尽；本次完成诊断，不修改应用代码或生产账户。

## Validation

- [x] 以实际任务时间、报错和供应商响应交叉验证结论，区分已证实原因与未验证推测。
- [x] 保留最小脱敏证据；不把完整用户会话、密钥或供应商全文写入仓库。
- [x] 执行文档差异检查；无代码改动，不运行 Rust/Web 测试。

## Documentation Sync

本任务跨运行链路并行调查，满足动态计划准入标准。

- [x] 更新 `docs/handoffs/2026-10-20-earnings-search-diagnosis.md`，记录结论、验证范围和剩余风险。
- [x] 完成后移至 `docs/archive/plans/`，移除 `docs/current-plan.md` 活跃项并更新 `docs/archive/index.md`。
- [x] 更新 `docs/runbooks/backend-deployment.md` 的实际配置、代理路由、用量/余额分层诊断步骤；没有长期行为变化，无需修改稳定架构文档。

## Risks / Open Questions

- 历史日志保留范围可能不足；当前供应商可用性不等于历史请求成功。
- 现有工作区包含无关改动及未推送提交，必须原样保留。
- 调查期间不重启服务、不修改生产凭证、不重放用户报告或发送消息。
- 不新增机械内容门禁；资料质量问题只从取数、搜索、原 Prompt 与真实样本核对入手。


## 2026-10-21 Recovery scope

用户已授权以现有 Gemini 替代财报 OpenRouter 路由，配置提供的 Tavily key 并上线验证。原工作区包含本地 V4 提交和无关变更，因此从 origin/main 创建独立 worktree 与 codex/earnings-gemini-recovery 分支。

- [x] 验证现有 Gemini 纯文本和工具调用能力，以及新 Tavily key 的真实搜索。
- [x] 在 config/earnings runner/OpenCode provider 层支持配置指定的可信供应商，保留当前轮隔离、管理员和 PDF/artifact 边界；不改原 Prompt、不新增内容门禁。
- [x] 补充旧 OpenRouter 兼容、自定义 provider、缺失/无效配置及凭证路由回归；运行定向 Rust/回归与部署必需门禁。
- [x] 修复现有 Gemini 代理的标准 OpenAI 工具结果兼容：按 tool_call_id 恢复函数名；保留最小补丁、无账号回归和真实双轮/整报告证据，更新 backend-deployment runbook 与原 handoff。
- [x] 原子更新受保护 canonical 配置与精确 revision 部署，保留回滚；避免打断活跃用户任务。
- [x] 使用独立 actor 做真实 WDC 前瞻，核对成功搜索、近期资料、模型身份、PDF 与持久化交付。
- [x] 更新 docs/runbooks/backend-deployment.md、docs/runbooks/opencode-setup.md、docs/invariants.md 与 docs/repo-map.md 的供应商路由约定，记录必要 decision。
- [x] 更新原 handoff 为已恢复/验证状态，归档本计划并同步 docs/current-plan.md 和 docs/archive/index.md。

### Recovery risks

现有 Gemini 代理的聊天能力与已失败的 Google grounding 是不同路径，必须分别实测。新 key 只通过 stdin 进入受保护文件，不进入仓库或日志。真实 canary 必须有时间/成本边界，失败保留报告和精确错误。

### 2026-10-21 canary finding

首次真实 WDC canary 在 todowrite 后第二轮返回 Gemini 400。对同一响应做有/无工具 name 对照：省略 name 时 400，提供正确 name 时 200/OK；OpenCode 按标准省略 name，既有代理错误补为 tool。范围扩展到代理协议修复，原 Prompt 和内容质量流程不变。

### 完整 canary 与上线阶段

- 现有 medium 档：WDC 完整前瞻 166.3 秒成功，10 次搜索/10 credits、4 份主机预取原文、1 次 renderer/1.543 秒、9 页 PDF；重启后历史/下载/哈希一致。
- 人工复核记录 DCF 无量化桥的调整、部分机构来源呈现缺口及五级标题排版问题；不能把技术闭环当作事实质量全验收，也未新增内容门禁。
- 同账号 flash-high 完整对照亦技术成功，但 DCF 算术错误、最新新闻使用和来源呈现未改善；最终选择现有 medium，不扩大模型试验。
- 线上代理最小补丁已接受：精确源码 SHA `11d831f2aa58dd5428d08bb4d63aca48cfaaae4884b6fe7159d30c43b13f01bc`，无 name 标准工具双轮均 200；canonical config 切换另行验收。精确 runtime 构建 `39608620192`、export `39613151201` 均成功，正部署同一 `5ad8e7ba` 不可变产物。


### 最终状态

- 2026-10-21 22:09:40（北京时间）`5ad8e7ba` 精确运行包和 canonical 配置已上线，provider/model/key 生效读回、真实搜索、Web/Feishu 及 PG/OSS 双轮验收完成。
- 独立 medium/high 报告对照已完成，选择 medium；原 Prompt/阶段顺序/renderer 内容边界不变。研究质量、legacy MCP 元数据和不同 OpenCode 版本验证范围在 handoff 明示。
- 原工作区未更改；临时 canary/PG/SSH/shadow 已清理，凭据不进入仓库。PR #59 承载应用/代理补丁与文档。
- 计划移入 `docs/archive/plans/`，活跃索引移除并更新历史索引；无需继续把已完成供应商恢复保留为活跃任务。
