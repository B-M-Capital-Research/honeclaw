# Bug: Heartbeat 监控使用 `mimo-v2.5-pro` 时批量命中 `Param Incorrect` 并漏发

- **发现时间**: 2026-05-12 23:03 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Closed
- **GitHub Issue**: 无

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-13 11:08 CST` 复核：该缺陷从 `Fixed` 更新为 `Closed`。本轮 07:30-10:00 CST 旧 live 运行态仍新增 `59` 条同类 heartbeat 失败，覆盖 `10` 个 job；错误均为 `LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)`，终态仍为 `execution_failed + skipped_error + delivered=0`。
  - `2026-05-13 10:22 CST` Feishu runtime 重启后未再看到 `mimo-v2.5-pro` provider 400；10:30 CST heartbeat 窗口已有 `DRAM 心跳监控`、`持仓重大事件心跳检测`、`Cerebras IPO与业务进展心跳监控` 成功 `completed + sent + delivered=1`，其余同窗为正常 `noop + skipped_noop`；11:00 CST 窗口全部为正常 `noop + skipped_noop`。
  - `data/runtime/logs/sidecar.log` 在 10:22 CST 记录 Feishu scheduler 启动并连接，此后同窗仅看到 web_search key quota warning 与正常 heartbeat 收口，未再出现 `[HeartbeatDiag] runner_error ... model=mimo-v2.5-pro ... Param Incorrect`。
  - `2026-05-13 07:08 CST` 复核：当前 HEAD 已是 `d3dffd6 Fix heartbeat mimo reasoning transcript replay`，但 live channel/backend 仍未确认重启到修复代码；本轮按旧运行态 / 未部署证据补充，不把状态从 `Fixed` 回退为 `New`。
  - 从 `2026-05-13T03:30:09+08:00` 到 `2026-05-13T07:00:18+08:00`，继续新增 `80` 条同类 heartbeat 失败，覆盖 `11` 个 job；终态均为 `execution_failed + skipped_error + delivered=0`，错误均为 `LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)`。
  - 失败覆盖 `DRAM 心跳监控`、`TEM破位预警`、`Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控`、`伦敦金跌破4500提醒`、`小米30港元破位预警`、`RKLB异动监控` 与 `全天原油价格3小时播报`。
  - 同窗仍有 `Oil_Price_Monitor_Closing`、`OWALERT_PostMarket`、`科技成长赛道大盘极值与情绪监控` 等非 heartbeat / 普通 scheduler 成功 `completed + sent`，故障仍集中在 heartbeat `mimo-v2.5-pro` function-calling 路径，而不是 Feishu 出站或 scheduler 全局停摆。
  - `data/runtime/logs/web.log.2026-05-12` 在 `2026-05-13 07:00 CST` 仍记录多个 `[HeartbeatDiag] runner_error ... model=mimo-v2.5-pro failure_kind=provider_http_error error="LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)"`。
  - `2026-05-13 03:02 CST` 复核：该缺陷仍为活跃 `New`。从 `2026-05-12T23:30:12+08:00` 到 `2026-05-13T03:00:25+08:00`，最近四小时继续新增 `82` 条同类 heartbeat 失败，覆盖 `11` 个 job；其中 10 个核心 heartbeat job 在 23:30、00:00、00:30、01:00、01:30、02:00、02:30、03:00 窗口基本连续失败。
  - `2026-05-12T22:30 CST`：`7` 条 heartbeat 同窗失败，覆盖 `DRAM 心跳监控`、`TEM破位预警`、`Cerebras IPO与业务进展心跳监控`、`持仓重大事件心跳检测`、`TEM大事件心跳监控`、`Monitor_Watchlist_11`、`TSLA 正负触发条件心跳监控`。
  - `2026-05-12T23:00 CST`：`9` 条 heartbeat 同窗失败，覆盖 `Cerebras IPO与业务进展心跳监控`、`DRAM 心跳监控`、`Monitor_Watchlist_11`、`伦敦金跌破4500提醒`、`TEM大事件心跳监控`、`TEM破位预警`、`RKLB异动监控`、`小米30港元破位预警`、`TSLA 正负触发条件心跳监控`。
  - 上述失败均落成 `execution_failed + skipped_error + delivered=0`。
  - `error_message` 均为 `LLM 错误: upstream HTTP 400: Param Incorrect (code: 400)`。
  - `detail_json.failure_kind=provider_http_error`，`detail_json.heartbeat_model=mimo-v2.5-pro`。
- 最近四小时同窗仍有非 heartbeat 定时任务成功送达，例如 `run_id=19503` 的 `核心观察股池晚间快报`、`run_id=19517` 的 `科技成长股持仓买卖点日内预警`、`run_id=19521/19526/19528` 的每日动态监控均为 `completed + sent + delivered=1`，说明不是 scheduler 或 Feishu 出站全局停摆。

## 端到端链路

1. Feishu heartbeat 调度在半点 / 整点窗口批量执行多个监控 job。
2. 公共 heartbeat runner 调用 `mimo-v2.5-pro`。
3. 上游返回 `HTTP 400 Param Incorrect`。
4. 本地已能把失败分类为 `provider_http_error`，但没有自动切换模型、重试兼容参数或降级到可用 provider。
5. 多个监控 job 同窗落成 `execution_failed + skipped_error + delivered=0`，用户侧收不到本轮 heartbeat 检查结果。

## 期望效果

- Heartbeat 公共链路遇到 provider 参数兼容类 `HTTP 400` 时，应有稳定止血方案，例如切换已知可用 heartbeat provider、移除不兼容参数后重试，或将整窗故障提升为可观测告警。
- 单个 provider / 模型参数错误不应在连续半点 / 整点窗口批量压垮多个 heartbeat 监控。
- 错误分类已记录后，应进一步避免同一参数组合在同窗重复打爆所有 job。

## 当前实现效果

- 最新四小时内已累计 `82` 条 heartbeat 因同一 `mimo-v2.5-pro` 上游 `HTTP 400 Param Incorrect` 失败。
- 失败已被正确记为 `provider_http_error`，没有被伪装成 noop；但业务效果仍是本轮监控漏发。
- 同窗普通 scheduler 仍可送达，故障集中在 heartbeat provider 参数 / 模型兼容路径。

## 用户影响

- 这是功能性 bug：影响自动监控告警链路的执行与送达，不是单纯回答质量问题。
- 多条 heartbeat 在两个连续窗口漏发，用户可能错过价格、重大事件、破位和观察池监控。
- 定级为 `P2`：影响多个监控 job，但当前证据不是全渠道不可用，也没有达到 P1 的整批连续长时间全量失效；同窗仍有普通定时任务成功送达。

## 根因判断

- 直接触发点不是基础 `chat/completions` contract 本身损坏：同一 `mimo-v2.5-pro` 在单轮 text-only 与最小 tools 请求下仍可 `HTTP 200` 返回。
- 真正根因出在 heartbeat 共享的 auxiliary function-calling 多轮 transcript：`mimo-v2.5-pro` 在 thinking mode 下会返回 `reasoning_content`，而 Hone 进入下一轮 tool-result 回传时只保留了 `content/tool_calls`，没有把上一轮 `reasoning_content` 一并回传，导致上游在第二轮开始拒绝请求并报 `Param Incorrect`。
- 活跃窗口里所有失败 heartbeat 都走 `runner=function_calling`，与上述多轮工具调用链路一致；同一时间普通非 heartbeat 定时任务仍可送达，也与“heartbeat 独有 transcript 兼容问题”相符。
- 既有 `scheduler_heartbeat_openrouter_402_credit_exhaustion_skips_alerts.md` 关注 OpenRouter credits / `HTTP 402` 额度问题；本单是不同 provider / model 的 `HTTP 400` 参数兼容问题。
- 既有 `scheduler_heartbeat_unknown_status_silent_skip.md` 关注模型已产出 triggered 正文但 JSON 结构坏掉导致漏发；本单在模型输出前就被 provider 拒绝，根因不同。

## 修复情况

- 2026-05-13 已在 `agents/function_calling/src/lib.rs` 保留 assistant `reasoning_content`，并通过 `AgentMessage.metadata -> hone_llm::Message.reasoning_content` 在多轮 tool loop 中回传给上游。
- 2026-05-13 已在 `crates/hone-llm/src/openai_compatible.rs` 收口 OpenAI-compatible 非流式请求：一旦消息里出现 `reasoning_content`，改走原始 JSON 请求体并显式携带该字段；同时从响应里提取 `reasoning_content` 供下一轮继续使用。
- 2026-05-13 已把 heartbeat auxiliary function-calling 工具集收窄为 `data_fetch` / `web_search` / `portfolio` / `missed_events` / `local_*`，移除 `skill_tool`、`load_skill`、`notification_prefs`、`deep_research` 等与 heartbeat 无关的 schema，降低同类 provider 兼容风险与请求体膨胀。

## 验证

- `cargo test -p hone-llm chat_with_tools_replays_reasoning_content_in_raw_request_body -- --nocapture`
- `cargo test -p hone-agent run_replays_reasoning_content_into_followup_tool_round -- --nocapture`
- `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- `cargo test -p hone-llm -p hone-agent -p hone-channels --no-run`

## 未验证项 / 后续建议

- 2026-05-13 11:08 CST 已观察到 live 重启后的 10:30 / 11:00 heartbeat 窗口恢复，因此本单关闭。
- 后续若部署后再次出现同一 `mimo-v2.5-pro` reasoning transcript / `Param Incorrect` 失败，应优先在本单追加复发证据，而不是新建重复文档。
