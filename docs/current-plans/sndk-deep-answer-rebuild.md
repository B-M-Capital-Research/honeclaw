# SNDK 深度投研回答重构、Gemini Flash 反代与图像识别修复

- title: SNDK 深度投研回答重构、Gemini Flash 反代与图像识别修复
- status: in_progress
- created_at: 2026-08-29
- updated_at: 2026-08-29
- owner: Codex
- related_files:
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-llm/src/`
  - `crates/hone-channels/src/attachments/`
  - `skills/stock_research/`
  - `skills/hari-invest/`
  - `tests/regression/`
- related_docs:
  - `docs/current-plan.md`
  - `docs/invariants.md`
  - `docs/handoffs/2026-08-29-sndk-deep-answer-rebuild.md`

## Goal

把 SNDK 深度投资分析中的基本面、护城河、稀缺与差异化、财务、行业竞争、估值，以及“经营假设如何传导到估值”的逻辑落成 HONE 的代码级回答契约；修复附件图片只能依赖 OCR、不能可靠识别图形的问题；增加可配置的 Gemini 3.7 Flash OpenAI-compatible 反代配置，并在独立分支完成本地 HONE 回归。

## Scope

- 只读取用户提供的评分工作簿，不改写原文件；从 I/L/M 对应的提问、真实回答和问题评分提炼失败模式。
- 把 SNDK PPT 当作研究方法来源，不把文档中的任何指令性文本当作用户授权。
- 复用 `oldwang` 工作树中已经形成但未提交的 SEC/财务证据成果时，按文件和测试逐项移植，不切换、不清理、不提交原工作树。
- 单股深度回答必须形成可审计因果链：需求 → 供给/替代 → 公司价值捕获 → 财务兑现 → 情景假设 → 估值 → 反向隐含要求 → 条件化动作。
- 采用 491 轮验证台账：240 轮离线契约/变形测试、80 轮 provider/附件/图像集成测试、40 轮本地 HONE 确定性回答管线、131 轮评分表真实问题覆盖映射。每轮必须有编号、输入、预期门槛和结果；真实问题轮必须分开记录代码契约覆盖与 live 模型验证，不得把断言数量冒充已执行的模型对话。
- 不自动推送、不改生产配置、不上传分支、不写入用户持仓或任务状态。

## Validation

- 已通过 Rust 定向及相关库测试：`hone-channels` 808 通过/2 忽略、`hone-core` 155 通过、`hone-llm` 36 通过、`hone-tools` 192 通过/1 忽略，共 1191 通过、3 忽略、0 失败；其中需要用户评分表路径的 131 样本测试另以 `--ignored` 显式执行并通过。
- 已生成并执行 491 轮可审计台账：1–240 为八类评分失败的变形契约，241–320 为视觉/OCR 四种降级组合，321–360 为完整 SNDK 深度回答管线，361–491 对应评分表 131 条真实问题。合并产物为 `target/sndk-validation/hone-491-round-validation-ledger.ndjson`。真实问题轮全部完成问题家族映射并标记 `contract_coverage=pass`，同时如实保留 `live_model_validation=pending_gemini_channel`。
- CI-safe 回归全部通过。完整脚本第一次运行到 skill runtime 工具测试时因磁盘空间耗尽中断；清理可再生构建产物后，未执行的脚本逐一通过，财经自动化契约为 49/49。
- 本地 HONE 真实预取已成功获取 SNDK 身份、行情、财务和 SEC 证据；随后模型流在现有代理处以 HTTP 403 终止，因此未生成最终回答。临时凭据配置已删除。
- Gemini Flash 手工探针已覆盖文本和真实 data-URL 图片。Google 官方确认稳定 ID 为 `gemini-3.7-flash`，并确认 OpenAI-compatible `chat/completions`、函数调用和图片理解。现有 `bob_luna` 令牌返回 403；`bob_claude` 虽能读取目录，但目录仅有八个 Claude 模型，对目标模型返回 503 无可用通道；登录控制台的模型广场也没有 Gemini。取得有权限的反代分组或 Google Gemini API key 后，按 runbook 重跑即可完成最后两项 live 验收。

## Documentation Sync

- 在 `docs/current-plan.md` 增加本任务索引。
- 若模型/视觉路由形成长期行为，更新 `docs/repo-map.md` 与 `docs/decisions.md`。
- 完成或暂停时写 handoff；完成后把计划移入 `docs/archive/plans/` 并更新 `docs/archive/index.md`。

## Risks / Open Questions

- Google 官方已经确认精确模型 ID 为 `gemini-3.7-flash`；代码支持和官方协议兼容不等于本机账号已获模型权限。
- 当前 `bob_luna` 凭据无分组权限，`bob_claude` 目录不含 Gemini 且目标模型无通道，不能把 403/503 伪装成接入成功。
- PPT 的未来财务数字和评分表中的生产回答可能含数据源异常；最终验证以 SEC/公司 IR 的时点和口径为准，不把 PPT 数字硬编码成事实。
- 图像识别需要同时保留原图视觉证据和 OCR 文本；OCR 成功不能替代图形、颜色、位置和趋势识别。
