# 宏观指标一等实体化 + 实体解析不再毁整轮（Codex 交接书）

- status: `in_progress`
- created_at: `2026-08-17`
- owner: Claude（计划/协调/验收）、Codex（实现）
- base revision: `32d3295c`
- 并行方式：4 条独立 git worktree，各自一个 commit，最后由 Claude 合并

## 背景（不要改变这个判断，除非代码打脸）

定时任务把 `PCE`、`ETF` 这类非证券缩写当股票代码去查行情，查不到就**整轮 `return Err`**，
把「已识别证券代码"PCE"…请稍后重试」当作客户订阅的报告推出去。

### 2026-08-17 生产实测（Claude 直查 GCE PG + journal，已定案）

| 事实 | 数字 |
|---|---|
| 近 3 天硬失败 | **2 次**（`PCE` 08-16 20:01、`ETF` 08-16 20:32，均为 web 渠道客户） |
| 同期正常降级到 Agent 发现 | **171 次** `entity_resolution.agent_loop` |
| `cloud_cron_job_runs.detail` 里该错误 | **全历史 0 条**（数据库完全看不见） |
| 近 14 天 execution_failed | 844（heartbeat 452 / 非 heartbeat 392） |

结论：`tentative_symbol` 防线基本在起作用（171:2），**问题是防线有洞，不是没有防线**。
不要推翻现有架构。

两个失败任务：
- `j_e447df29`「每日20点美股盘前要闻与持仓事件摘要」，片段：
  `…1. 当日/近期重要宏观数据与预期差：就业、非农、初请、CPI、PCE、ISM、零售销售等；2. 美联储和利率相关…`
- `j_047f5da6`「持仓每日新闻与月度复盘」，片段：
  `这是用户持有股票和ETF的每日新闻汇总与月度持仓复盘任务。北京时间每天08:00执行。请先读取用户当前持仓与关注列表，覆盖美股持仓 TEM、TSLA、FNMA、DBRG、ABSI、SGOV、SBET、RXRX、IBKR、AIRO，以及A股ETF 512690、563020、159797、515180；…`

**重要：Claude 按代码逐条推演，这两段文本都不该命中任何 `bound_to_a_security` 分支。**
所以真凶未知——可能在 prompt 后段，也可能是 scheduler 组装 runtime input 时加进去的内容。
**Track A 的 example 就是用来把这个问题变成 2 秒可查的。不要靠猜。**

---

## 绝对红线（违反即打回）

1. **不得引入任何"非证券缩写"黑名单/否决表。**
   `docs/invariants.md:93` 明确禁止：手工缩写表曾经静默删掉 11 个真实上市代码
   （ARM、NOW、ON、AA、BE、IT、BB、AS、OR、GOOD、BULL）。
   允许的唯一动作是**把候选标记为 `tentative_symbol = true`**（即"未确定"），
   由 Agent 读完整请求定夺。**标未确定可以，删候选不行。**
2. **以下既有测试必须保持绿，不得修改、不得放宽断言**
   （文件均为 `crates/hone-channels/src/investment_response_guard.rs`）：
   - `scheduled_scans_keep_real_listings_that_look_like_common_words` (~:13741)
   - `explicit_dollar_symbols_are_preserved_without_acronym_denylist` (~:12459)
   - `scheduled_and_heartbeat_skip_macro_regulatory_and_name_components` (~:13805)
   - `a_bound_uppercase_ticker_stays_an_explicit_code` (~:16683)
   - `clause_subject_grammar_alone_yields_a_tentative_seed_not_an_explicit_code` (~:16669)
3. **每个 bugfix 至少一个回归测试**（`AGENTS.md:203/213-216`），且测试必须**真的守得住**：
   写完后自己做一次变异验证——把你的改动注释掉，对应测试必须转红。
   （历史教训：曾有断言用合规输入去断言 `None`，因提前 return 恒真，守不住任何回归。）
4. **遇到与本文冲突以代码为准**，回来改本文，不要为了贴合计划写多余代码或兼容层。
5. **不要过度设计**：不加配置开关、不加 feature flag、不留向后兼容分支。

## 环境事实（本机）

- **没有 `timeout` 二进制**，不要用它包命令。
- **`PATH` 里没有 `rg`**，用 `grep -rn`。
- 主 worktree 是 `/Users/zhangxuanren/Workspace/honeclaw`，**不要动它**，只在自己的 worktree 里干活。
- 门禁命令：
  ```
  cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
  ```
  `hone-desktop` / `hone-user-app` 必须排除。跑完贴出 `passed=N failed=N`。
- 单包快速迭代用 `cargo test -p hone-channels`。

---

## Track A —— L0 可观测（worktree `../honeclaw-l0`）

**目标：让这类 bug 从"翻 journal"变成"一条 SQL + 一条命令"。本 track 不改任何判定逻辑。**

### A1 实体扫描诊断 example

新增 `crates/hone-channels/examples/entity_scan_explain.rs`（仿同目录既有
`heartbeat_prompt_llm_smoke.rs` 的写法）：

```
cargo run -p hone-channels --example entity_scan_explain -- --origin scheduled "<prompt 原文>"
```

输出每个候选一段，必须包含：
- `mention` / `normalized symbol` / `SecurityIdentifierKind`
- 全部绑定标志的**真值**：`exact_input`、`explicit_ticker_label`、`explicit_ticker_binding`、
  `strong_exact_shape`、`direct_market_binding`、`chinese_analysis_binding`、
  `english_analysis_binding`、`comparison_binding`、`symbol_cluster_binding`、
  `clause_subject_binding`、`numeric_market`、`numeric_asset`、
  `bound_to_a_security`、`unsettled_without_a_reader`、`only_clause_subject_support`
- 最终 `tentative_symbol`
- 若候选被 `continue` 丢弃，打印**丢弃原因**（哪个判断命中）
- 末尾打印 `extract_entity_scope` 的最终 scope（`Securities` / `AgentToolDiscovery` / `Portfolio` / `Broad` / `PassThrough`）

实现要求：
- 在 `plain_ticker_mentions`（`investment_response_guard.rs:8370`）里把现有局部变量收进一个
  `#[derive(Debug, Default)] struct MentionTrace`。**不得改动任何判定表达式本身**，
  只是把已经算出来的值另存一份。
- 提供 `pub(crate) fn explain_entity_scope(input: &str, origin: AgentTurnOrigin) -> ScopeExplain`，
  再由 example 通过一个 `pub` 薄封装（放 `lib.rs` 或新的 `pub mod diagnostics`）调用。
  `pub` 面尽量小。
- 正常路径**零额外开销**：trace 只在 explain 入口里收集（例如让 `plain_ticker_mentions`
  接受 `Option<&mut Vec<MentionTrace>>`，热路径传 `None`）。

**验收**：`cargo run -p hone-channels --example entity_scan_explain -- --origin scheduled "股票代码 AAPL 现在多少钱"`
能打出 `explicit_ticker_label=true`、`tentative_symbol=false`、`scope=Securities`。
再加一个单测断言这两条输入的 trace 关键字段。

### A2 实体解析失败落库

`crates/hone-web-api/src/routes/events.rs:508` 已经拿到错误文案，
终态写入点在同文件 `:261-298`。让 `cloud_cron_job_runs.detail` 带上：

```json
{"scheduler": {"failure_kind": "entity_resolution_unresolved", "unresolved_mention": "PCE"}}
```

- 从错误文案里提取被误判的 token（文案格式：`已识别证券代码"{}"，但当前数据供应商…`）。
  **不要靠正则猜**——优先让 `prepare_verified_investment_turn` 的错误带结构化信息；
  若改动面太大，退而在 `events.rs` 侧解析，但要写清楚这是权宜之计。
- `detail` 是 JSONB，注意既有写入是整体覆盖还是 merge，跟随现状。
- 回归测试：给定一个实体解析失败的错误，写出的 detail 含上述两个字段。

---

## Track B —— L1 Unresolved 不再毁整轮（worktree `../honeclaw-l1`）

**这是止血层，单独就修掉 PCE 和 ETF 两个生产失败，且对未来任何新缩写自动生效。**

`crates/hone-channels/src/investment_response_guard.rs` 三处
`EntityMatch::Unresolved => return Err(...)`：
- `:3678` 数字码分支
- `:3723` 显式码分支（生产上 PCE/ETF 走的就是这条）
- `:3761` 名称分支

现状是**扇出放大器**：一个候选查无此物 ⇒ 整个 `prepare_verified_investment_turn` 返回 `Err`
⇒ `crates/hone-channels/src/agent_session/core.rs:1459` 变成 turn 终态错误，Agent 一次都没跑。

### 改法

- 未解析的 mention **从实体集合中剔除，并记入契约的缺口披露**，不再 `return Err`。
- 只有当**全部** mention 都未解析时才降级——降级目标是
  `EntityResolutionScope::AgentToolDiscovery`（让 Agent 读完整请求自己判断），
  **不是** `Err`。这与 `docs/invariants.md:93` 规定的兜底路径一致。
- **必须保留的语义：绝不把未解析的代码映射到别的证券。**
  这才是原设计要守的东西，`return Err` 只是它的实现手段。剔除 ≠ 映射，不违反不变量。
- 缺口披露复用既有惯例：`InvestmentResponseContract` 已有
  `canonical_fact_block()`（~:428）和 `retry_block()`（~:541）。按同风格加一行，例如：
  `本轮未能核验的候选：PCE（无同代码行情覆盖，未按证券处理）`。
  字段建议加在契约结构体上（如 `unverified_mentions: Vec<String>`），由 block 渲染。

### 回归测试（新增）

1. 三个候选里一个未解析 ⇒ 另外两个正常成契约，未解析的出现在缺口披露里，**不返回 `Err`**。
2. 全部未解析 ⇒ 降级 `AgentToolDiscovery`，**不返回 `Err`**。
3. 三个 `Unresolved` 分支各覆盖一次（数字码 / 显式码 / 名称）。
4. **变异验证**：把改动改回 `return Err`，上述测试必须转红。

---

## Track C —— L2 宏观指标一等实体（worktree `../honeclaw-l2`）

### C1 词典：新增 `crates/hone-core/src/macro_indicator.rs`

放 hone-core，与同类"共享真值源" `crates/hone-core/src/provider_symbol.rs` 并列。

```rust
pub struct MacroIndicator {
    pub canonical: &'static str,      // "core_pce"
    pub display: &'static str,        // "核心PCE物价指数"
    pub agency: &'static str,         // "bea.gov"
    pub aliases: &'static [&'static str],
    /// 该缩写同时是真实上市代码（如 ADP = Automatic Data Processing）
    pub collides_with_listing: bool,
}
```

覆盖（中英别名都要）：
`非农/大非农/nonfarm/non-farm/NFP`、`小非农/ADP就业/ADP employment`、
`核心PCE/core PCE/PCE`、`CPI/消费者物价指数`、`PPI`、
`FOMC/议息会议/利率决议/interest rate decision`、`GDP`、
`ISM/ISM制造业/ISM服务业`、`初请失业金/jobless claims`、`零售销售/retail sales`、
`失业率/unemployment rate`、`消费者信心/consumer confidence`。

`collides_with_listing`：`ADP` = **true**（真实代码）。`非农`、`FOMC`、`初请失业金` = false。
其余项 Codex 自行判断时**宁可标 true**（保守 = 保留候选）。

**种子数据已存在，照抄不要重写**：
- `crates/hone-event-engine/src/pollers/macro_events.rs:17` `DEFAULT_HIGH_MACRO_KEYWORDS`（英文全套）
- `crates/hone-web-api/src/routes/public_finance_calendar.rs:389` `macro_seed_events()`
  （中文名 + 权威来源域名 bls.gov / bea.gov / federalreserve.gov / ismworld.org / census.gov）

**本轮只建立 hone-core 这一份并给 hone-channels 用。那 4 处重复词表的收敛是后续项，不要动。**

### C2 独立扫描器，**不要**给 `SecurityIdentifierKind` 加变体

理由是硬的，别绕：
- `normalize_and_classify`（`crates/hone-channels/src/security_identifier.rs:277`）
  对非 ASCII 直接 `return None` ⇒ `非农` / `议息会议` **根本不可能**成为 `SecurityIdentifier`。
- 给枚举加变体会波及 `investment_response_guard.rs:8438` 那个穷举 `match` 和 8 处 `!= Bare` 判断。

⇒ 实现 `pub fn scan(input: &str) -> Vec<MacroMention>`，
`MacroMention { start: usize, end: usize, canonical: &'static str, collides_with_listing: bool }`。
- 别名匹配需处理大小写与词边界；ASCII 别名要求**大小写不敏感但有词边界**
  （`PCEX` 不算命中 `PCE`），中文别名直接子串匹配。
- 重叠时取最长别名。

### C3 在 `plain_ticker_mentions` 挂两个钩子

**(a) 拆掉连坐。** 命中宏观词典的 token **不计入**
`identifier_has_symbol_cluster_binding`（~:9238）的 `>= 2` 法定人数。
现在真代码给假代码背书的通道就是这条——持仓越多越容易中招。

**(b) 强制未确定。** 命中宏观词典 ⇒ `tentative_symbol = true`，
压过 `bound_to_a_security`（~:8469）。

**唯一例外**：`has_explicit_ticker_label`（`股票代码 ADP`）是最强信号，
用户明写代码标注时宏观词典不得干预。

**再强调一次：只标 tentative，绝不 `continue` 掉候选。**

### 回归测试（新增）

1. `ADP 就业数据低于预期` ⇒ tentative / 非 `Securities`；
   `股票代码 ADP 的财报` ⇒ 非 tentative / `Securities`。同一 token 两种结果。
2. 上面两个生产任务片段（本文背景章节里的原文）⇒ scope 不是 `Securities`，且不产生 `Err`。
3. 连坐豁免：构造「有市场词 + 1 个真代码 + 1 个宏观词」的子句，宏观词不得把真代码凑成法定人数。
4. **不得回归**：红线第 2 条列出的 5 个既有测试全绿。
5. **变异验证**：分别注释掉 (a) 和 (b)，各自必须打红对应测试。

---

## Track D —— L3 宏观取数打通（worktree `../honeclaw-l3`）

**只做取数层，不碰 `investment_response_guard.rs`**（避免与 B/C 冲突）。

### D1 `data_fetch(data_type="macro")` 目前不可达 —— 真 bug

`crates/hone-tools/src/data_fetch.rs`：
- `:506` `"macro"` 分支实现完整（treasury-rates + GDP/CPI/失业率/联邦基金），有 TTL（`:965`），
  有测试（`:4109`），`description()`（`:2122`）也向模型宣传了它；
- 但 `parameters()` 的 `r#enum`（`:2137` 起）里**没有** `"macro"`，
  而该 enum 会原样进 JSON schema（`crates/hone-tools/src/base.rs:62`）⇒ 模型根本调不到。

→ 补上 `"macro"`。
**只补 `macro`。**（另有 8 个同样缺失：`valuation`、`segments`、`peers`、`ownership`、
`corporate_actions`、`press_releases`、`transcript`、`market_hours` —— 本轮不动，
但在 commit message 里记一笔。）

回归测试：断言 `parameters()` 里 `data_type` 的 enum 覆盖了 `data_fetch_urls`
实际支持的全部分支（这样以后漏一个就红）——**这条测试现在会因为那 8 个而失败，
所以先写成只断言 `macro` 在内，并留 `// TODO` 注明完整版本。**

### D2 宏观发布日历

现有 `"macro"` 分支只取存量序列。盘前要闻真正需要的是
「今晚 20:30 公布，预期 x，前值 y」——即 `/v3/economic_calendar`。
`crates/hone-event-engine/src/pollers/macro_events.rs:90` 已经在用同一个端点，照抄参数与解析。

→ 给 `"macro"` 分支加一个 `economic_calendar` 条目，窗口取 `today .. today+7d`。
TTL 跟随既有 `ttl_for_data_type` 的宏观档位。

---

## 交付要求（每个 track）

1. 一个 track 一个 commit，**做完停下等 Claude 评审，不要连着做下一个 track**。
2. commit message 写清：改了什么、为什么、变异验证做了哪几处、哪几处必须转红。
3. 贴出门禁输出的 `passed=N failed=N`。
4. 如果发现本文的判断与代码不符，**先停下来在本文的「实施记录」章节写明分歧**，不要自作主张扩大改动面。

## 实施记录

### Track D — L3 宏观取数打通

- status: `done`
- 实现：仅在 `data_fetch` 的 `data_type` schema enum 中补充 `macro`；macro bundle 新增
  `economic_calendar`，沿用 FMP `/v3/economic_calendar`，窗口为 UTC
  `today..today+7d`，继续复用既有 macro TTL、JSON 解析和 coverage。
- 范围：未改动其余 8 个已知 schema 缺口；未改动
  `crates/hone-channels/src/investment_response_guard.rs`。
- 回归：`cargo test -p hone-tools macro_ -- --nocapture` →
  `passed=2 failed=0`。
- 变异验证：临时删除 `macro` enum 项时 schema 回归测试
  `passed=0 failed=1`；临时删除 `economic_calendar` component 时窗口/端点回归测试
  `passed=0 failed=1`；恢复后两条均转绿。
- workspace 门禁：已运行
  `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`；
  已执行的 test-result 合计 `passed=821 failed=172`（`ignored=1`），在
  `hone-channels` 因 PostgreSQL 无法连接、沙箱禁止 loopback FMP stub 绑定及其导致的
  env-lock poison 级联而停止。
  前置 `bash scripts/dev_pg.sh up` 返回 Docker 不可访问；Track D 定向测试无失败。
- 分歧：计划描述与代码实现没有发现分歧；全量门禁失败属于当前执行环境前置条件。
