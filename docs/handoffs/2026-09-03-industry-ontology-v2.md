# 行业本体 v2：上游信号、在线编辑入口、面向那几十家核心公司的落地（2026-09-03）

## 用户要求

> 这块行业分析整体重构一下，需要启动一版，然后让管理员用户可以快速的更新，可以直接线上编辑，
> 也有比较明确的入口……很多估值不准确、很多公司的理解也不对，原因是没有构建一层类似于本体行业
> 的概念。比如 AI 基础设施跟英伟达息息相关，但很多回答都跟英伟达最近的行为没关系……
> 这几十家公司是我们最最最重点的，用户来问这些公司的估值、分析、财报，能够根据我们做的
> 这些行业的分析来回答。

## 先量出「跟英伟达最近的行为没关系」是不是真的

用 8/30 那六道成员公司的生产回答（RKLB / VST / CRDO / MU / SNDK / NBIS）做基线：

| 题 | 提及英伟达 | 接到英伟达最近财报/指引 |
|---|---:|---|
| rklb的估值 | 0 | 否 |
| vst现在贵不贵 | 0 | 否 |
| CRDO 值得买吗 | 0 | 否 |
| 给我讲讲MU的投资逻辑 | 2 | 否 |
| SNDK 的合理估值是多少 | 0 | 否 |
| NBIS 这家公司怎么看 | 3 | 否 |

**0/6。** 光通信那一行的 `core_watch` 明写着「看英伟达财报」，CRDO 的回答仍然一次没提。

同日在生产上又跑了一遍 8 题的隔离基线（`ont0`：RKLB / VST / CRDO / MU / SNDK / NBIS / COHR / AAOI，
actor `harness-eval-ont0-*`）：**0/8** 接到英伟达最近一季的实际动作——MU、NBIS、COHR 提到了「英伟达」
（2、3、1 次），但没有一题写到它最近一季的收入、指引或管理层原话，也没有一题带日期。
散文里的「关注」不会变成取数动作——这是本体缺的那条边。

## 改法：三件事

### 一、本体的边：`upstream_signals`

每个行业新增结构化字段：这一行的收入由哪几家**上市公司**的最近行为决定，以及写这一行的公司
之前该先取它们的哪几个读数。

```json
{ "symbol": "NVDA", "name": "英伟达", "relation": "demand_source",
  "why": "…落到 driver_chain 的具体环节…",
  "pull": ["最新一季数据中心收入与下季指引", "毛利率与毛利率指引", "电话会里关于 HBM 采购/规格的表述"],
  "cadence": "季度财报（2/5/8/11 月）" }
```

`relation` ∈ `demand_source` / `capex_source` / `supply_gate` / `peer_signal`。全部是美股代码——
必须是现有工具能取到财报的公司，不然写了也取不到。

注入（`prompt.rs::industry_baseline`）随之从「关注谁」改成「先去取什么」：每条信号渲染成
「写这家之前先取 X —— 取法 `data_fetch(earnings_outlook)` / `analyst_actions` / `transcript`」，
抬头加一句：带上游信号的行，需求侧第一段先写上游**最近一季实际做了什么**，再沿传导链传到这家；
没取到就明写未取到，不得略过、不得用记忆里的旧季度代替。`industry-map` skill 把它写成第 0 步，
`valuation-audit` / `fundamentals` 各接一句（三情景的经营输入、增长来源那一问先看上游最近一季）。
**没有加门禁**：不检查「有没有调过 earnings_outlook」，那是 validator 驱动的循环改写。

### 二、管理员直接线上编辑 + 明确入口

- 后端 `POST /api/public/industry-map/edits`，`is_web_admin` 把门（非管理员 403），body 与对话工具
  写进日志的是**同一个 `EditOp`**（有测试钉住 12 种 `kind` 都能反序列化）。响应带回完整快照，
  页面整体替换本地状态，「最近改动」卡片与树上的标记跟着一起动。
- 本体可编辑的范围补齐：新增/移除行业（移除只是让重放跳过，底稿不动、可恢复）、
  新增/移除上游信号；对话工具 `industry_map_edit` 同步加这四个 action。
- 页面：管理员看到「编辑本体」开关，进入后每一块就地改；研究台那张卡移到管理组第一张，
  kicker「本体 · 可在线编辑」。

### 三、面向那几十家公司

树里只收美股与 ADR（61 家）。每一家被问到时：公司卡（历史基线）+ 行业块（传导链、倍数锚、
反模式、上游信号）+ 上游最近一季的实际数——三层一起进同一轮。

## 注入预算与页面验证

- 底稿里每行 3–4 条上游信号、每条 `pull` 三到五个读数，整段注入会把两行的开销从约 1100 token
  推到 3700。`industry_baseline` 只注入两条（NVDA 永远第一）、每条前三个读数各截到一句、`why`
  截到 90 字，两行合计守在 2600 字以内（`industry_baseline_pulls_nvda_first_and_keeps_the_upstream_block_compact`）。
  完整版留给研究台页面和 `industry-map` skill。
- 页面编辑链路在本地用真实 `hone-console-page`（dev-login 管理员、隔离 `HONE_DATA_DIR`）走通：
  开关 →「改动说明」→ 改 `one_liner` 保存 → 绿色回执 + 「最近改动」卡 + 树上红点；再从表单新增
  一条上游信号（INTC，capex_source，两行 `pull`）→ 回执「新增上游信号 INTC」，卡片计数 2。两条都落在
  `industry_map/edits.json`，`by` 是网页用户 id，与对话里 `industry_map_edit` 写同一份日志。

## 发布与复测

- revision `91e78e74c197c6ea27ad7283e18e6ce939cf95a9`，镜像
  `ghcr.io/b-m-capital-research/honeclaw-runtime@sha256:001b0d52de780389b7a93eb505a4fcfae9e53503217906019d022d9af08369a3`
  （主机 `crane digest` 取得）。2026-09-03 12:46 UTC 切换：两次 `active-chat-runs` 均为 0，
  `current` → 91e78e74，`previous` → e7f855a9，`systemctl restart hone-web` 后 `NRestarts=0`，
  `/api/meta` git_sha 对上，二进制里 `上游信号 ·` / `最近一季实际做了什么` / `add_upstream_signal` 都在。
- harness 同包发布：`fundamentals` / `industry-map` / `valuation-audit` 三个目录换新，`soul.md` 未动，
  备份在 `/srv/honeclaw/skills/backups/pre-91e78e74…-20260903T124648Z`。`/api/skills` 42 项，
  `industry-map` 在；落盘的 SKILL.md 含「先取上游」，底稿 8 行都带 `upstream_signals`。
- 未登录探针：`GET /api/public/industry-map` → 401；`POST /api/public/industry-map/edits` 空体 → 422
  （axum 的 `Json` 提取器先于鉴权拒掉畸形体；合法体未登录才是 401。不泄露任何东西，但下次顺手改成
  先鉴权再解析）。
- 复测 `ont1`（同 8 题、同 actor 前缀，12:47–12:55 UTC）：**0/8**，与基线持平。工具日志
  （`runner.tool` 只记 `tool=data_fetch <kind>`，不记 ticker）显示 `earnings_outlook` 从 0 次涨到 4 次
  （CRDO / MU / NBIS / AAOI 各 1），MU、NBIS 的回答里「英伟达」提及从 2、3 次涨到 5、6 次，但没有一题写出
  英伟达最近一季的收入、指引或毛利率数字，也没有一题按注入要求写「本轮未取到 X 的最新财报」。
  用单测把注入原文打出来核对：块是完整进了系统提示的（RKLB 不在树里，其余 7 家都命中），
  问题在于**它给的是「去取什么」的指令，事实只藏在传导链和被截断的 `why` 里**——模型对指令不理，
  对手边的事实才会用。

## v2.1：把事实放在指令前面（同日跟进）

- `UpstreamSignal` 加 `latest`（带数字、带日期的一段：它最近一季实际做了什么）与 `latest_as_of`；
  新 op `set_upstream_latest{symbol, latest, as_of}`（页面 LatestEditor + 对话工具
  `industry_map_edit(action="set_upstream_latest")`），每季财报后管理员只改这一段。
- 注入重排：上游块移到传导链**之前**，先一行「上游最近动作 · NVDA（demand_source，截至 2026-08-26）：
  FY27Q2 数据中心 $89.0B（+117%）、Q3 指引 $108B、毛利率 75.0%→74.0%、Supply and capacity 承诺
  $119B→$279B…」，再一行「写这家之前先核对 NVDA 有没有更新的一季：data_fetch(...)」。头部改成
  「需求侧第一段就从那条最近动作写起（日期和数字都带上）…没有更新就照本体这条写并注明截至日期」。
  两行合计预算约 1,900 token（`industry_baseline_pulls_nvda_first_and_keeps_the_upstream_block_compact`
  守 3,000 字）。
- 底稿：8 行的 NVDA 都写了 FY27Q2（2026-08-26）的最近动作，按行各带一句（电力：PORTS 园区 1,050 亿担保；
  新云：担保敞口与 AI cloud agreements；设备：承诺措辞 primarily memory and manufacturing facilities…）；
  存储行的 MU 写了 FY26Q3 毛利率 84.9% / 指引约 86%。数字全部来自种子文件里已带日期的事实。
- 发布：revision `1950b5b864c060039eabc1c31df771c26bbbd988`，镜像
  `…honeclaw-runtime@sha256:8aa83ff01b1543cbcfcc352c926150ec4c854cc084a41f796b7e07a280a1472a`，
  2026-09-03 13:14 UTC 切换（两次 active-chat-runs=0；current → 1950b5b8，previous → 91e78e74，
  e7f855a9 已清；NRestarts=0，重启后无 error）。二进制里 `上游最近动作 ·` / `就从那条最近动作写起` /
  `set_upstream_latest` 都在；harness 只换了 `industry-map`（备份 `pre-1950b5b8…-20260903T131454Z`），
  落盘底稿 9 条 `latest` 非空。匿名空体 POST edits 现在是 401（先鉴权再解析）。
- 复测 `ont2`（同 8 题，13:15–13:20 UTC）：仍 **0/8**。工具面有动：`earnings_outlook` 5/7 题、
  `analyst_actions` 3/7 题；CRDO 写进了 2 个 FY27Q2 数字；但 SNDK / AAOI / VST 的回答一次都没提英伟达。
- 决定性诊断：用隔离 actor 让生产 agent「只复述系统提示里【本轮相关行业】关于 SNDK 的上游最近动作那一行」，
  它逐字复述出来了（NVDA FY27Q2 $89.0B / $108B / 75.0%→74.0% / $119B→$279B，截至 2026-08-26；
  MU FY26Q3 84.9% / 86%）；同样能复述【历史公司研究基线】。**块在提示里，问题是回答模板没有它的位置**：
  每题都加载 `valuation-audit`，终稿按「估值对账表 → 三问 → 三情景」写，注入头部说的「需求侧第一段」
  在这套结构里没有对应的段落。

## v2.2：给上游事实在模板里安一个位置（只发 harness，不发二进制）

- `valuation-audit`：对账表多一行「上游最近动作」（原句数字 + 截至日期，`earnings_outlook` 有更新一季则以
  更新为准）；一问的**增长来源**第一句就是那条动作、第二句写传导闸门；第四步基准档的关键经营输入第一项是它，
  悲观 / 乐观改传导不改上游数。`fundamentals` 四的第一句同样原样引用它。`industry-map` 步骤 0 改成
  「先写上游最近动作，再核对更新，再写公司」。
- 发布：commit `35a422cd`（skills only），harness 13:22 UTC 装入生产（fundamentals / industry-map /
  valuation-audit 三个目录，备份 `pre-35a422cd…-20260903T132223Z`），二进制仍是 1950b5b8。
- 复测 `ont3`（同 8 题，13:22–13:30 UTC）：**4/8**（VST、SNDK、COHR、AAOI 都把英伟达 FY27Q2 的数字和
  截至日期写进了对账表「上游最近动作」行并沿传导链写到公司；SNDK 6/6 个数字全中）。RKLB 不在树里（对照），
  三个没接上的是 CRDO「值得买吗」、MU「投资逻辑」、NBIS「怎么看」——都提了英伟达（0/4/4 次）但没带数字与日期。
  三轮对照（成员公司 7 家）：ont0 0 → ont2 0 → ont3 4。
- 若仍不达标，下一步是二进制侧：把注入头部的「需求侧第一段」改成点名这三个落位（对账表行 / 增长来源 /
  基准档输入），并考虑在 pre-turn enrichment 里对成员公司预取一次上游的 `earnings_outlook`。
