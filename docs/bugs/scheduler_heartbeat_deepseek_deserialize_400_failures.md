# Bug: Heartbeat 定时任务在多 provider 下仍会把上游 `HTTP 400` 误解析成 `invalid type: integer 400` 并整轮失败

- **发现时间**: 2026-04-28 11:01 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-03 14:00-14:01` 窗口最新完成样本里，`run_id=14654`（`持仓重大事件心跳检测`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `error_message` 再次回到同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 316`
- 最近一小时运行日志：`data/runtime/logs/sidecar.log`
  - `2026-05-03 14:00:37.753` 同窗先记录真实上游 bad request：`This endpoint's maximum context length is 262144 tokens. However, you requested about 1323585 tokens ... "code":400`
  - `2026-05-03 14:00:37.805-14:00:37.811` heartbeat 链路随后仍把这条上游 `HTTP 400` 压回 `failed to deserialize api response: invalid type: integer \`400\``
  - 同批次 `ORCL 大事件监控` 在 `14:00:31` 仍能落成 `completed + sent`，而 `Cerebras IPO与业务进展心跳监控` 则在 `14:01:36` 落成 `noop + skipped_noop`，说明当前复发仍属于 heartbeat 公共 provider 错误解析层的离散失效，而不是整批调度停摆。
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-03 07:30-07:31` 窗口最新完成样本里，`run_id=14355`（`持仓重大事件心跳检测`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `error_message` 再次回到同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 314`
- 最近一小时运行日志：`data/runtime/logs/sidecar.log`
  - `2026-05-03 07:30:34.029` 同窗先记录真实上游 bad request：`This endpoint's maximum context length is 262144 tokens. However, you requested about 387379 tokens ... "code":400`
  - `2026-05-03 07:30:34.083-07:30:34.084` heartbeat 链路随后仍把这条上游 `HTTP 400` 压回 `failed to deserialize api response: invalid type: integer \`400\``
  - 同批次 `ORCL 大事件监控` 与 `TEM大事件心跳监控` 又分别在 `07:30:22.079`、`07:30:18.285` 退化成 `parse_kind=Empty`，说明当前复发仍属于 heartbeat 公共 provider 错误解析层的离散失效，而不是整批调度停摆。
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-02 10:30-10:31` 窗口最新完成样本里，`run_id=13412`（`持仓重大事件心跳检测`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `error_message` 再次回到同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 314`
- 最近一小时运行日志：`data/runtime/logs/sidecar.log`
  - `2026-05-02 10:31:12.091` 同窗先记录真实上游 bad request：`This endpoint's maximum context length is 262144 tokens. However, you requested about 377581 tokens ... "code":400`
  - `2026-05-02 10:31:12.116-10:31:12.119` heartbeat 链路随后仍把这条上游 `HTTP 400` 压回 `failed to deserialize api response: invalid type: integer \`400\``
  - 同批次 `ORCL 大事件监控` 仍能在 `11:02:12.616-11:02:12.618` 落成 `JsonTriggered + deliver`，而 `Monitor_Watchlist_11` 则在 `11:02:02.320-11:02:02.321` 再次退化成 `error decoding response body`；这说明当前复发仍属于 heartbeat 公共 provider 错误解析层的离散失效，而不是整批调度停摆。
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-02 01:02-01:03` 窗口最新完成样本里，`run_id=12985`（`持仓重大事件心跳检测`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `error_message` 再次回到同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 316`
- 最近一小时运行日志：`data/runtime/logs/sidecar.log`
  - `2026-05-02 01:02:52.134` 同窗任务终态直接记录为上述二次反序列化错误，说明 `2026-05-01` 标记为 `Fixed` 的修复结论并未在当前真实 heartbeat 窗口生效。
  - 同批次 `ORCL 大事件监控` 仍能在 `01:02:17.104-01:02:17.105` 落成 `JsonTriggered + deliver`，而 `Monitor_Watchlist_11` 则在 `00:02:02.053-00:02:02.054` 退化成 `error decoding response body`；这说明当前复发仍属于 heartbeat 公共 provider 错误解析层的离散失效，而不是整批调度停摆。
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-01 14:30-14:31` 窗口最新完成样本里，`run_id=12484`（`ORCL 大事件监控`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `error_message` 仍保持同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 314`
- 最近一小时运行日志：`data/runtime/logs/web.log.2026-05-01`
  - `2026-05-01 14:30:29.430` 同窗任务终态直接记录为上述二次反序列化错误，说明当前活跃 provider 路径下仍没有保留原始 bad request 诊断。
  - 这次样本落在 `ORCL 大事件监控`，并与同窗其它 `noop` / `sent` heartbeat 并存，说明故障仍是 heartbeat 公共错误解析层的离散失效，而不是整批调度停摆。
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-05-01 03:00-03:03` 窗口最新完成样本里，`run_id=11923`（`持仓重大事件心跳检测`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `error_message` 仍保持同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 314`
- 最近一小时运行日志：`data/runtime/logs/sidecar.log`
  - `2026-05-01 03:01:16.418` 上游先返回真实 bad request：`This endpoint's maximum context length is 262144 tokens. However, you requested about 746997 tokens...`
  - `2026-05-01 03:01:16.458-03:01:16.461` heartbeat 链路随后没有把这条上游 `HTTP 400` 正确保留，而是再次塌缩成 `failed to deserialize api response: invalid type: integer \`400\``
  - 这说明 `2026-04-30` 记录的 OpenRouter raw HTTP 兜底解析修复结论仍未稳定覆盖当前生产 heartbeat 链路；当前活跃问题仍是“上游 400 被二次反序列化错误掩盖”
- 最近一小时真实调度窗口：`data/sessions.sqlite3` -> `cron_job_runs`
  - `2026-04-30 16:01` 窗口最新完成样本里，`run_id=11362`（`Cerebras IPO与业务进展心跳监控`）再次落成 `execution_failed + skipped_error + delivered=0`
  - 同一条 run 的 `error_message` 仍保持同一形态：`LLM 错误: failed to deserialize api response: invalid type: integer \`400\`, expected a string at line 1 column 316`
- 最近一小时运行日志：`data/runtime/logs/sidecar.log`
  - `2026-04-30 16:01:18.988` 上游先返回真实 bad request：`This endpoint's maximum context length is 262144 tokens. However, you requested about 1365907 tokens...`
  - `2026-04-30 16:01:19.046-16:01:19.051` heartbeat 链路随后没有把这条上游 `HTTP 400` 正确保留，而是再次塌缩成 `failed to deserialize api response: invalid type: integer \`400\``
  - 这说明 `2026-04-30` 记录的 OpenRouter raw HTTP 兜底解析修复结论仍未稳定覆盖当前生产链路；当前活跃问题仍是“上游 400 被二次反序列化错误掩盖”
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

- `2026-05-01` 本轮代码修复后，heartbeat 当前使用的 `llm.auxiliary -> OpenAiCompatibleProvider` 与备用 `OpenRouterProvider` 都会在 SDK `JSONDeserialize` 之外，对 `ApiError` 类上游非 2xx / schema 失败继续走 raw HTTP 兜底，不再只依赖单一 serde 失败分支。
- 同时补了 `chat_with_tools` 分支的数值 `code=400` 回归测试；这是 heartbeat 实际使用的调用路径，能直接覆盖此前线上 `function_calling` 任务把上游 bad request 压成 `invalid type: integer 400` 的场景。
- 但 `2026-05-02 10:31` 的 `run_id=13412` 说明上述修复结论仍未在当前真实 heartbeat 窗口生效；本单继续维持 `New`，待重新核查当前运行实例究竟命中了哪条 provider / SDK 错误路径。

## 用户影响

- 这是功能性缺陷。heartbeat 的核心价值是定时检查并在需要时提醒；当 provider 400 被反序列化错误放大为整轮失败时，用户会丢失这一轮监控覆盖。
- 定级为 `P2`，因为影响集中在 heartbeat 任务族，没有证据表明直聊主链路或全部 scheduler 全局不可用。
- 它不是 `P3` 质量问题，因为已经直接导致任务未执行完成，而不是单纯输出质量下降。

## 根因判断

- 直接触发点是上游 provider 返回 `HTTP 400` 错误体时，当前 OpenAI-compatible 解析预期过窄，导致 `400` 整数状态字段之类的内容在反序列化时再次失败。
- `10:30` 的 DeepSeek 样本与 `15:00` 的 Moonshot 样本共用同一种二次报错，且后者还能从 sidecar 看到真实上游错误是 `maximum context length`；这更像是公共错误解析缺陷，而不是某个单独 provider 的偶发兼容问题。
- 现有 heartbeat 链路缺少对这类 provider bad request 的稳态吸震，因此错误直接暴露成 `execution_failed + skipped_error`。
- `2026-05-01` 复核代码路径后确认，heartbeat 的 `AuxiliaryFunctionCalling` 实际优先取 `llm.auxiliary`，当前生产配置把它指到了 `https://openrouter.ai/api/v1`，因此线上主故障路径首先落在 `OpenAiCompatibleProvider::chat_with_tools`，而不是此前文档里假设的 `OpenRouterProvider` 主链路。

## 下一步建议

- 观察下一次真实 heartbeat 窗口，确认 `cron_job_runs.error_message` 不再出现 `invalid type: integer 400`，而是保留 `upstream HTTP 400` 与真实 bad request 文案。
- 若后续仍有同类失败，再补 provider 名 / 状态码 / body schema 的显式日志，进一步缩短线上归因路径。

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

## 修复情况（2026-05-01）

- 复核后确认 heartbeat 实际走 `AuxiliaryFunctionCalling`，当前生产配置优先命中 `llm.auxiliary` 的 `OpenAiCompatibleProvider`，且该 provider 的 `base_url` 指向 OpenRouter；此前仅修 `OpenRouterProvider` 不足以覆盖线上主路径。
- `crates/hone-llm/src/openai_compatible.rs`
  - raw HTTP 兜底触发条件从仅 `JSONDeserialize` 扩到 `JSONDeserialize | ApiError`，避免 SDK 在不同非 2xx / schema 失败分支下绕过原始错误体保留。
  - 新增 `chat_with_tools_preserves_numeric_provider_error_body_after_sdk_deserialize_failure`，直接覆盖 heartbeat 使用的 tool-calling 分支。
- `crates/hone-llm/src/openrouter.rs`
  - 同步把 raw HTTP 兜底触发条件放宽到 `JSONDeserialize | ApiError`，避免备用 OpenRouter provider 与主路径继续分叉。
  - 新增对应的 `chat_with_tools` 回归测试，保证两条 provider 实现保持一致。

## 回归验证（2026-05-01）

- `cargo test -p hone-llm --lib -- --nocapture`
- `rustfmt --edition 2024 crates/hone-llm/src/openai_compatible.rs crates/hone-llm/src/openrouter.rs`

## 复核结论（2026-05-02）

- 本轮按当前自动化约束，不再用当前机器旧生产窗口样本作为活跃判定依据。
- 代码复核确认当前仓库同时覆盖 `OpenAiCompatibleProvider` 与 `OpenRouterProvider` 的 `JSONDeserialize | ApiError` raw HTTP fallback。
- `chat_with_tools_preserves_numeric_provider_error_body_after_sdk_deserialize_failure` 与 `chat_with_tools_preserves_openrouter_numeric_error_body_after_sdk_deserialize_failure` 仍直接断言错误文案不再包含 `invalid type: integer`，并保留上游 `HTTP 400` 诊断。
- 但 `2026-05-02 10:31` 的最新真实 heartbeat 样本已经再次复现完全相同的坏态，说明仅凭仓库代码复核不足以维持 `Fixed` 结论；本轮状态回退为 `New`。若当前运行实例已部署最新代码，应优先排查是否存在第三条 provider 调用路径、旧二进制未重启，或 heartbeat 实际没有走到这两处 provider 实现。

## 风险

- 本轮未做运行态复验，因为任务约束明确不重启服务；真实进程仍需在下一个 heartbeat 窗口验证是否已拿到新的 provider 错误保留形态。
- 若上游后续再返回更非标准的错误体（例如既不是 OpenAI error wrapper，也没有 `message/msg/detail`），当前逻辑仍只保证不再把数值 `code` 重新压扁为 serde 整数类型错误。
