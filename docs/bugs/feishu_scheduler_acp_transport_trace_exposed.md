# Bug: Feishu scheduler 成功回复外露 Codex transport fallback 原始痕迹

## 发现时间

2026-06-19 15:01 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

Fixed

## GitHub Issue

无，非 P1

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-06-19 11:02-15:01 CST。
  - `data/sessions.sqlite3` 仍未追平最近真实会话，`sessions.max(updated_at)=2026-06-17T10:37:37.207669+08:00`、`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`cron_job_runs.max(executed_at)=2026-06-17T11:01:42.353141+08:00`；本轮用户可见文本以 ACP 流式日志重构。
  - 同窗 ACP 日志可重构 8 个 session、21 次 `session/prompt`、21 次 prompt 均有 response，未见 response error；可见回复均以 `stopReason=end_turn` 收口。
  - 2026-06-19 12:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 的 `每日公司资讯与分析总结` final 开头直接拼入 `Falling back from WebSockets to HTTPS transport. stream disconnected before completion: tls handshake eof`。
  - 同条 final 后续仍正常输出 TEM / CAI / NBIS / CRWV / NVDA / GOOGL / TSM 等观察结论、Juneteenth 休市口径、目标价 / 催化 / 财报节点和来源列表。
- 最近四小时无非文档代码提交。

## 端到端链路

1. Feishu scheduler 触发 `每日公司资讯与分析总结`。
2. runner 执行过程中出现 Codex ACP WebSocket 到 HTTPS 的 transport fallback 痕迹。
3. 虽然最终 prompt 返回 `end_turn`，但内部 transport fallback 文案被拼到用户可见 final 开头。
4. Feishu 用户收到的报告主体可读，但开头包含原始 runner / transport 排障文本。

## 期望效果

- scheduler 成功回复不应出现 `Falling back from WebSockets to HTTPS transport`、`stream disconnected before completion`、`tls handshake eof` 等内部 runner / transport 细节。
- 若 transport 抖动未阻断最终生成，应只输出业务报告；若已阻断，应进入产品化失败提示和可审计失败状态。
- 输出净化应覆盖“成功 final 中夹带内部错误痕迹”的路径，而不只覆盖失败分支。

## 当前实现效果

- 本轮回复有 `stopReason=end_turn`，业务报告主体完整，说明 scheduler 主生成链路没有中断。
- 但 raw transport trace 被直接放在用户可见文本开头，暴露内部运行状态和底层失败细节。
- 同一 final 还外露 `StockAnalysis`、长期画像同步等内部/实现口径；这些同根文案边界问题已另由 `feishu_scheduler_data_fetch_tool_name_exposed.md` 跟踪，本单只聚焦 raw Codex transport trace 外泄。

## 用户影响

- 用户仍能读到主要报告结论、来源和风险提示，调度生成链路最终完成，没有空回复、错投、重复投递、数据写坏或跨用户泄露证据。
- 问题主要影响产品感、信任感和实现边界：用户会看到底层 transport 排障文本，误以为系统处于故障态。
- 因为本轮没有阻断 scheduler 主功能链路，也没有造成错误投递或数据破坏，所以按规则定级为质量性 `P3`。

## 根因判断

- 共享错误净化曾覆盖失败分支中的 transport trace，但本轮说明成功 final 中的前缀污染仍可能绕过或晚于净化边界。
- 与 `codex_acp_transport_disconnect_request_failure.md` 不同：本轮没有最终失败或缺失 final，而是成功回复夹带 raw transport trace。
- 与归档的 `channel_raw_llm_error_exposure.md` 同属“底层传输错误痕迹进入用户态文本”大类，但本轮没有外泄 URL / cf-ray，严重等级低于历史 P1 样本。

## 修复记录

- 2026-06-21 19:09 CST 修复：
  - 共享 `sanitize_user_visible_output(...)` 新增 runner warning 句族剥离，覆盖成功 final 中夹带的 `Falling back from WebSockets to HTTPS transport`、`stream disconnected before completion`、`tls handshake eof` 等 transport trace。
  - 回归样本确认 warning 被剥离后仍保留后续业务正文，不把成功报告误判为空输出。
  - 验证：`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。

## 下一步建议

- 扩展 `sanitize_user_visible_output(...)` 或 scheduler 出站 finalizer，剥离成功回复开头/中段的 `Falling back from WebSockets`、`stream disconnected before completion`、`tls handshake eof` 等 runner transport trace。
- 增加回归：当最终回复主体可用但前缀包含 Codex transport fallback 文案时，用户可见文本应保留业务正文并删除内部错误痕迹。
- 后续巡检若只在 ACP event payload 中看到该文本、但 final 未外露，不应计为复发；只有用户可见 final 命中才补证。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码，未运行代码测试。
- 已验证范围：`data/sessions.sqlite3` 上界、ACP session / prompt / `stopReason` 统计、用户可见 final 关键词扫描、最近四小时非文档代码提交检查。
