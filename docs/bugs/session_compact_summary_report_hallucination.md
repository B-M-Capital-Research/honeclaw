# Bug: 会话压缩摘要幻觉生成“用户报告”并回灌正式回答

- **发现时间**: 2026-04-15
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing（2026-04-23 架构改造已落地，待 24h 灰度复核）
- **证据来源**:

- 2026-04-23 23:00-23:01 最新同小时状态变化复核：
   - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
   - 同一会话在 `2026-04-23T23:00:00.481663+08:00` 再次触发定时任务 `核心观察股池晚间快报`；`cron_job_runs.run_id=5233` 于 `2026-04-23T23:01:07.397080+08:00` 记录为 `completed + sent + delivered=1`。
   - `session_messages` 同轮 assistant final 于 `2026-04-23T23:01:05.630926+08:00` 开头已恢复为正常的 `当前北京时间2026-04-23 23:00...`，不再包含 `Context compacted`。
   - 但同一 session 的上一轮 `run_id=5199` 在 `2026-04-23T21:37:05.557097+08:00` 仍把 `Context compacted` 直接写进最终回复；两轮相隔不到 90 分钟，说明当前是“同链路间歇恢复”，不是可确认的稳定修复。
   - 结论：最新 23:01 样本显示止血逻辑已在部分场景生效，但由于同一任务 21:35 仍刚刚复现，状态继续保持 `Fixing`，不能降级为 `Fixed`。

- 2026-04-23 21:35-21:37 最新同小时状态变化复核：
   - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
   - 定时任务 `科技核心股池 · 晚间击球区快报` 在 `cron_job_runs.run_id=5199`、`executed_at=2026-04-23T21:37:09.792230+08:00` 记录为 `completed + sent + delivered=1`。
   - `session_messages.ordinal=24` 的 assistant final 于 `2026-04-23T21:37:05.557097+08:00` 仍直接以 `Context compacted` 开头，然后才进入 25 支观察池晚间快报正文。
   - 同一任务在 `2026-04-22T21:36:01.630843+08:00` 的上一日样本 `run_id=4641` 已恢复为正常前缀；说明 2026-04-23 21:35 不是历史脏样本残留，而是用户可见 compact 标记在最新生产窗口重新外泄。
   - 结论：21:02 `OWALERT_PreMarket` 之后并未稳定收口，21:35 新样本再次证明定时任务出站仍会把 compact 标记投给用户；状态继续保持 `Fixing`，不能降级或关闭。

- 2026-04-23 21:00-21:02 最新同小时状态变化复核：
   - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
   - 定时任务 `OWALERT_PreMarket` 在 `2026-04-23T21:00:00.700150+08:00` 触发后，日志显示同轮持续执行 `skill_tool`、`data_fetch`、`web_search`，并伴随 Tavily `usage limit` 降级告警。
   - `data/runtime/logs/sidecar.log`
     - `2026-04-23 21:02:12.080` 再次记录 `runner internal compact signalled via status text: "Context compacted\n"`。
     - `2026-04-23 21:02:44.904` 记录 `step=session.persist_assistant detail=done`，随后 `done ... success=true elapsed_ms=164072 tools=14(Tool: hone/data_fetch,Tool: hone/skill_tool,Tool: hone/web_search)`。
   - `session_messages` 同轮 assistant final 于 `2026-04-23T21:02:44.901261+08:00` 仍直接以 `Context compacted` 开头，然后才进入盘前扫描正文。
   - 结论：20:01/20:02 两条定时任务“未外泄”的止血迹象没有扩展到 21:00 的最新真实播报，compact 标记仍会进入用户可见最终回复；状态继续保持 `Fixing`，不能降级或关闭。

- 2026-04-23 19:38-20:02 最新同小时状态变化复核：
   - `session_id=Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a`
   - 用户在 `2026-04-23T19:36:35.245543+08:00` 提问 `量子计算股票有哪些`，`session_messages.ordinal=18` 的 assistant final 于 `19:38:29.383833+08:00` 仍直接以 `Context compacted` 开头，说明 19:02 runtime 重启后同一小时内仍存在用户可见外泄。
   - 同一会话在 `2026-04-23T20:00:00.608307+08:00` 收到定时任务 `美股盘前与持仓新闻综述` 后，`session_messages.ordinal=20` 于 `20:01:57.267126+08:00` 已输出正常长文，不再包含 `Context compacted`。
   - 另一个定时任务会话 `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7` 也在 `2026-04-23T20:02:39.437955+08:00` 输出正常长文，`sessions.last_message_preview` 未见 compact 标记。
   - `data/runtime/logs/sidecar.log`
     - `2026-04-23 20:01:29.172` 记录 `runner internal compact signalled via status text: "Context compacted\n"`，说明新逻辑已经开始在 runner 层识别并拦截内部 compact 信号。
   - 结论：这条缺陷在 19:38 仍真实影响用户，但 20:00 两条后续任务已出现“内部识别成功、用户侧未再外泄”的止血迹象；状态继续保持 `Fixing`，并继续观察是否只是部分场景生效。

- 2026-04-23 13:50 根因复盘 + 架构改造落地（本次）：
   - **新增证据**：Telegram 出站链路也复现，chat `8039067465`（hone-test-bot）收到一条只有 `Context compacted` 的机器人消息。
   - **根因复合（实测确认）**：
     1. **codex-acp 内置 compact**：实测当 ACP session 内 token 用到 ~98%（251K/258K）时，
        codex 在处理下一轮 prompt 之前先 compact，并通过 `agent_message_chunk text="Context compacted\n"`
        通知客户端。ACP 协议 `session/*` 方法只暴露 `new/load/list/prompt/cancel/update/set_config_option/set_mode`，
        客户端无法主动控制 compact 时机。
     2. **opencode-acp 内置 compact**：实测在 ~85%（221K/256K）触发，**没有**任何字面量信号；
        而是直接把一段 `OK\n---\n## Goal\n## Constraints\n## Progress\n...\n## Relevant Files\n---\n
        I don't have an active task yet. How can I help you today?` 形式的 markdown summary
        拼到本轮 reply 后面推回客户端 —— 同样会被 honeclaw 当作模型正常回复投到 sink，
        造成另一种用户可见外泄。
     3. **gemini-acp**：实测**完全不推 `usage_update`**，无任何可观测的 compact 信号，且本机
        ACP 大 input 性能问题严重。**已全局禁用**（factory 层报错引导切换）。
     4. **honeclaw 自带 SessionCompactor 是冗余且有害的**：ACP 系列 runner 自带 session 持久化与
        内置 compact，honeclaw 这边再走自己的 24msg/48KB 阈值压缩，会写 `Conversation compacted`
        边界 + `【Compact Summary】...` 系统消息，又被回灌进下一轮 prompt（即本 bug 的旧根因）。
   - **改造方向（本次落地）**：
     1. **客户端入口丢弃 compact 字面量** — `acp_common.rs::handle_acp_session_update_with_renderer`
        识别 `Context compacted` / `Conversation compacted` chunk，drop 不进 final_reply / 不投 sink，
        同时置 `state.compact_detected = true`。
     2. **opencode summary 切断** — 一旦 `compact_detected`，识别后续 chunk 中 `\n---\n## ` 边界，
        保留前缀（模型对本轮 prompt 的真实回答），从边界开始 drop，并把后续 chunk 整块 drop。
     3. **opencode usage 骤降识别** — opencode 没有字面量，改用"流内首次 `usage_update.used`
        相对上一轮 peak 下降超 50%"作为信号，落到同一个 `compact_detected` 路径。
     4. **honeclaw 不再对 ACP runner 自动 compact** — `AgentRunner::manages_own_context()` 返回 true
        的 runner（codex_acp / opencode_acp），`HoneBotCore::maybe_compress_session` 直接短路返回，
        不再触发 SessionCompactor。
     5. **ACP runner 不再灌 `latest_compact_summary`** — `agent_session.rs::resolve_prompt_input` 与
        `scheduler.rs::run_heartbeat_task` 在 self-managed runner 下把 `bundle.conversation_context`
        清空，节流 ~30KB / 轮 input。honeclaw 自己的 `session_messages` 仍完整保留，用作跨 runner
        切换、UI 展示与 debug。
     6. **末端 sanitize 兜底** — `runtime.rs::sanitize_user_visible_output` 加 `RE_COMPACT_MARKER_LINE`,
        逐行匹配 `(context|conversation) compacted` 并丢弃，覆盖 multi_agent / 历史脏数据回放等场景。
     7. **gemini_acp 全局禁用** — `core.rs::create_runner_with_model_override` 的 `gemini_acp` 分支
        直接返回错误，引导用户切换到 codex_acp / opencode_acp / multi-agent / function_calling。
   - **保留路径**：
     - `agent_session.rs::force_compact_for_context_overflow`（LLM 报 context overflow 时的兜底）保留
     - `agent_session.rs` 中用户 `/compact` 命令的 manual 触发保留
     - 仅 `auto` 触发对 ACP runner 短路
   - **后续 SP reseed 优化（Pass 2，未实施）**：`acp_common` + 两个 ACP runner 已经把
     `acp_needs_sp_reseed` flag 写入 session_metadata；当前 prompt 构建层每轮都发 SP，flag 未消费。
     如果后续要进一步省 token（每轮少发 ~3KB），可以让 prompt 构建在 reseed flag=false 且非首轮时
     skip SP，在 reseed flag=true（compact 已发生）时强制重发 SP；不在本次范围。
   - **状态**：架构改造已落地、单测覆盖 4 个新用例，等待 24h 灰度复核 —— 跟踪
     `data/runtime/logs/acp-events.log` 与 `session_messages` / `cron_job_runs.response_preview`
     是否还有 `Context compacted` 开头的 assistant final 写入。


  - 会话: `Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb`
  - 最近一小时复现会话: `Actor_feishu__direct__ou_5f988206c4f2b110f0f8ce93f89c1eb07c`
- 2026-04-22 21:03 最新用户可见外泄复核：
   - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
   - `cron_job_runs.run_id=4626`（`OWALERT_PreMarket`，`executed_at=2026-04-22T21:03:32.723942+08:00`）记录 `execution_status=completed`、`message_send_status=sent`、`delivered=1`。
   - 同轮 `session_messages.ordinal=6` 的 assistant final 在 `2026-04-22T21:03:28.051552+08:00` 直接以 `Context compacted` 开头，然后才进入盘前扫描正文。
   - 同会话在 `2026-04-22T21:30:36.563117+08:00` 又写入 `role=system` 的 `Conversation compacted` 和 `【Compact Summary】...`，说明 summary 角色继续维持 system 态，但用户可见正文净化仍未闭环。
   - 这次已经送达用户，说明 20:05 后问题仍在真实定时任务出站链路活跃；状态继续维持 `Fixing`，不能关闭。

- 2026-04-23 04:00 最新 compact summary 语义污染复核：
   - `session_id=Actor_feishu__direct__ou_5f0e001c305cfc075babe830a9b2c6079c`
   - 用户在 `2026-04-23T03:39:44.493988+08:00` 提问 `如果有新增订单呢`，assistant 在 `2026-04-23T03:40:24.000785+08:00` 已给出 1248 字回答。
   - 同会话后续在 `2026-04-23T04:00:09.635294+08:00` 自动 compact，并于 `04:00:09.635312` 写入 `role=system` 的 `【Compact Summary】...`。
   - 该 summary 开头仍写“你的最新提问 **‘如果有新增订单呢’** 属于**尚未回答的新问题**”，与实际已回答历史不符。
   - 本轮未看到新的 `role=user` compact summary，也未看到 `Context compacted` 直接进入可见 assistant final；但 summary 本身仍会把错误“未回答新问题”回灌后续 prompt，说明 compact summary 的事实/边界校验仍未闭环，状态不能切到 `Fixed`。
- 2026-04-22 20:05 最新用户可见外泄复核：
   - `session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`
   - 同会话在 `2026-04-22T20:00:37.102052+08:00` 写入 `role=system` 的 `Conversation compacted`，紧接着 `20:00:37.102063` 写入 `role=system` 的 `【Compact Summary】...`，说明新生成 summary 角色继续维持 system 态。
   - 用户在 `2026-04-22T20:02:11.211146+08:00` 追问 `不局限在科技股 关注A股所有标的 还有哪些值得现在投资？`
   - `session_messages.ordinal=10` 的 assistant final 在 `2026-04-22T20:05:22.460217+08:00` 仍直接以 `Context compacted` 开头，然后才进入 A 股全市场配置建议。
   - 这再次证明问题已经收敛为“压缩后首条/近邻最终输出缺少可见文本净化”：summary 角色已改善，但用户可见正文仍会泄漏内部压缩标记。状态继续维持 `Fixing`，不能关闭。
- 2026-04-22 16:50 最新用户可见外泄复核：
   - `session_id=Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a`
   - 用户在 `2026-04-22T16:49:02.504465+08:00` 提问 `分析一下今天VRT的财务会Beat还是Miss。`
   - `session_messages.ordinal=18` 的 assistant final 在 `2026-04-22T16:50:37.876162+08:00` 仍直接以 `Context compacted` 开头，然后才进入 VRT 财报 Beat/Miss 判断。
   - `data/runtime/logs/acp-events.log` 同轮在 `2026-04-22T08:50:17.678745+00:00` 也记录 `agent_message_chunk` 内容为 `Context compacted\n`，说明压缩状态标记来自 runner 输出流并进入最终可见正文。
   - 这再次证明 14:38 后问题没有自然收口：压缩后首条最终输出仍缺少可见文本净化。状态继续维持 `Fixing`，不能关闭。
- 2026-04-22 14:38 最新用户可见外泄复核：
   - `session_id=Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc`
   - 用户在 `2026-04-22T14:35:45.460823+08:00` 提问 `国产AI服务器厂商有哪些？`。
   - 同会话在 `2026-04-22T14:36:08.979650+08:00` 写入 `role=system` 的 `Conversation compacted`，紧接着 `14:36:08.979668` 写入 `role=system` 的 `【Compact Summary】...`，说明新生成 summary 角色仍维持 system 态。
   - 但 `session_messages.ordinal=8` 的 assistant final 在 `2026-04-22T14:38:43.349287+08:00` 仍直接以 `Context compacted` 开头，然后才回答国产 AI 服务器厂商分类、浪潮信息/新华三/紫光股份/联想/宁畅/超聚变/华为等内容。
   - 这说明问题已进一步收敛为“压缩后第一条最终输出缺少可见文本净化”：角色落库已改善，但用户可见正文仍泄漏内部压缩标记。状态继续维持 `Fixing`，不能关闭。
- 2026-04-22 09:03 最新用户可见外泄复核：
   - `session_id=Actor_feishu__direct__ou_5fe09f5f16b20c06ee5962d1b6ca7a4cda`
   - `cron_job_runs.run_id=4400`（`早9点市场复盘(XME及加密ETF)`，`executed_at=2026-04-22T09:03:58.992153+08:00`）记录 `execution_status=completed`、`message_send_status=sent`、`delivered=1`。
   - 同轮 `session_messages.ordinal=16` 的 assistant final 在 `2026-04-22T09:03:32.070747+08:00` 直接以 `Context compacted` 开头，然后才进入 XME、BTC/ETH/SOL 与宏观大盘复盘正文。
   - 这说明 08:33 之后同类用户可见格式污染仍在下一小时继续复现；问题不是单条报告偶发，而是压缩后第一条最终输出仍缺少发送前净化。
   - 状态继续维持 `Fixing`：新生成 summary 角色已有收敛迹象，但可见输出净化与存量 summary 隔离仍未闭环。
- 2026-04-22 08:33 最新用户可见外泄复核：
   - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
   - `cron_job_runs.run_id=4383`（`美股AI产业链盘后报告`，`executed_at=2026-04-22T08:33:21.473282+08:00`）记录 `execution_status=completed`、`message_send_status=sent`、`delivered=1`。
   - 同轮 `session_messages.ordinal=10` 的 assistant final 在 `2026-04-22T08:33:18.287474+08:00` 直接以 `Context compacted` 开头，然后才进入正式盘后 AI 产业链报告。
   - 这次不再只是 prompt 里带入存量 `role=user` summary；压缩状态标记已经进入已送达正文，属于用户可见格式污染和内部状态外泄。
   - 同一会话随后 `08:34:58` 与 `08:47:28` 的两条后续 assistant final 不再以 `Context compacted` 开头，说明症状可能是压缩后第一条输出穿透，但发送侧尚未统一清洗该标记。
   - 状态继续维持 `Fixing`：新生成 summary 角色已有收敛迹象，但可见输出净化与存量 summary 隔离仍未闭环。
- 2026-04-22 05:00 最新 prompt 污染复核：
   - `session_id=Actor_feishu__direct__ou_5f895bed1573d53053e89bfc382b523a44`
   - `session_messages` 中仍保留 `2026-04-20T21:31:24.260047+08:00` 的 `role=user` `【Compact Summary】...`，内容覆盖 `TEM / RKLB / BE / MSFT / BOXX / YINN / MU / LITE` 等持仓与交易纪律。
   - 最新真实任务是 `2026-04-22T05:00:00.105203+08:00` 的 `[定时任务触发] 任务名称：科技成长赛道大盘极值与情绪监控`，用户只要求盘后扫描纳指、ARKK、成长股与 VIX 极值信号。
   - `data/runtime/logs/acp-events.log` 在 `2026-04-21T21:00:03.002554+00:00` 的 `session/update` 中把上述旧 `【Compact Summary】`、`【历史对话总结】` 与本轮定时任务输入一起送入 runner。
   - `2026-04-22T05:00:48.663741+08:00` assistant 最终正常生成盘后扫描并送达；这说明当前主要风险不是本轮无回复，而是存量 `role=user` compact summary 仍会作为真实用户上下文进入 prompt，继续影响后续投资类回答的事实边界和个性化偏置。
   - 因此状态继续维持 `Fixing`：新生成 summary 角色已有收敛迹象，但历史 `role=user` summary 未迁移/隔离前，生产 prompt 污染仍未闭环。
- 2026-04-22 04:00 最新 prompt 污染复核：
   - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
   - `session_messages` 中仍保留 `2026-04-20T21:30:28.320382+08:00` 的 `role=user` `【Compact Summary】...`，内容覆盖 `COHR / RKLB / GEV / SNDK / MU / BE / VST / CIEN / GOOGL / AVGO / TEM` 等股票关注表。
   - 最新真实任务是 `2026-04-22T04:00:00.213839+08:00` 的 `[定时任务触发] 任务名称：Oil_Price_Monitor_Closing`，用户只要求收盘前原油价格与对 COHR/RKLB 等科技股的影响判断。
   - `data/runtime/logs/acp-events.log` 在同轮 `user_message_chunk` 中把这条旧 `【Compact Summary】`、`【历史对话总结】` 与本轮定时任务输入一起送入 runner，并且 summary 里还带有“OWALERT_PreMarket 尚未完成正式输出”等旧任务状态。
   - `2026-04-22T04:01:10.452480+08:00` assistant 虽然给出了原油回复，但说明生产 prompt 仍会消费历史 `role=user` compact summary；问题已从“新生成 summary 是否仍写成 user”转为“存量 user-summary 污染仍未迁移/隔离”。因此状态继续维持 `Fixing`，不能关闭。
- 2026-04-22 00:00 最新状态变化复核：
   - `session_id=Actor_feishu__direct__ou_5fe09f5f16b20c06ee5962d1b6ca7a4cda`
   - `2026-04-21T23:16:37.180473+08:00` 会话 auto compact 后写入 `role=system` 的 `Conversation compacted`
   - `2026-04-21T23:16:37.180494+08:00` 随后的 `【Compact Summary】...` 也已写成 `role=system`，不再是此前的 `role=user`
   - `session_id=Actor_feishu__direct__ou_5f79ee8185333e5db4a55e5eca0d8d2f7e`
   - `2026-04-21T23:59:54.708566+08:00` 写入 `role=system` 的 `Conversation compacted`
   - `2026-04-21T23:59:54.708583+08:00` 随后的 `【Compact Summary】...` 同样为 `role=system`
   - 这说明“新生成 compact summary 继续以 `role=user` 落库”的症状在最新两条直聊 auto compact 样本里已有收敛迹象；但 2026-04-21 21:02 仍有 `Context compacted` 穿透进最终 assistant 正文，且本轮未完整验证 `role=system` summary 是否一定不会进入 prompt，因此状态维持 `Fixing`。
- 2026-04-21 21:02 最新可见文本外泄样本：
   - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
   - `2026-04-21T21:00:00.426553+08:00` 真实用户消息为 `[定时任务触发] 任务名称：OWALERT_PreMarket`
   - `2026-04-21T21:02:47.997793+08:00` assistant 最终内容直接以 `Context compacted` 开头，然后才进入盘前扫描正文
   - 对应 `cron_job_runs.run_id=4136` 记录该任务 `execution_status=completed`、`message_send_status=send_failed`，说明压缩标记已经进入最终可见正文，即使本轮又被 Feishu 发送 400 阻断，落库文本本身已经污染。
   - 这不是单纯的内部 `role=user` summary 落库问题；压缩状态标记已经穿透到用户可见答复正文，属于输出格式/内部状态外泄。
- 2026-04-21 20:00-20:02 最新复现：
   - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
   - `2026-04-21T20:00:00.291431+08:00` 真实用户消息为 `[定时任务触发] 任务名称：A股盘后高景气产业链推演`
   - `data/runtime/logs/sidecar.log` 在 `20:00:00.299913` 记录 `Compressing session ... with 24 messages (~82140 bytes)`，`20:00:36.440438` 记录 `summary_chars=2641`，随后 `20:00:36.449352` 记录 `compacted to boundary + summary + 6 retained items`
   - `session_messages` 在 `2026-04-21T20:00:36.441080+08:00` 写入 `role=system` 的 `Conversation compacted`
   - 紧接着 `2026-04-21T20:00:36.441099+08:00` 又写入 `role=user` 的 `【Compact Summary】...`，内容是 A 股高景气观察池表格，覆盖 `300308 / 300502 / 300394 / 688498` 等标的与“助手的观点 / 用户的观点”列
   - 日志在 `20:00:36.449574` 继续进入 `restore_context + build_prompt + create_runner`，并在 `20:01-20:02` 连续调用 `skill_tool` 与 `data_fetch`，说明最新 scheduler 会话仍在压缩后把内部摘要当作真实 user transcript 继续进入回答链路。
- 2026-04-21 18:55-18:57 最新复现：
   - `session_id=Actor_feishu__direct__ou_5ff0946a82698f7d16d9a5684696c84185`
   - `2026-04-21T18:54:52.082570+08:00` 用户真实输入为 `预判一下美股纳斯达克指数今天开盘后的走势`
   - `data/runtime/logs/web.log` 在 `18:54:52-18:55:04` 记录同一会话 `Compressing session ... with 22 messages`，随后 `compacted to boundary + summary + 6 retained items`
   - `session_messages` 在 `2026-04-21T18:55:04.787603+08:00` 先写入 `role=system` 的 `Conversation compacted`
   - 紧接着 `2026-04-21T18:55:04.787621+08:00` 又写入 `role=user` 的 `【Compact Summary】...`，内容甚至写出 `这不是历史总结任务，无法执行`，并列出 `ANET / EQIX / DLR / SMCI / BE / WDC / COHR / LITE / AXTI / GOOGL`
   - `2026-04-21T18:57:04.401376+08:00` assistant 随后继续产出纳指开盘预判正式回答，说明最新生产链路仍会在真实用户新问题前实时生成并落库 `role=user` compact summary；问题不是旧会话存量污染。
- 2026-04-21 17:49-17:51 最新复现：
   - `session_id=Actor_feishu__direct__ou_5f988206c4f2b110f0f8ce93f89c1eb07c`
   - `session_messages` 中仍保留 `2026-04-20T11:37:42.915681+08:00` 的 `system` 消息 `Conversation compacted`
   - 紧接着 `2026-04-20T11:37:42.915697+08:00` 仍是 `role=user` 的 `【Compact Summary】...`，内容覆盖 `TEM / TSLA / 海力士 / VCX / 9992.HK / 06656.HK / AVGO / 地平线` 等股票关注表
   - 最新真实用户输入为 `2026-04-21T17:49:30.011371+08:00` 的 `那rklb 呢`
   - `data/runtime/logs/acp-events.log` 在 `2026-04-21T09:49:35Z` 的 `user_message_chunk` 中仍把同一 `【Compact Summary】` 与本轮用户输入一起送入 runner
   - `2026-04-21T17:51:42.901297+08:00` assistant 随后产出 RKLB 正式分析，说明最新生产回答仍会消费这条以 `role=user` 身份回灌的 compact summary；问题不是只停留在历史落库层。
- 2026-04-21 15:54-15:58 最新复现：
   - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
   - `2026-04-21T15:54:56.700908+08:00` 会话先写入 `system` 消息 `Conversation compacted`
   - 紧接着 `2026-04-21T15:54:56.700944+08:00` 又写入 `role=user` 的 `【Compact Summary】...`，内容是 `22支观察池 · 正式总表`，覆盖 `MSFT / NVDA / GOOGL / AAPL / AVGO / AMZN / META` 等观察池与击球区信息
   - 同一会话随后处理真实用户请求 `再加一个BE，AMD进来，然后24支股排优先级名单，还有一版击球区距离表。`，并在 `2026-04-21T15:58:24.885618+08:00` 正式回答 24 支观察池更新
   - 这说明最新生产链路仍会把 compact summary 以真实 `user` transcript 身份写回，并继续驱动后续投资组合类回答；问题不是上午旧样本残留。
- 2026-04-21 16:05-16:08 最新复现：
   - `session_id=Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34`
   - `2026-04-21T16:05:35.978776+08:00` 会话先写入 `system` 消息 `Conversation compacted`
   - 紧接着 `2026-04-21T16:05:35.978791+08:00` 又写入 `role=user` 的 `【Compact Summary】...`，内容是 `股票关注表`，覆盖 `RKLB / TEM / IONQ / 量子计算 / 美股科技股` 等历史关注项
   - 同一会话真实用户新问题是 `美股亚川在光通信行业被低估吗`，随后 `2026-04-21T16:08:57.128142+08:00` assistant 继续基于该会话上下文回答 ADTRAN/ADTN 分析
   - 这说明 16:00 之后生产链路仍会在新问题前把 compact summary 作为真实 `user` transcript 插入，而不是只保存在系统态 summary 字段。
- 2026-04-21 10:52-11:14 最新连续复现：
   - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
   - `2026-04-21T10:52:02.385261+08:00` 会话先写入 `system` 消息 `Conversation compacted`
   - 紧接着 `2026-04-21T10:52:02.385272+08:00` 又写入 `role=user` 的 `【Compact Summary】...`，内容是完整 `股票关注表` 与“助手的观点 / 用户的观点”列
   - `session_id=Actor_feishu__direct__ou_5f44da57b6746474d4497f091b9f772b87`
   - `2026-04-21T10:59:14.072609+08:00` 写入 `system` 边界消息后，`2026-04-21T10:59:14.072620+08:00` 又写回 `role=user` 的 `【Compact Summary】...`，内容覆盖 `002353.SZ / 300049.SZ / 02259.HK` 等前序股票关注表
   - 同一会话随后在 `2026-04-21T11:01:30.668657+08:00` 继续正式回答“如果黄金上涨，它和02899对比哪个更合适？”，说明这条 `role=user` summary 仍进入后续上下文窗口
   - `session_id=Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`
   - `2026-04-21T11:09:54.781705+08:00` 写入 `system` 边界消息后，`2026-04-21T11:09:54.781714+08:00` 又写回 `role=user` 的 `【Compact Summary】...`，内容覆盖 `CRDO / VST / NUAI` 等历史画像与观点
   - 同一会话随后在 `2026-04-21T11:14:05.205080+08:00` 正式回答“请详细分析下高通”，说明最新生产链路仍是“summary 以 user 身份回灌，再继续回答新问题”
   - 这三个会话在同一小时内连续复现，说明问题不是单个旧会话残留，而是当前 auto compact 持续把摘要写入真实 user transcript。
- 2026-04-21 10:52-10:59 最近一小时最新复现：
   - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
   - `2026-04-21T10:52:02.385261+08:00` 会话先写入 `system` 消息 `Conversation compacted`
   - 紧接着 `2026-04-21T10:52:02.385272+08:00`，`session_messages` 又再次写回 `role=user` 的 `【Compact Summary】...`，内容仍是完整 `股票关注表` 与“助手的观点 / 用户的观点”列，而不是系统态摘要元数据
   - 随后同一会话在 `2026-04-21T10:54:09.279122+08:00` 与 `2026-04-21T10:58:15.980432+08:00` 连续给出仓位复盘正式回答，说明所谓“2026-04-20 已改为写 `role=system` 并在 restore 跳过”的修复结论并未落到当前生产链路；auto compact 仍在持续生成新的 `role=user` 污染样本
- 2026-04-20 16:51 最近一小时最新复现：
   - `session_id=Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc`
   - `2026-04-20T16:50:45.690+08:00` 用户真实输入是：`分析一下DELL，并综合对比DELL、HPE、CRWV三家公司`
   - `data/runtime/logs/web.log` 记录 `2026-04-20 16:50:45.691` `Compressing session ... with 21 messages`，随后 `16:51:21.587` 记录 `compacted to boundary + summary + 6 retained items`
   - 紧接着 `session_messages` 在 `2026-04-20T16:51:21.579381+08:00` 再次写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 仍然不是系统态元数据，而是完整股票关注表与观点文本，直接列出 `CLS / DELL / HPE / CRWV / 曦智科技 / 特斯拉` 等标的的“助手的观点 / 用户的观点”
   - 随后 `2026-04-20T16:55:48.792364+08:00` assistant 虽然成功返回 DELL 正式分析，但说明到最近一小时窗口结束时，compact summary 仍会实时以真实 `user` transcript 身份插到新问题前进入 prompt，而不是只保存在内部 summary 字段
- 2026-04-20 15:36 最近一小时最新复现：
   - `session_id=Actor_feishu__direct__ou_5fe09f5f16b20c06ee5962d1b6ca7a4cda`
   - `2026-04-20T15:36:31.232945+08:00` 会话自动 compact 后，先写入 `system` 边界消息 `Conversation compacted`
   - 紧接着 `2026-04-20T15:36:31.233259+08:00` 又再次写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 仍然不是系统态元数据，而是完整的股票关注表与结论文本，直接列出 `3042.HK / DRAM / BTC / 港股科技股` 等标的的“助手的观点 / 用户的观点”
   - 同轮真实用户新问题是 `股價的上漲是由什麽推動的，資金量？`，随后 `2026-04-20T15:37:05.557727+08:00` assistant 给出 DRAM 驱动分析；一分钟后用户又追问 `成交量和换手率具體怎麽看`
   - `data/runtime/logs/web.log` 同步记录 `2026-04-20 15:36:02.437` `Compressing session ... with 22 messages`、`15:36:31.239` `compacted to boundary + summary + 6 retained items`，随后 `15:37:05.564` `session.persist_assistant detail=done`
   - 这说明到本轮巡检时，compact summary 仍会以真实 `user` transcript 身份插在新问题前进入 prompt，而不是只保存在内部 summary 字段；问题已从 11 点窗口继续延续到 15:36 的最新直聊样本
- 2026-04-20 11:37 最近一小时最新复现：
   - `session_id=Actor_feishu__direct__ou_5f988206c4f2b110f0f8ce93f89c1eb07c`
   - `2026-04-20T11:37:42.915697+08:00` 会话自动 compact 后，再次写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 不是系统态元数据，而是完整的股票关注表与结论文本，直接列出 `TEM / 地平线机器人 / Dell / IREN / COHR` 等标的的“助手的观点 / 用户的观点”
   - 紧接着同轮真实用户新问题是：`cohr呢`
   - `2026-04-20T11:38:51.090650+08:00` assistant 虽然表面成功回答了 COHR，但这说明在最近小时窗里，compact summary 仍会作为真实 `user` transcript 插入新问题前的 prompt，而不是只存入内部 summary 字段
- 2026-04-20 10:11-10:46 最近一小时最新复现：
   - `session_id=Actor_feishu__direct__ou_5f721e2f6a672bf212d3056d02d931faa0`
   - `2026-04-20T10:11:17.155738+08:00` 会话自动 compact 后，再次写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 不是系统态元数据，而是完整的股票关注表与观点摘要，直接列出 `SK海力士 / WDC / CLS / CAI / TEM` 等标的的“助手的观点 / 用户的观点”
   - 紧接着同轮真实用户输入是：`群核科技现在能买入吗？市值预计是多少？`
   - `2026-04-20T10:12:23.235671+08:00` assistant 虽然成功回答了群核科技问题，但这说明最新生产链路仍会在新问题开始前把 summary 作为真实 `user` transcript 注回上下文，而不是隔离到内部 summary 字段
   - `session_id=Actor_feishu__direct__ou_5fe38548e3fd8217c052d3ddb70fb3c918`
   - `2026-04-20T10:46:53.190178+08:00` 另一条 Feishu 直聊在 auto compact 后又一次写回 `role=user` 的 `【Compact Summary】...`
   - 该 summary 同样是长表格和结论文本，不是系统态压缩记录；内容覆盖 `SDCKQ / TEM / ORCL / GOOGL / CRCL / CLS` 等标的
   - 同轮真实用户新问题是：`详细分析DELL这家公司，并详细分析其是否可以买入`
   - `2026-04-20T10:51:43.214064+08:00` assistant 随后给出 DELL 正式分析，说明这个问题在最近一小时仍以“summary 先回灌、再继续正常答新题”的形态活跃存在，而不是旧样本残留
- 2026-04-19 23:37-23:47 最近一小时最新复现：
   - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
   - `2026-04-19T23:37:58.122025+08:00` 会话自动 compact 后，再次写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 不是系统态元数据，而是完整的股票关注表与结论摘要，直接写出 `GOOGL / MSFT / NVDA / META / AVGO / TSM / AMZN / AAOI / ASTS` 等标的的“助手的观点 / 用户的观点”
   - 紧接着同轮真实用户输入是：`Hi hone.看一下台机电财报Q1营收1.1亿同比35%，可以买台机电吗？`
   - `2026-04-19T23:39:14.513774+08:00` assistant 虽然成功回答了 TSM 问题，但这说明最新生产链路仍会在新问题开始前把 summary 作为真实 `user` transcript 注回上下文，而不是隔离到内部 summary 字段
   - `session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`
   - `2026-04-19T23:47:26.795360+08:00` 另一条 Feishu 直聊在 auto compact 后又一次写回 `role=user` 的 `【Compact Summary】...`
   - 该 summary 同样是长表格和持仓/结论文本，不是系统态压缩记录；内容覆盖 `CAR / GOOGL / NVDA / MRVL / DELL / BRK.B` 等标的
   - 同轮真实用户新问题是：`你这个版本，年化利率和回撤大概是多少`
   - `2026-04-19T23:46:37.910834+08:00` assistant 先回落成友好超时文案，之后 `2026-04-19T23:56:54.987236+08:00` 才继续给出正式组合评价；说明 compact summary 角色错误在最近一小时不只持续存在，还叠加了同会话超时/重试抖动
- 2026-04-19 21:37 最近一小时最新样本：
   - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`
   - `2026-04-19T21:37:07.311200+08:00` 会话再次自动 compact，并写回 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 已经不是中性历史摘要，而是直接写出“这个问题涉及卫星变轨和部署的具体技术时间表，我没有确切的最新数据可以准确回答”，本质上是对上一轮 ASTS 问题的答复草稿
   - 同轮真实用户输入是：`你思想逻辑现在有一点错乱我刚刚的回答是对于你发给我的asts 你和我说到甲骨文去`
   - `2026-04-19T21:37:27.746598+08:00` assistant 最终回复虽然承认“上下文串线了”，但 `web.log` 记录这一轮仍是先 `Compressing session ... with 22 messages`，再继续 `restore_context + build_prompt + create_runner`
   - 说明线上会话到 `2026-04-19 21:37` 仍会把 compact summary 当作真实用户消息写回，再进入后续回答链路；影响已从“旧持仓/旧报告污染”扩展到 `ASTS -> ORCL` 的话题串线
- 2026-04-19 16:41 最近一小时导入样本：
   - `session_id=Actor_feishu__direct__ou_5fb47bd113e7776b05e7a5c2c56e310652`
   - `session_messages.imported_at=2026-04-19T16:41:31.133907+08:00` 的最新导入批次里，再次出现 `role=user` 的 `【Compact Summary】...`
   - 这条 summary 的原始 `timestamp=2026-04-19T14:49:30.823580+08:00`，内容仍是 `TEM / RKLB` 等持仓表与“助手的观点 / 用户的观点”列，而不是系统态摘要元数据
   - 同一批导入里，随后还能看到 `16:12-16:36` 的正式 assistant 连续回答都围绕该用户既有持仓与投资风格展开，说明这条 summary 仍作为真实 transcript 被保留下来，而不是只存在于内部 summary 字段
   - 最后 `2026-04-19T16:41:31.132922+08:00` 又落入一条用户可见 `抱歉，这次处理失败了。请稍后再试。`，说明本轮导入不只是历史脏数据回灌，而是当前活跃直聊在被污染 transcript 上继续运行并收口失败
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
 - 2026-04-19 12:44-12:45 最近一小时复核：
   - `session_id=Actor_feishu__direct__ou_5f0b28ce5f1fb395b9f677fdf52a4401be`
   - `2026-04-19T12:44:05.314086+08:00` 用户真实输入仅为：`研究一下900943这家公司`
   - `2026-04-19T12:44:28.433467+08:00` 会话再次写入 `system` 消息 `Conversation compacted`
   - 紧接着 `2026-04-19T12:44:28.433529+08:00`，`session_messages` 又写入 `role=user` 的 `【Compact Summary】...`，内容仍是 `TEM / RKLB` 等持仓与“助手的观点 / 用户的观点”表格，而不是系统态摘要元数据
   - `2026-04-19T12:45:15.488756+08:00` assistant 正式回复虽然表面回答了 `900943`，但正文直接引入“完全脱离你的美股科技投资主线”“你目前的持仓（MU、ALAB、RKLB等）”等个性化判断，明显继承了刚被回灌的旧摘要上下文，而不是围绕新问题本身做中性研究
   - 这说明最新小时窗里，compact summary 角色错误已经不只污染失败链路，也会直接把旧持仓语境带入全新的个股问答，改写正式回答的立场和范围
- 2026-04-19 23:37-23:47 最近一小时再次复核：
   - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
   - `2026-04-19T23:37:58.122025+08:00` auto compact 后，`session_messages` 再次写入 `role=user` 的 `【Compact Summary】...`，内容仍是完整股票关注表，而不是系统态摘要元数据
   - `2026-04-19T23:39:14.513774+08:00` assistant 随后回答 `TSM` 财报问题，说明普通直聊新问题在进入回答前仍会先消费这条被回灌的 summary
   - `session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`
   - `2026-04-19T23:47:26.795360+08:00` 同样再次写入 `role=user` 的 `【Compact Summary】...`，内容是 `CAR / GOOGL / NVDA / MRVL / DELL / BRK.B` 等标的的结构化表格
   - 随后同一会话先在 `2026-04-19T23:46:37.910834+08:00` 回落成“抱歉，处理超时了。请稍后再试。”，再在 `2026-04-19T23:56:54.987236+08:00` 给出正式组合结论
   - 这说明最近一小时里，`Compact Summary` 的角色错误依旧是实时生成的生产问题，而不是白天旧样本延迟导入；同时它已经与同会话超时/重试链路叠加

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

- 2026-04-21 20:00 最新样本说明，scheduler 触发会话在 auto compact 后仍实时把 `Compact Summary` 写成 `role=user`，随后立即进入 `restore_context + build_prompt + create_runner`；问题仍是当前生产链路实时生成，不是旧会话存量污染。
- 2026-04-21 21:02 最新样本说明，`OWALERT_PreMarket` 的最终 assistant 文本直接以 `Context compacted` 开头，压缩状态不再只停留在 transcript 角色污染，而是会进入最终可见输出。
- 2026-04-22 08:33 最新样本说明，同类 `Context compacted` 外泄已出现在 `completed + sent + delivered=1` 的 `美股AI产业链盘后报告`，说明发送成功路径仍缺少压缩标记清洗。
- 2026-04-21 18:55 样本说明，auto compact 仍在当前生产链路实时把 `Compact Summary` 写成 `role=user`；即使回答表面完成，真实 transcript 已被内部压缩产物污染。
- 2026-04-21 17:49 最新样本说明，旧 `role=user` compact summary 不只是存量脏数据；它在新的 `那rklb 呢` 直聊请求中仍被恢复进 runner 输入，并与本轮真实 user turn 同时进入 prompt。
- 2026-04-21 16:05 最新样本说明，compact summary 仍会以 `role=user` 写入会话；同轮后续正式回答继续处理“美股亚川”新问题，生产链路没有收口到“summary 只作为系统态元数据”。
- 2026-04-21 15:54 样本同样说明，另一条观察池会话在 compact 后写入 `role=user` 的 22 支观察池 summary，并继续处理 24 支观察池更新请求。
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
- `16:41` 的最新导入样本说明，这个问题不只表现为“旧时间窗还有脏数据没清掉”。`Actor_feishu__direct__ou_5fb47bd113e7776b05e7a5c2c56e310652` 在最近一次导入里仍把 `14:49` 的 `Compact Summary` 作为 `role=user` 保存进当前活跃会话，同时同批次还包含 `16:12-16:36` 的后续正式回答和 `16:41` 的失败收口；也就是说，生产 transcript 仍在被这一类 summary 真实参与、真实消费。
- `21:37` 的 ASTS 最新样本进一步说明，污染范围已从“旧持仓/旧报告”扩展到当前话题边界本身：用户刚指出系统把 `ASTS` 说成了 `ORCL`，auto compact 却仍把上一轮答复草稿写回为 `role=user`，随后正式回答只能在被污染的上下文里承认“串线”而非真正隔离错误摘要。
- 因而当前缺陷的主表现一度收敛为两点：一是 summary 角色仍错误，二是 summary 仍会在后续回答前重写本轮输入语义；这两点都没有被此前修复覆盖。2026-04-22 00:00 最新复核显示，`role=user` 写库症状在最新两条样本里已有收敛，但仍需继续验证 prompt 隔离与最终可见文本外泄是否彻底停止。

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
9. `2026-04-19 12:44` 的最新直聊样本说明，这条缺陷已经继续改写正式回答的价值判断：新问题只是“研究一下900943这家公司”，但 answer 仍直接沿用刚回灌的美股持仓上下文，扩写成“是否符合你的美股科技主线”的组合建议。
10. `2026-04-19 23:37` 与 `23:47` 的双样本进一步说明，问题并未局限在某个历史污染会话。最近一小时两个不同 Feishu 直聊都再次生成新的 `role=user` `Compact Summary`，证明线上 transcript 污染仍是当前时态的活跃问题。
11. `2026-04-20 10:11` 与 `10:46` 的最新双样本进一步说明，哪怕上一轮正式回答表面可读，这个缺陷依旧会在“新问题进来之前”实时生成新的 `role=user` `Compact Summary`；因此当前线上仍在持续制造新污染，而不是只剩历史遗留数据。
12. `2026-04-20 16:51` 的最新 DELL 对比样本进一步证明，这个问题到本轮巡检结束前仍在生产持续发生。虽然随后正式回答可读，但 `Compact Summary` 仍先以真实 `user` 消息落库，说明“旧污染还在库里”已经不是准确描述，当前链路仍在继续制造新的污染 transcript。

## 修复情况（2026-04-17）

1. `crates/hone-channels/src/session_compactor.rs` 现在把 compact summary 以 `role=system` 写回会话，不再伪装成新的 `role=user` 输入。
2. `crates/hone-channels/src/agent_session.rs` 的 `restore_context()` 现在会显式跳过 `session.compact_summary` 消息，避免这段摘要再作为普通用户消息进入后续 runner transcript。
3. `crates/hone-channels/src/prompt.rs` 现在优先读取 `session.summary` 并统一转换为 `【历史会话总结】` 注入本轮 prompt；旧会话里遗留的 `【Compact Summary】` 消息只作为兼容 fallback 读取，不再把原始标记文本直接塞回用户输入区。
4. 新增/更新的回归测试已经覆盖：
   - prompt 组装优先使用 `session.summary`，不会继续引用旧的 compact summary 消息正文
   - compact boundary 后的 restore 不再把 compact summary 当成普通用户消息恢复
   - `recv_extra` 仍然位于历史摘要之前，避免群聊补充上下文顺序被这次修复破坏
5. 代码层修复已完成并通过 crate 级测试；当时文档曾更新为 `Fixed`。但 2026-04-19 06:52 的真实 Feishu 会话再次落入 `role=user` 的 `【Compact Summary】`，而且 2026-04-19 09:00 的 `特斯拉与火箭实验室新闻日报` 也在 auto compact 后重现同样落库方式，说明“代码修复通过测试”并不等于线上 transcript 已恢复。
6. `2026-04-19 12:44` 的 `900943` 新样本进一步说明，线上问题不只停留在 transcript 污染本身；`role=user` 的旧摘要已经会直接进入 answer 立场判断，把原本中性的公司研究题改写成围绕既有持仓风格的建议输出。
7. `2026-04-19 23:37` 与 `23:47` 的最新双样本又证明，这并不是某条旧摘要未清理干净的残留问题，而是 auto compact 在当前生产会话里仍会新生成 `role=user` summary，并在后续正常问答前参与真实上下文组装。
8. `2026-04-20 10:11` 与 `10:46` 的最新双样本说明，这种“新生成 summary 再参与下一题”的行为今天上午仍在继续；即使后续 `群核科技` 与 `DELL` 答案可读，真实 transcript 仍先被错误角色的压缩摘要污染。

## 最新真实样本复核（2026-04-22 00:00 CST）

- `2026-04-21 23:16` 与 `23:59` 两条最新直聊 auto compact 样本均显示 `【Compact Summary】` 已改为 `role=system` 落库。
- 本轮没有发现新的 `role=user` compact summary 样本，也没有发现 23:00-00:00 窗口内新的 `Context compacted` 可见外泄。
- 但 `2026-04-21 21:02` 的可见正文污染仍是同日生产证据；因此本轮只更新为“部分症状收敛”，不把状态改为 `Fixed` 或 `Closed`。

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

## 修复情况（2026-04-20）

根因确认：前序修复（b3d5102）只收紧了摘要内容，但 `session_compactor.rs` 仍把 compact summary 写成 `role=user`，导致 `restore_context` 继续把它作为用户消息注回历史。

本次修复：
1. `crates/hone-channels/src/session_compactor.rs:331` — 将 compact summary 写库角色从 `"user"` 改为 `"system"`
2. `crates/hone-channels/src/agent_session.rs` — `restore_context` 的 `"user"` 分支：遇到 compact_summary 元数据直接跳过（兼容旧数据）；`"system"` 分支：同时跳过 compact_boundary 和 compact_summary
3. 更新所有相关回归测试（共 4 个），期望值从"含 summary"改为"只含真实消息"；`resolve_prompt_input` 测试仍通过（summary 通过 `conversation_context` 正常出现在 runtime_input 里）
4. `cargo test -p hone-channels` 全部 213 个测试通过
5. 但 `2026-04-21T10:52:02.385272+08:00` 的最新 Feishu 真实会话仍再次落入 `role=user` 的 `【Compact Summary】...`，证明仓库曾记录的修复并没有让当前生产写库路径真正收口；本单状态因此重新回到 `Fixing`，README 也需要同步撤回“已修复”结论。

## 后续建议

1. 后续仍应优先补一条 scheduler 跨任务回归测试，直接锁住“前一轮 compact summary 不得串入后一轮独立任务答案”的正式 contract。
2. 如果真实流量里仍观测到摘要幻觉数字，可再补更强的 summary-output contract test，直接约束不得输出历史中未出现的证券价格、持仓数量和目标价。
3. 需要优先核对当前线上写库路径与 prompt 组装路径是否出现分叉：即便 runner restore 已跳过 compact summary，只要 `session_messages` 仍把摘要落成 `role=user`，后续导出、排障和任何依赖消息库的能力都会继续读到受污染 transcript。
