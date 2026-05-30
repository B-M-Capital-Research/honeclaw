# Bug: Codex version probe 资源耗尽导致直聊和定时任务批量失败并外露原始 runner 错误

- 发现时间：2026-05-20 11:06 CST
- Bug Type：System Error
- 严重等级：P1
- 状态：New
- GitHub Issue：[#43](https://github.com/B-M-Capital-Research/honeclaw/issues/43)

## 证据来源

- `data/sessions.sqlite3` 最近四小时真实会话窗口（`2026-05-20 07:02-11:00 CST`）：
  - `session_id=Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c`
    - `2026-05-20T08:58:59+08:00` 用户请求 `AMPX的画像`
    - `2026-05-20T08:59:00+08:00` assistant 直接返回 `failed to probe codex version via codex: Resource temporarily unavailable (os error 35)`
  - `session_id=Actor_feishu__direct__ou_5f1ed3244e3a7b34789cea10eeabe4da98`
    - `2026-05-20T09:22:28+08:00` 用户请求 `接下去80后转型什么好`
    - `2026-05-20T09:22:28+08:00` assistant 直接返回同一 runner probe 原始错误
    - `2026-05-20T09:22:50+08:00` 用户追问 `？`
    - `2026-05-20T09:22:50+08:00` assistant 再次返回同一原始错误
  - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-05-20T09:53:54+08:00` 用户请求 `谷歌io大会怎么看`
    - `2026-05-20T09:53:54+08:00` assistant 直接返回同一原始错误
- `data/sessions.sqlite3` -> `cron_job_runs` 最近四小时：
  - 至少 15 条普通 scheduler run 落成 `execution_failed + sent + delivered=1`，底层 `error_message` 为 `failed to probe codex version via codex: Resource temporarily unavailable (os error 35)`。
  - 影响任务包括 `每日SemiAnalysis与Citrini文章追踪`、`美股AI产业链盘后报告`、`每日CNN贪婪指数`、`创新药持仓每日动态推送`、`Hone_AI_Morning_Briefing`、`OKLO每日重要事件跟踪`、`闪迪(SNDK)每日行情与行业简报`、`港股持仓与关注股早间行情研判`、`每日有色化工标的新闻追踪`、`特斯拉与火箭实验室新闻日报`、`早9点市场复盘(XME及加密ETF)`、`核心观察池早间简报`、`每日美股降息概率推送`、`Citrini AI 供应链文章跟踪`。
  - `09:00 美股AI与航空科技晨报` 在 Web scheduler 侧使用了产品化失败文案，但 `error_message` 仍记录同一 runner probe 失败。
- 最近四小时运行日志：
  - `data/runtime/logs/web.log.2026-05-20`
    - `2026-05-20 08:00-10:00 CST` 多条 `[MsgFlow/feishu] runner.error kind=SpawnFailed`，错误均为 `failed to probe codex version via codex: Resource temporarily unavailable (os error 35)`。
    - `2026-05-20 09:00 CST` `[MsgFlow/web] runner.error` 与 `定时任务执行失败` 同样命中该错误。
    - `2026-05-20 09:30 CST` `[MsgFlow/discord] runner.error` 同样命中该错误。
  - `data/runtime/logs/hone-feishu.runtime-recovery.log`
    - `2026-05-19 23:28 CST` 起已能看到同类 `SpawnFailed`，本轮 07:02-11:00 CST 确认仍在真实用户/调度窗口影响主链路。

## 端到端链路

1. Feishu 直聊、Feishu scheduler、Web scheduler 或 Discord scheduler 收到用户输入 / 到点触发。
2. `AgentSession` 准备启动 Codex runner。
3. runner 在正式 `session/prompt` 前执行 `codex` version probe。
4. 当前机器资源耗尽或进程创建受限，version probe 立即失败并返回 `Resource temporarily unavailable (os error 35)`。
5. 上层将该 `SpawnFailed` 当成本轮 agent 失败。
6. 多数 Feishu direct / scheduler 失败分支把原始 runner 错误作为 assistant final 或 `response_preview` 送达用户；Web scheduler 其中一条已用较产品化文案，但任务正文仍未产出。

## 期望效果

- Codex runner version probe 失败不应把原始进程错误直接发给用户。
- 用户可见侧应得到脱敏、稳定、可理解的系统繁忙/稍后重试提示。
- scheduler 台账应能区分“runner 启动前失败”和“模型执行中失败”，避免把未执行任务误看成已完成内容投递。
- 高频 `os error 35` 应触发健康检查、退避或并发保护，避免多个渠道在同一窗口批量失败。

## 当前实现效果

- 直聊用户的真实任务没有被执行，只收到原始 runner probe 错误。
- 多个普通 scheduler 任务落成 `execution_failed + sent + delivered=1`，其中多数 `response_preview` 就是原始错误文本。
- 同一根因同时影响 Feishu、Web、Discord 三类入口，说明不是单个用户 prompt 或单个任务配置问题。
- 2026-05-30 11:03 CST 复核：原始 runner 错误外露已被 2026-05-20 修复净化，但 runner probe 资源耗尽本身仍在真实直聊和普通 scheduler 中批量复发；用户可见回复变为脱敏文案，任务正文仍未执行。

## 用户影响

- 这是功能性缺陷：用户直聊请求和定时任务正文均未完成。
- 这是错误边界缺陷：用户可见回复暴露了 `codex` version probe、进程资源错误和本地 runner 启动细节。
- 影响范围跨直聊与定时任务、跨 Feishu/Web/Discord，且最近四小时多次复现，因此定级为 P1。

## 根因判断

- 直接根因是 Codex runner 启动前的 version probe 受本机资源限制影响，返回 `Resource temporarily unavailable (os error 35)`。
- 下游错误净化层没有覆盖 `failed to probe codex version via codex` / `SpawnFailed` / `os error 35` 这类 runner 启动前失败，导致原始错误进入用户可见内容。
- scheduler 对部分 runner 启动前失败仍按 `sent + delivered=1` 登记，使台账更像“发送了有效失败回复”，而不是“任务未能进入 agent 执行”。
- 2026-05-30 复发证据显示：错误净化边界已经生效，`cron_job_runs.detail_json.scheduler.failure_kind=internal_error_suppressed`，但 runner 启动健康、并发 / 资源保护、重试或退避机制仍不足，导致主功能链路继续失败。

## 复发记录

- 2026-05-30 11:03 CST：按最近四小时窗口 `2026-05-30 07:02-11:02 CST` 复核真实会话与日志，确认同根因从 `Fixed` 回退为 `New`。
  - `session_messages` 中至少 5 条 Feishu direct assistant final 向用户返回脱敏失败文案 `当前本机执行环境暂时不可用，请稍后再试。`：`Actor_feishu__direct__ou_5fdb997ed67ac0b7f5403701682185d67a`（07:52）、`Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3`（08:46 / 09:44）、`Actor_feishu__direct__ou_5fe40dc70caa78ad6cb0185c21b53c4732`（09:21）、`Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`（09:49）。
  - `cron_job_runs` 普通 scheduler 有 10 条 `execution_failed + sent + delivered=1`，覆盖 Feishu / Web / Discord；代表任务包括 `每日SemiAnalysis与Citrini文章追踪`、`AI硬件与云厂商相关新闻晨报`、`每日CNN贪婪指数`、`创新药持仓每日动态推送`、`OKLO每日重要事件跟踪`、`09:00 美股AI与航空科技晨报`、`特斯拉与火箭实验室新闻日报`、`核心观察池早间简报`、`每日美股降息概率推送`。
  - `data/runtime/logs/web.log.2026-05-30` 在 08:46-09:49 CST 多次记录 `runner.error kind=SpawnFailed`，底层仍是 `failed to probe codex version via codex: Resource temporarily unavailable (os error 35)`；Web direct `web-user-879a3b18fce2` 在 09:04 / 09:43 / 09:44 也连续命中该失败。
  - 用户可见侧未再暴露原始 `failed to probe` / `os error 35`，但真实请求和定时任务正文未完成，影响直聊与 scheduler 主链路，仍符合 P1。

## 修复记录

- 2026-05-20 12:10 CST：`crates/hone-channels/src/runtime.rs` 新增 runner resource-unavailable 分类。
- 覆盖包含 `codex` / `codex-acp` / `runner` / `acp` 且同时包含 `Resource temporarily unavailable`、`os error 35`、`would block`、`failed to probe`、`version probe` 或 `failed to spawn` 的错误。
- 直聊和通用出站错误映射为：`当前本机执行环境暂时不可用，请稍后再试。`
- scheduler 的 `user_visible_error_message_or_none(...)` 同样返回该脱敏文案，避免 `response_preview` / 用户送达内容包含原始 runner 错误。
- 本修复只做通用错误边界加固，不为单次资源耗尽写重试、绕过或硬编码特殊流程。

## 验证

- `cargo test -p hone-channels user_visible_error_message_maps_codex_probe_resource_errors --lib -- --nocapture`
- `cargo test -p hone-channels user_visible_error_message_or_none_keeps_codex_probe_resource_errors_sanitized --lib -- --nocapture`
- `cargo test -p hone-channels user_visible_error_message --lib -- --nocapture`
- `cargo check -p hone-channels --tests`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/runtime.rs`

## 后续建议

- 为 Codex runner version probe 评估缓存、超时、退避或启动健康检查，避免每轮请求都因短时进程资源耗尽同步失败。
- 为 scheduler 增加 runner 启动前失败分类，例如 `runner_spawn_failed`，便于区分“任务未进入 agent 执行”和“模型执行中失败”。
- 若部署后仍出现 runner 启动前错误原文外发，保留脱敏错误关键词并扩展同一分类函数，不要针对单个日志样本写渠道特判。
