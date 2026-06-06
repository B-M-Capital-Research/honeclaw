# Bug: Codex ACP transport 断连导致直聊和定时请求失败且缺少自动恢复

- **发现时间**: 2026-06-06 11:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-06 07:01-11:01 CST。
  - 本窗共有 12 个 user turn 与 12 个 assistant final，Feishu direct / Discord scheduler 会话均有 assistant 记录收口。
  - assistant final 污染扫描未命中空回复、`company_profiles/...`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic 或 `index out of bounds`。
  - `session_id=Actor_feishu__direct__ou_5f0bdff19e3e341fbbbffe811abecaac61` 在 2026-06-06 09:25 CST 收到用户追问：小分子化学药 / 生物药用药方式 / 是否借助 AI 研发。
  - 2026-06-06 09:29 CST assistant final 只返回脱敏通用失败文案：`抱歉，这次处理失败了。请稍后再试。`，用户本轮问题没有得到回答。
- `data/runtime/logs/acp-events.log`
  - 2026-06-06 09:26 CST 同一 Feishu direct prompt 已启动，随后 runner 输出内部 transport fallback 事件。
  - 2026-06-06 09:29 CST 同一 prompt 返回 `stream disconnected before completion` 内部错误；用户侧没有看到原始 URL 或 transport 细节。
  - 2026-06-06 09:30-09:34 CST Discord scheduler session `Session_discord__group__g_3a1469549745654468692_3ac_3a1469549746518622371` 也出现同类 transport fallback 和 `stream disconnected before completion` 错误。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=38431` / `job_name=每日美股降息概率推送` 在 2026-06-06 09:34 CST 落成 `noop + skipped_noop + should_deliver=0 + delivered=0`。
  - `detail_json.failure_kind=internal_error_suppressed`，说明 scheduler 没有把内部失败外发，但这轮定时任务也没有产出正文。
- 最近四小时无非文档代码提交。

## 端到端链路

1. 用户通过 Feishu direct 发起一个连续追问，前几轮同主题药物机制解释均正常回答。
2. runner 初始化、创建 session 并发起 `session/prompt`。
3. Codex ACP 从 WebSocket fallback 到 HTTPS transport 后，最终返回 `stream disconnected before completion`。
4. Feishu direct 外层把内部错误净化成通用失败文案并写入会话。
5. 用户本轮问题没有被完成，只能手动重试。
6. 同窗 Discord scheduler 也命中同类 transport 断连，但当前 scheduler 失败被抑制为不外发、不送达。

## 期望效果

- ACP transport 断连应有自动重试、备用 transport / runner fallback，或至少在保留安全净化的同时给出更可操作的失败分类。
- 对 Feishu direct 用户主动提问，单次可恢复的 transport 抖动不应直接让整轮清零。
- 对 scheduler，内部失败可以不外发，但应保留明确失败终态与可审计分类，避免误判为正常 `noop`。

## 当前实现效果

- Feishu direct 对用户可见的结果是通用失败，原始错误没有外泄，说明错误净化是生效的。
- 但主请求没有完成，且系统没有在同轮自动恢复或基于既有上下文降级回答。
- Discord scheduler 没有外发通用失败，`should_deliver=0` 是正确止血；但 `execution_status=noop` 与 `failure_kind=internal_error_suppressed` 同时出现，容易把 transport 失败和真正无须发送的业务 `noop` 混在一起。

## 用户影响

- 这是功能性 bug，不是单纯输出质量问题。
- Feishu 用户主动追问没有得到答案，定时任务也有一轮因同类 runner transport 断连未产出正文。
- 定级为 `P2`：影响请求完成率和 scheduler 结果生成，但本窗只有 1 条 Feishu direct 用户可见失败和 1 条 Discord scheduler 抑制失败；没有跨用户大面积不可用、错投、数据破坏或原始错误外泄证据，因此不是 `P1`。

## 根因判断

- 直接根因是 Codex ACP transport 在执行中断连，返回 `stream disconnected before completion`。
- 与 `web_scheduler_acp_stream_disconnect_no_final.md` 同属 ACP transport 断连大类，但本轮新增受影响链路是 Feishu direct 用户主动提问和 Discord scheduler 抑制失败，不是原 Web scheduler SSE / 无终态问题。
- 与 `channel_raw_llm_error_exposure.md` 不同：本轮没有把 `chatgpt.com/backend-api/codex/responses`、transport fallback 或内部错误文本暴露给最终用户。
- 与 `feishu_direct_codex_usage_limit_generic_failure.md` 不同：本轮不是 usage limit / quota，错误分类为 transport disconnect。

## 下一步建议

- 在 ACP runner 调用层为 `stream disconnected before completion` 增加一次短重试，或在同类错误下切换备用 runner / transport。
- 将 scheduler 的 transport 失败终态从业务 `noop` 中区分出来，例如保持 `execution_failed + skipped_error + failure_kind=internal_error_suppressed`，避免巡检和用户侧误读为“条件未命中”。
- 保留现有错误净化规则，继续禁止内部 URL、transport fallback、raw error 进入最终用户文本。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码，未运行代码测试。
- 已验证范围：SQLite 会话收口、assistant final 污染扫描、ACP 事件错误分类、`cron_job_runs` 终态与最近四小时非文档提交检查。
