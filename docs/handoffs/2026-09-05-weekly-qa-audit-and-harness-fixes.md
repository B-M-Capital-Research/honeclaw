# 2026-09-05 近一周线上问答盘点与 harness 修复

## 取数

- 生产 PG `cloud_sessions`（`updated_at` 近 7 天，去掉 `harness-eval-*`）：43 个会话，按消息时间戳筛出
  2026-08-29 之后的真实用户轮次 **134 轮 / 35 个用户（3 个管理员）**，web 82、飞书 52；
  去掉 `[定时任务触发]` / `【Invoked Skill Context】` 伪用户消息，assistant 终稿在 `content[].type == "final"` 块里。
- journald `runner.tool` 9,537 行（只记 `tool=<name> <kind>`，不记 ticker），`recv` 525 条（含定时）。
- 四个独立评审 agent 各读 33–34 轮，按 对题 / 完整 / 可信 / 形式 四维 1–5 分打分并贴标签，逐轮结果在
  scratchpad `audit7d/review1-4.json`。

## 结果

| 维度 | 均分 | ≤3 分轮次 |
|---|---|---|
| 对题 | 4.28 | 18 |
| 完整 | 4.26 | 19 |
| 可信 | 3.54 | 53 |
| 形式 | 3.54 | 54 |

四维全 ≥4 的只有 50/134。标签：事实存疑 39、过长 26、忽略行业上游 21、未回答核心问题 12、模板堆砌 11、
工具未用 11、过短 9、时间口径混乱 9、语言混乱 8、数字汉化 6、A股港股处理不当 6。无回答 0，
上下文过长导致无法继续 2（同一用户）。

## 根因与修复（本次已改）

1. **涨跌归因兜底桩吞掉整篇回答（11/134，3 次用户重问仍得同一桩）**。`is_market_move_final_check_enabled`
   只要原话带 大跌/暴涨 就开机械终稿检查，`original_market_move_request` 在没有【本轮用户输入】分段时把注入的
   证据也当原话；检查里「每个确定性原因段落同段必须带本轮 URL + 目标日期」几乎不可能满足，纠正两轮后发
   `deterministic_market_move_gap_response`（SPY/QQQ 涨跌 + 「原因本轮未完全核验」）。命中的问题包括
   非农 / 加息预期、CRDO 暴跌后值不值得买、SNDK 割仓后再入场、BE 能否到 330。这正是 AGENTS.md「生成型工作流治理」
   点名的反模式。改法：触发需同时有涨跌词与「为什么 / 原因 / 怎么回事 / because…」等归因线索；只看用户原话
   （截到下一段落标记）；删掉逐段 URL 规则，保留日期、星期、quote 数字、宽基范围这些客观检查；兜底桩只剩客观
   违规仍在时才会出现。`hone-agent` 162 通过，另 2 条是 8/29 起就在的既有失败。
2. **pre-turn enrichment 一周被丢弃 289 次**。`web_search` 生产中位 7–9 s、p90 10–13 s（Tavily），却放在 6 s 的
   identity join 里；join 超时整段证据作废（日志文案还写成「exceeded its deadline」）。改法：web 搜索与 identity
   并行、自带 10 s 预算、超时只算「没有 web 结果」；成功路径加 INFO（calls / web），下次可以算分母。
3. **数字 / 术语汉化**（用户 09-04 投诉，投诉后仍出现 3 轮）。根因是 `【语言要求】必须全程以中文回复，禁止中英文混排`，
   模型据此把 ticker、Forward P/E、$228.45 写成汉字。改成：中文回复，但证券代码、财务与技术缩写、阿拉伯数字与货币单位
   一律原样。
4. **company_portrait 让模型去读 `references/*.md`**，而 `local_read_file` 沙箱只到用户空间（一周 2 次失败）：参考内容内联进 SKILL.md。
5. **估值算术 / 单位 / 币种自相矛盾（事实存疑 39 轮里的主体）**：ASML 用美元股价除欧元 EPS、SK 海力士净利润大于营收、
   81 亿写成 $81.04B、同文两个 TTM P/E、情景概率加不到总概率、AVGO 一天两个 forward EPS 与调参凑「高度吻合」。
   `valuation-audit` 红旗自检加：币种折算、单位单一写法、回代自检、预测窗口写死、交叉验证不许调参。
6. **无行情仍给价位、估算价代替 quote、技术位当触发**：协创数据首行「未取到行情」正文却给 253.88 元与三情景；
   16 只持仓 10 只「~$414（公开行情）」然后清仓 6 只；均线 / 整数关口 / 「成交量萎缩确认支撑」当买卖触发 6 轮以上。
   `valuation-audit` 与 `position_advice` 加硬规则：未取到行情不出现价格结论；估算价禁止；触发只认估值端点 / 事件日期 /
   经营指标；仓位只用相对语言，不给「目标 25%」与整仓清仓；「翻倍」目标先讲概率与回撤。

## 没改、需要另一条线处理的

- **飞书渠道**：52 轮里过半 4–7k 字、多张 6–7 列宽表、`$$` LaTeX。`FEISHU_FORMAT_GUIDANCE` 与 soul.md 的篇幅下限
  正在另一会话的未提交改动里，本次不碰；建议：篇幅下限只对 web 生效，飞书 ≤2000 字、禁表格与 LaTeX。
- **web 首行「行情口径：本轮仅使用可核验资料…」模板句**（18/33 轮）：`agent_session/core.rs` 的 service-owned prefix，
  终稿完成后应回填本轮 quote 的时间与时段。
- **gemini grounding 重定向长链接**（`vertexaisearch.cloud.google.com/grounding-api-redirect/…`，5 轮以上）：
  需要在 provider 代理层把 grounding URL 解析成真实链接或只留域名。
- **宏观口径**：非农解读用 2025-11 的联邦基金利率旧序列、同日「加息概率 58–66%」与「封死降息 50bp」并存——
  `market_analysis`（另一会话在改）应要求先落当前目标区间与会前定价。
- **上下文过长无法继续**（1 个用户 2 次）：压缩后仍超限，engine 侧。
- **虚假跟进承诺**（「我会继续紧盯…」）与韩文 / 繁体混入：soul.md 层。

## 发布与复测

- 第一版 `e867eae3`（在另两个会话的 7 个提交之上 cherry-pick；镜像
  `…@sha256:a95d7b0a20a8ccc5a49562dd124ec221fe7c04e359f611470e0c3ed185dff9b9`），2026-09-05 04:51 UTC 切换：
  current → e867eae3，previous → 6cf77eaa（另一会话 04:28 刚发的版本），清掉 1950b5b8 / 505cf737 / e0278ed1 /
  a2d76ea4 后磁盘 5.2G；NRestarts=0，无 error；harness 换了 company_portrait / position_advice / valuation-audit。
- 复测 `fix1`（一周里被兜底桩吞掉的 6 道原题）：CRDO 暴跌后值不值得买 5,237 字、LITE 估值 4,931 字、SNDK 割仓再入场
  2,925 字、BE 为什么涨/能否到 330 4,194 字——都是完整回答，无汉字数字；pre-turn enrichment 首次出现
  `loaded calls=6/18 web=true`。但「为什么今天光通信大跌」与「非农…市场跳水」仍是 320 字桩：前者是客观检查
  （把 quote 写成「收盘价」、SPY 涨跌符号）两轮纠正后仍不过；后者是生产 runtime input 没有【本轮用户输入】头，
  收窄后的触发词扫到了服务端追加的「归因门禁」「原因」字样。
- 第二版 `05fdf675`：无头时也截到第一个分段标记（单测覆盖 6 道原题 × 有头 / 无头 / 夹门禁段三种布局）；纠正用尽后
  不再整篇换桩，改为正文照发 + 末尾「口径提示」列出未对齐的几处（空稿才用桩）。复测 `fix2`（05fdf675，镜像 `…@sha256:813756818db959cac8bad9dd1827999e32d5d982ca02a83a36e9662a923702c8`，
  05:1x UTC 切换，current → 05fdf675，previous → e867eae3，6cf77eaa 已清）：**6/6 都是完整回答**——
  光通信大跌 3,702 字（写清 09-04 反而走强、回调在 09-02/03、Ciena 指引与增发稀释是原因）、非农 5,083 字
  （数据超预期→加息预期抬升→估值端承压，带 CPI / 10Y / FOMC 证伪条件）、CRDO 6,821、LITE 4,523、SNDK 1,985、
  BE 1,010；无汉字数字；六题 pre-turn enrichment 全部 `loaded`（calls 1–18）。光通信与非农两题仍各触发了
  3 次客观检查（把 quote 写成收盘价、SPY 交易所写法），纠正用尽后正文照发、末尾附「口径提示」一行，
  不再是 320 字的桩。SNDK 那题仍出现「50 日均线附近」这类技术位措辞——skill 规则已装但模型只部分照做，待观察。
- 上线后真实流量的分母：新版本每轮记 `pre-turn enrichment loaded calls=N web=bool` 或
  `pre-turn identity phase did not finish within its budget`；一周后按这两条算成功率，再按「原因本轮未完全核验」
  出现次数看桩是否绝迹。
