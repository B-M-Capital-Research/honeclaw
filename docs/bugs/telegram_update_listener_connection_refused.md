# Bug: Telegram update listener 持续网络不可达，近一个月无新会话落库

- **发现时间**: 2026-04-21 10:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
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

1. Telegram 渠道进程启动后进入 `getUpdates` 长轮询。
2. listener 在访问 `https://api.telegram.org/.../GetUpdates` 时直接命中 TCP 连接失败，错误为 `Connection refused (os error 61)`。
3. 当前实现只做固定退避重试，没有恢复到可用轮询状态。
4. 因为入站更新拉取失败，新的 Telegram 用户消息无法进入正常处理链路，也不会落到 `sessions` / `session_messages`。

## 期望效果

- Telegram listener 应稳定完成 `getUpdates` 轮询，而不是在传输层持续 `Connection refused`。
- 即使上游网络短时异常，也应具备明确的可恢复策略或更清晰的链路告警，而不是长时间反复重试无新消息。
- Telegram 渠道至少应能恢复到“用户发消息后能够创建/更新会话并落库”的基本功能状态。

## 当前实现效果

- 到 `2026-04-21 18:36` 最新窗口，Telegram listener 仍在 `GetUpdates` 阶段超时，且下一次重试继续连接超时；没有看到 Telegram 新消息恢复落库。
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

- 直接触发点是 Telegram listener 在请求 `GetUpdates` 时发生网络连接失败；最新样本已覆盖 TCP 连接拒绝与连接超时两种形态，当前更像网络可达性、代理/DNS、上游屏蔽或本机出站链路问题，而不是单条消息 payload 处理错误。
- 由于 listener 能持续进入重试分支，说明 bot 进程本身仍在运行；失效点集中在与 Telegram API 的连接建立阶段。
- 目前没有证据表明这是已有 Feishu/Heartbeat 缺陷的同根因复用，应作为独立渠道故障跟踪。

## 下一步建议

- 优先排查当前环境到 `api.telegram.org` 的网络可达性、代理/防火墙、DNS 解析与证书链路。
- 修复后优先验证两件事：
  1. `sidecar.log` 不再持续出现 `GetUpdates ... Connection refused`
  2. Telegram 新消息能重新写入 `sessions` / `session_messages`
