# Bug: Heartbeat 定时任务切到 DeepSeek provider 后批量报 `invalid type: integer 400` 并整轮失败

- **发现时间**: 2026-04-28 11:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `2026-04-28 10:30` 窗口共 11 条 heartbeat 完成样本，其中 8 条统一落成 `execution_failed + skipped_error + delivered=0`，错误完全相同：`LLM 错误: failed to deserialize api response: invalid type: integer 400, expected a string at line 1 column 56`
    - 失败样本分别为：
      - `8633` `小米破位预警`
      - `8634` `CAI破位预警`
      - `8635` `ASTS 重大异动心跳监控`
      - `8636` `小米30港元破位预警`
      - `8637` `RKLB异动监控`
      - `8638` `TEM破位预警`
      - `8639` `持仓重大事件心跳检测`
      - `8640` `Monitor_Watchlist_11`
    - 同一批样本的 `detail_json.heartbeat_model` 全部指向 `deepseek/deepseek-v4-pro`。
    - 同窗仅有 `8632`（`全天原油价格3小时播报`）、`8641`（`ORCL 大事件监控`）、`8642`（`TEM大事件心跳监控`）保留 `noop + skipped_noop`，且 `parse_kind=JsonNoop`。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 10:30:06-10:30:54` 同一窗口 heartbeat 连续收口，其中失败样本没有留下正常 `parse_kind`，只在 `cron_job_runs` 里表现为 provider 反序列化错误。
    - `2026-04-28 11:00:12-11:01:19` 下一窗口 provider 又切到 `moonshotai/kimi-k2.5`，说明 `10:30` 的 `deepseek/deepseek-v4-pro` 失败不是所有 heartbeat 共通的稳定输出形态，而是 provider 级异常后被切走。
  - 同一根因的旁证：`data/sessions.sqlite3` -> `cron_job_runs`
    - `2026-04-28 11:00` 窗口 12 条 heartbeat 已全部不再报 `invalid type: integer 400`，但转为 `moonshotai/kimi-k2.5` 的 `JsonNoop` 或 `Empty`，说明 `10:30` 的 400 反序列化失败是独立的 provider 退化，而不是业务条件突然全部触发。

## 端到端链路

1. Feishu heartbeat 任务在 `10:30` 窗口按时启动。
2. scheduler 把同批 heartbeat 路由到 `deepseek/deepseek-v4-pro`。
3. provider 返回的错误响应在 OpenAI-compatible 反序列化阶段失败，报出 `invalid type: integer 400, expected a string`。
4. 当前 heartbeat 链路没有把这类 provider 400 解析成可恢复的失败或自动切换到备用 provider。
5. 最终 8 条任务直接落成 `execution_failed + skipped_error`，用户收不到这一轮监控结果。

## 期望效果

- Heartbeat 切换 provider 后，provider 返回 `HTTP 400` 或等价 bad request 时应被稳定解析，并落成可诊断的错误，而不是在 SDK 反序列化层二次崩成 `invalid type: integer 400`。
- 对同批 heartbeat 成片出现的 provider 级错误，应有自动重试、备用 provider fallback，或至少清晰的降级标记。
- 不应让用户监控任务在单个 provider 返回体格式变化后直接整轮失效。

## 当前实现效果

- `10:30` 窗口里 8 条 heartbeat 被同一个 `deepseek/deepseek-v4-pro` 反序列化错误整轮打断。
- 同批并非所有 heartbeat 都应触发业务提醒，因为仍有 3 条样本成功收口为 `JsonNoop`。
- 到 `11:00` 窗口时，同批任务已被切到 `moonshotai/kimi-k2.5`，`invalid type: integer 400` 不再出现；这进一步说明 `10:30` 的问题集中在 DeepSeek provider 返回体/协议兼容，而不是 heartbeat 业务逻辑本身。

## 用户影响

- 这是功能性缺陷。heartbeat 的核心价值是定时检查并在需要时提醒；当 provider 400 被反序列化错误放大为整轮失败时，用户会丢失这一轮监控覆盖。
- 定级为 `P2`，因为影响集中在 heartbeat 任务族，没有证据表明直聊主链路或全部 scheduler 全局不可用。
- 它不是 `P3` 质量问题，因为已经直接导致任务未执行完成，而不是单纯输出质量下降。

## 根因判断

- 直接触发点是 `deepseek/deepseek-v4-pro` 返回的错误体与当前 OpenAI-compatible 解析预期不一致，导致 `400` 整数状态字段之类的内容在反序列化时再次失败。
- 由于同批样本都记录同一个 `heartbeat_model`，且下一窗口切换 provider 后同类错误立即消失，这更像是 provider 响应协议兼容问题，而不是单个 heartbeat prompt 或数据源输入问题。
- 现有 heartbeat 链路缺少对这类 provider bad request 的稳态吸震，因此错误直接暴露成 `execution_failed + skipped_error`。

## 下一步建议

- 先排查 `deepseek/deepseek-v4-pro` 在 heartbeat 运行时的真实上游返回体，确认是 provider 协议变更、模型路由错误，还是 OpenAI-compatible 错误解析假设过窄。
- 为 heartbeat provider 错误补最小可观测性：至少记录上游状态码、错误字段类型和 provider 名，而不是只留下二次反序列化报错。
- 若 DeepSeek 仍会被继续用于 heartbeat，应补一次 provider 级 fallback 或短重试，避免同类窗口继续整批失效。
