# Bug: 会话压缩摘要幻觉生成“用户报告”并回灌正式回答

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
  - 会话: `Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb`
  - 最近一小时复现会话: `Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
- Prompt audit: `data/runtime/prompt-audit/feishu/20260415-171407-Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb.json`
- LLM audit: `data/llm_audit.sqlite3`
- 运行日志: `data/runtime/logs/web.log`
- 2026-04-16 最近一小时再次复现：
   - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
   - `2026-04-16 01:07:39.381` 会话自动 compact，并写回 `role=user` 的 `【Compact Summary】...`
   - 同条 summary 直接伪造了“根据截图内容，一鸣的持仓情况如下”表格，包含 `RKLB 500股 / 成本$68.50`、`SNDK 200股 / 成本$245.00 / 当前价$887.00` 等未验证字段
   - `2026-04-16T01:10:01.999236+08:00` assistant 后续正式回复继续引用该伪摘要中的两只持仓，称“根据compact summary，看起来之前已经有部分分析结果了”
 - 2026-04-16 08:47-09:00 最新复核：
   - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
   - `2026-04-16 08:47:56.682377+08:00` 会话再次写回 `role=user` 的 `【Compact Summary】...`，内容仍是带明确投资结论的 A 股股票表，而不是系统内部摘要
   - `2026-04-16 08:51:51.243193+08:00` 同轮 scheduler 任务最终 assistant 为空，说明这份 summary 在进入本轮任务前已被回灌进上下文，但没有产生新的正常回答
   - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
   - `2026-04-16 09:00:44.238102+08:00` 新一轮定时任务又在触发后被写回 `role=user` 的 `【Compact Summary】...`
   - 同轮 `web.log` 记录 `09:00:28.557` `context overflow detected`，随后 `09:00:44.239` `context_overflow_recovery compacted=true`，说明 scheduler 会话在本轮任务运行中再次把摘要回灌到上下文
 - 2026-04-16 20:31-20:49 最新复核：
   - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
   - `2026-04-16T20:30:59.783610+08:00` 新一轮 `每日仓位复盘` scheduler 任务触发
   - `2026-04-16T20:31:25.422365+08:00` 会话再次写回 `role=user` 的 `【Compact Summary】...`，内容仍然是上一轮 `RKLB vs SpaceX` 的完整对比表和分析结论，而不是系统态摘要
   - 该轮 `2026-04-16T20:32:51.450868+08:00` assistant 虽然完成送达，但说明 compact summary 仍会在 scheduler 任务执行前注入可见用户消息
   - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
   - `2026-04-16T20:45:59.758405+08:00` 新一轮 `美股盘前AI及高景气产业链推演` scheduler 任务触发
   - `2026-04-16T20:46:13.650765+08:00` 会话再次写回 `role=user` 的 `【Compact Summary】...`，内容明确写出“助手已完成全量9个定时任务的梳理”“建议删除任务4，将完整的新指令写入任务5，待用户确认后执行”
   - 紧接着 `2026-04-16T20:49:04.746325+08:00` assistant 正式回复开头直接引用该 summary：`关于定时任务系统的梳理，已确认删除原任务4并将完整合并指令写入任务5，待您最终核准`
   - 同轮 `cron_job_runs.run_id=1989` 被记为 `completed + sent + delivered=1`，说明当前缺陷已从“提前替用户作答”延伸为“把前序任务配置上下文串进下一条 scheduler 结果”
 - 2026-04-16 23:45-23:58 最近一小时复核：
   - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`
   - `2026-04-16T23:45:18.280077+08:00` 用户真实输入仅为：`现金还有多少呢？`
   - `2026-04-16T23:45:34.534352+08:00` 系统再次写回 `role=user` 的 `【Compact Summary】...`，内容仍是结构化持仓表，含 `GOOGL / CAI / TEM / BRK.B` 等历史组合信息，而不是系统内部摘要元数据
   - 随后 `2026-04-16T23:46:03.184074+08:00` assistant 仍继续基于被回灌后的上下文答复“系统记录中目前没有你的现金余额数据”，说明本轮问答前仍先把 summary 当用户消息注回 prompt
   - `session_id=Actor_feishu__direct__ou_5f69970af6b0ef6ce8e233ef0e0cc0bd79`
   - `2026-04-16T23:57:33.381412+08:00` 用户真实输入只有：`1`
   - `2026-04-16T23:57:40.124340+08:00` 会话再次写回 `role=user` 的 `【Compact Summary】...`，内容直接把“1”解释为三个候选方向：`校验 RKLB 增发后的最新资产负债表`、`拆解 WULF`、`提出新的期权风控策略`
   - 这条 summary 已经不是中性历史摘要，而是在压缩阶段主动补足并改写用户意图；随后 `2026-04-16T23:58:32.383200+08:00` assistant 直接沿着 RKLB 增发与 SpaceX IPO 逻辑展开正式分析
   - 最近一小时这两个样本说明：即使不再直接伪造整篇长报告，`Compact Summary` 仍持续以 `role=user` 进入真实会话，并会在“现金台账查询”“模糊指令澄清”这类普通直聊里抢先改写上下文和问题方向
 - 2026-04-17 00:03-00:38 最近一小时复核：
   - `session_id=Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b`
   - `2026-04-17T00:03:02.540084+08:00` 会话在连续执行 `RKLB -> TEM -> AAOI 每日动态监控` 时再次自动 compact，并写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 直接整理出 `【当前任务清单】` 表，枚举 `TEM / RKLB / AAOI` 等监控任务与触发条件；随后 `2026-04-17T00:04:08.587209+08:00` assistant 继续输出 `AAOI 每日动态监控简报`
   - `hone-feishu.release-restart.log` 同时记录 `2026-04-16T16:03:02.539385Z [SessionCompress] ... summary_chars=1262`，说明最新 scheduler 链路仍会在运行中把 summary 注回真实会话，而不是仅保存在系统态
   - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`
   - `2026-04-17T00:35:45.574918+08:00` 用户真实输入仅为：`m7就是指美股科技七巨头`
   - `2026-04-17T00:36:07.554072+08:00` 会话再次 compact，并写回 `role=user` 的 `【Compact Summary】...`，内容仍是持仓/关注列表表格，包含 `GOOGL / CAI / TEM / BRK.B` 及“助手的观点 / 用户的观点”列
   - 紧接着 `2026-04-17T00:38:24.093143+08:00` assistant 继续基于被回灌后的上下文给出 `M7` 买入时机结论；`hone-feishu.release-restart.log` 记录同轮 `search_tool_calls=0`、`combined_tool_calls=0`，说明这轮回答并没有新搜索纠偏，而是直接在被污染后的上下文里完成
   - 这两个样本表明：即使 summary 不再伪造全新投研报告，只要它继续以 `role=user` 回灌，就仍会在 scheduler 任务串行执行与普通直聊澄清场景中重写后续 prompt 的事实边界
- 2026-04-17 01:02-01:06 最近一小时复核：
   - `session_id=Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb`
   - `2026-04-17T01:02:59.378263+08:00` 会话再次自动 compact，并写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 仍是完整的 `股票关注表`，包含 `MU / WDC / TEM / RKLB` 等标的，以及“助手的观点 / 用户的观点”两列，显然不是系统内部压缩元数据
   - 紧接着 `2026-04-17T01:06:39.078023+08:00` assistant 正式回复用户“帮我分析一下hims”；同轮 `web.log` 记录 `search_tool_calls=2`、`answer_tool_calls=0`、`combined_tool_calls=2`
   - 这说明即使进入了新的直聊分析请求，compact summary 仍先以用户消息身份参与本轮上下文组装；问题已不限于 scheduler 串话，也继续存在于普通 direct session 的压缩恢复路径中
- 2026-04-19 06:52-06:54 最近一小时复核：
   - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
   - `2026-04-19T06:52:23.700401+08:00` 会话写入 `system` 消息 `Conversation compacted`
   - 紧接着 `2026-04-19T06:52:23.700584+08:00`，`session_messages` 再次落入 `role=user` 的 `【Compact Summary】...`，内容不是系统态摘要元数据，而是带明确投资结论的长文：
     - “光模块产业链的核心投资机会已从三巨头……向上游扩散”
     - “真正值得重点研究的是上游材料端的估值洼地机会”
     - 还继续枚举 `源杰科技 / 长光华芯 / 云南锗业` 等标的与产业链判断
   - 同一会话在 `2026-04-19T06:54:13.701580+08:00` 已继续产出正式 assistant 回答，说明这条 `role=user` 的 compact summary 不是历史遗留脏数据，而是就在本轮 auto compact 后再次进入真实会话上下文
   - 这与文档里“2026-04-17 已改存 `role=system`、restore 跳过”的修复结论直接冲突，说明生产链路至少在消息落库层面仍未收口；即便最终回答表面可读，压缩摘要角色错误仍在持续污染真实 transcript
 - 2026-04-19 12:00-12:02 最近一小时复核：
   - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
   - `2026-04-19T12:00:00.416514+08:00` 新一轮 `每日公司资讯与分析总结` 定时任务触发
   - `2026-04-19T12:01:38.970370+08:00` 会话再次写入 `system` 消息 `Conversation compacted`
   - 紧接着同一 `imported_at` 下，`session_messages` 又写入 `role=user` 的 `【Compact Summary】...`，内容仍是持仓/观点表，而不是系统态摘要元数据：
     - `RKLB | 高弹性标的之一，5月7日财报为重要观察节点 | 持仓257股，成本72.22美元`
     - `TEM | 待补 | 持仓193股，成本49.53美元`
     - `CRWV / NBIS / GOOGL / TSM` 也被整理进同一张表
   - 对应 `hone-feishu.release-restart.log` 记录该轮在 `2026-04-19T04:01:18Z` 命中 `context overflow detected`，`04:01:38Z` 完成 `context_overflow_recovery compacted=true`，随后本轮任务仍失败收口为“当前会话上下文过长...仍无法继续”
   - 这说明 compact summary 的 `role=user` 污染不只会伴随“看似成功送达的日报”，也会在 scheduler 失败分支里继续写入真实 transcript，并与最新定时任务触发混在同一会话上下文里

## 端到端链路

1. 这条 Feishu direct session 在 2026-04-15 17:14:07 命中自动压缩。
2. `SessionCompactor` 没有只总结“旧消息”，而是把整个 active window 连同最后一条新的 Rocket Lab 用户问题一起送给压缩模型。
3. 压缩模型没有输出“历史摘要”，而是直接生成了一整篇 `Rocket Lab (RKLB) 全面深度分析` 长文，并在文中编入 `22至25 美元`、`2025 年底首飞`、`FY2025 收入 9.5-10 亿美元` 等未验证数字。
4. 系统随后把这段压缩结果以 `role=user` 的 `【Compact Summary】...` 写回会话。
5. 17:15:10 与 17:16:57，回答链路又因为 `context window exceeds limit (2013)` 触发 `context_overflow_recovery` 强制压缩，进一步把这份伪摘要固化进会话上下文。
6. 最终 17:21:59 的正式回答阶段把该伪摘要当成“用户已提供的报告/原始请求”，于是出现“报告中假设的 22至25 美元”“报告遗漏了……”这类错误引用。

## 期望效果

- 会话压缩只应总结已有历史，不应把本轮最后一个未回答问题当作自由发挥的答题对象。
- `Compact Summary` 应被明确标识为系统内部压缩产物，而不是长得像“用户提供的材料”。
- 回答阶段不应把压缩摘要解释为用户上传报告、用户笔记或外部附件。

## 当前实现效果（问题发现时）

- 压缩模型实际使用的是 `llm.auxiliary.model = MiniMax-M2.7-highspeed`，而不是主对话模型。
- 2026-04-15 17:14:07 的自动压缩记录显示 `active_messages=26`、`trigger=auto`，已经满足 direct session 自动压缩条件。
- 2026-04-15 17:15:47 的恢复压缩记录显示 `trigger=context_overflow_recovery`、`forced=true`，会在上下文溢出后再次强制压缩并重试。
- 压缩结果被写回为 `role=user` 的 `【Compact Summary】...`，后续 prompt 组装与 multi-agent answer 会直接看到这段内容。
- 最终回答引用了压缩摘要中的伪“报告假设”，但用户本轮没有上传任何报告文件，也没有在真实历史里提供 `22至25 美元` 这一数字。

## 当前实现效果（2026-04-15 23:56-23:58 最近一小时复核）

- 同类问题已在另一条 Feishu direct 会话再次复现，说明这不是单次压缩偶发，而是当前 active window 压缩链路仍会持续污染后续回答：
  - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
  - `2026-04-15T23:56:57.340991+08:00` 用户真实输入只有：`rklb呢 马上SpaceX上市 那rklb估值是不是应该会高一些`
  - `2026-04-15 23:56:57.348` `web.log` 记录：`Compressing session ... with 27 messages (~59763 bytes)`
  - `2026-04-15 23:57:23.713` 会话被自动 compact，随后同一时刻写回一条 `role=user` 的 `【Compact Summary】...`
- 这次 `Compact Summary` 仍然不是对旧历史的中性摘要，而是直接生成了一整段带明确结论的 RKLB 投研文本，例如：
  - `SpaceX IPO对RKLB的估值拉动效果有限且逻辑存在误区，不建议以"SpaceX影子股"逻辑建仓RKLB`
  - `Rocket Lab是美国纳斯达克上市的商业火箭公司，专注中小型卫星发射`
- 后续 answer 阶段没有重新完成独立判断，而是继续沿着这段伪摘要输出结论：
  - `2026-04-15T23:58:43.545035+08:00` assistant 回复直接从 `SpaceX的IPO不会系统性抬高Rocket Lab（RKLB）的合理估值` 起笔
  - 同轮日志显示搜索阶段 `tool_calls=0`，但 answer 阶段仍额外执行了 `hone_data_fetch`，说明它是在被 compact summary 污染后的上下文里继续补证，而不是纠正 compact summary 的语义
- 最近一小时这次复现和 17:14 那次事故虽然会话不同，但症状完全一致：新问题进入压缩窗口后，被系统以 `role=user` 的“摘要”形式提前回答，随后正式回答把它当成可信上下文继续展开。

## 当前实现效果（2026-04-16 01:07-01:10 最近一小时复核）

- 同一缺陷在图片附件会话里继续以另一种题材复现，说明问题已经不限于 RKLB 投研问答，而是会把“最后一个未回答任务”直接改写成伪造 summary：
  - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
  - `2026-04-16 01:07:39.381291+08:00` 系统写入 `Conversation compacted`
  - 紧接着 `2026-04-16 01:07:39.381312+08:00` 写回 `role=user` 的 `【Compact Summary】...`
- 这次 `Compact Summary` 没有总结旧历史，而是直接替用户“完成”了尚未成功的图片识别任务，伪造出一张持仓表：
  - `RKLB | 500股 | 成本$68.50 | 当前价$72.00`
  - `SNDK | 200股 | 成本$245.00 | 当前价$887.00`
  - 还附带“已记录一鸣的持仓信息”“持仓分析建议”等结论性文本
- 随后的 assistant 持续把这段伪 summary 当成可信前情：
  - `2026-04-16T01:10:01.999236+08:00` assistant 落库内容明确写出：`根据compact summary，看起来之前已经有部分分析结果了：- RKLB: 500股，成本$68.50 - SNDK: 200股，成本$245.00`
  - 最终回复继续要求用户基于这两只股票补录其它持仓，证明 compact summary 已经污染本轮“识别四张截图”的主任务链路
- 这次复现和前两次事故共享同一根因：系统不是在概括旧上下文，而是在 `role=user` 的 summary 中提前作答，并把伪结论回灌给正式回答阶段。

## 当前实现效果（2026-04-16 08:47-09:00 最近一小时复核）

- 缺陷继续出现在 Feishu scheduler 会话中，且已不局限于单条图片会话：
  - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
  - `2026-04-16 08:47:56.682377+08:00` 先写回 `role=user` 的 `【Compact Summary】...`
  - 紧接着 `2026-04-16 08:51:51.243193+08:00` scheduler 任务完成时 assistant 为空，`sessions.last_message_preview` 也为空，说明 compact summary 已被回灌但这轮没有形成新的可用答复
- 到 `09:00`，同类问题又在另一条 Feishu scheduler 会话上叠加了上下文溢出重压缩：
  - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
  - `2026-04-16 09:00:28.557` `web.log` 记录 `context overflow detected, compacting and retrying`
  - `2026-04-16 09:00:44.239` 记录 `context_overflow_recovery compacted=true`
  - 同一时间 `session_messages` 新增 `role=user` 的 `【Compact Summary】...`，其中仍是结构化股票关注表，而不是隔离在系统态的压缩元数据
- 这说明当前缺陷不仅会“把最后一个问题提前答掉”，还会在 scheduler 场景里把旧会话总结持续注入新一轮定时任务上下文；一旦再叠加 `context_overflow_recovery`，污染会在同轮任务内被再次固化。

## 当前实现效果（2026-04-16 20:31-20:49 最近一小时复核）

- `Compact Summary` 仍然继续以 `role=user` 写回真实会话，说明此前“只总结旧消息”的修复并没有解决“摘要仍被当成用户可见上下文参与后续推理”这个核心问题。
- `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 在 `20:31:25` 写回的 compact summary 继续保留上一轮 `RKLB vs SpaceX` 的完整结论型表格；虽然这轮 `每日仓位复盘` 最终成功送达，但说明 scheduler 任务前仍会先把旧结论作为用户消息注回上下文。
- `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7` 的症状更明确：`20:46:13` compact summary 总结的是“是否删除任务4、把完整指令写入任务5”的任务编排讨论，而 `20:49:04` 下一条本该独立生成的“美股盘前AI及高景气产业链推演”结果，开头却直接继承了这段上下文。
- 这表明当前缺陷已经不只是“summary 内容本身有幻觉”，而是 `Compact Summary` 仍然作为 `role=user` 进入 prompt，导致不同 scheduler 任务之间发生明显的跨任务串话与回答污染。
- 本轮 `run_id=1989` 最终被记为 `completed + sent + delivered=1`，说明系统不会把这类污染识别为失败；如果只看台账执行状态，会误以为结果完全正常。
- `23:45` 的现金查询样本说明，这个问题已经不限于 scheduler 或深度分析场景；即便用户只是在追问组合里“现金还有多少”，系统仍会先把一大段持仓表以 `role=user` 重新注回上下文。
- `23:57` 的单字输入样本则更直接：summary 把用户的“1”擅自扩写成三个解释方向，随后正式回答沿着其中一个方向继续展开，说明压缩摘要仍会主动替用户补全意图并改变本轮任务走向。
- `00:03` 的 scheduler 串行样本说明，即便每轮任务最终都成功送达，compact summary 仍会把“任务清单/触发条件”作为用户消息夹在任务之间，导致后续任务共享被污染的上下文。
- `00:36` 的澄清样本说明，这种回灌已经不是极端长会话专属问题；即便用户只是补一句对 `M7` 的定义，系统也会先注入大段历史表格，再在零额外工具调用的前提下沿着被回灌后的上下文作答。
- `01:02` 的最新样本进一步说明，即使本轮新问题是独立的 `HIMS` 分析，请求进入 answer 前仍会先插入一条 `role=user` 的股票关注表 compact summary；也就是说，问题并没有收敛到 scheduler 场景，而是继续存在于普通 direct session 的自动 compact 路径。
- `09:00` 的最新样本再次证明 scheduler 普通 auto compact 仍在生产生效：`Actor_feishu__direct__ou_5f95ab3697246ded86446fcc260e27e1e2` 在 `2026-04-19T09:00:26.593495+08:00` 又写回 `role=user` 的 `TSLA / RKLB` `【Compact Summary】`，随后同一任务仍在 `run_id=2861` 被记为 `completed + sent + delivered=1`。这说明问题不是“旧污染仍留在库里”，而是当前定时任务运行前仍会主动生成并消费这类 summary。
- `12:01` 的最新样本进一步说明，污染并不依赖任务最终成功送达：`Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 在 overflow recovery 后再次写回 `role=user` 的持仓表 `【Compact Summary】`，随后本轮 `run_id=2923` 只返回“当前会话上下文过长”失败提示。也就是说，compact summary 角色错误仍会在 scheduler 失败路径里实时生成新的 transcript 污染。
- 因而当前缺陷的主表现已经收敛为两点：一是 summary 角色仍错误，二是 summary 仍会在后续回答前重写本轮输入语义；这两点都没有被此前修复覆盖。

## 已确认事实

- 本次事故里没有用户上传的 PDF / 图片 / 附件报告。
- 根目录 `data/uploads/feishu` 未发现这条消息对应上传物。
- actor sandbox 下 `data/agent-sandboxes/feishu/direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb/uploads/Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb/` 为空。
- `22至25 美元` 只出现在压缩摘要和最终污染后的回答里，未在这条会话之前的真实输入中找到来源。

## 触发条件

1. direct session active messages 超过 20 条，或 active 内容字节超过 80,000，触发自动压缩。
2. provider 返回 `context window exceeds limit` / `too many tokens` 等错误时，`AgentSession` 会额外触发一次 `context_overflow_recovery` 强制压缩并自动重试。
3. 当 active window 末尾恰好是一个新的深度分析请求时，压缩模型更容易从“总结历史”漂移成“直接回答最后的问题”。

## 用户影响

- 用户会被误导为“系统看到了一个我上传过的报告”，从而破坏对回答可信度的判断。
- 正式回答会把压缩幻觉当成事实背景继续扩散，导致二次污染；最近一小时的图片会话里，这种污染已经从“伪造投研报告”扩展到“伪造持仓识别结果”。
- 在金融分析场景里，这类伪上下文会直接引入错误估值、错误时间线和错误事件判断，属于高风险质量故障。
- 之所以不是 `P3`，是因为问题并不只是“回答写得不够好”，而是系统内部压缩产物污染了真实会话上下文，后续工具调用与正式结论都会围绕伪上下文继续执行，已经影响主回答链路的正确性。

## 根因判断

1. `SessionCompactor` 当前总结的是整个 active window，而不是“将被裁掉的旧上下文”。
2. 压缩结果被存成 `role=user` 消息，语义上过于像用户自己提供的材料。
3. 回答链路没有对 `session.compact_summary` 做足够强的隔离或降权，导致 multi-agent search / answer 会把它理解成原始用户请求的一部分。
4. 压缩提示词只要求“总结历史”，但没有显式禁止“回答最后一个问题”或“生成新的投研报告”。
5. 最近多次复现证明，即使没有再次命中 `context_overflow_recovery`，仅靠一次普通 auto compact 就足以把伪结论写回会话并污染后续 answer 阶段；一旦叠加 overflow recovery，这份污染还会在同轮 scheduler 任务内再次被固化。
6. 从 `20:46 -> 20:49` 的跨任务串话样本看，即使 summary 本身更接近“历史总结”，只要它继续以 `role=user` 参与后续 prompt 组装，answer 阶段仍会把其中的待办、结论和上下文当成当前任务事实继续复述。
7. `2026-04-19 09:00` 的定时任务样本说明，这个缺陷并不限于直聊恢复路径。scheduler 在普通 auto compact 后仍会把 summary 作为真实 `role=user` transcript 落库，再继续执行并成功送达最终日报，意味着线上生产路径仍在实时生成新的污染样本。
8. `2026-04-19 12:01` 的最新定时汇总样本说明，即使任务最终没有成功送达正文，scheduler 在 `context_overflow_recovery` 失败路径里仍会把 compact summary 写成真实 `role=user` transcript；因此当前问题不只是“成功回答前污染上下文”，还会继续污染失败任务后的会话库与排障视图。

## 修复情况（2026-04-17）

1. `crates/hone-channels/src/session_compactor.rs` 现在把 compact summary 以 `role=system` 写回会话，不再伪装成新的 `role=user` 输入。
2. `crates/hone-channels/src/agent_session.rs` 的 `restore_context()` 现在会显式跳过 `session.compact_summary` 消息，避免这段摘要再作为普通用户消息进入后续 runner transcript。
3. `crates/hone-channels/src/prompt.rs` 现在优先读取 `session.summary` 并统一转换为 `【历史会话总结】` 注入本轮 prompt；旧会话里遗留的 `【Compact Summary】` 消息只作为兼容 fallback 读取，不再把原始标记文本直接塞回用户输入区。
4. 新增/更新的回归测试已经覆盖：
   - prompt 组装优先使用 `session.summary`，不会继续引用旧的 compact summary 消息正文
   - compact boundary 后的 restore 不再把 compact summary 当成普通用户消息恢复
   - `recv_extra` 仍然位于历史摘要之前，避免群聊补充上下文顺序被这次修复破坏
5. 代码层修复已完成并通过 crate 级测试；当时文档曾更新为 `Fixed`。但 2026-04-19 06:52 的真实 Feishu 会话再次落入 `role=user` 的 `【Compact Summary】`，而且 2026-04-19 09:00 的 `特斯拉与火箭实验室新闻日报` 也在 auto compact 后重现同样落库方式，说明“代码修复通过测试”并不等于线上 transcript 已恢复。

## 历史修复情况（2026-04-16，已确认未收口）

1. `crates/hone-channels/src/session_compactor.rs` 已改为只总结“将被压掉的旧消息”：
   - 正常 auto compact 不再把保留窗口里的最近消息送进压缩 prompt
   - 这意味着最后一个未回答问题不会再被压缩模型提前“接管作答”
2. direct-session 的压缩提示词已收紧：
   - 明确要求“只能总结已发生的历史，不能回答尚未解决的问题”
   - 明确禁止新增价格目标、持仓明细、时间线或未在历史中出现的事实数字
   - 明确禁止把摘要写成正式报告、正式结论或投资建议正文
3. `context_overflow_recovery` 的强制压缩边界也已保留：
   - 当会话里只剩 1 条活跃消息时，强制 compact 仍可工作，不会把 overflow recovery 退化成“完全不 compact”
4. 但 `2026-04-16 20:31` 与 `20:46` 的最新样本证明：上述修复最多只缓解了“把最后一个问题直接写成伪答案”的部分场景，并没有消除 `Compact Summary` 作为 `role=user` 回灌后续任务上下文的问题，因此本缺陷状态从 `Fixed` 重新打开为 `New`。
5. `2026-04-16 23:45` 与 `23:57` 的样本进一步说明，即使 summary 文本更像“历史整理”或“澄清问题”，只要它继续以 `role=user` 写回会话，就仍会在普通直聊中抢先定义用户意图并影响 answer 阶段的方向选择。
6. `2026-04-17 00:03` 与 `00:36` 的样本继续证明：当前问题已经稳定跨越 scheduler 串行任务与直聊澄清场景复现，且即使没有新的搜索工具调用，answer 阶段仍会直接消费被回灌的 summary。
7. `2026-04-17 01:02` 的新样本说明，即使在普通 direct session 中继续执行新的证券分析请求，summary 仍会先以 `role=user` 进入 prompt；此前修复并没有把 compact summary 从真实会话语义中隔离出去。

## 回归验证

- `cargo test -p hone-channels build_prompt_bundle_uses_session_summary_over_compact_summary_message -- --nocapture`
- `cargo test -p hone-channels restore_context_uses_only_messages_after_latest_compact_boundary -- --nocapture`
- `cargo test -p hone-channels restore_context_ -- --nocapture`
- `cargo test -p hone-channels resolve_prompt_input_places_recv_extra_before_session_summary -- --nocapture`
- `cargo test -p hone-channels auto_compact_summary_excludes_latest_user_turn_from_prompt -- --nocapture`
- `cargo test -p hone-channels auto_compact_uses_low_group_threshold_and_keeps_recent_window -- --nocapture`
- `cargo test -p hone-channels context_overflow_auto_compacts_and_retries_successfully -- --nocapture`
- `cargo test -p hone-channels context_overflow_failure_is_rewritten_to_friendly_message -- --nocapture`
- `cargo test -p hone-channels`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/session_compactor.rs crates/hone-channels/src/agent_session.rs crates/hone-channels/src/prompt.rs`

## 后续建议

1. 后续仍应优先补一条 scheduler 跨任务回归测试，直接锁住“前一轮 compact summary 不得串入后一轮独立任务答案”的正式 contract。
2. 如果真实流量里仍观测到摘要幻觉数字，可再补更强的 summary-output contract test，直接约束不得输出历史中未出现的证券价格、持仓数量和目标价。
3. 需要优先核对当前线上写库路径与 prompt 组装路径是否出现分叉：即便 runner restore 已跳过 compact summary，只要 `session_messages` 仍把摘要落成 `role=user`，后续导出、排障和任何依赖消息库的能力都会继续读到受污染 transcript。
