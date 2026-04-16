# Bug: Feishu 直聊消息在已有同 session 任务处理中时仍先发送 placeholder，但未真正进入 agent 主链路

- **发现时间**: 2026-04-16 13:40 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
  - 最近真实会话：
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-16 13:35:53`、`13:36:06`、`13:39:29` 连续三次只记录到 `step=reply.placeholder ... detail=sent`
    - 对应 message_id:
      - `om_x100b51331da2fcb0b372d4261515e4d`
      - `om_x100b51331af1c8a8b25f3dadee4a13a`
      - `om_x100b51332e157888b351106abb9185b`
    - `2026-04-16 13:54:47`、`13:56:32`、`13:58:20`、`13:58:30` 再次复现同样模式，只记录到 `step=reply.placeholder ... detail=sent`
    - 对应 message_id:
      - `om_x100b5133d4f02ca4b2169b0d10fe903`
      - `om_x100b5133ee7cc488b3d7932076ddbd1`
      - `om_x100b5133e96da4a4b219307b64cda0a`
      - `om_x100b5133e6f830acb220cccba1b5145`
    - 用户口径上，最新两条即“喂喂喂”和“1”；二者都未进入会话主链路，也未落入 `session_messages`
    - `2026-04-16 14:58:49` 再次收到一条 `text_chars=9` 的 Feishu 文本消息
    - `2026-04-16 14:58:50` 仍只记录到 `step=reply.placeholder ... detail=sent`
    - 到 `2026-04-16 15:01` 为止，`session_messages` 仍没有新增这条用户消息，`sessions.last_message_at` 也仍停留在 `2026-04-16T12:53:32.600190+08:00`
  - 最近运行日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - 在上述各条 placeholder 之后，没有出现同 message_id 的：
      - `session.persist_user`
      - `recv`
      - `agent.prepare`
      - `agent.run`
      - `failed`
    - 同时间窗内 Feishu 渠道进程仍在线，说明不是整个 listener 进程退出。
    - 最近一小时新增样本来自 `data/runtime/logs/web.log`
      - `2026-04-16 14:58:49.167` `step=message.accepted ... text_chars=9`
      - `2026-04-16 14:58:50.840` `step=reply.placeholder ... detail=sent`
      - `2026-04-16 14:59:42.224` `runtime_admin_override denied ... reason=not_whitelisted`
      - 同一条消息仍没有后续 `session.persist_user`、`recv`、`agent.prepare` 或 `failed`
  - 最近消息落库：`data/sessions.sqlite3`
    - `sessions.session_id='Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15'` 在 `2026-04-16T13:58:20.668278+08:00` 之后 `updated_at` 被刷新
    - 但 `last_message_at` 仍停留在 `2026-04-16T12:53:32.600190+08:00`
    - 说明新消息只触发了入口更新，没有成功持久化为用户消息
    - 到 `2026-04-16T14:58:50.841774+08:00`，同一 session 的 `updated_at` 再次被刷新，但 `last_message_at` 仍未前进，说明最新文本消息也停在入口层
  - 代码线索：
    - `bins/hone-feishu/src/handler.rs` 中 direct / group 共用同一条 placeholder 发送逻辑
    - `crates/hone-channels/src/agent_session.rs` 中 `AgentSession::run()` 会在写 `session.persist_user` 日志前先等待 per-session run lock
    - `bins/hone-feishu/src/handler.rs` 已将 Feishu 私聊纳入入口层 `SessionLockRegistry` busy 检查，并把 placeholder 发送移动到获得处理权之后
    - 但最新四条复现未命中 `direct.busy`，说明除入口 busy 缺口外，`send_placeholder_message()` 之后到 `session.run()` 真正启动前仍有未收口的异常路径
  - 修复结论回撤：
    - 2026-04-16 早些时候补的“私聊 busy 短路”只能覆盖同 session 活跃态可见的场景
    - `13:54` 之后的新证据表明该缺陷仍然活跃，原“Fixed”结论不成立，现回调为 `New`
  - 2026-04-16 当前修复进展：
    - `bins/hone-feishu/src/handler.rs` 已把“空解析内容”兜底前移到 placeholder 之前，避免无内容消息再出现 placeholder 假启动
    - placeholder 发送时机已继续后移到 `AgentSession` 对象准备完成之后，进一步缩小静默区间
    - 当前运行配置 `data/runtime/config_runtime.yaml` 已为 `+8613871396421` 补入 `feishu_mobiles`，并为其补入 `open_id=ou_39103ac18cf70a98afc6cfc7529120e5` 到管理员名单
    - 定向回归：`cargo test -p hone-feishu actionable_user_input_detects_empty_payload -- --nocapture`、`cargo test -p hone-feishu direct_busy_text_is_explicit -- --nocapture` 通过

## 端到端链路

1. 用户在 Feishu 私聊里连续发送多条消息或附件。
2. 新消息进入 Feishu handler 后，系统先发送“正在思考中...”或附件确认 placeholder。
3. 但如果同一 `session_id` 已有上一条消息仍在处理中，新消息会在更深层的 `AgentSession::run()` 入口等待 session run lock。
4. 当前已知至少存在两条独立异常路径：
   - 一条是此前已修补的“入口未命中 busy，直接进入深层 session lock 等待”；
   - 另一条是最新复现出来的“placeholder 已发送，但 `session.run()` 前后没有任何后续日志”，表现为链路在更前层就中断。

## 期望效果

- 如果同一 Feishu 私聊 session 已有消息在处理中，应在入口期直接返回明确 busy 提示，而不是先发送 placeholder。
- 只有真正拿到处理权的消息，才应发送 placeholder 并进入 `agent.run`。
- 日志应能清晰区分“真正开始处理”与“因 busy 被短路”。
- 即使 handler 在 placeholder 之后出现异常，也应有明确的失败日志与用户态兜底，而不是静默停在 placeholder。

## 当前实现效果

- 修复前，群聊已经有 busy / pretrigger 策略，但 Feishu 私聊没有同等级入口保护。
- 修复前，私聊用户连续发送消息时，系统会先给 placeholder，随后卡在更深层 session 锁等待，体感上像“处理失败”或“系统没反应”。
- 当前处于修复中。Feishu 私聊入口已有 `direct.busy` 短路，本轮又补了“空解析内容先兜底、后发 placeholder”的顺序修复，并把 placeholder 发送时机继续后移。
- 新版本 `hone-feishu` release 二进制已经重编并重启，管理员配置也已生效。
- 但最近一小时已经再次观察到新的同类真实文本消息：链路停在 `message.accepted -> reply.placeholder -> runtime_admin_override denied`，依然没有进入 `session.persist_user` 或 `agent.run`。
- 这说明问题并未随着 placeholder 后移和管理员配置修复一起收口，当前仍不能把状态提升为 `Fixed`。

## 用户影响

- 这是功能性缺陷。用户会误以为消息已经开始处理，但实际没有进入 agent 主链路。
- 之所以定级为 `P1`，是因为它直接影响 Feishu 私聊主链路的可用性与可解释性，且会持续误导用户反复重试。
- 之所以不是 `P0`，是因为当前证据仍集中在单渠道、单 session 并发场景，并非系统全局不可用。

## 根因判断

- 根因不在 Tavily、MiniMax 或 answer provider。最新 “喂喂喂” / “1” 两条甚至没有进入 `session.persist_user`，说明失败早于搜索或回答阶段。
- 当前更合理的判断是：此前确认的“私聊入口 busy 缺口”确实存在，但不是唯一根因。
- 最新证据显示，在 `send_placeholder_message()` 成功之后、`session.run()` 真正开始写库之前，仍存在未被日志覆盖的中断点或任务异常退出路径。
- `14:58` 的新样本进一步表明，这个中断点并不一定走到 session lock 或 runner 初始化，甚至可能在更前层被 `runtime_admin_override` 等入口逻辑拦住，但拦截发生时 placeholder 已经对用户可见。
- 因为最新复现没有记录 `direct.busy`，所以它不完全等同于“session run lock 等待”，应继续沿 handler 本地逻辑和异步任务边界排查。

## 下一步建议

- 先在 `bins/hone-feishu/src/handler.rs` 里为 placeholder 发送后、`session.run()` 调用前后补显式步骤日志和 panic/错误兜底，缩小静默区间。
- 把最新 `13:54`、`13:56`、`13:58` 以及 `14:58` 的真实会话证据作为同一 bug 的持续复现样本继续跟踪，不要再视为已修复。
- 优先核对 `runtime_admin_override`、管理员白名单判断以及 placeholder 发送顺序之间的关系，确认是否存在“权限拒绝发生在 placeholder 之后”的新前置短路。
- 若补日志后确认是独立于 busy 的第二个中断点，再拆成新 bug；在此之前继续归并到当前文档，避免重复建档。
