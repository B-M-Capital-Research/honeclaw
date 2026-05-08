# Bug: Event-engine mainline distill 复用全局 OpenRouter max_tokens 触发 HTTP 402

- **发现时间**: 2026-05-09 07:03 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **GitHub Issue**: 无
- **证据来源**:
  - `data/runtime/logs/desktop_release_app.log`
    - `2026-05-09 06:42:19-06:42:24 CST`，`mainline distill cron` 因 `missing_holdings` 触发多个 actor 的投资主线蒸馏。
    - 同一窗口 OpenRouter 连续返回 `HTTP 402`，错误摘要为“requested up to 30000 tokens, but can only afford 539”。
    - `mainline_distill` 对 `RKLB`、`TEM`、`LITE`、`DELL`、`CAI`、`CRWV`、`GOOGL`、`NBIS` 等 ticker 记录 `mainline distill failed`。
    - 多个 actor 随后落成 `distilled=0`，例如 `feishu::ou_64ee7ca7af22d44a83a31054e6fb92a3`、`feishu::ou_0a88f4c2105e8388aa2a63ae847f7f28`、`feishu::ou_2ccd43e67b89664af3a72e13f9d48773`、`discord::483641214445551626` 等。
  - 代码真相源：
    - `crates/hone-web-api/src/lib.rs` 仍用 `OpenRouterProvider::from_config(&state.core.config)` 注入 `global_digest_provider`。
    - `crates/hone-event-engine/src/global_digest/mainline_distill.rs` 对每个 profile 独立发起 LLM distill；失败后只把 ticker 放进 `skipped_tickers`，不会写入可用主线。
  - 已有相关文档：
    - `docs/bugs/archive/sec_enrichment_openrouter_max_tokens_402.md` 的“后续”已明确留下风险：Global digest / mainline distill 仍复用全局 OpenRouter provider，后续若出现 `HTTP 402` 应按路径做独立 cap 或语义摘抄。

## 端到端链路

1. Event-engine global digest 的 `mainline distill cron` 发现 actor 缺少 holdings 主线缓存。
2. 运行时扫描 actor sandbox 中的 `company_profiles/*/profile.md`，按 ticker 调用 LLM 蒸馏投资主线，并额外调用一次 style distill。
3. 当前 provider 由 `OpenRouterProvider::from_config(...)` 创建，沿用全局 OpenRouter completion budget。
4. OpenRouter 在请求预授权阶段按约 `30000` completion tokens 估算，当前 key 只可承受 `539`，直接返回 `HTTP 402`。
5. mainline distill 捕获失败后继续推进 cron，但该 actor 本轮 `distilled=0`，只留下 skipped ticker 列表。

## 期望效果

- Mainline distill 这类短结构化摘要不应复用全局长输出 `max_tokens`。
- 即使 OpenRouter 额度较低，也应通过较小 completion cap、语义摘抄、换用便宜模型或明确失败状态，尽量保留可用主线缓存。
- 失败时应便于区分“ticker 没有 profile”“profile 无 ticker 标识”和“LLM provider quota / token 预算失败”，避免把可修复的 provider 配置问题埋进普通 skipped ticker。

## 当前实现效果

- 最近四小时真实运行中，mainline distill 对多个 actor 批量触发，但核心 LLM 请求因 `HTTP 402` 全部或大部分失败。
- Cron 仍继续落成 actor done，并把结果写成 `distilled=0` / `skipped_tickers=N`，后续 global digest 个性化会缺少这些 actor 的投资主线缓存。
- 这是功能性 bug：损害点在 event-engine 个性化与主线缓存生成链路，而不是单次回答写作质量。

## 用户影响

- 用户后续收到的 global digest / event-engine 摘要可能缺少基于持仓画像的投资主线重排与风格约束，表现为个性化下降、重点排序变差或重复蒸馏失败。
- 当前没有证据显示已造成错误投递、跨用户投递或整条用户消息链路中断，因此不定为 `P1`。
- 该问题也不是 `P3`：它不是单条 AI 表达质量问题，而是后台画像主线缓存的批量生成链路失败，影响后续产品功能质量和可维护性，因此定为 `P2`。

## 根因判断

- 直接根因是 mainline distill 复用全局 OpenRouter provider，导致短摘要任务也携带约 `30000` completion token 预算。
- 与已修复的 SEC filing enrichment `HTTP 402` 同属“后台短摘要路径不应复用全局 max_tokens”问题族，但受影响链路不同：本单是 `global_digest/mainline_distill`，不是 SEC filing enrichment，也不是 heartbeat auxiliary runner。
- 当前 OpenRouter key 额度很低会放大问题，但代码仍应避免对短输出任务申请明显过大的 completion budget。

## 下一步建议

- 为 global digest / mainline distill 创建独立 capped provider，默认 completion cap 可从 `800-1500` token 起步，并增加配置项或复用 event-engine 下的短摘要 cap。
- 对 profile markdown 先做语义摘抄或长度上限，避免 prompt input 也在低额度时触发 provider 预授权失败。
- 在 `distill_and_persist_one` 或 cron 观测字段中区分 `provider_quota_exhausted`、`missing_profile`、`profile_parse_failed`，避免所有失败都只体现为 skipped ticker。

## 修复记录（2026-05-09 CST）

- 状态更新为 `Fixed`。
- `hone-web-api` 启动 event-engine 时，mainline distill cron 不再复用 `global_digest_provider` / 全局 `llm.openrouter.max_tokens`。
- 新增 `build_mainline_distill_provider(...)`，通过 `OpenRouterProvider::from_config_with_max_tokens(...)` 装配独立短输出 provider，completion cap 固定为 `1200` tokens；global digest curator 仍保留原 provider。
- 该修复不针对单次 OpenRouter 余额波动写特判，只收窄后台短摘要任务的通用 completion budget，避免 1-2 句主线蒸馏申请 30k completion tokens。
- 验证：
  - 通过：`cargo test -p hone-web-api mainline_distill_uses_short_completion_budget --lib -- --nocapture`
  - 通过：`cargo check -p hone-web-api --tests`
  - 通过：`rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/lib.rs`

## 后续建议

- 如后续低额度下仍出现 prompt/input 预算相关 402，再对 profile markdown 增加 section-aware excerpt 或字符预算分档重试。
- `distill_and_persist_one` 的失败分类仍可继续增强，但不阻塞本轮 completion budget 根因闭环。
