# oldwang 内容整合 + 实测驱动的 harness 优化（2026-08-29）

## 用户要求

> 你再拉一下代码，看下 oldwang 有很多东西推上来了，基于这些完整的继续优化我们的 harness，
> 尤其是 skill、各个公司的核心观点和模式，并且推上线后进行下评测

## 先搞清楚 oldwang 是什么

`origin/oldwang` 从 `832e6b9e`（2026-08-11）拉出，此后 **main 走了 331 个提交、oldwang 走了 9 个**。
所以它**不是一个"更新"的分支**，而是一个两周前的旁支：有些内容比 main 新，有些比 main 旧。

已证实的两个方向：

| | 谁更新 |
|---|---|
| `references/kernel-manifest.md`、`CHANGELOG.md`、`evals.json`、`prompt.rs` 的三条新规则、`DeepAnalysisKind::EquityMove` 路由 | **oldwang 独有** |
| `company-cards.json` 的 MU / SNDK 卡片（main 已在 8/27 手工加深，oldwang 是 7/16 的通用版）、`stock_research`、`scheduled_task`、7 个视角 skill | **main 更新**（照搬会倒退） |

因此**全程禁止 merge / rebase / cherry-pick**，只按内容逐项移植。用 8 个 agent 分片盘点 + 8 个 agent
逐条证伪覆盖（`origin/main` vs `origin/oldwang` 双向比对），135 条候选里最终判定：
`port_adapted` 37 / `port_partial` 25 / `port_as_is` 1 / `skip_main_better` 46 / `skip_risky` 15 / `skip_irrelevant` 11。

## 但先做了一件更要紧的事：量出问题在哪

发现生产站的 `POST /api/chat`（loopback 8077，`{channel,user_id,message}` → SSE）可以直接跑真实 Agent。
用它把老王 8/22–8/28 评测里的 20 道代表题重放了一遍（含 3 道 8/10 的控制题），
再按老王点评归纳的评分卡逐维度打分。

**基线：平均 6.25 / 10。**

| 维度 | 均分（满分 2） | 0 分占比 |
|---|---:|---:|
| **C1 结论落到数**（合理股价区间 + 现价在区间里的位置） | **0.60** | **60%** |
| **C4 估值先定类型**（先定这是哪类公司 → 用哪套倍数 → 跨方法族交叉） | **0.65** | **47%** |
| **C5 Bull/Bear 量化** | 0.72 | 39% |
| **C3 稀缺度与差异化** | 0.82 | 18% |
| C2 胜负手 | 1.16 | 5% |
| C6 可执行（带阈值的触发条件） | 1.45 | 10% |

典型失败（q01 分析 MU）：回答里有毛利率、EPS、trailing/forward PE、EV/EBITDA，
甚至算出了「常态化 Forward PE 约 12x~15x」，**唯独差最后一步乘法**——没把常态化 EPS × 倍数
算成股价区间，也没说现价 $932.86 落在哪，最后只交付「持有区（置信度：中）」。

### 顺带量出了根因

把每题实际加载的 skill 和分数对上之后：

- 加载了场景/视角 skill 的 7 题：平均 **7.07**
- 只加载 `hari-invest` / `company-thesis-ratings` 或没加载 skill 的 13 题：平均 **5.81**
- 5 道几乎同形的「帮我分析一下 X」里，唯一加载了 `stock_research` 的那道 **8.5**，其余 4 道平均 **5.25**

原因是 `DEFAULT_HARI_INVEST_POLICY` 每轮强制加载 `hari-invest`，而 `hari-invest` 的资源路由
**只列自己的 references，从不指向别的 skill**。模型加载完它就停手，8/28 写进 `stock_research`
和 `valuation-audit` 的规则根本没机会生效。

## 改了什么

### 一、把 hari-invest 从"终点"改成"路由点"（最高杠杆，两处各几行）

- `skills/hari-invest/SKILL.md` 的资源路由前面加一张「本轮问题 → 除本 skill 外还要加载谁」的表，
  只给指针、不重写被指向 skill 的口径。
- `prompt.rs` 的 `DEFAULT_HARI_INVEST_POLICY` 加一条同义规则，覆盖不读 SKILL.md 正文的路径。

### 二、把 C1 / C4 补到最靠近调用点的两层

- `skills/valuation-audit/SKILL.md`：第四步加「收尾把倍数乘完」（下沿/上沿各一条算式 → 合理价值区间 →
  现价位置）；明确「两种方法指两个倍数族」，TTM P/E 与 Forward P/E 不算交叉；补区间对账
  （两法结论相反时写明以哪个为准，禁止「无论哪种方法都…」抹平）；反向估值要与已核验实际值逐项对照；
  三情景要披露四类输入；补分母为负写 N/M；缺一致预期时自建分母而不是放弃。
- `skills/stock_research/SKILL.md`：把「带算式区间」明确成**每股股价区间**（走市值/EV 口径要除稀释股本），
  且第 6 段与结论段必须是同一个区间、同一个现价。
- `investment_response_guard.rs` 的单股深度九段路由：§1 带上区间与现价位置、§3 拆成
  护城河/稀缺性/差异化三问、§5 区分结构改善与周期反弹、§6 先定类型+跨族交叉+反向隐含要求、
  §7 每档推到每股数字、§8 用历史基线证伪条件检验最新事实，并补一句反降级
  （次要数据缺失不得把整篇降级成「等待补数据」）。语义取自 oldwang，**去掉了它的 SNDK 门控**。

### 三、修好「aaoi为什么突然大跌」（基线 0.5/10）

日志定位：该轮只调了两次 `skill_tool`，**一次外部检索都没做**，于是
`agents/function_calling/src/lib.rs:6555` 的 market-move 终稿检查判定
`eligible_source_count=0 / cause_evidence_missing=true`，**直接跳过重试**、输出 242 字模板兜底。

而那条检查要求的「每个原因段落同段写出目标日期和原始 URL」**从来没有写进模型收到的提示里**——
提示里只有一个日期锚点块。这是 prompt 与 gate 的口径错配。

修法（纯提示，不新增 gate）：在模型已经收到的那个日期锚点块里写清终稿要求与后果，
并明确「先取证再动笔」的最小取证集（quote + 一次带绝对日期的外部检索，
`web_search / news / press_releases / sec_filings / analyst_actions` 任一——留了非 Tavily 的路径）。
`skills/market_analysis/SKILL.md` 同步补 aliases（「美股今天为什么夜盘就跌个没完」原先命中 **0 个 skill**）、
证伪信号不得用股价技术位、归因题不输出买卖点。

### 四、各个公司的核心观点和模式（用户点名的重点）

52 张公司卡里 **49 张是模板**：

- `valuation_method` 只有 11 种文本，**24 家共用同一句**（台积电、Vistra、Cisco、BWXT、SBET 都用它）
- `watch_items` **45 张只有 1 条**，而且是生成脚本留下的元信息（「当前价格/市值取自公开行情；逐字稿强调…」）
- `falsifiers` **46 张共用同一对通用句**

按估值原型分 11 组，每组一个 agent 改写 + 一个 agent 逐条证伪，重写这三个字段：
`valuation_method` 写成「属于哪一类 → 用哪套倍数 → 拆什么 → 第二种方法交叉 → 一句反模式」，
`watch_items` 写成 3 条「可观测量 + 核验口径 + 为什么会改判断」，`falsifiers` 写成 2 条该公司特有的可观测事件。

红线自查全部通过：**0 处当前数值**（股价/市值/份额%/毛利率/收入额/增速/未来日期）、0 处目标价或买卖动作、
`valuation_method` 0 处重复、falsifiers 0 处旧模板残留；MU / SNDK / RKLB 三张已加深的卡未动，
其余 14 个字段（含 `source_updated_at`，因为**本轮没有新素材**）逐字段断言未变。

配套：
- 修 4 处 theme 错标（SBET「电力/液冷」→ 数字资产财库、STX/WDC「其他」→ 存储/HDD、ADEA「主题观察」→ IP授权）。
  theme 会驱动 `company_ratings.rs` 的同业组与毛利假设。
- `build_company_cards.mjs` 顶部标注为 2026-06-20 的一次性 bootstrap，并写明重跑会静默覆盖全部手工加深
  且不会有任何测试报错。
- `company-thesis-ratings/SKILL.md` 输出结构 6 段 → 8 段（护城河 / 产业稀缺性 / 公司差异化拆成三段独立回答），
  并补卡片三字段的分工纪律。

**成本**：注入系统提示的单卡平均 335 字 → 565 字，命中 1 家约 +161 token，命中上限 8 家约 +1286 token。

### 五、一个真 bug：extended_hours 的日涨跌分母

`pct_change_vs_prev_session_close` 逐个相邻窗口计算，对 regular 窗口来说**分母是当日盘前收盘**，
不是上一常规收盘。而 `hone_session_policy` 和 quote 的 `cannot_prove` 文案都在指示模型
"常规时段涨跌另取这个字段"——等于主动把错的那个数指为日涨跌。

用真实 MRVL 数据钉死：正确 -5.57%，错误口径 -6.12%。

修法：regular 窗口增加 `pct_change_close_to_close` + `previous_regular_close`，
pre 增加 `pct_change_vs_previous_regular_close`，post（且同日）增加 `pct_change_vs_regular_close`，
各自带 `canonical_change_basis`；三处指令文案同步改指新字段；
合并成一个 `hone_session_policy` 而不是并列两个互相矛盾的策略串；
补回归测试 `regular_daily_change_does_not_use_the_premarket_close_as_denominator`。

### 六、其余 skill 的小改

`fundamentals`（首次写 GAAP 盈利就交代成色、受一次性项目污染的口径标注了也不能当锚；
领先指标要补历史转化率）、`analyst-coverage`（"机构"指真正出研报的卖方，
Seeking Alpha/Zacks/TipRanks 只算来源；结尾要把 `targetMedian` 换成隐含倍数接回自己的判断）、
`etf-analysis`（点名两件事落在服务端九段的哪一段；前十持仓分 A/B 两档取数共 15 次调用、降级口径必须写出来）、
`hari-invest/references/provenance.md`（补「当前事实截止：每次问答重新确定」；修 main 现存的
0.1.0 vs 0.2.0 版本号不一致；并入 A–E 来源分层表）、`logic-index.md`（产品方法与候选隔离）。

## 明确没有移植的

- **`symbol == "SNDK"` 的专项提示词**与 **`missing_sndk_deep_logic` 关键词校验器**（9 处引用）。
  前者是 ticker 硬编码，与用户「要扩展性的，不是硬代码」直接冲突；后者是 `AGENTS.md` 141–154 行
  明令禁止的关键词命中式内容门禁。**语义已泛化进 skill 层，实现一行没进。**
- **oldwang 的「没有一致预期时给 12x/15x/18x 机械反向门槛」**：与 `valuation-audit`「倍数必须本轮推导、
  类型表只是选区间的先验」直接冲突，且它自己前半句要固定倍数、后半句禁止固定倍数。
  改成「用一手数据自建分母，倍数仍走三问推导」。
- **oldwang 的 7 步传导链 / 8 步流程进 hari-invest**：逐环节被 main 8/28 的
  `first-principles` / `moat` / `scarcity-differentiation` / `fundamentals` / `valuation-audit` /
  `stock_research` 覆盖且更可判定。搬过去等于同一规则两处口径。
- **oldwang 的公司卡**（比 main 旧）、**stock_research / scheduled_task**（比 main 旧）。
- **`kernel-manifest.md` 的「代码强制点 / 默认调用顺序 / 验收资产」三节**：点名的
  `investment_decision_context.rs` 统一点时决策状态、`evals.json` runner、SNDK 491 轮台账
  在 main 都不存在或未接线，照搬等于写入不实文档。只取了 A–E 来源分层表。
- **`web_search` 的 DuckDuckGo 零密钥兜底**（280 增 / 116 删）：见下方「留给你决定的事」。

## 发布

一次同版发布二进制与 harness，因为 **`company-cards.json` 被 `prompt.rs:116` 用 `include_str!`
编进二进制**——卡片改动不随 harness bundle 生效，必须重新构建镜像；同时磁盘上的
`/srv/honeclaw/skills/.../company-cards.json` 又会被 `skill_tool` 读到，两份不同版会让模型看到两套卡。

- 提交 `b428ca09`，镜像 `sha256:a896eb90…`，`[PASS] verified`。
- harness bundle 60 文件 SHA-256 清单，sha `8b8bc87897691b8b16f962d8` 本地/远端一致；
  dry-run 报 8 个 skill 变更、`soul.md unchanged`；备份在
  `/srv/honeclaw/skills/backups/pre-b428ca09…-20260829T045831Z`。
- 两次 `active-chat-runs` 均 `{"count":0}` 后原子换链，`previous` 指向 `897f66fd`。
- 核验：exe 指向 b428ca09、`NRestarts=0`、`cloud_mode=cloud`、
  `local_durable_dependency_count=0`、40 skills、公网 401、`journalctl -p err` 无条目；
  二进制里逐条 grep 到新串（含「数字资产财库」「带零售对冲的重资产发电商」，
  证明新卡确实编进去了）。清理最老 release，磁盘回到 4.6G。

## 测试

- `hone-tools --lib --test-threads=1`：**201 passed / 5 failed**（比基线多一个通过的新测试，5 条失败仍是既有的同 5 条 `skill_tool::tests::*` 漂移）
- `hone-channels --lib`：**826 passed / 40 failed**，与暂存基线逐条相同；`prompt::tests`(21) 与
  `investment_response_guard::tests`(138) 单独跑全绿。
- `hone-core --lib`：161 / 1，失败的 `config::tests::soul_prompt_keeps_the_full_investment_contract`
  **把我的改动 stash 掉后同样失败**，是 main 上的既有问题，不是本轮引入。
- `cargo fmt`：我改的三个文件干净；`agents/function_calling/src/lib.rs`、`hone-llm/`、
  `hone-web-api/src/routes/chat.rs` 有既有格式漂移，未动。

## 留给你决定的事

1. **Tavily 配额天天打满**。你们自己的 `docs/bugs/web_search_tavily_payg_quota_exhausted_degrades_realtime_research.md`
   记录了连续多天每 4 小时窗口 19–35 次 `pay-as-you-go limit`，且 `key_count=1`。
   这会让本轮「异动题必须先取证」的要求更难满足。oldwang 有一份 DuckDuckGo 零密钥兜底
   （`web_search.rs` 280 增 / 116 删，抓 DDG HTML）。我没有上：它把用户查询发给一个新的第三方，
   属于对外行为变更，应该你来定；而且真正的修法可能是加 key 或提额度。
2. **market-move 的兜底仍然是硬失败**。`cause_evidence_missing=true` 时代码直接跳过重试。
   本轮的提示修法是让模型别走到那一步，但如果它仍然一次检索都没做，用户还是会拿到模板。
   要不要让这种情况重试一次（让模型去补检索）需要你确认——它会改热路径的循环行为。
3. **`hone-cli chat --once --json --actor-id`**（oldwang 有、main 没有）没有移植：
   本地没有可用 key，而生产的 `POST /api/chat` 已经能做同样的评测，成本更低。
4. **NVDA、TSLA、ORCL、AAPL、HPE、APLD、MSTR 没有公司卡**。评测 131 题里 NVDA 出现 5 次、TSLA 6 次。
   补卡需要授权逐字稿，我不能凭空造。
5. **英文短片段的路由误命中是既有问题**：`portfolio_management` / `position_advice` /
   `deep_stock_research` 的 `when_to_use` / `description` 里有 `on` 这样的两字母片段，
   会被「帮我写个 python 脚本」这类问题子串命中。本轮没动这三个文件。

