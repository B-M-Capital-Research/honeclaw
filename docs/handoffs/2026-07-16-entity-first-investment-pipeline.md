# 投研实体优先执行管线改造交接

- title: 投研实体优先执行管线改造交接
- status: done
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/scheduler.rs
  - crates/hone-tools/src/data_fetch.rs
  - tests/regression/ci/test_finance_automation_contracts.sh
  - tests/regression/manual/test_entity_search_live.sh
- related_docs:
  - docs/archive/plans/entity-first-investment-pipeline.md
  - docs/archive/plans/asset-aware-investment-preflight.md
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
- related_prs: N/A

## Summary

投研链路已从“正则猜 ticker + 单股特例”改为实体优先的统一执行管线。当前请求先结构化提取全部命名证券，再逐一使用 DataFetch search 解析规范实体并核验同 symbol 行情，之后才允许生成公司特定数字或进入单股/比较回答契约。

## What Changed

- 移除 `REPEAT`/`AI` 一类缩写黑名单、`Nebius → NBIS` 硬编码、搜索首条候选和最终阶段重猜 ticker。
- 显式 ticker 最初只接受 `$TICKER` 确定性输入；此限制已被下方同日 bare-ticker 回归阶段替代，`$TICKER` 仍是最高置信度输入，但不再是普通代码的唯一入口。
- 中文名/别名经过结构化提取后使用 DataFetch search；候选按名称、symbol、交易所评分，相同分数的 share class 保持歧义。
- 引入 typed `AgentTurnOrigin`，scheduler/heartbeat 使用原始任务正文做实体输入，delivery envelope 不再参与。
- 实体、证据和回答契约每轮只准备一次，context overflow 与 heartbeat budget recovery 复用同一份 runtime suffix。
- 单股深度保留九章节强制格式；多标的增加所有 symbol、数据时间和风险/证伪的最终校验。

## Verification

- `hone-channels`：488/488 passed。
- `hone-tools`：123 passed，0 failed，1 ignored。
- 全量运行时二进制构建（CLI、MCP、Web、iMessage、Discord、Feishu、Telegram）通过；所有 `AgentRunOptions` 字面量已使用默认补全，typed origin 在 scheduler 内覆盖。
- 投研 CI 契约：16/16 success。
- 真实 NBIS MCP 链路：search 精确返回 `NBIS / Nebius Group N.V.`，quote 返回正数价格；FMP 与 Tavily 探针均 healthy。
- rustfmt changed-files 和 `git diff --check` 通过。
- 服务已完整重启到 supervisor `84863`：8077/8088 正常监听，Web/Discord/Feishu 单进程在线，Postgres/OSS healthy，启动后错误扫描为空。

## Risks / Follow-ups

- provider 搜索覆盖不足时会向用户澄清，这是故意的正确性优先行为。
- 大型 scheduler 观察列表的 search fan-out 可继续做缓存/批处理优化，但不能跳过每轮同 symbol 行情核验。
- 若引入新的证券数据源，应实现相同的候选字段和歧义语义，不要恢复公司名硬编码。

## Next Entry Point

先读 `D-2026-07-16-01` 和 `crates/hone-channels/src/investment_response_guard.rs`；真实 provider 回归从 `tests/regression/manual/test_entity_search_live.sh` 进入。

## 2026-07-16 普通 ticker 回归修复阶段

### Root Cause

`7a18f552` 将 `$TICKER` 设为唯一确定性代码输入，普通 `NBIS/nbis` 全部依赖辅助模型返回严格 JSON。辅助输出包含 reasoning 或格式偏差时，请求会在 DataFetch 前直接返回“证券实体识别结果不完整”；同一缺陷也让多条 ticker heartbeat 周期性失败。

### Follow-up Changes

- 普通大写 ticker 与证券语境中的小写 ticker 先形成词法候选；`今天nbis怎么样`、`NBIS最近怎么样` 和多 ticker 比较不再等待辅助 JSON。
- 候选不是实体真相：每个代码仍必须由本轮 DataFetch search 返回 exact symbol，之后才允许查同 symbol quote、financials 或生成结论；不接受搜索首条猜测，也没有公司别名硬编码。
- assignment key/value、`Q1`–`Q4`、行业/指标缩写和无关小写词不进入快路径。复杂公司名仍走结构化提取；结构化模型明确返回空数组时，不会重新塞回未经确认的大写 token。
- 辅助模型响应可包含 reasoning 或多个 JSON 对象，解析器选择最后一个完整 `entities` 对象；普通 ticker 已确定时，辅助解析失败不再阻断。
- “怎么样”纳入单股深度意图，因此最新 NBIS 问法继续执行九章节回答契约，而不是退化为草率行情短答。

### Follow-up Verification

- `cargo test -p hone-channels investment_response_guard --lib --no-fail-fast`：19/19 passed。
- `cargo test -p hone-channels --no-fail-fast`：494/494 passed。
- 投研 CI 契约：16/16 success。
- 真实 MCP：NBIS search 精确返回 `Nebius Group N.V.`，同代码 quote 为正，financials 返回 4 组非空数据。
- 全量 source runtime binaries（CLI、MCP、Web、iMessage、Discord、Feishu、Telegram）构建通过。
- 部署后临时 Web actor 端到端输入“今天nbis怎么样”：成功完成，正文 5036 字符，`1`–`9` 九个编号章节齐全，无实体错误或 stream error。
- 服务重启到 supervisor `62767`、backend `62779`；8077/8088、管理端/用户端均为 HTTP 200，Discord/Feishu/Web 各单进程在线，Postgres/OSS healthy，重启后 fatal/error 扫描为 0。

### Follow-up Risk

普通词和短 ticker 天然可能重名，因此词法层只决定“是否值得 exact lookup”，不决定证券身份。后续若扩展语境规则，应继续增加正反回归，不得恢复静态公司映射、首结果命中或把辅助模型当唯一 ticker 入口。

## 2026-07-16 资产类型与可见正文门禁阶段

### Root Cause

- FMP 没有宕机。`INTL` exact search/profile 明确表明它是 `Main International ETF`，quote 和 9 项 holdings 正常；公司 financials 返回成功空数组是 ETF 的合法语义。
- 实体优先管线最初仍把所有深度证券当公司，强制要求利润表，因而把 ETF 的合法空财务误报为“无法稳定核验本轮财务数据”。
- 修复资产路由后的首次真实回复又暴露了第二层边界：FunctionCalling runner 的 raw `AgentResponse.content` 保留 `<think>`，SSE 和最终出站则只展示清理后的正文。旧门禁在 raw 中先命中内部推理的 `1.` / `2.`，把完整公开答案误判为缺章节和时间。
- 时间门禁原本只接受字面量“数据时间/北京时间”，还会拒绝与当前报价同句的明确 provider 日期。

### Follow-up Changes

- exact entity + quote 后，按结构化字段独立路由 equity、ETF/fund 和 crypto；未知类型不靠空财务猜测。
- equity 使用 profile/meaningful financials/news；fund 使用 profile/holdings/news；crypto 使用 exact crypto market/quote/news。禁用的 company/fund 工具调用跨 retry 累积审计。
- 三类资产分别拥有实质性九章节契约；价格冲突、币种、时间、动作触发和混合资产比较都在最终正文上检查。
- 门禁先复用 `sanitize_user_visible_output` 取得与 SSE/最终持久化一致的 canonical visible content，再做章节和数字校验。raw response 仍可留在审计，不再污染用户正文契约。
- 数据时间同时接受明确标签、`截至 YYYY-MM-DD` 和贴在当前报价上的 provider 日期，但只检查前言与第 1 节，避免成立日或未来催化日期误命中。
- DataFetch 不缓存 semantic-empty evidence，并限制 FMP key 轮换到认证、配额和限流错误。

### Follow-up Verification

- `investment_response_guard`：32/32 passed；新增正反时间语义、fund/company/crypto、价格冲突和 tool audit 回归。
- `hone-channels` 串行全量：509/509 passed；新增 raw `<think>` + 完整可见基金答案的集成回归。
- DataFetch：24/24 passed；财经 CI：17/17 success。
- 真实 provider：NBIS 财务 4 期；INTL `isEtf=true/isFund=false`、quote 30.495、holdings 9、financials `[]`；BTCUSD `CRYPTO` quote 正常、stock profile `[]`。
- 最终生产 Web E2E “现在intl怎么看”：`deep_analysis=Fund`，HTTP 200、`success=true`、0 reset、一次完成；正文 1–9 节、北京时间、现价、基金目标与持仓证据齐全。
- 最终运行时 supervisor `63086`、backend `63098`；8077/8088、Postgres/OSS、Web/Discord/Feishu 均健康且各一个进程，最终启动后错误扫描为空。

### Follow-up Risk

- FMP ticker news 可能包含仅在正文中碰巧出现同字符串、但并非同证券实体的文章；后续应增加新闻实体匹配，不能仅凭 provider 的 symbol 字段把内容写成已核验事件。
- 长期可在 `AgentRunnerResult` 显式区分 canonical visible content 与 raw audit content，避免其它门禁重复做 sanitize；本轮最小修复已保证投资门禁与最终出站共用同一规则。

## 2026-07-16 投研回答校验后一次提交阶段

### Root Cause

- 投资契约在 runner 已把完整首稿流给客户端后才验证。误判会发送 `StreamReset` 并启动第二次生成，因此用户先看到一版正常结束的回答，随后闪烁清空并再次出现。
- RMBS 首稿实际使用了 `Forward PE` 和 `EV/EBITDA`，但估值方法识别只接受较窄的 `P/E` 别名；重试稿又把“对应股价/目标股价”当成本轮现价冲突。
- Web listener 已通过 `Done` 发送终止事件，route 尾部在失败时又补发一次 `run_finished`，形成双终止帧。
- 19:01 的自动 bug patrol 越界执行 `git restore`，清掉了尚未提交、但已在 18:36 运行二进制中的资产感知源码；部署前已从原始 Codex patch 记录逐文件恢复并核对 blob hash。

### Follow-up Changes

- `DeferredUserOutputEmitter` 在所有受投研契约保护的 runner 尝试外建立统一边界，只转发进度/工具状态，屏蔽草稿 delta、reset、thought 和尝试级 error。
- 空成功、瞬态失败、投资契约和 context overflow 的内部重试都复用该边界；最终清洗、附件处理和契约全部通过后，session 只提交一次 canonical `StreamDelta`，失败则由外层只发一个终态错误。
- Web route 删除额外失败 `run_finished`，`SseSessionListener::Done` 成为唯一终止事件来源。
- P/E 方法使用边界正则，`Forward PE`、`Forward P/E`、`目标 PE`、`PE 40x` 只计一种方法；当前价解析跳过对应/目标/隐含/折算股价，明确错误的当前价/最新价仍失败。

### Follow-up Verification

- `cargo test -p hone-channels --lib -- --test-threads=1`：510/510 passed。
- DataFetch：24/24 passed；Web SSE：3/3 passed；财经 CI 契约：17/17 success。
- 运行中四个二进制（CLI、Web、Discord、Feishu）重新构建；补齐失败终态单帧后最终完整重启到 supervisor `26981`、backend `26991`；8077/8088 正常监听，Postgres/S3 `ok=true`，Discord/Feishu 单进程在线。
- 真实 Web E2E “现在rmbs怎么看”：`assistant_delta=1`、`assistant_reset=0`、`run_error=0`、`run_finished=1`、`success=true`；最终正文 3709 字符、九节完整，现价 `$102.89`，Forward PE 与 EV/EBITDA 同时通过。
- 真实 Web E2E “现在intl怎么看”：同样一次正文、0 reset、0 error、一次成功终止；正文 2594 字符、九节完整并包含基金/ETF/持仓证据。

### Follow-up Risk

- 受保护的投研回答不再逐 token 显示，而是在终审后一次显示；这是避免向所有渠道泄露草稿与重试闪烁的明确延迟权衡。
- 普通非投研请求仍保持原生流式和工具分支 reset；不要把本边界扩展成全局禁流。
