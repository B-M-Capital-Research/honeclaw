# Bug: Telegram update listener 持续不可用，近一个月无新会话落库

- **发现时间**: 2026-04-21 10:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - 2026-04-26 22:20-22:31 最新运行日志：
    - `data/runtime/logs/desktop.log`
      - `2026-04-26 22:20:48.104` bundled runtime 再次启动 managed channels。
      - `2026-04-26 22:20:48.990` 继续记录 `managed channel telegram skipped because it exited during startup`。
      - `2026-04-26 22:31:01.520` 又重启一轮 bundled runtime。
      - `2026-04-26 22:31:02.423` 再次出现 `managed channel telegram skipped because it exited during startup`。
    - `data/sessions.sqlite3`
      - 最近 Telegram 会话仍停留在 `Actor_telegram__direct__8039067465`，`updated_at=2026-03-18T11:06:59.182313+08:00`
      - 最近一小时仍无任何 `actor_channel='telegram'` 新会话或消息落库
    - 结论：到 `2026-04-26 22:31` 为止，Telegram 渠道在 21:03 的“旧进程占锁/启动即退出”之后仍未恢复；bundled runtime 连续两次重试都没把 listener 拉回可用状态
  - 2026-04-26 21:03 最新运行日志：
    - `data/runtime/logs/desktop.log`
      - `2026-04-26 21:03:29.627` bundled runtime 再次启动 managed channels。
      - `2026-04-26 21:03:30.951` 继续记录 `managed channel telegram skipped because it exited during startup`。
    - `data/runtime/logs/hone-telegram.release-restart.log`
      - `2026-04-26T13:03:31.568213Z` 直接报 `检测到旧的 Telegram Bot 进程仍占用启动锁，hone-telegram 不会启动。请先清理之前的进程后再重试。`
    - `data/sessions.sqlite3`
      - 最近 Telegram 会话仍停留在 `Actor_telegram__direct__8039067465`，`updated_at=2026-03-18T11:06:59.182313+08:00`
      - 最近一小时仍无任何 `channel='telegram'` 新消息落库
  - 2026-04-26 19:01-19:03 最新运行日志：
    - `data/runtime/logs/desktop.log`
      - `2026-04-26 19:01:59.349` bundled runtime 再次启动 managed channels。
      - `2026-04-26 19:02:00.722` 记录 `managed channel telegram skipped because it exited during startup`。
      - `2026-04-26 19:03:14.733` 随后又重启一轮 bundled runtime；`2026-04-26 19:03:16.088` 再次出现同样的 `managed channel telegram skipped because it exited during startup`。
    - `data/runtime/logs/desktop_release_screen_current.log`
      - `2026-04-26T11:02:01.569991Z` 与 `2026-04-26T11:03:17.058629Z` 连续两次报 `无法获取 Telegram Bot 信息: A Telegram's error: Invalid bot token`
      - 两次报错后都立即跟着 `sidecar terminated code=Some(1)`，说明 Telegram 不是进入 `GetUpdates` 后再退避，而是启动校验阶段直接退出
    - `data/sessions.sqlite3`
      - 最近一小时仍无任何 `channel='telegram'` 新消息落库，最近 Telegram 会话依旧停在 `2026-03-18`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-22 11:14:41.977` Telegram Bot 再次启动。
    - `2026-04-22 11:14:43.273` 随即报 `无法获取 Telegram Bot 信息: A Telegram's error: Invalid bot token`。
    - `2026-04-22 11:32:20.693` 又出现一次 Telegram Bot 启动记录，但截至本轮巡检未看到新的 Telegram 会话落库。
    - `data/sessions.sqlite3` 中最近 Telegram 会话仍停留在 `Actor_telegram__direct__8039067465`，`updated_at=2026-03-18T11:06:59.182313+08:00`。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-22 03:10:21`、`03:11:35`、`03:12:28` release app 启动 Telegram 渠道时连续报 `无法获取 Telegram Bot 信息: A Telegram's error: Invalid bot token`。
    - 同一窗口 `data/runtime/logs/desktop.log` 在 `03:10:20`、`03:11:34`、`03:12:27` 记录 `managed channel telegram skipped because it exited during startup`，说明当前不是单纯 `GetUpdates` 网络抖动，而是 Telegram sidecar 在启动阶段就因 bot token 无效退出。
    - 这与此前 `GetUpdates` 连接超时同属 Telegram 入站不可用链路，因此更新原缺陷，不新建重复单；根因判断从“网络可达性/代理为主”扩展为“凭据有效性或运行配置错误也会导致当前生产 Telegram 完全不可用”。
    - `2026-04-21 18:36:09` 最新样本仍为 `GetUpdates` 网络失败：`Telegram update listener error ... GetUpdates): operation timed out`
    - `2026-04-21 18:36:15` 下一次重试继续失败：`error trying to connect: operation timed out`
    - 同步日志显示退避从 `1s` 切到 `2s` 后继续重试，仍未看到 Telegram 入站链路恢复。
    - `2026-04-21 14:31:46`、`14:33:12`、`14:51:11`、`14:51:17`、`14:51:24`、`14:51:33`、`14:51:47`、`14:52:08`、`14:52:45`、`14:53:54`、`14:55:03`、`14:56:12`、`14:57:21`、`14:58:30` 最新窗口继续反复报 `Telegram update listener error ... GetUpdates ... operation timed out`
    - 退避从 `1s` 逐步升到 `64s` 后仍未恢复，说明截至 15:00 CST 前 Telegram 入站轮询仍不可用。
    - `2026-04-21 13:49:04.964` 最新样本仍为 `GetUpdates` 网络失败：`Telegram update listener error: A network error: error sending request for url (https://api.telegram.org/token:redacted/GetUpdates): error trying to connect: operation timed out`
    - 这说明 `10:01-10:17` 的 `Connection refused` 不是单次瞬时异常；到 `13:49` 最新错误形态变成连接超时，但端到端结果仍是 Telegram listener 无法稳定拉取更新。
    - `2026-04-21 10:01:02.189` 开始，Telegram listener 连续报错：`Telegram update listener error: A network error: error sending request for url (https://api.telegram.org/token:redacted/GetUpdates): error trying to connect: tcp connect error: Connection refused (os error 61)`
    - 同类错误在 `10:02:06`、`10:03:10`、`10:04:14`、`10:05:18`、`10:06:22`、`10:07:26`、`10:08:30`、`10:09:34`、`10:10:37`、`10:11:41`、`10:12:45`、`10:13:49`、`10:14:53`、`10:15:57`、`10:17:01` 持续重复
    - 每轮报错前都有 `retrying getting updates in 64s`，说明 listener 并未正常恢复，只是在固定退避后重试并再次失败
  - `data/sessions.sqlite3` -> `sessions`
    - 最近 Telegram 会话停留在 `Actor_telegram__direct__8039067465`，`updated_at=2026-03-18T11:06:59.182313+08:00`
    - 次新的 Telegram 会话为 `Actor_telegram__direct__7890339825`，`updated_at=2026-03-17T16:33:02.550896+08:00`
  - `data/sessions.sqlite3` -> `session_messages`
    - 最近 24 小时窗口内没有任何 `channel='telegram'` 的新消息落库，和最近一小时 listener 持续拉取失败相互印证

## 端到端链路

1. Telegram 渠道进程启动后需要先完成 bot token 校验，再进入 `getUpdates` 长轮询。
2. 最新 release app 窗口里，进程甚至会在启动锁阶段就被旧进程拦住；更早窗口则在 `bot.get_me()` 阶段因 `Invalid bot token` 退出，desktop 统一只记录 managed channel skipped；再早窗口即使进入 listener，也会在 `GetUpdates` 阶段持续连接超时或拒绝。
3. 当前实现没有把“旧进程占锁导致无法拉起”、“凭据无效导致 sidecar 退出”和“长轮询网络失败”恢复到可用入站状态。
4. 因为 Telegram 渠道无法稳定进入可监听状态，新的 Telegram 用户消息无法进入正常处理链路，也不会落到 `sessions` / `session_messages`。

## 期望效果

- Telegram listener 应先用有效 bot token 完成启动校验，再稳定完成 `getUpdates` 轮询，而不是在启动阶段退出或在传输层持续 `Connection refused` / timeout。
- 即使上游网络短时异常，也应具备明确的可恢复策略或更清晰的链路告警，而不是长时间反复重试无新消息。
- Telegram 渠道至少应能恢复到“用户发消息后能够创建/更新会话并落库”的基本功能状态。

## 当前实现效果

- 2026-04-26 22:31 CST 最新窗口显示，Telegram 渠道在 `21:03` 被记录为“旧进程仍占启动锁”之后并没有恢复；`desktop.log` 在 `22:20:48.990` 和 `22:31:02.423` 连续两次再次记录 `managed channel telegram skipped because it exited during startup`，而 `sessions` 中最近 Telegram 会话仍停在 `2026-03-18`。
- 2026-04-26 21:03 CST 最新窗口显示，Telegram 已经不只是 `Invalid bot token` 或 `GetUpdates` 网络失败；bundled runtime 再次尝试拉起 sidecar 时，`hone-telegram.release-restart.log` 直接报“旧的 Telegram Bot 进程仍占用启动锁”，desktop 则继续把渠道记成 `managed channel telegram skipped because it exited during startup`。当前入站链路仍完全没有恢复，最近 Telegram 会话依旧停在 2026-03-18。
- 2026-04-22 14:03 CST 最新样本仍在 `data/runtime/logs/web.log:2167` 报 `Telegram update listener error ... GetUpdates): connection closed before message completed`；本轮只读巡检没有调用 Telegram API。
- 2026-04-23 04:03 CST 与 06:03 CST 最新窗口继续复现 `GetUpdates` 连接中断：`data/runtime/logs/web.log.2026-04-22:1196` 和 `:1326` 均记录 `Telegram update listener error ... GetUpdates): connection closed before message completed`；本轮只读巡检没有调用 Telegram API。对应错误处理入口仍是 `bins/hone-telegram/src/handler.rs:220-230`，除 `TerminatedByOtherGetUpdates` 外只记录 error 并依赖 listener 后续重试，没有持久健康状态或用户侧告警。
- 2026-04-26 19:01-19:03 CST 最新窗口显示问题仍未止血：bundled runtime 两次尝试拉起 Telegram sidecar，都在 `bot.get_me()` 阶段直接报 `Invalid bot token` 并退出；`desktop.log` 同步把渠道标成 `managed channel telegram skipped because it exited during startup`，最近 Telegram 会话仍无新增落库。
- 2026-04-23 10:53-11:03 CST 最新窗口再次出现连续 `GetUpdates` 网络失败，其中 `10:53:55` 到 `11:03:29` 基本按退避节奏重复 `Connection refused (os error 61)`；12:54 CST 又出现 `operation timed out`，14:03 CST 又回到 `connection closed before message completed`。本轮 event-engine 巡检没有调用 Telegram API。
- 2026-04-22 12:03 CST 同一窗口还出现 `data/runtime/logs/web.log:2125` 的 `GetUpdates` 连接中断；`telegram.pid=75490` 和 heartbeat 均存活，说明这是监听请求层面的持续错误，而不是 Telegram sidecar 进程已退出。
- 到 `2026-04-22 11:14` 最新 release app 窗口，Telegram Bot 再次启动后仍立即报 `Invalid bot token`；`11:32` 又出现启动记录，但没有新的 Telegram 会话落库。
- 2026-04-22 07:22 CST 出现两条用户入站消息后，runner 在 `data/runtime/logs/web.log:1906-1917` 因 `hone-mcp binary not found near current executable` 失败并只发送 placeholder；这属于 Telegram 对话执行链路的新近失败证据，和 GetUpdates 拉取错误不同，但同样影响 Telegram 用户端可用性。
- 2026-04-22 10:03 CST 最新样本仍在 `data/runtime/logs/web.log` 报 `Telegram update listener error ... GetUpdates): connection closed before message completed`；本轮只读巡检没有调用 Telegram API。
- 2026-04-22 06:03 最新样本仍在 `data/runtime/logs/web.log` 报 `Telegram update listener error ... GetUpdates): connection closed before message completed`；同类错误还在 `2026-04-22 00:03`、`02:03`、`04:03` 出现。
- 到 `2026-04-22 03:10-03:12` release app 窗口，Telegram sidecar 已从此前 `GetUpdates` 超时演变为启动即失败：`Invalid bot token`，desktop 标记 `managed channel telegram skipped because it exited during startup`。
- 本轮 event-engine 巡检没有调用 Telegram API；上述结论仅来自本地运行日志。与此同时，`data/events.sqlite3` 在 `2026-04-21 21:19:33` UTC 记录过 event-engine high `sink/sent`，说明 `sendMessage` 路径至少曾成功，当前错误集中在 update listener 的 `GetUpdates` 入站长轮询。
- 到 `2026-04-21 18:36` 窗口，Telegram listener 仍在 `GetUpdates` 阶段超时，且下一次重试继续连接超时；没有看到 Telegram 新消息恢复落库。
- 到 `2026-04-21 14:31-14:58` 最新窗口，Telegram listener 仍持续 `GetUpdates` 超时并固定退避重试，没有看到入站链路恢复或新 Telegram 会话落库。
- 到 `2026-04-21 13:49` 的最新日志，Telegram listener 仍在 `GetUpdates` 网络阶段失败；错误从 `Connection refused` 变为 `operation timed out`，但没有看到入站链路恢复或新 Telegram 会话落库。
- 最近一小时里 Telegram listener 基本每分钟都在固定重试一次 `getUpdates`，且每次都因 `Connection refused (os error 61)` 失败。
- 仓库内最近的 Telegram 会话更新时间仍停留在 2026-03-18，最近 24 小时没有任何 Telegram 新消息落库，说明问题不是单次抖动，而是当前入站链路处于持续不可用状态。
- 这是功能性问题，不是内容质量问题；损害点在于 Telegram 用户消息进不来，而不是回答写得不够好。

## 用户影响

- Telegram 渠道当前很可能无法接收任何新消息，用户即使发起提问也不会进入正常会话处理。
- 之所以定级为 `P2`，是因为这是整条 Telegram 入站链路的功能中断，但目前证据仍主要来自日志与落库缺失，尚未拿到最近一小时真实用户投诉或单条会话超时样本。
- 这不是 `P3`：问题不在回答质量，而在渠道监听本身失效。

## 根因判断

- 最新直接触发点已经扩展到三类：旧 Telegram 进程残留并持续占用启动锁、Telegram bot token 无效导致 sidecar 在进入长轮询前退出，以及此前已经观测到的 TCP 连接拒绝/连接超时。
- 因此当前根因不再只能解释为网络可达性、代理/DNS、上游屏蔽或 bot token 配置错误；还必须核对 bundled runtime 的旧进程回收与启动锁释放链路，确认 restart 后是否仍残留僵尸或孤儿 sidecar。
- 目前没有证据表明这是已有 Feishu/Heartbeat 缺陷的同根因复用，应作为独立渠道故障跟踪。

## 下一步建议

- 优先核对 bundled runtime 为什么在 `previous managed children stopped` 之后仍留下占用 Telegram 启动锁的旧进程；随后再核对当前读取的 Telegram bot token 来源、是否与生产 bot 一致、是否被错误配置覆盖，并补排查当前环境到 `api.telegram.org` 的网络可达性、代理/防火墙、DNS 解析与证书链路。
- 修复后优先验证两件事：
  1. `sidecar.log` 不再出现 `Invalid bot token`，也不再持续出现 `GetUpdates ... Connection refused` / timeout
  2. Telegram 新消息能重新写入 `sessions` / `session_messages`
