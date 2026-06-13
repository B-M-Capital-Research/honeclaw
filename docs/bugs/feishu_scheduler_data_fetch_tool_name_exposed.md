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
- `data/sessions.sqlite3` -> `session_messages`
  - 2026-06-10 23:02 CST 巡检窗口：2026-06-10 19:01-23:02 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 21:35 CST 收到 `[定时任务触发] 任务名称：科技核心股池 · 晚间击球区快报`，assistant `ordinal=318` 于 21:37:52 CST 正常落库 final。
  - final 开头写出：`本轮使用 StockAnalysis 最新可见美股价格... data_fetch 当前不可用，已用可靠网页源补充校验`。
  - 同 session 在 23:00 CST 收到 `[定时任务触发] 任务名称：核心观察股池晚间快报`，assistant `ordinal=320` 于 23:02:04 CST 正常落库 final。
  - final 开头写出：`本轮 23:00 刷新未能取得新的 data_fetch / 网页行情返回；以下沿用本会话 21:35 已校验的 StockAnalysis 最新可见美股价格...`。
- 同窗摘要：
  - 2026-06-10 19:01-23:02 CST `data/sessions.sqlite3` 有 53 个 user turn 与 54 个 assistant 记录，最近 Feishu direct / scheduler 会话均以 assistant final 收口。
  - 普通 Feishu scheduler 33 条均 `completed + sent + delivered=1`，最近四小时无非文档代码提交。
  - 本轮两个样本都正常完成观察池快报，没有投递失败、空回复、错投或数据破坏证据；复发仍集中在内部工具名 / 站点名进入用户可见降级说明。
- `data/sessions.sqlite3` -> `session_messages`
  - 2026-06-11 11:01 CST 巡检窗口：2026-06-11 07:01-11:01 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 收到 `核心观察池早间简报` 定时触发，assistant `ordinal=322` 于 09:04:12 CST 正常落库 final。
  - final 开头写出：`可用行情接口未返回有效结果，已用 StockAnalysis 页面补充校验；击球区沿用本地固定区间。`
  - 本样本没有继续出现字面量 `data_fetch`，但仍把内部/实现侧行情接口失败和 `StockAnalysis` 站点名作为用户态降级说明发出，说明已有净化没有覆盖同一链路的站点名 / 内部数据链路同义口径。
- 同窗摘要：
  - 2026-06-11 07:01-11:01 CST `data/sessions.sqlite3` 有 19 个 user turn 与 20 个 assistant 记录，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant final 收口。
  - 普通 scheduler 17 条为 `completed + sent + delivered=1`，本条 Feishu scheduler 正常完成观察池早间简报，没有投递失败、空回复、错投或数据破坏证据。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误、quota、panic、`company_profiles/...`、技能状态或 cron 内部存储口径；本轮问题仍集中在内部行情链路 / 站点名进入用户可见降级说明。
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-11 23:03 CST 巡检窗口：2026-06-11 19:02-23:02 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 21:35 CST 收到 `科技核心股池 · 晚间击球区快报` 定时触发，assistant `ordinal=324` 于 21:37:19 CST 正常落库 final；对应 `cron_job_runs.run_id=40508` 为 `completed + sent + delivered=1`。
  - final 开头写出：`可用行情接口未返回有效结果，以下用网页源补充校验；击球区沿用本地固定区间。`
  - 同 session 在 23:00 CST 收到 `核心观察股池晚间快报` 定时触发，assistant `ordinal=326` 于 23:02:19 CST 正常落库 final；对应 `cron_job_runs.run_id=40548` 为 `completed + sent + delivered=1`。
  - final 开头写出：`专用行情工具本轮未返回有效结果；价格与财报日期用网页源补充校验，击球区沿用本地固定区间。`
  - 两个样本都没有字面量 `data_fetch`，但仍把内部行情工具 / 专用行情工具失败作为用户态解释输出；由于本单 20:12 CST 已有代码层修复记录，本轮按 live / 未确认部署运行态补证记录，不直接回退状态。
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-12 11:01 CST 巡检窗口：2026-06-12 07:01-11:01 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 收到 `核心观察池早间简报` 定时触发，assistant `ordinal=328` 于 09:02:43 CST 正常落库 final；对应 `cron_job_runs.run_id=40830` 为 `completed + sent + delivered=1`。
  - final 开头写出：`data_fetch 未返回有效结果，价格与财报日期用 StockAnalysis 页面补充校验；击球区沿用本地固定区间。`
  - 该样本晚于 2026-06-11 20:12 CST 语义扩展修复记录，且重新出现 `data_fetch` 与 `StockAnalysis` 字面量，说明同一 Feishu scheduler 降级说明仍会进入用户可见回复。
- 同窗摘要：
  - 2026-06-12 07:01-11:01 CST `data/sessions.sqlite3` 有 19 个 user turn 与 20 个 assistant 记录，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口，没有 user-only 残留。
  - 普通 scheduler 17 条为 `completed + sent + delivered=1`，本条 Feishu scheduler 正常完成观察池早间简报；没有旧价格成功态、投递失败、空回复、错投或数据破坏证据。
  - assistant final 污染扫描命中本单降级句族；本单仍是用户可见文案边界问题。
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-13 11:03 CST 巡检窗口：2026-06-13 07:01-11:03 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 收到 `核心观察池早间简报` 定时触发，assistant `ordinal=334` 于 09:03:29 CST 正常落库 final；对应 `cron_job_runs.run_id=41500` 为 `completed + sent + delivered=1`。
  - final 开头写出：`本轮使用 data_fetch quote 校验，价格口径为 2026-06-12 美股收盘附近最新可得行情`。
  - final 尾部再次写出：`本轮 25 支价格和下一次财报日期均由 data_fetch quote 返回`。
- 同窗摘要：
  - 2026-06-13 07:01-11:03 CST `data/sessions.sqlite3` 有 14 个 user turn 与 14 个 assistant turn，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口，没有 user-only 残留。
  - 普通 scheduler 11 条为 `completed + sent + delivered=1`，本条 Feishu scheduler 正常完成观察池早间简报；Discord scheduler 本轮也恢复为 `completed + sent + delivered=1`。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误、quota、panic、`company_profiles/...`、cron 内部存储口径或 SQLite 口径；本轮问题继续集中在内部行情工具名进入用户可见 scheduler final。

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
- 2026-06-10 21:35 / 23:00 CST 复发样本继续使用 `data_fetch 当前不可用`、`未能取得新的 data_fetch / 网页行情返回` 与 `StockAnalysis 最新可见美股价格` 等措辞，说明 sanitizer / prompt guard 仍未覆盖同义降级句族。
- 2026-06-11 09:04 CST 复发样本虽然没有字面量 `data_fetch`，但仍写出“可用行情接口未返回有效结果，已用 StockAnalysis 页面补充校验”，把内部数据链路失败和站点名继续作为用户态说明暴露。
- 2026-06-11 21:35 / 23:00 CST live 样本继续写出“可用行情接口未返回有效结果”和“专用行情工具本轮未返回有效结果”；该样本晚于代码层修复记录，但本轮未确认 live 已部署 / 重启到该修复，因此只作为运行态观察证据。
- 该样本不同于旧的 `Feishu 晨报在 data_fetch 连续失败后仍以成功态发送旧价格早报`：本轮没有看到旧价格被当作实时价送达，且使用 StockAnalysis 明确补充校验；主要问题是内部工具名外露。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 用户仍收到完整观察池快报，没有投递失败、空回复、错投、会话状态错乱或数据破坏证据。
- 影响集中在产品感和信任口径：用户看到内部工具名后，会把正常降级说明理解成调试过程或系统异常。
- 因此它不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 初步判断是 scheduler final guidance 或共享用户可见输出净化层没有覆盖自然语言形式的 `data_fetch` 降级说明。
- 2026-06-10 03:27 修复只覆盖了 `data_fetch 本轮未返回可用结果，已用 StockAnalysis 补充校验` 这一精确或窄形态，未覆盖 `data_fetch 当前未返回可用行情，已用 StockAnalysis 实时页补充校验价格与页面显示财报日期` 等同义变体。
- 2026-06-11 新样本进一步说明，修复还未覆盖不含 `data_fetch` 字面量、但表达为“行情接口未返回有效结果 + StockAnalysis 页面补充校验”的同链路降级句族。
- 2026-06-11 23:03 CST live 样本进一步把句族扩展到“专用行情工具未返回有效结果 + 网页源补充校验”；若确认部署 20:12 CST 修复后仍复现，再评估是否从 `Fixed` 回退。
- 2026-06-12 11:01 CST 样本晚于 2026-06-11 20:12 CST 语义扩展修复记录，且恢复为 `data_fetch` + `StockAnalysis` 字面量外露。当前证据来自真实 scheduler final 与 cron 台账，因此把状态从 `Fixed` 调回 `New`；仍按质量性 `P3` 处理。
- 2026-06-13 11:03 CST 样本继续晚于 2026-06-11 20:12 CST 语义扩展修复记录，且不再是“失败降级说明”单一形态，而是“本轮使用 / 均由 data_fetch quote 校验”的成功口径直接进入 final；说明现有净化或 prompt guard 没有覆盖“工具名作为来源背书”的句型。状态保持 `New`，仍按质量性 `P3` 处理。
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
- 2026-06-10 23:02 CST 补充同根复发证据：
  - 21:35 CST `科技核心股池 · 晚间击球区快报` final 外露 `data_fetch 当前不可用，已用可靠网页源补充校验`。
  - 23:00 CST `核心观察股池晚间快报` final 外露 `未能取得新的 data_fetch / 网页行情返回`，并继续提 `StockAnalysis 最新可见美股价格`。
  - 两轮都正常送达并输出观察池列表，因此仍不影响主功能链路，保持质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-06-11 11:01 CST 补充同根复发证据：
  - 09:04 CST `核心观察池早间简报` final 写出 `可用行情接口未返回有效结果，已用 StockAnalysis 页面补充校验；击球区沿用本地固定区间`。
  - 任务仍正常送达并输出核心股 / 拓展股列表、击球区与财报日期；没有旧价格成功态、投递失败或功能阻断证据，因此仍不影响主功能链路，保持质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-06-11 23:03 CST 补充 live / 未确认部署运行态证据：
  - 21:35 CST `科技核心股池 · 晚间击球区快报` final 写出 `可用行情接口未返回有效结果，以下用网页源补充校验；击球区沿用本地固定区间`。
  - 23:00 CST `核心观察股池晚间快报` final 写出 `专用行情工具本轮未返回有效结果；价格与财报日期用网页源补充校验，击球区沿用本地固定区间`。
  - 两轮均为 `completed + sent + delivered=1`，观察池列表、价格口径和击球区正常输出；没有旧价格成功态、投递失败或功能阻断证据。
  - 因本单 20:12 CST 已有代码层修复和回归记录，且本轮未确认 live 已部署 / 重启到该修复，状态保持 `P3 / Fixed`；若部署后继续复现，再回退为 `New`。非 P1，不创建 GitHub Issue。
- 2026-06-12 11:01 CST 重新打开：
  - 09:02 CST `核心观察池早间简报` final 写出 `data_fetch 未返回有效结果，价格与财报日期用 StockAnalysis 页面补充校验；击球区沿用本地固定区间`。
  - 对应 `cron_job_runs.run_id=40830` 为 `completed + sent + delivered=1`，观察池列表、价格口径和击球区正常输出；没有旧价格成功态、投递失败或功能阻断证据。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3`；非 P1，不创建 GitHub Issue。
- 2026-06-12 23:02 CST 补充同根复发证据：
  - 23:00 CST `核心观察股池晚间快报` final 写出 `本轮专用 data_fetch 未返回可调用结果；价格改用 StockAnalysis 页面校验`。
  - 对应 `cron_job_runs.run_id=41224` 为 `completed + sent + delivered=1`，观察池列表、价格口径、击球区和财报日期仍正常输出；没有投递失败、空回复、错投或数据破坏证据。
  - 本轮 19:02-23:02 CST `data/sessions.sqlite3` 有 42 个 user turn 与 42 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 34 条均为 `completed + sent + delivered=1`。
  - 该样本晚于 2026-06-11 20:12 CST 语义扩展修复记录，且重新出现 `data_fetch` 与 `StockAnalysis` 字面量外露；状态保持 `P3 / New`。因为不阻断 scheduler 主功能链路，非 P1，不创建 GitHub Issue。
- 2026-06-13 11:03 CST 补充同根复发证据：
  - 09:03 CST `核心观察池早间简报` final 开头写出 `本轮使用 data_fetch quote 校验`，尾部再次写出 `25 支价格和下一次财报日期均由 data_fetch quote 返回`。
  - 对应 `cron_job_runs.run_id=41500` 为 `completed + sent + delivered=1`，观察池列表、击球区距离和财报日期正常输出；没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 本轮 07:01-11:03 CST `data/sessions.sqlite3` 有 14 个 user turn 与 14 个 assistant turn，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口；普通 scheduler 11 条均为 `completed + sent + delivered=1`。
  - 该样本把 `data_fetch quote` 从失败降级句变成来源背书句，但用户可见内部工具名外露的根因和影响范围相同；状态保持 `P3 / New`。因为不阻断 scheduler 主功能链路，非 P1，不创建 GitHub Issue。

## 修复记录

- 2026-06-10 03:27 CST 修复：
  - 共享 `sanitize_user_visible_output(...)` 新增内部行情工具降级口径改写：`data_fetch 本轮未返回可用结果，已用 StockAnalysis 补充校验` 会统一改成“主行情源本轮未返回可用结果，已改用公开页面补充校验”。
  - 新增 `sanitize_user_visible_output_rewrites_market_data_tool_fallback_copy` 回归，锁定内部工具名和站点名不再进入 scheduler 用户态 final。
  - 本轮只修用户可见文案边界，不涉及 scheduler 执行流、行情 fallback 策略或旧价格成功态判定。

- 2026-06-11 20:12 CST 语义扩展修复并关闭：
  - 共享 `sanitize_user_visible_output(...)` 将 `data_fetch` / `StockAnalysis` 降级句族扩展到 `data_fetch 当前不可用`、`data_fetch 当前未返回可用行情`、`未能取得新的 data_fetch / 网页行情返回`、`可用行情接口未返回有效结果，已用 StockAnalysis 页面补充校验` 等同义形态。
  - Feishu scheduler 出站文本复用该净化链路，用户可见降级说明统一为“主行情源本轮未返回可用结果，已改用公开页面补充校验”，不再暴露内部工具名或具体站点名。
  - 新增 / 扩展 runtime 与 scheduler 回归，覆盖 2026-06-10 09:03、21:35、23:00 与 2026-06-11 09:04 CST 复发句族。
  - 无关联 GitHub Issue；本轮未依赖生产日志、线上渠道状态或本机 live 服务复核。

## 验证

- `cargo test -p hone-channels sanitize_user_visible_output_rewrites_market_data_tool_fallback_copy --lib -- --nocapture`
- `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
- `cargo test -p hone-channels scheduler_delivery_text_rewrites_data_fetch_degradation_copy --lib -- --nocapture`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/runtime.rs crates/hone-channels/src/scheduler.rs`
- `cargo check -p hone-channels --tests`
- `git diff --check`
