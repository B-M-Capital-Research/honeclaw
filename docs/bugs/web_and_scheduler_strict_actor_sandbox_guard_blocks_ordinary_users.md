# Bug: Web direct 与 scheduler 被 strict actor sandbox guard 批量拦截

## 发现时间

2026-07-15 15:02 CST

## Bug Type

System Error

## 严重等级

P2

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/runtime/logs/web.log.2026-07-15`
  - 巡检时间窗：2026-07-15 11:01-15:02 CST。
  - 13:00、13:30、14:00 CST 三个 scheduler 批次均出现 52 条 `安全执行器不可用：普通用户不能使用具备宿主机访问能力的 CLI/ACP；请切换到 hone_cloud，或由管理员运行。`，合计 156 条；另有 13:14 / 13:20 / 13:24 / 13:41 CST Web direct 同类失败。
  - 受影响 scheduler 覆盖 Web 与 Feishu heartbeat，包括 `AI与科技持仓观察关键事件心跳提醒`、`NVDA 关键事件心跳提醒`、`闪迪关键事件心跳提醒`、`持仓财报与重大新闻心跳提醒`、`AAOI 1.6T 光模块心跳检测`、`RKLB异动监控`、`Monitor_Watchlist_11`、`全天原油价格3小时播报` 等。
  - 13:14-13:24 CST Web direct session `Actor_web__direct__web-user-be13e1f84d14` 连续 4 次收到同一 IBM 当日异动 / 抄底问题，均在 `agent.prepare` 后触发 `strict actor sandbox guard`，随后 `session.persist_assistant detail=failed`，没有生成业务回答。
  - 13:41 CST 同一 Web direct 用户追问“怎么切换到 hone_cloud”，仍被同一 guard 拒绝，说明用户无法通过当前会话获得自助恢复说明。
  - 14:30 CST 后 heartbeat 又出现多条 `run_finish success=true` 与 `deliver_preview`，14:55 CST Feishu direct 有 `reply.send segments.sent=3/3`，说明这不是全渠道持续停摆；但本窗直接请求和多批 scheduler 已被阻断。
- `data/sessions.sqlite3`
  - 本轮 `session_messages.timestamp` 最大值仍停在 `2026-07-15T10:55:26.462700+08:00`，`sessions.last_message_at` 最大值同样停在 10:55 CST；15:00 CST 只有旧数据重新导入。
  - 本地 `cron_job_runs` 在 11:01-15:02 CST 仍无新增行，`max(executed_at)` 继续停在 `2026-07-10T14:01:27.621121+08:00`。因此本轮以 runtime log 作为真实运行态证据。
- `docs/bugs/`
  - 未找到已登记的 `安全执行器不可用`、`strict actor sandbox guard`、`hone_cloud` 或普通用户 CLI/ACP guard 同根缺陷。
  - 该问题不同于既有 MiniMax transport、实时核验门禁、ACP transport timeout 或 quota 收口缺陷：失败发生在 runner 创建前的 actor sandbox guard，且错误明确指向普通用户 runner/profile 配置不匹配。

## 端到端链路

1. 普通 Web / Feishu 用户或其 scheduler 到点触发 direct / heartbeat 任务。
2. `AgentSession` 进入 `agent.prepare`，尝试为普通用户创建 runner。
3. strict actor sandbox guard 判断当前 runner 具备宿主机访问能力，要求切换 `hone_cloud` 或管理员运行。
4. direct 请求写入失败 assistant 状态，scheduler 记录 `runner_error` 并跳过发送。
5. 用户收不到 IBM 问答、切换说明或本应完成的 heartbeat 覆盖。

## 期望效果

- 普通用户应默认路由到允许的云端 / 受控 runner profile；如果配置错误，应在调度前被发现并降级到可用 runner。
- 当 guard 拒绝执行时，direct 用户至少应收到产品化说明或可操作恢复路径，而不是同一 guard 阻断“如何切换”的追问。
- scheduler 应避免在多个批次重复用不可用 runner 执行，必要时将任务标为配置错误并进入可观测重试 / 告警路径。

## 当前实现效果

- 13:00、13:30、14:00 CST 多批 Web / Feishu heartbeat 被同一 guard 批量拒绝，用户收不到正常 noop / triggered 结果。
- Web direct 用户连续重试同一个强时效金融问题仍无法进入回答链路，追问如何切换也失败。
- 14:30 后部分 scheduler 恢复成功，说明故障可能与 runner profile / actor config 的动态状态有关，但没有文档或台账记录解释这段不可用窗口。

## 用户影响

- 这是功能性缺陷：普通用户 direct 问答和 scheduler 监控任务会在 runner 创建前失败，导致当前请求无法完成。
- 定级为 P2：本窗有多用户、多任务失败，但 14:30 后同一 runtime 出现 scheduler 成功样本，14:55 Feishu direct 也有成功发送；暂未证明所有渠道持续不可用、错投、数据破坏或敏感信息泄露，因此不定为 P1。

## 根因判断

- 初步判断是普通用户 actor 被分配到具备宿主机访问能力的 CLI/ACP runner，触发严格 sandbox guard；预期应使用 `hone_cloud` 或等价受控 runner。
- direct 和 scheduler 共享同一 runner/profile 解析入口，导致配置漂移时同时影响实时问答和定时任务。
- 本地 `sessions.sqlite3` 与 `cron_job_runs` 未同步本窗真实状态，进一步降低了该类 runner 配置错误的审计可见性；台账停滞本身仍归入既有 `sessions_sqlite_mirror_stalled_after_successful_direct_replies.md`。

## 下一步建议

- 检查普通用户 actor 的 runner profile 解析优先级，确保 Web / Feishu 普通用户默认进入 `hone_cloud` 或受控 function-calling profile，而不是宿主机 CLI/ACP。
- 为 strict actor sandbox guard 增加产品化失败收口：direct 返回可操作说明，scheduler 记录配置错误并避免同一批次重复重试。
- 增加回归：普通 Web actor 与 Feishu scheduler actor 在非管理员身份下创建 runner 时不会命中 host CLI/ACP guard。
- 运行态修复后复核 30 分钟窗口：不再出现 `安全执行器不可用`，且 Web direct 能回答同类问题或给出受控说明。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`data/runtime/logs/web.log.2026-07-15` 11:01-15:02 CST、`data/sessions.sqlite3` 会话/调度台账时间戳、`docs/bugs/*.md` 去重搜索。
