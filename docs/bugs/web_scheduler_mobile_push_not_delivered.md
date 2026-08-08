# Bug: Web scheduler claims mobile notification delivery but only writes web session events

- 发现时间：2026-05-15 15:04 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：New
- 修复情况：2026-08-04 06:02 运行态继续复发：event-engine Web sink 在 `web push broadcast failed: channel closed` 后退到 `[dryrun sink]`，但 dispatch 仍记 `status=sent`；这会继续把未送达的 Web push / SSE 当作已送达。2026-05-16 的手机系统通知能力边界修复仍成立，但投递结果台账语义再次不可信。
- GitHub issue：无；当前不是 P1，未创建 issue。

## 最新进展

- `2026-08-08 10:01 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-08 06:01-10:01 CST`。
    - 08:31 CST event-engine digest 出站连续记录 14 条 `web push broadcast failed: channel closed channel=web user=...`，随后进入 31 条 `[dryrun sink]` fallback，覆盖 web / feishu / legacy / discord 等 actor sink preview。
  - `data/sessions.sqlite3`
    - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`，06:01 CST 后 `web_push_messages` 增量为 0。
  - 判断：这是既有 Web push channel closed 后 fallback / 台账语义不可靠问题的持续复发。它影响 Web push 投递可观测性和真实触达，但 source runtime 仍运行、其它 sink 有 fallback，暂不升 P1。

- `2026-08-07 22:01 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-07 18:01-22:01 CST`。
    - 近窗有 1 条 `web push broadcast failed: channel closed`，随后出现 2 条 `[dryrun sink]` fallback；代表样本为 20:30 CST event-engine digest sink 对 `web-user-879a3b18fce2` channel closed 后退到 dryrun sink。
  - `data/sessions.sqlite3`
    - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`，18:01 CST 后 `web_push_messages` 增量为 0。
  - 判断：本轮规模较小，但仍证明 Web push channel closed 后会退到 dryrun sink，且本地 Web push 台账不推进；维持功能性 `P2 / New`，非 P1。

- `2026-08-06 22:01 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-06 18:01-22:01 CST`。
    - 近窗有 1 条 `web push broadcast failed: channel closed`，随后出现 1 条 `[dryrun sink]` fallback；代表样本为 20:31 CST event-engine digest sink 对 `web-user-be13e1f84d14` channel closed 后退到 dryrun sink。
  - `data/sessions.sqlite3`
    - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`，18:01 CST 后 `web_push_messages` 增量为 0。
  - 判断：本轮规模较小，但仍证明 Web push channel closed 后会退到 dryrun sink，且本地 Web push 台账不推进；维持功能性 `P2 / New`，非 P1。

- `2026-08-06 10:01 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-06 06:01-10:01 CST`。
    - 近窗有 1 条 `web push broadcast failed: channel closed`，随后出现 9 条 `[dryrun sink]` fallback。
  - `data/sessions.sqlite3`
    - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`，06:01 CST 后 `web_push_messages` 增量为 0。
  - 判断：本轮规模较小，但仍证明 Web push channel closed 后会退到 dryrun sink，且本地 Web push 台账不推进；维持功能性 `P2 / New`，非 P1。

- `2026-08-05 22:02 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-05 18:03-22:02 CST`。
    - 近窗有 3 条 `web push broadcast failed: channel closed`，随后出现 5 条 `[dryrun sink]` fallback；代表样本为 20:31 CST event-engine digest sink 对 `web-user-879a3b18fce2` channel closed 后退到 dryrun sink。
  - `data/sessions.sqlite3`
    - `web_push_messages.max(created_at)=2026-07-19T13:30:44.965959+08:00`，18:03 CST 后 `web_push_messages` 增量为 0。
  - 判断：本轮规模小于 08:30 后批量复发，但仍证明 Web push channel closed 后退到 dryrun sink 且本地 Web push 台账不推进；维持功能性 `P2 / New`，非 P1。

- `2026-08-05 10:05 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-05 06:00-10:05 CST`。
    - 近窗统计到 22 条 `web push broadcast failed: channel closed`，以及 45 条 `[dryrun sink]` fallback。
    - 代表样本：08:30 CST 后多个 Web actor 先记录 `channel digest sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview；`data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 留在无法审计送达的状态，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 event-engine 和 scheduler 仍有运行 / 送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

- `2026-08-05 02:01 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-04 22:01-2026-08-05 02:01 CST`。
    - 近窗统计到 32 条 `web push broadcast failed: channel closed`，以及 38 条 `[dryrun sink]` fallback。
    - 代表样本：22:03 CST 后多个 Web actor 先记录 `channel sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview；`data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 当成可继续记录的投递结果，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 event-engine 和 scheduler 仍有运行 / 送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

- `2026-08-04 22:02 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-04 18:00-22:02 CST`。
    - 近窗统计到 49 条 `web push broadcast failed: channel closed`，以及 51 条 `[dryrun sink]` fallback。
    - 代表样本：18:00 CST 后多个 Web actor 先记录 `channel sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview；dispatch 层仍把这些事件计为 `sent`，近窗 `status=sent` 命中 66 次。
    - `data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 记为 `sent`，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 event-engine 和 scheduler 仍有运行 / 送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

- `2026-08-04 18:01 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-04 14:00-18:01 CST`。
    - 近窗统计到 5 条 `web push broadcast failed: channel closed`，以及 5 条 `[dryrun sink]` fallback。
    - 代表样本：16:06 / 17:36 CST 多个 Web actor 先记录 `channel sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview；dispatch 层仍把这些事件计为 `sent`。
    - `data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 记为 `sent`，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 event-engine 和 scheduler 仍有运行 / 送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

- `2026-08-04 10:01 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-04 06:02-10:01 CST`。
    - 近窗统计到 35 条 `web push broadcast failed: channel closed`，以及 38 条 `[dryrun sink]` fallback。
    - 代表样本：06:02 CST 后多个 Web actor 先记录 `channel sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview；dispatch 层仍把这些事件计为 `sent`。
    - `data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 记为 `sent`，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 event-engine 和 scheduler 仍有运行 / 送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

- `2026-08-07 10:02 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-07 06:02-10:02 CST`。
    - 08:30-08:31 CST 多个 Web actor 先记录 `web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 `盘前摘要` body preview；同窗也有多条 Feishu / Discord / legacy dryrun sink 样本。
    - `data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 记为送达候选，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 event-engine 和 scheduler 仍有运行 / 送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

- `2026-08-04 06:02 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-04 02:00-06:02 CST`。
    - 近窗统计到 26 条 `web push broadcast failed: channel closed`，以及 30 条 `[dryrun sink]` fallback。
    - 代表样本：02:00 CST 后多个 Web actor 先记录 `channel sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview；dispatch 层仍把这些事件计为 `sent`。
    - `data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 记为 `sent`，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 event-engine 和 scheduler 仍有运行 / 送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

- `2026-08-04 02:02 CST` 运行态继续复发，状态维持 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-03 22:01-2026-08-04 02:01 CST`。
    - 近窗统计到 151 条 `web push broadcast failed: channel closed`，以及 170 条 `[dryrun sink]` fallback。
    - 代表样本：22:01 / 22:31 / 23:01 / 23:31 / 02:00 CST 多个 Web actor 先记录 `channel sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview，紧接着 `hone_event_engine::router::dispatch` 仍记录同一事件 `status=sent`。
    - `data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 记为 `sent`，会误导用户、调度审计和后续补发判断，因此维持功能性 `P2/New`。
    - 同窗 Feishu / event-engine 仍有成功送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏；另有 9 条 Discord DM `HTTP 401` 后 dryrun fallback 仍记 sent，暂作为同类投递台账语义观察，不新建独立缺陷，非 P1，不创建 GitHub Issue。

- `2026-08-03 22:02 CST` 运行态回退为 `New/P2`：
  - `data/logs/hone-console-page-source.log`
    - 巡检窗口：`2026-08-03 18:02-22:02 CST`。
    - 近窗统计到 63 条 `web push broadcast failed: channel closed`。
    - 代表样本：18:03 / 18:32 / 19:03 / 19:33 / 21:00 / 21:30 CST 多个 Web actor 先记录 `channel sink failed, falling back to log: web push broadcast failed: channel closed`，随后 `[dryrun sink]` 写出 body preview，紧接着 `hone_event_engine::router::dispatch` 仍记录同一事件 `status=sent`。
    - `data/sessions.sqlite3` 同窗 `web_push_messages` 增量为 0，`web_push_messages.max(created_at)` 仍停在 `2026-07-19T13:30:44.965959+08:00`。
  - 判断：
    - 这不是 2026-05 原始问题中的“assistant 承诺手机系统通知”复发；旧能力边界提示仍可视为已修。
    - 但同一 Web 投递链路仍把实际 channel closed / dryrun fallback 记为 `sent`，会误导用户、调度审计和后续补发判断，因此回退为功能性 `P2/New`。
    - 同窗 Feishu / Discord / event-engine 仍有成功送达样本，未见全渠道不可用、错对象投递、敏感泄露或数据破坏，非 P1，不创建 GitHub Issue。

## 证据来源

- `data/sessions.sqlite3`
  - `session_id=Actor_web__direct__web-user-ba50cb9401c0`
  - `2026-05-15T12:23:03+08:00` 起，用户追问 Web 定时任务如何在手机收到提醒；assistant 引导用户打开手机上的 Hone 网页/App、允许通知，并说明可用一次性任务测试手机提醒。
  - `2026-05-15T12:32:55+08:00` 用户要求 3 分钟后发测试通知；`2026-05-15T12:40:39+08:00` 用户反馈“没收到”。
  - `2026-05-15T12:45:33+08:00` 用户要求重新 1 分钟后发一条；`2026-05-15T12:48:44+08:00` 用户再次反馈“你还是没发”。
  - `2026-05-15T12:49:09+08:00` assistant 最终承认：任务创建并触发了，但没有变成手机系统通知；当前 web 通道不等于手机系统级 push。
- `cron_job_runs`
  - `run_id=21817`，`job_name=12:35 测试通知`，`executed_at=2026-05-15T12:35:11+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`，`detail_json.console_event_sent=false`，`delivery_channel=web`。
  - `run_id=21818`，`job_name=12:47 二次测试通知`，`executed_at=2026-05-15T12:47:13+08:00`，`execution_status=completed`，`message_send_status=sent`，`delivered=1`，`detail_json.console_event_sent=false`，`delivery_channel=web`。
- 代码确认
  - `packages/app/src/context/sessions.tsx` 只监听 SSE `scheduled_message` / `push_message` 并 append 到当前会话。
  - `rg` 未发现 Web 端对 `Notification.requestPermission`、`PushManager` 或真正 Web Push 订阅的实现；现有 service worker 只用于 asset recovery。
  - `crates/hone-web-api/src/routes/events.rs` 的 Web scheduler 记录 `console_event_sent`，但 `web_scheduler_delivery_status(false)` 仍会把会话落库视为 `sent + delivered=1`。

## 端到端链路

Web 用户创建持仓新闻晚报 -> 用户询问如何在手机收到提醒 -> assistant 将 Web 会话通知解释为可通过手机网页/App 通知权限接收 -> 用户请求一次性测试通知 -> scheduler 到点触发并把 assistant final 写回 Web 会话 -> Web 投递层只尝试 SSE `scheduled_message`，且本轮 `console_event_sent=false` -> `cron_job_runs` 仍记为 `completed + sent + delivered=1` -> 用户手机系统通知中心没有收到提醒。

## 期望效果

- 如果 Web scheduler 只支持会话内消息，应明确告诉用户：当前不能保证手机系统通知，也不能把它当作可靠手机提醒。
- 如果产品要支持手机提醒，应有真正的 Web Push / App Push / 邮件 / 短信等可审计投递目标，并把送达结果与会话落库区分开。
- assistant 在创建或测试 Web 定时任务时，应基于可用 channel capability 给出准确承诺；无法系统级 push 时，不应引导用户反复排查手机通知权限。

## 当前实现效果

- 任务执行链路成功，测试通知正文也写入 Web 会话。
- 两条测试通知均 `console_event_sent=false`，说明实时控制台事件没有送到活跃 SSE 监听者。
- 后端仍把这类 Web scheduler 记录为 `sent + delivered=1`，这符合旧修复里的“会话落库即送达”定义，但不能代表手机系统通知已送达。
- assistant 前几轮把“当前 Hone 网页/会话通知通道”与手机系统级通知排查混在一起，直到用户两次反馈没收到后才明确承认当前没有真正打到手机通知中心。

## 用户影响

- 用户创建的 20:00 持仓新闻晚报可能只落在 Web 会话里，无法作为手机提醒使用。
- 台账显示 `delivered=1`，运维或后续 agent 容易误判为通知已送达，实际用户侧没有收到系统提醒。
- 用户被引导做手机权限排查和重复测试，增加信任损耗；这不是单次表达偏好问题，而是能力边界和送达语义不一致。

## 根因判断

- Web scheduler 的当前送达语义只覆盖“写入会话 / 尝试 SSE 事件”，没有覆盖手机系统级 push。
- 前端没有可见的 Web Push 订阅与浏览器通知授权链路；后端也没有独立的 push 订阅表或外部通知投递结果。
- assistant 缺少 channel capability 约束：当 `delivery_channel=web` 且没有 Web Push 能力时，仍按“手机通知权限”给出操作建议，导致用户预期与系统实际能力错位。

## 修复进展（2026-05-16 00:06 CST）

- 已在 Web 渠道且允许 cron 的对话提示中注入 `【Web 定时任务送达边界】`：
  - 当前 Web 定时任务结果只保证写入当前 Hone 会话；
  - 网页在线且 SSE 连接存在时会实时追加到页面；
  - 当前没有 Web Push / 手机系统通知能力，不允许承诺会出现在手机通知中心，也不再引导用户排查手机通知权限。
- 已在 Web scheduler 执行台账 detail 中补充：
  - `system_push_supported=false`
  - `system_push_sent=false`
  - 继续保留 `console_event_sent`，用于区分页面实时 SSE 是否送达。
- 保留既有“会话落库即 Web delivered”的语义，不把离线页面视为 send_failed；但台账不再让后续排障误读为手机系统 push 已送达。
- 新增回归：
  - `resolve_prompt_input_warns_web_cron_cannot_send_mobile_system_push`
  - `web_scheduler_detail_distinguishes_session_delivery_from_system_push`
- 验证：
  - `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/prompt.rs crates/hone-channels/src/turn_builder.rs crates/hone-channels/src/agent_session/tests.rs crates/hone-web-api/src/routes/events.rs`
  - `cargo test -p hone-channels resolve_prompt_input_warns_web_cron_cannot_send_mobile_system_push -- --nocapture`
  - `cargo test -p hone-web-api web_scheduler_ -- --nocapture`
  - `cargo check -p hone-web-api --tests`
- 修复提交：`fbba5342`
- 状态更新为 `Fixed`。若后续产品要求真正手机提醒，应另开功能/缺陷补 Web Push/App Push 订阅、授权状态检查和系统级投递台账。

## 下一步建议

1. 若保持仅会话内消息：后续 UI 可继续补显式说明，但当前 prompt 与台账已先阻断错误承诺。
2. 若要支持手机提醒：新增 Web Push/App Push capability、订阅状态检查与投递台账字段，区分 `session_persisted`、`sse_event_sent`、`system_push_sent`、`system_push_failed`。
