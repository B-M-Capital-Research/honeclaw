# Bug: Feishu scheduler 降级说明外露 `data_fetch` 内部工具名

## 发现时间

2026-06-09 23:04 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=39170`
  - `job_name=核心观察股池晚间快报`
  - `actor_channel=feishu`
  - `executed_at=2026-06-09T23:02:02.969971+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 开头写出：`data_fetch 本轮未返回可用结果，已用 StockAnalysis 补充校验`
- `data/sessions.sqlite3` -> `session_messages`
  - 时间窗：2026-06-09 23:00-23:02 CST
  - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
  - assistant `ordinal=314`
  - `timestamp=2026-06-09T23:01:59.827773+08:00`
  - assistant final 已输出核心股与拓展股价格、击球区、财报日期和来源，并正常收口；正文开头同样包含 `data_fetch 本轮未返回可用结果`
- 本轮巡检汇总：
  - 2026-06-09 19:03-23:04 CST `data/sessions.sqlite3` 有 97 个 user turn 与 99 个 assistant 记录；最近活跃 Feishu direct / scheduler session 均以 assistant final 收口。
  - 普通 Feishu scheduler 34 条均为 `completed + sent + delivered=1`。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误、quota、panic 或 stream disconnect。
- `data/sessions.sqlite3` -> `session_messages`
  - 2026-06-10 11:03 CST 巡检窗口：2026-06-10 07:03-11:03 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 收到 `核心观察池早间简报` 定时触发，assistant `ordinal=316` 于 09:03:28 CST 正常落库 final。
  - final 开头写出：`data_fetch 当前未返回可用行情，已用 StockAnalysis 实时页补充校验价格与页面显示财报日期`。
  - 该样本晚于 2026-06-10 03:27 CST 共享 sanitizer 修复确认，且措辞从旧样本的 `本轮未返回可用结果` 变成 `当前未返回可用行情`，说明现有净化规则没有覆盖同义变体。
- 同窗摘要：
  - 最近四小时共有 25 个 user turn 与 26 个 assistant 记录，普通 scheduler 16 条 `completed + sent + delivered=1`。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、`company_profiles/...`、raw tool 字段、思维痕迹、provider 原始错误、quota、panic 或 `enabled=true/false`；本轮问题集中在内部行情工具名 / 站点名进入用户可见降级说明。

## 端到端链路

1. Feishu scheduler 触发 `核心观察股池晚间快报`。
2. runner 尝试使用行情 / 数据工具获取观察池价格与财报信息。
3. 某个内部数据链路未返回可用结果，runner 改用 StockAnalysis 页面完成补充校验。
4. 最终回复正常送达，但把内部工具名 `data_fetch` 作为用户可见降级说明发出。

## 期望效果

- 用户可见文本可以说明“本轮主行情源未返回可用结果，已改用公开页面交叉校验”。
- 不应暴露 `data_fetch` 这类内部工具名、工具编排或执行状态。
- 降级说明应保留数据口径与可信度边界，但使用产品化业务语言。

## 当前实现效果

- 任务按时完成并送达，核心股 / 拓展股列表、击球区、价格口径和来源均可读。
- 但 final 开头直接写出 `data_fetch 本轮未返回可用结果`，把内部工具名当作业务说明暴露给 Feishu 用户。
- 2026-06-10 09:03 CST 复发样本改写为 `data_fetch 当前未返回可用行情，已用 StockAnalysis 实时页补充校验...`，仍把内部工具名和站点名作为用户态说明暴露。
- 该样本不同于旧的 `Feishu 晨报在 data_fetch 连续失败后仍以成功态发送旧价格早报`：本轮没有看到旧价格被当作实时价送达，且使用 StockAnalysis 明确补充校验；主要问题是内部工具名外露。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 用户仍收到完整观察池快报，没有投递失败、空回复、错投、会话状态错乱或数据破坏证据。
- 影响集中在产品感和信任口径：用户看到内部工具名后，会把正常降级说明理解成调试过程或系统异常。
- 因此它不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 初步判断是 scheduler final guidance 或共享用户可见输出净化层没有覆盖自然语言形式的 `data_fetch` 降级说明。
- 2026-06-10 03:27 修复只覆盖了 `data_fetch 本轮未返回可用结果，已用 StockAnalysis 补充校验` 这一精确或窄形态，未覆盖 `data_fetch 当前未返回可用行情，已用 StockAnalysis 实时页补充校验价格与页面显示财报日期` 等同义变体。
- 现有 `web_direct_internal_skill_and_local_store_terms_exposed.md` 覆盖 Web direct 的 `skill` / `data/portfolio` / 本地 json 口径；本轮是 Feishu 普通 scheduler 的行情工具降级说明，链路和触发位置不同。
- 现有 `feishu_scheduler_stale_price_fallback_after_data_fetch_failure.md` 覆盖关键行情失败后旧价格 fallback 被记成功；本轮证据不足以判断旧价成功态复发，只确认内部工具名外露。

## 下一步建议

- 扩展共享出站净化或 scheduler prompt guard，将 `data_fetch 本轮未返回可用结果` 等内部工具名口径改写为“主行情源本轮未返回可用结果”。
- 对 Feishu scheduler final 增加回归样本：当内部行情工具失败但有公开来源补充校验时，用户可见文本不得出现 `data_fetch`、tool 名称或内部执行状态。
- 后续巡检继续区分两类证据：若同时复用旧价格并记成功，应回看 stale-price fallback 缺陷；若只是工具名进入最终回复，则按本单跟踪。
- 扩展 sanitizer / prompt guard 时应按语义覆盖 `data_fetch` + `StockAnalysis` 降级句族，而不是只匹配单个固定句。

## 复发记录

- 2026-06-10 11:03 CST 状态从 `Fixed` 回退为 `New`：
  - 09:03 CST `核心观察池早间简报` final 再次外露 `data_fetch 当前未返回可用行情，已用 StockAnalysis 实时页补充校验价格与页面显示财报日期`。
  - 该任务仍正常输出核心股 / 拓展股价格、击球区与财报日期，也没有投递失败、空回复、错投或数据破坏证据。
  - 因为问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍按质量性 `P3` 处理；非 P1，不创建 GitHub Issue。

## 修复记录

- 2026-06-10 03:27 CST 修复：
  - 共享 `sanitize_user_visible_output(...)` 新增内部行情工具降级口径改写：`data_fetch 本轮未返回可用结果，已用 StockAnalysis 补充校验` 会统一改成“主行情源本轮未返回可用结果，已改用公开页面补充校验”。
  - 新增 `sanitize_user_visible_output_rewrites_market_data_tool_fallback_copy` 回归，锁定内部工具名和站点名不再进入 scheduler 用户态 final。
  - 本轮只修用户可见文案边界，不涉及 scheduler 执行流、行情 fallback 策略或旧价格成功态判定。

## 验证

- `cargo test -p hone-channels sanitize_user_visible_output_rewrites_market_data_tool_fallback_copy --lib -- --nocapture`
- `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`
- `git diff --check`
