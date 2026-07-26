# Bug: Feishu 直聊取消关注 SIVE/SIVEF 后外发内部 persistent tool 错误

- 发现时间：2026-07-26 15:02 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-26 11:00-15:00 CST。
  - `session_messages` 新增 8 条 user、4 条 assistant、2 条 system compact，覆盖 2 个更新 session；最近 assistant 到 `2026-07-26T14:48:19.973648+08:00`。
  - session_id：`Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`
  - `2026-07-26T12:00:13.429561+08:00` 用户发送 `SIVE`、`SIVEF`，并要求“这三个都取消关注”。
  - `2026-07-26T12:00:29.178236+08:00` assistant 正常澄清只看到了两处 ticker，要求用户补充第三个标的。
  - `2026-07-26T12:01:44.090762+08:00` 用户明确补充：`SIVE\nSIVEF 取消这两只代码`。
  - `2026-07-26T12:02:25.803421+08:00` assistant final 只返回 `agent_owned_finance_persistent_tool_error`，`metadata_json` 标记 `error_kind=AgentFailed`、`run_failed=true`。
- `data/runtime/logs/web.log.2026-07-26`
  - `12:01:44` 同 session 接收用户补充消息后触发自动 compact。
  - `12:02:14-12:02:15` runner 执行了两次 `data_fetch search`，均显示工具执行成功。
  - `12:02:25` runtime 记录 `agent-owned finance blocked persistent tool before registry execution`。
  - `12:02:25` message flow 以 `error="agent_owned_finance_persistent_tool_error"` 失败收口，并走 `failure_fallback segments.sent=1`。
- 现有文档去重：
  - `feishu_direct_watchlist_cancel_misses_active_heartbeat.md` 覆盖的是取消 `CBRS` 时只执行 `portfolio unwatch`、没有关联并取消已存在 heartbeat job；本轮是 SIVE/SIVEF 取消关注请求在 agent-owned finance persistent tool 边界直接失败，且用户可见 final 外发内部错误 key。
  - `web_direct_image_attachment_not_readable_internal_debug_leak.md` 提到 Web direct 图片链路中的同名 `agent_owned_finance_persistent_tool_error`，但受影响渠道、输入类型和用户任务不同；本轮是 Feishu direct 纯文本取消关注链路。
  - 其它 raw error 文档多覆盖 `hone-mcp binary not found`、ACP 断连或 provider 错误，不覆盖本次 persistent tool block 的用户可见错误 key。

## 端到端链路

1. Feishu direct 用户要求取消 SIVE / SIVEF 两只代码的关注。
2. assistant 首轮要求补齐第三只标的；用户随后明确改为只取消 SIVE / SIVEF 两只。
3. 旧会话自动 compact 后进入 function-calling runner。
4. runner 先执行金融搜索工具，随后在 persistent tool 边界被阻断。
5. runtime 将本轮标记为 `AgentFailed`，但最终用户可见消息只有内部错误 key。
6. 用户没有收到取消成功、取消失败的可理解原因，也没有收到下一步确认。

## 期望效果

- 对“取消关注 ticker”这类状态变更请求，应调用真实关注 / watchlist / cron 任务管理工具完成操作，或返回用户态失败说明。
- 如果 persistent tool 因副作用不确定被阻断，最终回复应说明“本次取消关注没有完成，请稍后重试或联系管理员”，并保留内部错误分类到日志。
- 用户可见文本不得包含 `agent_owned_finance_persistent_tool_error` 这类内部错误 key。

## 当前实现效果

- 用户明确给出两只 ticker 后，本轮业务动作没有完成可见确认。
- assistant final 直接外发内部错误 key，既不可操作，也暴露实现层错误分类。
- 同窗 14:47 同一 Feishu direct session 对 ORCL 问答可正常工具调用并收口，说明不是 Feishu direct 全渠道不可用；问题集中在取消关注 / persistent tool 副作用收口链路。

## 用户影响

- 这是功能性 bug：用户明确要求取消关注两只代码，但系统没有给出成功确认、失败原因或安全重试路径。
- 同时存在质量 / 安全边界问题：内部错误 key 进入用户可见 final。
- 当前证据覆盖单个 Feishu direct session、单次取消关注请求；同窗其它 direct 能正常收口，未见跨用户错投、数据破坏或全渠道不可用，因此定级为 `P2` 而不是 `P1`。

## 根因判断

- 初步判断 agent-owned finance 对可能产生持久副作用的工具调用做了 replay / registry 前阻断，但错误没有被用户态净化层映射为安全文案。
- 自动 compact 后保留的上下文和工具选择可能让取消关注请求误入 `data_fetch search` 研究链路，随后 persistent tool guard 无法确认副作用状态，最终落成 raw error。
- 这与 CBRS 取消关注未关联 heartbeat 的根因不同：本轮尚未走到正确的取消工具或 cron/portfolio 状态源消费阶段，失败发生在 runner / persistent tool 安全边界与 final fallback。

## 下一步建议

- 为 `agent_owned_finance_persistent_tool_error` 增加共享用户可见错误净化，禁止裸 key 进入 final。
- 对取消关注 / unwatch / 删除监控等副作用请求增加专用恢复路径：如果 persistent tool 被阻断，应明确告知未完成，不要继续金融搜索或生成投研回答。
- 增加 Feishu direct 回归：用户在旧会话 compact 后发送 `SIVE\nSIVEF 取消这两只代码`，最终回复不得包含内部错误 key，且必须给出成功确认或安全失败说明。
