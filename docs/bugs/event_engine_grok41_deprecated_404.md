# Bug: Event-engine still uses deprecated `x-ai/grok-4.1-fast` and loses LLM-backed enrichment

- **发现时间**: 2026-05-17 19:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **GitHub Issue**: 无；非 P1，本轮不创建 issue。

## 证据来源

- `data/runtime/logs/hone-console-page-prod.log`
  - `2026-05-17 16:28:24-17:28:33 CST` 最近四小时真实运行窗口内，OpenRouter 对 `x-ai/grok-4.1-fast` 连续返回 `HTTP 404`，摘要为 `Grok 4.1 Fast is deprecated. xAI recommends switching to Grok 4.3`。
  - 同窗共检出 `48` 条 `Grok 4.1 Fast is deprecated` 日志。
  - 受影响链路包括 `global_digest::mainline_distill` 的 per-ticker 主线蒸馏和 `mainline style distill`；代表 ticker 包括 `ORCL`、`ASTS`、`RKLB`、`TEM`、`VST`、`PDD`、`NBIS`、`GOOGL`、`CRWV`、`CAI`、`DELL`、`LITE` 等。
  - 更早同日还可见 `sec_enrichment LLM call failed` 与 `event_dedupe LLM call failed; falling back to pass-through` 命中同一模型下线错误，说明影响不止单个 mainline distill job。
- `data/sessions.sqlite3`
  - 最近四小时直聊会话仍有 `45` 个 user turn 与 `45` 个 assistant turn，普通 scheduler 有 `11` 条 `completed + sent + delivered=1`，未见直聊无回复或出站全局停摆。
  - 同窗 assistant final 污染扫描未命中 `/Users/`、`data/agent-sandboxes`、`rawOutput`、`tool_call`、`assistant.tool_calls`、`session/update`、原始飞书标签、compact marker、`reasoning_content` 或 `Param Incorrect`。
- 代码/配置确认：
  - `config.yaml`、`config.example.yaml`、`crates/hone-core/src/config/event_engine.rs`、`crates/hone-web-api/src/lib.rs`、`packages/app/src/pages/settings-model.ts` 等仍把 `x-ai/grok-4.1-fast` 用作 event-engine / global digest / news classifier / SEC enrichment 相关默认或示例模型。
  - 既有缺陷 [`event_engine_mainline_distill_openrouter_402.md`](./event_engine_mainline_distill_openrouter_402.md) 已修复的是短摘要链路复用全局 `max_tokens` 导致 `HTTP 402`；本次是同一大功能域中的新外部模型下线 / 默认模型过期问题，根因不同。

## 端到端链路

1. Event-engine poller / global digest / mainline distill 触发需要 LLM 的后台增强任务。
2. 运行时按配置或默认值选择 `x-ai/grok-4.1-fast`。
3. OpenRouter 返回模型下线的 `HTTP 404`。
4. 调用方把部分失败降级为 skipped ticker、style distill failed、SEC enrichment failed 或 event dedupe pass-through。
5. 主功能链路继续运行，但个性化主线、SEC 摘要、新闻分类 / 聚类质量和去重能力下降。

## 期望效果

- event-engine 的默认模型、示例配置、桌面设置默认值和运行时配置应使用当前 OpenRouter 可用模型。
- 外部模型下线时应有明确的 provider/model unavailable 分类，便于巡检快速区分额度不足、输入过长、模型不存在和网络抖动。
- 对关键后台增强链路，应至少有可配置 fallback 或降级说明，避免多个功能同时静默退化为 skipped / pass-through。

## 当前实现效果

- 多条 event-engine LLM-backed enhancement 仍会请求已经下线的 `x-ai/grok-4.1-fast`。
- mainline distill 对部分 actor 继续完成 cron tick，但主线蒸馏和 style 蒸馏失败，后续 digest 个性化缺少或陈旧。
- event dedupe 在模型调用失败后走 pass-through，可能增加重复或弱相关摘要进入后续候选。
- SEC enrichment 命中同类错误时无法生成 LLM 摘要，只能保留基础事件。

## 用户影响

- 用户不会立刻看到全局无回复或错误投递，但 event-engine 的投资主线、新闻摘要、去重、SEC enrichment 等后台质量链路会持续降级。
- 后续 global digest / event-engine 推送可能表现为个性化变差、重点排序变浅、重复内容增加或公司画像主线更新失败。
- 这不是 P3：问题不只是单条 AI 表达质量，而是多个后台功能链路因已下线模型稳定失败。
- 当前没有证据显示直聊、普通 scheduler 出站、跨用户投递或用户可见消息主链路中断，因此定为 P2 而不是 P1。

## 根因判断

- 直接根因是 event-engine 多处默认 / 示例 / 运行配置仍固定到 `x-ai/grok-4.1-fast`，但 OpenRouter 已将该模型下线。
- 既有 402 token cap 修复不能覆盖本问题，因为本轮失败发生在模型可用性检查 / provider 调用层，错误码为 404。
- 影响面可能跨 `global_digest.pass1/pass2`、`mainline_distill_llm`、`event_dedupe_model`、`news_classifier_model`、SEC enrichment profile 和桌面设置默认项，需要统一替换和回归，而不是只改一个调用点。

## 下一步建议

- 选择当前可用且质量/成本接近的替代模型，例如 OpenRouter 上的 `x-ai/grok-4.3` 或经 POC 验证的其它 event-engine 模型。
- 同步更新 `config.yaml`、`config.example.yaml`、默认配置函数、桌面设置默认值和相关测试期望，避免 UI 保存旧模型把修复回滚。
- 增加 `model_unavailable` / `provider_model_deprecated` 类错误归因，必要时对 LLM-backed enrichment 走可配置 fallback。
- 修复后用 event-engine baseline 或至少一条 mainline distill / SEC enrichment / event dedupe 定向 smoke 证明新模型可用。

## 修复记录（2026-05-17 20:10 CST）

- 状态更新为 `Fixed`。
- 复核 OpenRouter 当前模型页与 xAI docs 后，选择 `x-ai/grok-4.3` 作为 `x-ai/grok-4.1-fast` 的当前可用替代。
- `crates/hone-core/src/config/event_engine.rs` 的 event-engine 默认模型已统一切到 `x-ai/grok-4.3`，覆盖：
  - `news_classifier_model`
  - earnings quality review
  - SEC filing enrichment
  - global digest pass1 / pass2
  - event dedupe
- `config.example.yaml` 与桌面设置默认 LLM profiles 已同步切到 `x-ai/grok-4.3`，避免新配置或 UI 保存重新写回废弃模型。
- 新增 / 更新配置回归：`config_example_yaml_matches_current_schema` 会断言 `config.example.yaml` 不再包含 `x-ai/grok-4.1-fast`，并检查 event-engine 示例模型与 `mainline_short` profile 均为 `x-ai/grok-4.3`；新增 `event_engine_default_models_avoid_deprecated_grok41_fast` 锁住默认配置不再回退。
- 本轮未写真实 OpenRouter 调用 smoke，避免把当前机器外部网络 / API key / provider 可用性作为完成门槛；模型 ID 可用性以 OpenRouter 模型页和 xAI docs 为准。

## 验证（2026-05-17 20:10 CST）

- 通过：`cargo test -p hone-core config_example_yaml_matches_current_schema -- --nocapture`
- 通过：`cargo test -p hone-core event_engine_default_models_avoid_deprecated_grok41_fast -- --nocapture`
- 通过：`cargo test -p hone-web-api mainline_distill_uses_short_completion_budget --lib -- --nocapture`
- 通过：`cargo check -p hone-core -p hone-web-api -p hone-event-engine --tests`
- 通过：`cargo fmt --all -- --check`
- 通过：`git diff --check`
- 未执行：`bun test packages/app/src/pages/settings-model.test.ts`，当前环境缺少 `bun`。
