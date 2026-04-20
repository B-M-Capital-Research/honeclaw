# Bug: Feishu 直聊在工具链尚未结束时提前持久化短答，导致用户只收到过渡性半成品回复

- **发现时间**: 2026-04-16 16:12 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fb47bd113e7776b05e7a5c2c56e310652`
    - `2026-04-20T20:39:38.246022+08:00` 用户提问：`asts  送错轨道这次影响力大吗`
    - `2026-04-20T20:40:28.632136+08:00` assistant 先返回 64 字过程句：`本地用户画像目录在当前沙箱里没有直接暴露出来，我先核验事件本身和时间线，再判断这是“噪音级事故”还是会改写商业化节奏的实质问题。`
    - 用户在 `2026-04-20T20:42:02.063934+08:00` 原样重问同一请求后，系统才在 `2026-04-20T20:42:44.540689+08:00` 给出正式长答
    - 这说明最近一小时里，同根因仍会把“本地目录/沙箱暴露情况”这种内部执行计划直接当成 `final` 发给用户，并迫使用户再次追问才能拿到正式分析
  - `data/runtime/logs/web.log`
    - `2026-04-20 20:40:18.726` 同一 `ASTS` 会话在过程句落库前仍继续启动 `Tool: hone/local_list_files`
    - `2026-04-20 20:40:18.729` 紧接着又继续启动 `Tool: hone/local_search_files`
    - `2026-04-20 20:40:25.172` 之后继续启动 `Tool: hone/data_fetch`
    - `2026-04-20 20:40:28.626` 还在继续启动 `Tool: hone/web_search`
    - 但 `2026-04-20 20:40:28.635` 已经先记录 `step=session.persist_assistant detail=done`，同秒落成 `done ... success=true ... reply.chars=64`
    - `2026-04-20 20:40:29.635` 又直接执行 `reply.send ... segments.sent=1/1`
    - 这说明当前收口时序仍允许“后续工具刚启动 -> 先把内部过程句持久化并外发”
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
    - `2026-04-20T20:00:00.154161+08:00` scheduler 注入用户请求：`GEV earnings reminder`
    - `2026-04-20T20:00:36.997863+08:00` assistant 仅落下 72 字计划句：`这次是明确的财报前预判任务。我先调取GEV的历史研究记录和财报前关键事实，再核验最新市场预期...`
    - 到本轮巡检结束，这条 assistant 之后没有新的正式财报前分析正文；`cron_job_runs.run_id=3590` 同时保留相同 `response_preview`
    - 这说明半成品先落库的问题已经扩散到 scheduler 注入的 direct 会话，而不只发生在用户手动发起的直聊提问
  - `data/runtime/logs/web.log`
    - `2026-04-20 20:00:34.680` 同一 `GEV earnings reminder` 会话在短答落库前仍继续启动 `Tool: hone/local_list_files`
    - `2026-04-20 20:00:34.834` 紧接着记录 `runner.stage=acp.tool_failed`，说明系统还在尝试补上下文文件
    - `2026-04-20 20:00:36.985` 又继续启动 `Tool: hone/web_search`
    - `2026-04-20 20:00:37.031` 才记录 `step=session.persist_assistant detail=done`
    - 同一秒落成 `done ... success=true ... reply.chars=72`
    - 这说明当前收口时序依然允许“仍有后续工具动作 -> 先把计划句持久化成最终答复”
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`
    - `2026-04-20T16:18:54.098436+08:00` 用户提问：`请对 vistra energy 进行详细分析`
    - `2026-04-20T16:19:32.990566+08:00` assistant 先返回过程句：`本地用户空间里没有现成的 company_profiles/ 目录，我先补查当前 actor 目录结构，再抓取 VST 的实时数据、财务和最新新闻...`
    - 用户在 `2026-04-20T16:21:36.849335+08:00` 原样重问同一请求后，系统才在 `2026-04-20T16:24:37.497871+08:00` 给出正式长答
    - 这说明最近一小时又出现“过程句先落库、用户被迫重问、正式答复才在下一次触发里出现”的同根因样本
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fe09f5f16b20c06ee5962d1b6ca7a4cda`
    - `2026-04-20T08:53:43.082970+08:00` 用户提问：`美股TEMPUS AI 的value analysis`
    - `2026-04-20T08:54:47.412231+08:00` assistant 最终只返回 84 字过程句：`我还缺一件事：如果把 Tempus 作为后续持续跟踪对象，我需要按现有画像格式沉淀一份主画像...`
    - 到本轮巡检结束，这条 user turn 之后没有新的正式分析答复；同会话下一条消息已跳到 `09:00:59` 的定时任务注入，说明本次请求被过程句截断后就结束了
  - `data/runtime/logs/web.log`
    - `2026-04-20 16:22:58.175` 同一 `vistra energy` 会话仍在继续执行 `Edit .../company_profiles/vistra-energy/profile.md`
    - `2026-04-20 16:23:22.384` 又继续执行 `Edit .../company_profiles/vistra-energy/events/2026-04-20-thesis-refresh.md`
    - 直到 `2026-04-20 16:24:37.502` 才落成 `session.persist_assistant detail=done`
    - 这与 `16:19:32` 已先出现过渡句 assistant 落库相互印证，说明用户看到的第一条可见答复确实早于后续研究动作完成
  - `data/runtime/logs/web.log`
    - `2026-04-20 08:54:47.406` 同一会话仍在启动 `Tool: hone/local_list_files`
    - `2026-04-20 08:54:47.417` 紧接着就出现 `step=session.persist_assistant detail=done`
    - 同一时间点落成 `done ... success=true ... reply.chars=84`
    - `2026-04-20 08:54:49.155` 继续执行 `step=reply.send ... segments.sent=1/1`
    - 这说明本轮不是“工具跑完后只答得短”，而是 answer 在仍有工具动作时就把内部计划句当成最终结果出站
  - 历史同根因样本：
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-16T16:00:09+08:00` 用户要求“组合风险评估 + 核心持仓评估”
    - `2026-04-16T16:01:05+08:00` assistant 只收到 55 字过渡句：`美股行情已经拿到。港股代码格式在底层数据里没直接回出...`
    - 同轮日志同样显示 `session.persist_assistant/done` 之后仍继续启动 `Tool: hone/web_search`

## 端到端链路

1. Feishu 直聊用户发起需要正式分析的请求，例如 `美股TEMPUS AI 的value analysis`。
2. agent 正常进入 `agent.prepare` 与 `agent.run`，并在同一轮先后调用 `skill_tool`、`local_search_files`、`data_fetch`、`web_search` 等工具。
3. 在工具链尚未结束时，系统先把一段过程性说明句持久化为 assistant 最终文本，并将本轮标记为 `success=true`。
4. 发送链路随后立即把这段短句发送给用户，整轮会话收口。
5. 用户拿到的是“还要去补画像/补数据”的中间句，而不是其明确要求的正式分析。

## 期望效果

- 当用户明确要求 `value analysis`、组合评估或深度分析时，系统应在工具完成后输出正式结论，而不是把“还要去做什么”的计划句当作最终答复。
- `session.persist_assistant`、`done` 与 `reply.send` 应只在 answer 真正收敛、且不再继续拉起新工具后发生。
- 如果工具阶段失败导致无法完成分析，也应给出明确的失败/降级说明，而不是伪装成任务已经完成。

## 当前实现效果

- `2026-04-20 20:40` 的 `ASTS` 最新样本说明，这条缺陷在最近一小时里仍是活跃问题，而不是只停留在早盘和下午的孤立样本。
- 这轮 64 字文本不是简短但完整的事件判断，而是把“本地用户画像目录在当前沙箱里没有直接暴露出来”“我先核验时间线”这种内部执行限制与计划直接包装成 `final` 发给用户。
- 更关键的是，日志显示 `local_list_files`、`local_search_files`、`data_fetch`、`web_search` 都还在继续启动时，`session.persist_assistant/done` 与 `reply.send` 已先发生；用户收到过程句后只能在两分钟后重问同一句，系统才给出正式长答。
- `2026-04-20 08:54` 的 `TEMPUS AI` 最新样本说明，这条缺陷仍是当前线上活跃问题，而不是 4 月 16 日的单次偶发。
- `2026-04-20 20:00` 的 `GEV earnings reminder` 说明问题已经不只局限于人工直聊提问；scheduler 注入的同类分析任务也会在研究动作尚未结束时先落一条计划句。
- 这轮最新样本里，`local_list_files` / `local_search_files` 还先后失败，随后 `web_search` 才继续启动，但系统已经把 72 字过渡句记成整轮 `success=true`；即使没有后续投递失败，用户实际能看到的也只会是半成品。
- `2026-04-20 16:19` 的 `vistra energy` 最新样本进一步说明，问题不只表现为“最终只剩一句过程句”；它也会先把过程句作为一条可见 assistant 消息落进真实会话，导致用户在没有拿到正式分析前只能重复提问。
- 本轮返回内容不是简短但完整的摘要，而是明显的内部执行计划句：系统告诉用户“还缺一件事”“先看本地已有画像模板和写法”，却没有继续给出 `TEMPUS AI` 的估值或基本面判断。
- `GEV earnings reminder` 的最新 72 字文本同样不是财报判断本身，而只是“我先调取历史研究记录、再核验市场预期”的执行计划；这进一步证明问题是答复结构被截断，而不是模型选择了简短风格。
- `vistra energy` 这轮过程句同样暴露了内部执行轨迹：系统先告诉用户要去补查 actor 目录结构、抓取实时数据和新闻，随后才在下一次用户重问后完成正式答复。
- 日志还显示 `Tool: hone/local_list_files status=start` 与 `session.persist_assistant/done` 在同一秒交错，证明收口时序仍然允许“工具未结束 -> 先落最终答复”。
- 这已经不只是“答得偏短”的质量波动，而是答复结构被截断成过程说明，导致用户任务没有真正完成。

## 用户影响

- 这是质量类缺陷，不影响消息送达、会话持久化或系统稳定性，因此不属于 `P1/P2` 功能性故障。
- 之所以定级为 `P3`，是因为用户仍然收到了可读文本，没有出现无回复、错投、数据损坏或系统级失败。
- `2026-04-20 20:00` 的 scheduler 样本虽然最终又叠加了 Feishu `send_failed`，但这不改变本缺陷的定性：如果单看 answer 收口，系统仍先生成了错误的半成品 assistant。
- 但该文本没有完成用户明确提出的分析任务，用户需要重新追问或自行判断这句过程说明是否代表“系统还没答完”，体验明显劣化。

## 根因判断

- `2026-04-20 20:40` 的 `ASTS` 新样本说明，这个根因并不只会输出“我还缺一件事”“先补数据”这种抽象过渡句；它也会把 `company_profiles`、`沙箱`、`本地目录是否暴露` 这类明显属于内部执行面的排障语句直接当成用户可见最终答复。
- 高概率是 answer 阶段对“最终可见文本”的判定仍然过早，会把中间计划句或过渡句消费成 final。
- 从 `2026-04-20 08:54:47` 的日志顺序看，当前链路没有把“仍有新的 tool start 事件”视为禁止收口的信号。
- `TEMPUS AI` 最新样本还表明，这类提前收口不仅会输出“我正在补数据”的短句，也会把“我需要先看画像模板/决定是否回写”的内部执行计划直接暴露给用户。

## 下一步建议

- 优先排查 Feishu 直聊 answer 出站链路如何判定 `final`，确认是否会把中间计划句、画像准备句或进度句提前视为最终可发送文本。
- 对 `session.persist_assistant` / `done` 增加约束：若同一轮仍存在新的 tool start 事件，或最后一条可见文本明显是计划句/过渡句，不应直接结束本轮。
- 为这类样本补质量巡检信号：
  - 用户请求是分析型长答
  - `reply.chars` 极短
  - `done` 前后仍有工具事件
  - 最终文本含“我还缺一件事”“先看模板”“我再补数据”之类过程性措辞
