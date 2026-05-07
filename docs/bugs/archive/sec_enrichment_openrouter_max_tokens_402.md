# Bug: SEC filing enrichment 复用全局 OpenRouter max_tokens 触发 `HTTP 402`

- **发现时间**: 2026-05-07 11:15 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **GitHub Issue**: N/A

## 证据来源

- 运行日志：
  - `data/runtime/logs/web.log.2026-05-07`
  - `2026-05-07 11:15:45-11:16:08 CST` 多次记录 OpenRouter `HTTP 402`：
    - `This request requires more credits, or fewer max_tokens`
    - `You requested up to 30000 tokens, but can only afford 26248`
  - 同一窗口紧接 `sec_enrichment LLM call failed`，说明失败链路是 SEC filing enrichment，不是前一行 `poller.fmp.analyst_grade`。
- live API smoke：
  - 当前 OpenRouter key 可认证，网络路径可用。
  - `x-ai/grok-4.1-fast` + `max_tokens=800` 成功返回 `ok`。
  - 同一模型 + `max_tokens=30000` 即使只有一句短 prompt，也返回 `HTTP 402`，提示当前 key 只能承受约 `26244` 输出 token 的单次预算。
- 追补日志：
  - `2026-05-07 15:23:00 CST` 同一链路在 TEM 10-Q 上继续失败：
    - `Prompt tokens limit exceeded: 54381 > 6713`
  - 这次不是 output `max_tokens` 过大，而是 filing input 本身过长。
  - `2026-05-07 19:59:00-19:59:04 CST` 摘抄逻辑部署后继续出现更小一档 prompt budget 失败：
    - TEM 8-K: `Prompt tokens limit exceeded: 5198 > 3256`
    - TEM 10-Q: `Prompt tokens limit exceeded: 3956 > 3256`
    - 另一条 filing: `Prompt tokens limit exceeded: 5129 > 3256`
  - 这说明 section-aware 摘抄已经把 10-Q 从 54k prompt tokens 降到约 4k,但当前 OpenRouter key 的可承受 prompt budget 会随 weekly limit 余额继续下降,不能依赖单个固定字符上限。

## 根因判断

- SEC filing enrichment 的配置字段 `event_engine.sec_filings.enrichment.max_summary_tokens` 默认是 `800`，目标只是一段约 200 字中文摘要。
- 旧实现中 `LlmSecFilingSummarizer` 复用 `global_digest_provider`，而该 provider 由 `OpenRouterProvider::from_config(...)` 构建，实际使用全局 `llm.openrouter.max_tokens`。
- 当前部署全局值为 `32768`，OpenRouter 对请求做最坏情况预授权时按约 `30000` 输出 token 预算校验，超过当前 key 的单次可承受预算后返回 `HTTP 402`。
- 这不是账户余额或 API key 连通性问题，而是短摘要链路错误继承了长输出预算。
- 追补根因：即使 completion budget 修好，旧实现仍把 10-Q/10-K 的清洗后全文送给 LLM。TEM/AMD/COHR 真实 10-Q 清洗后仍约 128k-215k 字符，其中大量是目录、普通财务表、exhibit index、inline XBRL hidden/header/resource 噪声，以及对摘要目标低价值的表格行。OpenRouter 当前 key 在该时点只能承担约 6.7k prompt tokens，所以 TEM 10-Q 的约 54k prompt tokens 会继续触发 `HTTP 402`。

## 用户影响

- SEC filing 事件本身不会丢失；enrichment 是 best-effort，失败后 renderer 回退到原始 form/link body。
- 用户会少看到 filing 的 LLM 摘要；运行日志会持续出现 OpenRouter 402，并可能消耗调试注意力。
- 若同一 tick 有多条 filing，旧实现会对每条都发起过大的预授权请求，放大 provider 错误噪声。

## 修复记录（2026-05-07 11:26 CST）

- 状态更新为 `Fixed`。
- `EventEngine` 新增 `with_sec_filings_enrichment_provider(...)`，SEC filing enrichment 可使用独立 LLM provider。
- `hone-web-api` 在装配 event-engine 时使用 `OpenRouterProvider::from_config_with_max_tokens(...)` 为 SEC filing enrichment 单独创建 provider，completion cap 来自 `sec_filings.enrichment.max_summary_tokens`。
- `llm.openrouter.max_tokens` 与 global digest provider 行为保持不变，避免影响普通长输出路径。
- `max_summary_tokens` 被安全 clamp 到 `1..=u16::MAX`，避免无效或溢出配置。

## 追补修复记录（2026-05-07 15:50 CST）

- `sec_enrichment` 不再把 filing 清洗后全文直接交给 LLM。
- 新增 `extract_filing_llm_context(...)`：先抽 SEC HTML 直接文本块，跳过 `<script>/<style>`、hidden 元素和 inline XBRL `ix:hidden` / `ix:header` / `ix:resources` / `ix:references` 噪声，再按 form 生成精选摘抄。
- 10-Q / 10-K 路径优先：
  - Item 2 MD&A 的 high-signal 窗口；
  - 财务附注与全文中的战略合作、大客户/订单、并购、债务、回购、capex、法律/监管、风险关键词窗口；
  - Item 1A Risk Factors 中的明确变化或 `no material changes` 口径；
  - Part II Legal Proceedings 的短小节。
- 8-K 路径优先保留前置 exhibit / press release narrative，预算到达后丢弃尾部表格和 exhibit 元数据。
- 送 LLM 的最大输入从“约 300k 字符全文截断”改为 `18_000` 字符的 section-aware 摘抄；这是语义筛选后的预算边界，不是盲截断。
- 默认 system prompt 改为说明输入是“精选摘抄，不是全文”，避免模型假设自己看到了完整 filing。

## 追补修复记录（2026-05-07 20:10 CST）

- 将默认 section-aware 摘抄上限从 `18_000` 字符收紧到 `10_000` 字符,让 TEM 8-K / 10-Q 这类样本默认落入当前约 3.2k prompt-token 预算附近。
- 对 OpenRouter 明确返回的 `Prompt tokens limit exceeded` / `HTTP 402` 增加同类语义摘抄重试:
  - 默认 `10_000` 字符；
  - 第一次重试 `7_000` 字符；
  - 第二次重试 `4_500` 字符；
  - 第三次重试 `2_800` 字符。
- 重试仍走同一个 filing-aware extractor,优先保留 MD&A / 战略合作 / 资本配置 / 风险法律窗口或 8-K 前置 narrative；不是把全文盲目截短。

## 验证

- 通过：`cargo test -p hone-web-api sec_filings_enrichment --lib`
- 通过：`cargo test -p hone-event-engine sec_filings_enrichment --lib`
- 通过：`cargo check -p hone-web-api`
- 通过：`rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/lib.rs crates/hone-event-engine/src/engine.rs`
- 未通过但非本次改动阻塞：`cargo fmt --all -- --check`，失败 diff 位于既有 `bins/hone-cli/src/*`、`crates/hone-core/src/quiet.rs`、`crates/hone-event-engine/src/global_digest/fetcher.rs`、`crates/hone-event-engine/src/router/policy.rs` 格式漂移，本次未修改这些文件。
- 追补 POC：
  - TEM/AMD/COHR 10-Q 与 TEM 8-K 真实文件确认：摘要信号集中在 MD&A、财务附注中的战略/资本配置窗口、风险/法律变化和 8-K exhibit 新闻稿前段；全文并非都同等有用。
  - TEM 10-Q live smoke 使用 15k 字符精选摘抄调用 `x-ai/grok-4.1-fast` 成功，OpenRouter 返回 `prompt_tokens=3170`、`completion_tokens=798`、成本约 `$0.0010`。
- 追补通过：`cargo test -p hone-event-engine sec_enrichment --lib`
- 追补通过：`cargo test -p hone-event-engine sec_enrichment --lib`，覆盖 `Prompt tokens limit exceeded` 识别与 `10_000 -> 7_000 -> 4_500 -> 2_800` 重试预算序列。

## 后续

- Global digest / mainline distill 仍复用全局 OpenRouter provider；如果后续在这些短摘要/结构化输出路径继续出现 `HTTP 402`，应先区分 output budget 与 prompt input 两类失败，再按路径做独立 cap 或语义摘抄。
- SEC S-1 / DEF 14A 目前走通用 front narrative + keyword-window 摘抄路径；本次 POC 样本主要覆盖 10-Q 与 8-K，后续若这些 form 触发质量问题，应补真实样本再调 extractor。
- 如果 OpenRouter key 的 prompt 可承受预算继续低于约 2.8k 字符摘抄对应的 token 量,当前链路仍会 best-effort 降级为无 LLM 摘要；那已经不是“文件选择”问题,而是需要补额度、换更便宜模型或改为本地/离线规则摘要。
