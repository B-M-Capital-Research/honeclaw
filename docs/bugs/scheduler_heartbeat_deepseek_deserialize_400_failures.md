# Bug: Heartbeat 定时任务在多 provider 下仍会把上游 `HTTP 400` 误解析成 `invalid type: integer 400` 并整轮失败

- **发现时间**: 2026-04-28 11:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-29 19:30` 窗口最新完成样本里，`run_id=10300`（`持仓重大事件心跳检测`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `detail_json.heartbeat_model` 仍是 `moonshotai/kimi-k2.5`
  - 错误仍保持同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 316`
- 最近一小时运行日志：`data/runtime/logs/sidecar.log`
  - `2026-04-29 19:30:52.576` 上游先返回真实 bad request：`This endpoint's maximum context length is 262144 tokens. However, you requested about 1956444 tokens...`
  - `2026-04-29 19:30:52.657-19:30:52.668` heartbeat 链路随后没有把这条上游 `HTTP 400` 正确保留，而是再次塌缩成 `failed to deserialize api response: invalid type: integer \`400\``
  - 这说明 `2026-04-28` 记录的 raw HTTP 兜底解析修复并没有在当前生产链路稳定生效，当前活跃问题仍是“上游 400 被二次反序列化错误掩盖”
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-28 15:00` 窗口最新完成样本里，`run_id=8858`（`持仓重大事件心跳检测`）再次落成 `execution_failed + skipped_error + delivered=0`
    - 同一条 run 的 `detail_json.heartbeat_model` 已不是 DeepSeek，而是 `moonshotai/kimi-k2.5`
    - 错误仍保持同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer 400, expected a string at line 1 column 316`
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-28 15:00:58.821` 上游先返回真实 bad request：`This endpoint's maximum context length is 262144 tokens. However, you requested about 1988312 tokens...`
    - `2026-04-28 15:00:58.852-15:00:58.859` heartbeat 链路随后没有把这条上游 `HTTP 400` 正确保留，而是二次塌缩成 `failed to deserialize api response: invalid type: integer 400`
    - 这说明当前活跃问题已经不是“DeepSeek 专属协议兼容”，而是 heartbeat 公共 OpenAI-compatible 错误解析在不同 provider 下都可能把上游 400 错误掩盖掉
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
    - `2026-04-28 11:00:12-11:01:19` 下一窗口 provider 又切到 `moonshotai/kimi-k2.5`，说明 `10:30` 的 `deepseek/deepseek-v4-pro` 失败不是所有 heartbeat 共通的稳定输出形态，而是一次具体窗口中的 provider / 请求错误。
  - 同一根因的旁证：`data/sessions.sqlite3` -> `cron_job_runs`
    - `2026-04-28 11:00` 窗口 12 条 heartbeat 已全部不再报 `invalid type: integer 400`，但转为 `moonshotai/kimi-k2.5` 的 `JsonNoop` 或 `Empty`，说明 `10:30` 的 400 反序列化失败是独立的 provider 退化，而不是业务条件突然全部触发。

## 端到端链路

1. Feishu heartbeat 任务在 `10:30` 窗口按时启动。
2. scheduler 把同批 heartbeat 路由到具体 provider（已确认至少覆盖 `deepseek/deepseek-v4-pro` 与 `moonshotai/kimi-k2.5`）。
3. provider 返回的 `HTTP 400` 错误响应在 OpenAI-compatible 反序列化阶段再次失败，报出 `invalid type: integer 400, expected a string`。
4. 当前 heartbeat 链路没有把这类 provider 400 解析成可恢复的失败或自动切换到备用 provider。
5. 最终相关任务直接落成 `execution_failed + skipped_error`，用户收不到这一轮监控结果。

## 期望效果

- Heartbeat 在任意 provider 下遇到 `HTTP 400` 或等价 bad request 时应被稳定解析，并落成可诊断的错误，而不是在 SDK 反序列化层二次崩成 `invalid type: integer 400`。
- 对同批 heartbeat 成片出现的 provider 级错误，应有自动重试、备用 provider fallback，或至少清晰的降级标记。
- 不应让用户监控任务在单个 provider 返回体格式变化后直接整轮失效。

## 当前实现效果

- `2026-04-29 19:30` 的 `run_id=10300` 说明，这条缺陷并未随着 `2026-04-28` 的修复结论退出线上：同一类 `maximum context length` 上游 400 仍被压扁成 `invalid type: integer \`400\``。
- 当前不能再把本单视为 `Fixed`；最新真实窗口已经再次复现相同错误形态，因此状态回调为 `New` 并重新进入活跃缺陷队列。
- `10:30` 窗口里 8 条 heartbeat 被同一个 `deepseek/deepseek-v4-pro` 反序列化错误整轮打断。
- `15:00` 窗口里 `run_id=8858` 又在 `moonshotai/kimi-k2.5` 下复现同一类二次反序列化失败；sidecar 已明确给出真实上游原因是 `maximum context length` 超限，但最终落库错误仍只剩 `invalid type: integer 400`
- 同批并非所有 heartbeat 都应触发业务提醒，因为仍有 3 条样本成功收口为 `JsonNoop`。
- 这说明当前问题已经扩大为“上游 bad request 被公共错误解析掩盖”，而不是仅限于某个 provider 的单次协议兼容。

## 用户影响

- 这是功能性缺陷。heartbeat 的核心价值是定时检查并在需要时提醒；当 provider 400 被反序列化错误放大为整轮失败时，用户会丢失这一轮监控覆盖。
- 定级为 `P2`，因为影响集中在 heartbeat 任务族，没有证据表明直聊主链路或全部 scheduler 全局不可用。
- 它不是 `P3` 质量问题，因为已经直接导致任务未执行完成，而不是单纯输出质量下降。

## 根因判断

- 直接触发点是上游 provider 返回 `HTTP 400` 错误体时，当前 OpenAI-compatible 解析预期过窄，导致 `400` 整数状态字段之类的内容在反序列化时再次失败。
- `10:30` 的 DeepSeek 样本与 `15:00` 的 Moonshot 样本共用同一种二次报错，且后者还能从 sidecar 看到真实上游错误是 `maximum context length`；这更像是公共错误解析缺陷，而不是某个单独 provider 的偶发兼容问题。
- 现有 heartbeat 链路缺少对这类 provider bad request 的稳态吸震，因此错误直接暴露成 `execution_failed + skipped_error`。

## 下一步建议

- 先排查 heartbeat 公共 OpenAI-compatible 错误解析，确认对上游 `HTTP 400` 的字段类型假设为什么会在不同 provider 下都失败。
- 为 heartbeat provider 错误补最小可观测性：至少记录上游状态码、错误字段类型和 provider 名，而不是只留下二次反序列化报错。
- 若 heartbeat 继续依赖多 provider，应补 provider 级 fallback 或短重试，并在上下文超限时优先保留原始 `maximum context length` 诊断，避免再次被二次反序列化掩盖。

## 修复情况（2026-04-28）

- `crates/hone-llm/src/openai_compatible.rs` 在 SDK `JSONDeserialize` 失败时改用 raw HTTP 兜底请求解析，非 2xx 响应会保留 `HTTP status`、错误 `message` 与数字或字符串 `code`。
- 该修复不为某个 provider 特判，只处理 OpenAI-compatible 错误体 schema 兼容性，避免把真实 `maximum context length` / 403 等原因压扁成 serde `invalid type`。
- 验证：`cargo test -p hone-llm extracts_ --lib`。
- `2026-04-29 19:30` 的 `run_id=10300` 说明上述修复结论未稳定覆盖当前生产 heartbeat 链路；本单由 `Fixed` 回调为 `New`，待重新核查实际生效路径。

## 修复情况（2026-04-30）

- 本轮复核确认 heartbeat 的 OpenRouter 模型路径走 `OpenRouterProvider`，而不是已修过的 `OpenAiCompatibleProvider`，因此仍会直接把 SDK 的 `JSONDeserialize` 错误向上抛出。
- `crates/hone-llm/src/openrouter.rs` 已为 OpenRouter 多 key provider 补同类 raw HTTP 兜底解析：
  - 每个 key 同时保存 SDK client 与 raw HTTP 所需的 `reqwest::Client` / API key / base URL。
  - 当 SDK 因 provider 错误体 schema 变化触发 `JSONDeserialize` 时，用同一请求重放一次 raw HTTP。
  - 非 2xx 响应保留 `upstream HTTP <status>`、错误 `message`，并兼容数字或字符串 `code`。
- 该修复不特判 DeepSeek、Moonshot 或某个模型，只收窄 OpenRouter 兼容错误体解析边界，避免 `maximum context length` 这类真实上游原因再次被压成 `invalid type: integer 400`。

## 回归验证（2026-04-30）

- `cargo test -p hone-llm openrouter -- --nocapture`
- `rustfmt --edition 2024 --check crates/hone-llm/src/openrouter.rs`
- `cargo check -p hone-llm --tests`
