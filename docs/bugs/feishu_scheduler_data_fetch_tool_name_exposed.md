# Bug: Feishu scheduler 外露内部工具 / 画像流程与 `data_fetch` 名称

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

## 最新进展

- 2026-07-03 23:02 CST 运行态继续复发，状态维持 `New`：
  - 19:02-23:02 CST `data/sessions.sqlite3` 有 3 条新的 assistant final，均正常收口；assistant final 污染扫描未命中空回复、本机绝对路径、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、panic、quota 或资源耗尽原文。
  - 21:35 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `科技核心股池 · 晚间击球区快报` final 开头写出 `本轮 data_fetch 已返回 25 支标的 quote_short 行情`，数据说明继续写 `价格来自本轮 data_fetch quote_short`。
  - 23:00 CST 同 actor 的 `核心观察股池晚间快报` final 再次写出同样 `data_fetch quote_short` 用户态工具口径。
  - 两个样本报告主体均正常收口，没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-03 03:02 CST 运行态继续复发，状态维持 `New`：
  - 23:02-03:03 CST `data/sessions.sqlite3` 有 1 条新的 Feishu direct assistant final，正常收口；assistant final 污染扫描未命中空回复、本机绝对路径、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、panic、quota 或资源耗尽原文。
  - 02:47 CST Feishu direct session `Actor_feishu__direct__ou_5f95ab3697246ded86446fcc260e27e1e2` 的 `cohr估值分析` final 完成估值分析，但来源段继续出现 `https://公开行情页.com/stocks/cohr/statistics/`、`https://公开行情页.com/stocks/cohr/forecast/` 这类净化占位域名，把内部来源改写残留暴露给用户。
  - 同条 final 还给出 COHR 精确盘中价、估值倍数和财务预期，但未见空回复、错投、投递失败或原始工具 JSON；问题仍只影响用户可见来源 / 工具口径边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-02 23:03 CST 运行态继续复发，状态维持 `New`：
  - 19:02-23:02 CST `data/sessions.sqlite3` 有 5 条新的 assistant final，均正常收口；assistant final 污染扫描未命中空回复、本机绝对路径、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、panic、资源耗尽或 binary-not-found 原文。
  - 21:35 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `科技核心股池 · 晚间击球区快报` final 开头写出 `本轮 data_fetch 行情时间戳约为 21:36`，数据说明继续写 `价格与财报日期来自本轮 data_fetch quote 返回`。
  - 23:00 CST 同 actor 的 `核心观察股池晚间快报` final 开头写出 `本轮 data_fetch 已返回 25 支标的最新行情`，并把 `data_fetch quote_short` 作为用户态来源说明。
  - 两个样本报告主体均正常收口，没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-01 23:02 CST 运行态继续复发，状态维持 `New`：
  - 19:06-23:02 CST `data/sessions.sqlite3` 有 4 条新的 assistant final，均正常收口；assistant final 污染扫描未命中空回复、本机绝对路径、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、panic、资源耗尽或 binary-not-found 原文。
  - 21:35 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `科技核心股池 · 晚间击球区快报` final 开头写出 `本轮 data_fetch 已返回 25 支标的最新行情`。
  - 23:00 CST 同 actor 的 `核心观察股池晚间快报` final 再次写出 `本轮 data_fetch 已返回 25 支标的最新行情`，并把 `data_fetch quote_short` 作为用户态来源说明。
  - 两个样本报告主体均正常收口，没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-01 03:01 CST 运行态继续复发，状态维持 `New`：
  - 23:00-03:01 CST `data/sessions.sqlite3` 已有最近 Feishu direct 会话镜像，`session_messages.max(timestamp)=2026-07-01T00:40:49.460547+08:00`、`session_messages.max(imported_at)=2026-07-01T00:40:49.568582+08:00`。
  - 本窗 SQLite 有 4 个 Feishu direct user turn 与 4 个 assistant final，均正常收口；assistant final 污染扫描未命中空回复、本机绝对路径、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、panic、资源耗尽或 binary-not-found 原文。
  - `data/runtime/logs/acp-events.log` 同窗可见 14 次 `session/prompt`、14 次 `session/set_model`、14 次 `stopReason=end_turn`、0 个 response error。
  - 用户可见 chunk 重构继续命中多类内部执行口径：23:00 CST 观察池快报仍写 `本轮 data_fetch 已返回 25 支标的最新行情`；23:20 CST CRCL 分析写 `我已经加载单股研究流程`；23:23 CST HPE 分析写 `本地没有 HPE 画像`、`把长期判断沉淀下来`；00:38 CST OUST 分析来源段把净化后的占位标签拼成 `https://公开行情页.com/stocks/oust/` 等不可用占位域名。
  - 这些样本都正常输出业务主体并收口，没有投递失败、空回复、错投或链路级数据破坏证据；问题仍只影响用户可见文案边界和产品感，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-30 23:01 CST 运行态继续复发，状态维持 `New`：
  - 19:02-23:01 CST `data/sessions.sqlite3` 已恢复最近会话镜像，`session_messages.max(timestamp)=2026-06-30T23:01:28.103232+08:00`、`session_messages.max(imported_at)=2026-06-30T23:01:28.304784+08:00`。
  - 本窗 SQLite 有 4 个 user turn 与 4 个 assistant final，均正常收口；assistant final 污染扫描未命中空回复、`company_profiles/`、本机绝对路径、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、panic、资源耗尽或 binary-not-found 原文。
  - 23:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 `核心观察股池晚间快报` final 开头写出 `本轮 data_fetch 已返回 25 支标的最新行情`，把内部工具名作为用户态来源说明。
  - `data/runtime/logs/acp-events.log` 同窗可见 56 次 `session/prompt`、56 次 `stopReason=end_turn` 对应 response 无错误；用户可见 chunk 污染扫描共命中 3 个 `data_fetch` 片段。
  - 这些样本没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-30 07:03 CST 运行态继续复发，状态维持 `New`：
  - 03:00-07:03 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 chunk。
  - 本窗 ACP 可见 11 次 `session/prompt`、11 次 `stopReason=end_turn`、0 个未收口会话；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 用户可见污染扫描命中 2 条 `data_fetch` 名称片段：Feishu direct session `Actor_feishu__direct__ou_5f895bed1573d53053e89bfc382b523a44` 的 `data_fetch 实时`，以及 Web direct session `Actor_web__direct__web-user-14f4cadb069f` 的 `data_fetch quote`。
  - 这些样本没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-29 23:01 CST 运行态继续复发，状态维持 `New`：
  - 19:01-23:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 chunk。
  - 本窗 ACP 可见 43 次 `session/prompt`、41 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 用户可见污染扫描仅命中 Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a` 的 `data_fetch 行` 片段，继续把内部工具名作为用户态来源说明。
  - 该样本没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-29 11:01 CST 运行态继续复发，状态维持 `New`：
  - 07:00-11:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 chunk。
  - 本窗 ACP 可见 38 次 `session/prompt`、36 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 用户可见污染扫描命中 Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a` 的 `hone data_fetch 行情与新闻` 片段，以及 Web direct session `Actor_web__direct__web-user-c0ce2faf7c49` 的 `StockAnalysis SND...` / `StockAnalysis` 片段，继续把内部工具名或站点名作为用户态来源说明。
  - 这些样本没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-27 03:01 CST 运行态继续复发，状态维持 `New`：
  - 23:03-03:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 chunk。
  - 本窗 ACP 可见 9 个 prompt session、16 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 用户可见污染扫描仅命中 Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5f895bed1573d53053e89bfc382b523a44` 的 `Hone data_fetch 实时` 片段，继续把内部工具名作为用户态数据来源说明。
  - 该样本没有链路级失败证据；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-25 11:01 CST 运行态继续复发，状态维持 `New`：
  - 07:04-11:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 36 次 `session/prompt`、36 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 08:30 CST Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34` 的 SNDK / 存储链复盘正常收口，但正文继续写出 `StockAnalysis` 作为行情 / 评级口径。
  - 09:00 CST session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 25 支观察池节奏管理正常收口，但正文写出 `data_fetch 返回的是 2026-06-24 美股常规盘收盘附近 quote`，并继续把工具字段当作用户态数据来源说明。
  - 两个样本报告主体均正常收口，未见投递失败、空回复或链路级数据破坏证据。问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-25 03:04 CST 运行态继续复发，状态维持 `New`：
  - 23:02-03:04 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 8 个 session、15 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 23:01 CST Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的观察池节奏管理完成价格、击球区和财报字段概览，但正文写出 `data_fetch 返回的美股常规盘盘中 quote`。
  - 00:02 CST session `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 的 TEM / AAOI / RKLB 监控汇总正常收口，但继续写出 `StockAnalysis` 作为行情口径。
  - 00:12 CST session `Actor_feishu__direct__ou_5f03129a3b3ef1e1f93caa23989cdb5554` 的 NBIS 分析正常收口，但来源段继续写出 `StockAnalysis NBIS 行情页`。
  - 三个样本报告主体均正常收口，未见投递失败、空回复或链路级数据破坏证据。问题仍只影响用户可见文案边界和产品感，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-24 23:02 CST 运行态继续复发，状态维持 `New`：
  - 19:00-23:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 30 个 session、42 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 20:48 CST Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7` 的 A股 / AI 硬件复盘完成事实、映射和动作建议，但正文继续写出 `StockAnalysis` 作为行情口径。
  - 21:35 CST session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的 25 支观察池晚间快报完成分组简表，但开头写出“用行情工具校验 25 支观察池的最新价格和财报字段”。
  - 21:45 CST session `Actor_feishu__direct__ou_5fea712445d905e8418bde07dbcf2cbfb2` 的美股早盘口径复盘正常收口，但正文写出“再尝试用行情工具补 QQQ、SPY、VIX 的同源报价”。
  - 三个样本报告主体均正常收口，未见投递失败、空回复或链路级数据破坏证据。问题仍只影响用户可见文案边界和产品感，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-23 23:02 CST 运行态复发，状态从 `Fixed` 回退为 `New`：
  - 19:02-23:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`session_messages.max(imported_at)=2026-06-17T10:37:41.827657+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 29 个 session、49 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、绝对路径、token、provider 原始错误或思维痕迹进入 final。
  - 20:46 CST Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7` 的 A股 / AI 硬件复盘完成趋势、事实、映射和动作建议，但正文继续写出 `StockAnalysis` 作为行情口径；21:32 CST session `Actor_feishu__direct__ou_5f895bed1573d53053e89bfc382b523a44` 的持仓纪律复核来源段继续写出 `StockAnalysis MU、LITE、BE、RKLB、TEM、YINN 页面`。
  - 两个样本晚于 2026-06-23 03:04 CST 共享净化修复记录；报告主体均正常收口，未见投递失败、空回复或链路级数据破坏证据。问题仍只影响用户可见文案边界和产品感，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-23 03:04 CST 代码级修复，状态更新为 `Fixed`：
  - 共享 `sanitize_user_visible_output(...)` 新增 `Hone 行情工具` / `Hone 行情口径` 改写为 `公开行情页`，并补齐“我先按北京时间 ... 对齐交易日”“我再核验 ...”这类执行过程句式收口。
  - 新增回归 `sanitize_user_visible_output_rewrites_hone_market_tool_copy`，并复用既有 `sanitize_user_visible_output_` 套件覆盖 scheduler 共享净化入口。
  - 验证：`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。
  - 本轮未重启当前服务；按当前代码与回归验证回写 `Fixed`，后续仅在确认加载当前代码的新运行态仍复现时重新打开。

- 2026-06-22 19:00 CST 运行态复发，状态从 `Fixed` 回退为 `New`：
  - 15:04-19:00 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 5 次 `session/prompt`、4 个 session、5 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径、transport trace、provider 原始错误或思维痕迹进入 final。
  - 17:30 CST Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8` 的 A/H 收盘后复盘正常收口，但开头写出“我先按北京时间 2026-06-22 17:30 对齐交易日，再核验...”等执行过程，正文继续写出 `Hone 行情口径`，来源段写出 `Hone 行情工具：000001.SS、^HSI...`。
  - 该报告主体完成 A股 / 港股收盘后复盘、AI 硬件、A/H 个股与美股盘前变量梳理；没有投递失败、空回复、错投或链路级数据破坏证据。
  - 该样本晚于 2026-06-22 07:08 CST scheduler / 共享净化修复记录；问题仍只影响用户可见文案边界和产品感，不影响主功能链路，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

## 修复记录

- 2026-06-22 07:08 CST 修复：
  - 共享 `sanitize_user_visible_output(...)` 补齐 TEM 样本里的漏网句式，新增覆盖 `本地长期画像`、`本轮没有新增事实改变 ... 长期画像`、`我先核验 ... 行情口径 / 本地长期画像`。
  - scheduler delivery 仍复用同一净化入口，因此 Feishu scheduler / direct actor final 会统一剥离内部画像读取 / 写入口径，同时保留业务主体结论。
  - 保留 2026-06-21 已有 `StockAnalysis` / `data_fetch` 用户态改写规则。
  - 验证：`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo test -p hone-channels scheduler_delivery_text_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。
  - 无关联 GitHub Issue；状态更新为 `Fixed`。

- 2026-06-22 11:03 CST 旧/未确认部署运行态观察，状态保持 `Fixed`：
  - 07:02-11:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 27 次 `session/prompt`、20 个 session、28 次 `stopReason=end_turn`、0 个 response error；未见空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径、transport trace、provider 原始错误或思维痕迹进入 final。
  - 08:30 CST Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34` 的 SNDK 简报正常收口，但用户可见 final 开头写出“我先核行情、新闻和本地 SNDK 画像”，并继续写出“我会把今天的结论沉淀到 SNDK 画像”等内部画像读取 / 写入动作。
  - 该报告主体完成 Juneteenth 休市口径、行情边界、风险与简报结论；没有投递失败、空回复、错投或功能阻断证据。
  - 当前机器 live 进程未确认已加载 2026-06-22 07:08 CST 的 scheduler / 共享净化修复，因此本轮只补充观察，不回退状态。若部署 / 重启到当前代码后仍复现，再改回 `New`；非 P1，不创建 GitHub Issue。

- 2026-06-22 03:02 CST 运行态复发，状态从代码级 `Fixed` 回退为 `New`：
  - 23:02-03:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 有 8 次 `session/prompt`、8 个 session、0 个 response error；未见空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径、transport trace、provider 原始错误或思维痕迹进入用户可见 chunk。
  - 00:00-00:03 CST Feishu direct / scheduler actor session `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 的 TEM 简报正常收口，但用户可见 chunk 开头写出“核验 TEM 的官方公告、最新行情口径和本地长期画像”，正文继续写出“本轮没有新增事实改变 TEM 长期画像”。
  - 同批样本仍命中既有 `StockAnalysis` / 画像流程文案边界；该样本晚于 2026-06-21 19:09 CST 共享净化层修复记录，因此按真实运行态回退。
  - 回复主体完成官方公告、行情、财务口径、动作和证伪条件；没有投递失败、空回复、错投或功能阻断证据。问题仍只影响用户可见文案边界和产品感，因此为质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-21 19:09 CST 修复：
  - 共享 `sanitize_user_visible_output(...)` 扩展用户可见净化规则，新增剥离自然语言画像读取 / 写入过程句式（如“已有的公司画像里...”“只更新本轮...”“追加到画像”“回写到长期画像”）和 runner warning 句族，并把 `StockAnalysis` 用户态执行口径统一改写为“公开行情页”。
  - `主行情工具返回额度错误` 等内部行情工具状态现在统一收口为“主行情源本轮未返回可用结果，已改用公开页面补充校验”，不再把内部工具名 / 额度状态当作报告正文。
  - 验证：`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。

- 2026-06-21 19:03 CST 补充同根复发证据，状态保持 `New`：
  - 15:03-19:03 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`cron_job_runs.max(executed_at)=2026-06-17T11:01:42.353141+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 2 个 session、2 次 `session/prompt`、2 次 `stopReason=end_turn`、0 个 ACP response error；未见空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径、transport trace 或思维痕迹进入 final。
  - 17:30 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8` 的 `A股港股收盘后复盘` 正常收口，但开头写出“我会先核实交易日与最近可用盘面”“接下来只补充可核验的最近交易日信号”“我再核验核心个股估值口径”等执行过程口径，尾部来源继续写出 `Hone行情工具`。
  - 该报告主体完成周日 / 端午 / Juneteenth 休市口径、QQQ/SMH/SOXX 等 ETF 与核心个股估值分层；没有投递失败、空回复、错投或链路级数据破坏证据。
  - 问题仍只影响 scheduler final 的用户可见文案边界和产品感，不影响主功能链路，因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-21 15:03 CST 补充同根复发证据，状态保持 `New`：
  - 11:03-15:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`cron_job_runs.max(executed_at)=2026-06-17T11:01:42.353141+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 2 个 session、3 次 `session/prompt`、3 次 `stopReason=end_turn`、0 个 ACP response error；未见空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径、transport trace 或思维痕迹进入 final。
  - 12:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 的 `每日公司资讯与分析总结` 正常收口，但开头写出 `已有的公司画像里长期框架没有变化`、`只更新本轮能核验到的新增事实` 和 `追加到画像` 等内部画像读取 / 写入流程。
  - 该报告主体完成周末休市口径、Nasdaq 100 调整、AI 基建 / 医疗 AI 分层、财报日期和分析师口径摘要；没有投递失败、空回复、错投或链路级数据破坏证据。
  - 问题仍只影响 scheduler final 的用户可见文案边界和产品感，不影响主功能链路，因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-21 11:03 CST 补充同根复发证据，状态保持 `New`：
  - 07:01-11:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 14 个 session、20 次 `session/prompt`、20 次 `stopReason=end_turn`、0 个 ACP response error；未见空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径、transport trace 或思维痕迹进入 final。
  - 09:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的核心观察池早间简报正常收口，但来源段继续写出 `StockAnalysis` 作为用户态行情来源 / 执行口径。
  - 该报告主体完成周末 / Juneteenth 休市价格口径、核心股与拓展股分组、击球区和数据待确认说明；没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据。
  - 问题仍只影响 scheduler final 的用户可见文案边界和产品感，不影响主功能链路，因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-21 03:02 CST 补充同根复发证据，状态保持 `New`：
  - 23:03-03:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 7 次 `session/prompt`、7 次 `stopReason=end_turn`、0 个 ACP response error；相关 scheduler / direct actor final 均正常收口。
  - 00:00-00:04 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 的 AAOI / TEM / RKLB 休市监控 final 均完成行情口径、催化筛选、风险和证伪条件，但继续写出 `StockAnalysis` 作为用户态来源 / 估值口径，并外露“长期画像 / 公司画像事件 / 不追加画像事件”等内部画像流程。
  - 三条报告主体均完成休市 / 行情口径和风险说明，没有空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径或链路级数据破坏证据；问题仍只影响用户可见文案边界和产品感，因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-20 23:03 CST 补充同根复发证据，状态保持 `New`：
  - 19:01-23:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 25 次 `session/prompt`、25 次 `stopReason=end_turn`、0 个 ACP response error。
  - 20:01 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f58ff884640e647a1792f618f45209251` 的期权墙 / 市场口径 final 正常收口，但开头写出 `本地没有可直接调用的 yfinance 包，我改用公开网页和行情接口做交叉核验`，把本机依赖与执行路径暴露给用户。
  - 23:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的核心观察池快报正常收口，但开头写出 `主行情工具返回额度错误，不能作为本轮价格源`，来源段继续写出 `StockAnalysis` 作为用户态校验口径。
  - 两条报告主体均完成休市 / 行情口径、观察或风险说明，没有空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径或链路级数据破坏证据；问题仍只影响用户可见文案边界和产品感，因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-20 15:03 CST 补充同根复发证据，状态保持 `New`：
  - 11:02-15:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 12:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 的 `每日公司资讯与分析总结` 以 `stopReason=end_turn` 收口，final 写出“把能复用的结论沉淀到本地公司画像”和“追加一条 2026-06-20 更新”等内部画像流程口径。
  - 报告主体仍完成公司资讯总结、指数纳入、目标价分歧、休市口径和风险边界说明，没有投递失败、空回复、错投或链路级数据破坏证据。问题仍是 scheduler final 文案边界，保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-20 11:02 CST 补充同根复发证据，状态保持 `New`：
  - 07:02-11:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 08:30 CST Web scheduler / direct actor session `Actor_web__direct__web-user-fe88bce3a53f` 的 AI 硬件晨报以 `stopReason=end_turn` 收口，final 正常完成 AMZN / INTC / DELL / TSM / AMAT / GLW 等高权重增量筛选，但仍写出 `主行情工具` 这类内部工具口径。
  - 09:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的核心观察池早间简报以 `stopReason=end_turn` 收口，final 写出 `已拿到 StockAnalysis 对 25 支标的的最新可得统一口径`，继续把站点名作为用户态来源 / 执行口径。
  - 两条报告均正常完成并收口，没有投递失败、空回复、错投或链路级数据破坏证据；问题仍是 scheduler final 文案边界，保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-19 15:01 CST 补充同根复发证据，状态保持 `New`：
  - 11:02-15:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 12:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 的 `每日公司资讯与分析总结` 以 `stopReason=end_turn` 收口，final 写出 `StockAnalysis/MarketBeat 等第三方用于评级和财报日期`、`把本轮结论同步到长期跟踪画像`、`画像里昨天已经沉淀了同一组公司的主线` 等内部数据源 / 画像流程口径。
  - 同条 final 还夹带 Codex transport fallback 原始痕迹；该 raw transport trace 另由 `feishu_scheduler_acp_transport_trace_exposed.md` 单独跟踪。本单只补充 scheduler 内部工具 / 画像流程口径外露证据。
  - 报告主体仍完整输出 TEM / CAI / NBIS / CRWV / NVDA / GOOGL / TSM 等观察结论，没有投递失败、空回复、错投或链路级数据破坏证据。问题仍是 scheduler final 文案边界，保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-19 11:02 CST 补充同根复发证据，状态保持 `New`：
  - 07:02-11:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 08:30 CST Web scheduler / direct actor session `Actor_web__direct__web-user-fe88bce3a53f` 的 AI 硬件晨报以 `stopReason=end_turn` 收口，final 虽完成 AMZN / INTC / DELL / TSM / AMAT / GLW 等主体，但仍写出 `主行情工具` 这类内部工具口径。
  - 09:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的核心观察池早间简报以 `stopReason=end_turn` 收口，final 写出 `主行情工具本轮未返回可用结果，已用 StockAnalysis 公开行情页校验`，把内部工具状态和站点名作为用户态降级说明。
  - 相关报告主体均完整输出并收口，没有投递失败、空回复、错投、原始 provider 报错或链路级数据破坏证据。问题仍是 scheduler final 外露内部工具 / 数据源执行状态，保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-19 07:04 CST 补充同根复发证据，状态保持 `New`：
  - 03:02-07:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 06:00 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f11da38ad70c47cf87c0b106b6408b190` 的美股盘后复盘以 `stopReason=end_turn` 收口，但 final 前段写出 `已加载市场分析技能` 和“把检索词改写为绝对日期口径”等内部流程动作。
  - 该 final 正常输出指数、VIX、利率、美元、板块、AI / 半导体与宏观事件归因；没有投递失败、空回复、错投、原始 provider 报错或链路级数据破坏证据。问题仍是 scheduler final 外露内部流程状态，保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-18 23:03 CST 补充同根复发证据，状态保持 `New`：
  - 19:03-23:03 CST `data/sessions.sqlite3` 仍未追平最近真实会话；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 23:01 CST Feishu scheduler / direct actor session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 的核心股池快报以 `stopReason=end_turn` 收口，但 final 前段写出 `主行情工具返回额度限制，无法完成 data_fetch 校验`。
  - 该 final 正常输出核心股与拓展股池、击球区、财报日期和数据口径边界；没有投递失败、空回复、错投、原始 provider 报错或链路级数据破坏证据。问题仍是 scheduler final 外露内部工具名 / 工具状态，保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-18 15:03 CST 补充同根复发证据，状态保持 `New`：
  - 11:03-15:03 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages` 与 `cron_job_runs` 在本窗均为 0；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 12:00 CST Feishu scheduler `每日公司资讯与分析总结`（`session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`）以 `stopReason=end_turn` 收口，final 前段写出 `本地长期画像目录当前没有可读内容`，并说明会为 AI 基建与医疗 AI 跟踪链补记录。
  - 该样本正常输出 TEM / CAI / NBIS / CRWV / NVDA / GOOGL / TSM 等观察结论、目标价 / 催化 / 财报节点和来源，没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 问题与 08:30 / 09:30 CST 样本同根，仍是 scheduler final 外露本地画像 / 内部流程动作；不影响主功能链路，保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-18 11:04 CST 补充同根复发证据，状态保持 `New`：
  - 07:01-11:04 CST 真实窗口中，`data/sessions.sqlite3` 仍未追平最近会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`cron_job_runs.max(executed_at)=2026-06-17T11:01:42.353141+08:00`；本轮继续以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 09:30 CST Feishu scheduler `美股收盘资金流复盘`（`session_id=Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768`）以 `stopReason=end_turn` 收口，final 开头写出 `已加载市场分析流程`，尾部来源写出 `Hone data_fetch：SPY、QQQ、11只 Select Sector SPDR ETF 收盘行情`。
  - 08:30 CST Feishu scheduler `闪迪(SNDK)每日行情与行业简报`（`session_id=Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34`）以 `stopReason=end_turn` 收口，final 开头写出 `现在补充本地 SNDK 画像`、`我现在更新 SNDK 长期画像和今日简报事件`，把内部长期画像读取 / 写入动作当作用户态正文。
  - 两个样本均正常输出业务报告、来源和风险结论，没有空回复、投递失败、错投、会话悬挂或链路级数据破坏证据；问题仍只影响用户可见文案边界和产品感，因此保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-18 07:02 CST 运行态复发，状态从代码级 `Fixed` 回退为 `New`：
  - 03:11 CST 非文档提交 `e10d7b2b Fix user-visible sanitizer regressions` 已落地，但 05:30 CST Feishu scheduler `美股收盘后跨市场复盘` 的用户可见 final 仍写出 `已加载市场复盘技能`、`行情与板块工具`、`Hone data_fetch` 等内部技能 / 工具口径。
  - 本轮证据来自 `data/runtime/logs/acp-events.log` 重构出的 ACP final；`data/sessions.sqlite3` 的 `session_messages` 仍停在 2026-06-17 10:37 CST，不能作为最近四小时真实会话唯一来源。
  - 该 scheduler final 正常完成市场复盘、A/H 映射和来源列表，没有投递失败、空回复、错投或数据破坏证据；问题仍只影响用户可见文案边界和产品感，因此保持质量性 `P3`，非 P1，不创建 GitHub Issue。

- 2026-06-18 03:04 CST 再次修复：
  - 共享 `sanitize_user_visible_output(...)` 现将“失败降级说明”和“成功校验背书”拆成两类规则处理：失败句型继续统一收口为 `主行情源本轮未返回可用结果，已改用公开页面补充校验`，成功句型则改写为不含内部工具名的“已完成校验”表达，不再把 `data_fetch quote` 成功样本误改成失败口径。
  - 新增回归 `sanitize_user_visible_output_rewrites_market_data_verified_copy`，并扩展既有 `sanitize_user_visible_output_rewrites_market_data_fallback_variants`，锁住 `data_fetch` / `StockAnalysis` / `quote` 句族的失败与成功两条路径。
  - 验证通过：`cargo test -p hone-channels sanitize_user_visible_output_rewrites_market_data_fallback_variants --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_rewrites_market_data_verified_copy --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests`。
  - 本轮未重启当前 Feishu 服务，也不把当前机器 live 运行态当作恢复证据；状态更新为代码级 `Fixed`，后续若部署后仍有新的内部行情工具名进入最终回复，再基于新样本重新打开。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 2026-06-21 19:03 CST 巡检窗口：2026-06-21 15:03-19:03 CST。
  - ACP 本窗可重构 2 个 session、2 次 `session/prompt`、2 次 `stopReason=end_turn`，0 个 ACP response error；本条 scheduler / direct actor final 正常收口。
  - 17:30 CST `Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8` final 写出交易日核实、最近可用盘面核验、核心个股估值核验等执行过程口径，并在来源中继续出现 `Hone行情工具`。
  - 该样本没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据；本单只记录内部工具 / 数据源 / 执行口径外露。
- `data/runtime/logs/acp-events.log`
  - 2026-06-21 15:03 CST 巡检窗口：2026-06-21 11:03-15:02 CST。
  - ACP 本窗可重构 2 个 session、3 次 `session/prompt`、3 次 `stopReason=end_turn`，0 个 ACP response error；本条 scheduler / direct actor final 正常收口。
  - 12:00 CST `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` final 写出本地公司画像长期框架、只更新新增事实和追加画像等内部画像流程动作。
  - 该样本没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据；本单只记录内部工具 / 数据源 / 画像流程口径外露。
- `data/runtime/logs/acp-events.log`
  - 2026-06-21 11:03 CST 巡检窗口：2026-06-21 07:01-11:02 CST。
  - ACP 本窗可重构 14 个 session、20 次 `session/prompt`、20 次 `stopReason=end_turn`，0 个 ACP response error；本条 scheduler / direct actor final 正常收口。
  - 09:00 CST `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` final 在“来源”段写出 `StockAnalysis`，作为用户态行情 / 财报日期校验口径。
  - 该样本没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据；本单只记录内部工具 / 数据源 / 执行口径外露。
- `data/runtime/logs/acp-events.log`
  - 2026-06-21 03:02 CST 巡检窗口：2026-06-20 23:03-2026-06-21 03:02 CST。
  - ACP 本窗可重构 7 次 `session/prompt`、7 次 `stopReason=end_turn`，0 个 ACP response error；本条 scheduler / direct actor final 均正常收口。
  - 00:00 CST `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 的 AAOI 监控 final 写出 `StockAnalysis` 作为行情 / 估值口径，并出现“长期画像 / 公司画像事件”类内部流程表达。
  - 00:01 CST 同 session 的 TEM 监控 final 写出本地画像核验和 `StockAnalysis` 估值 / 财报日期口径；00:02 CST RKLB 监控 final 继续写出 `StockAnalysis` 与“不追加 RKLB 长期画像事件”。
  - 三个样本没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据；本单只记录内部工具 / 数据源 / 画像流程口径外露。
- `data/runtime/logs/acp-events.log`
  - 2026-06-20 23:03 CST 巡检窗口：2026-06-20 19:01-23:01 CST。
  - ACP 本窗可重构 25 次 `session/prompt`、25 次 `stopReason=end_turn`（含 23:00 CST 边界 prompt 于 23:02:38 CST 收口）、0 个 ACP response error；本条 scheduler / direct actor final 均正常收口。
  - 20:01 CST `Actor_feishu__direct__ou_5f58ff884640e647a1792f618f45209251` final 写出本机 `yfinance` 包不可用和改用网页 / 行情接口的执行路径。
  - 23:00 CST `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` final 写出 `主行情工具返回额度错误`，并在来源段把 `StockAnalysis` 作为用户态校验口径。
  - 两个样本没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据；本单只记录内部工具 / 数据源 / 本机执行环境口径外露。
- `data/runtime/logs/acp-events.log`
  - 2026-06-20 15:03 CST 巡检窗口：2026-06-20 11:02-15:02 CST。
  - ACP 本窗可重构 9 次 `session/prompt`、8 次 `stopReason=end_turn`、0 个 ACP response error；本条 scheduler / direct actor final 正常收口。
  - 12:00 CST `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` final 写出本地公司画像沉淀与追加更新动作；报告主体完成公司资讯总结与可复用结论。
  - 该样本没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据；本单只记录内部工具 / 数据源 / 画像流程口径外露。
- `data/runtime/logs/acp-events.log`
  - 2026-06-20 11:02 CST 巡检窗口：2026-06-20 07:02-11:02 CST。
  - ACP 本窗可重构 13 个 session、20 次 `session/prompt`、20 次 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误进入用户可见 final。
  - 08:30 CST `Actor_web__direct__web-user-fe88bce3a53f` final 写出 `主行情工具` 内部工具口径；报告主体完成 AI 硬件晨报与高权重事件筛选。
  - 09:00 CST `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` final 写出 `StockAnalysis 对 25 支标的的最新可得统一口径`；报告主体完成核心观察池早间简报。
  - 两个样本没有旧价格 fallback 成功态、投递失败、空回复、错投或功能阻断证据；本单只记录内部工具 / 数据源口径外露。
- `data/runtime/logs/acp-events.log`
  - 2026-06-19 15:01 CST 巡检窗口：2026-06-19 11:02-15:01 CST。
  - ACP 同窗可重构 8 个 session、21 次 `session/prompt`、21 次 prompt 均有 response，未见 response error；可见回复均以 `stopReason=end_turn` 收口。
  - 12:00 CST `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` final 写出 `StockAnalysis/MarketBeat`、长期跟踪画像同步和画像沉淀等内部/实现口径。
  - 该样本正常完成公司资讯总结、分层结论、财报节点和来源列表，没有旧价格成功态、投递失败、空回复、错投或功能阻断证据。
- `data/runtime/logs/acp-events.log`
  - 2026-06-19 07:04 CST 巡检窗口：2026-06-19 03:02-07:02 CST。
  - ACP 同窗有 3 个 session、3 次 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误进入用户可见 final。
  - 06:00 CST `Actor_feishu__direct__ou_5f11da38ad70c47cf87c0b106b6408b190` final 写出 `已加载市场分析技能`，并说明把检索词改写为绝对日期口径。
  - 该样本正常完成美股 2026-06-18 收盘复盘、Juneteenth 休市口径、指数 / 波动率 / 利率 / 板块和 AI 半导体归因，没有旧价格成功态、投递失败、空回复、错投或功能阻断证据。
- `data/runtime/logs/acp-events.log`
  - 2026-06-18 23:03 CST 巡检窗口：2026-06-18 19:03-23:03 CST。
  - ACP 同窗有 55 次 prompt、55 次 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误进入用户可见 final。
  - 23:01 CST `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` final 写出 `主行情工具返回额度限制，无法完成 data_fetch 校验`，随后以“今日 09:00 已校验参考”输出观察池快报。
  - 该样本正常完成快报主体和风险口径，没有旧价格被包装成实时价的证据；复发仍集中在内部行情工具名 / 工具失败状态进入用户可见 scheduler final。
- `data/runtime/logs/acp-events.log`
  - 2026-06-18 15:03 CST 巡检窗口：2026-06-18 11:03-15:03 CST。
  - 同窗 `data/sessions.sqlite3` 未追平最近真实会话；`data/sessions/*.json` 仅 `Actor_feishu__direct__ou_5f8d3431a2b9ca4af0044ff8970fa36a52.json` 在 15:02 CST 更新，因此本轮仍使用 ACP 事件重构 final。
  - ACP 同窗有 14 个 session、10 次 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误进入用户可见 final。
  - 12:00 CST Feishu scheduler `每日公司资讯与分析总结`（`session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`）final 写出 `本地长期画像目录当前没有可读内容`，并继续说明会补 AI 基建与医疗 AI 跟踪链记录。
  - 该 final 主体完成公司资讯、分析师口径、下一次财报日期、分层结论和来源列表；没有旧价格成功态、投递失败、空回复、错投或功能阻断证据。
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
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-13 23:04 CST 巡检窗口：2026-06-13 19:01-23:04 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 21:35 CST 收到 `科技核心股池 · 晚间击球区快报` 定时触发，assistant `ordinal=336` 于 21:36:20 CST 正常落库 final；对应 `cron_job_runs.run_id=41857` 为 `completed + sent + delivered=1`。
  - final 开头写出：`本轮使用 data_fetch quote 校验；当前为周六晚，对应最新可得美股价格为 2026-06-12 美股收盘附近行情`。
  - 同 session 在 23:00 CST 收到 `核心观察股池晚间快报` 定时触发，assistant `ordinal=338` 于 23:01:15 CST 正常落库 final；对应 `cron_job_runs.run_id=41892` 为 `completed + sent + delivered=1`。
  - 23:00 final 已改写为 `价格用可检索市场源校验，财报日期沿用今日 21:35 同会话已校验结果`，未再外露 `data_fetch`；但 21:35 同窗可见样本仍说明该问题未稳定收口。
- 同窗摘要：
  - 2026-06-13 07:01-11:03 CST `data/sessions.sqlite3` 有 14 个 user turn 与 14 个 assistant turn，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口，没有 user-only 残留。
  - 普通 scheduler 11 条为 `completed + sent + delivered=1`，本条 Feishu scheduler 正常完成观察池早间简报；Discord scheduler 本轮也恢复为 `completed + sent + delivered=1`。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误、quota、panic、`company_profiles/...`、cron 内部存储口径或 SQLite 口径；本轮问题继续集中在内部行情工具名进入用户可见 scheduler final。
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-14 11:03 CST 巡检窗口：2026-06-14 07:02-11:03 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 收到 `核心观察池早间简报` 定时触发，assistant 于 09:02:26 CST 正常落库 final；对应 `cron_job_runs.run_id=42167` 为 `completed + sent + delivered=1`。
  - final 开头写出：`当前未取得新的 data_fetch 返回，价格沿用本会话 6月13日已校验口径，并用公开市场报道交叉核对部分核心股`。
  - 同窗 `data/sessions.sqlite3` 有 15 个 user turn 与 15 个 assistant turn，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口；普通 scheduler 12 条均为 `completed + sent + delivered=1`。
  - 本轮没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据；复发仍集中在内部行情工具名进入用户可见 scheduler final。
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-14 23:02 CST 巡检窗口：2026-06-14 19:02-23:02 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 21:35 CST 收到 `科技核心股池 · 晚间击球区快报` 定时触发，assistant `ordinal=342` 于 21:37:24 CST 正常落库 final；对应 `cron_job_runs.run_id=42518` 为 `completed + sent + delivered=1`。
  - final 开头写出：`专用 data_fetch 本轮未返回可用结果，以下价格与财报日期改用 StockAnalysis 页面校验`。
  - 同 session 在 23:00 CST 收到 `核心观察股池晚间快报` 定时触发，assistant `ordinal=344` 于 23:01:28 CST 正常落库 final；对应 `cron_job_runs.run_id=42559` 为 `completed + sent + delivered=1`。
  - 23:00 final 开头写出：`本轮 data_fetch 已返回最新可得 quote，口径为 2026-06-12 美股收盘附近行情`。
  - 同窗 `data/sessions.sqlite3` 有 18 个 user turn 与 18 个 assistant turn，最近 Feishu scheduler 会话均以 assistant 收口；普通 scheduler 18 条均为 `completed + sent + delivered=1`。
  - 本轮没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据；两条样本分别覆盖“失败降级说明”和“工具名作为来源背书”两类句型。
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-15 23:04 CST 巡检窗口：2026-06-15 19:03-23:04 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 21:35 CST 收到 `科技核心股池 · 晚间击球区快报` 定时触发，assistant 于 21:37:31 CST 正常落库 final；对应 `cron_job_runs.run_id=43191` 为 `completed + sent + delivered=1`。
  - 21:35 final 开头写出：`专用 data_fetch 未返回可用结果，以下改用 StockAnalysis 校验`。
  - 同 session 在 23:00 CST 收到 `核心观察股池晚间快报` 定时触发，assistant 于 23:02:31 CST 正常落库 final；对应 `cron_job_runs.run_id=43244` 为 `completed + sent + delivered=1`。
  - 23:00 final 开头写出：`data_fetch 本轮未返回可用结果，价格改用公开行情页校验`。
  - 同窗 `data/sessions.sqlite3` 有 45 个 user turn 与 45 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 34 条均为 `completed + sent + delivered=1`。
  - 两轮均输出观察池列表、击球区、价格和财报日期；没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
- `data/sessions.sqlite3` -> `session_messages` / `cron_job_runs`
  - 2026-06-16 11:01 CST 巡检窗口：2026-06-16 07:03-11:01 CST。
  - session `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773` 在 09:00 CST 收到 `核心观察池早间简报` 定时触发，assistant `ordinal=352` 于 09:03:49 CST 正常落库 final；对应 `cron_job_runs.run_id=43527` 为 `completed + sent + delivered=1`。
  - final 开头写出：`data_fetch 未取得可用返回，价格改用 StockAnalysis 校验，击球区沿用你的本地固定区间`。
  - 该样本晚于 2026-06-16 04:08 CST 共享净化层代码级修复记录，说明仍有未覆盖句型或 live 出站路径未稳定经过净化。
  - 同窗 `data/sessions.sqlite3` 有 26 个 user turn 与 26 个 assistant turn，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口；普通 scheduler 16 条为 `completed + sent + delivered=1`。
  - 本条 Feishu scheduler 正常完成观察池早间简报，没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据；问题仍集中在内部行情工具名 / 站点名进入用户可见 scheduler final。
- `data/runtime/logs/acp-events.log`
  - 2026-06-18 11:04 CST 巡检窗口：2026-06-18 07:01-11:04 CST。
  - 同窗 `data/sessions.sqlite3` 未追平最近真实会话，`data/sessions/*.json` 虽有 4 个文件在 07:51-09:50 CST 更新，但文件内 `messages[].timestamp` 最大值仍停在 2026-06-17 或更早；因此本轮仍使用 ACP 事件重构 final。
  - ACP 同窗有多条 Feishu / Web scheduler 与 direct 会话；代表性响应均以 `stopReason=end_turn` 收口，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误进入用户可见 final。
  - 08:30 CST Feishu scheduler `闪迪(SNDK)每日行情与行业简报`（`session_id=Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34`）final 写出 `现在补充本地 SNDK 画像`、`我现在更新 SNDK 长期画像和今日简报事件`。
  - 09:30 CST Feishu scheduler `美股收盘资金流复盘`（`session_id=Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768`）final 开头写出 `已加载市场分析流程`，尾部来源写出 `Hone data_fetch：SPY、QQQ、11只 Select Sector SPDR ETF 收盘行情`。
  - 上述 final 分别完成 SNDK 行情 / 行业简报、资金流复盘与数据不可验证边界说明，没有旧价格成功态、投递失败、空回复、错投或功能阻断证据。
- `data/runtime/logs/acp-events.log`
  - 2026-06-18 07:02 CST 巡检窗口：2026-06-18 03:02-07:02 CST。
  - 同窗 `data/sessions.sqlite3` 未追平最近四小时真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`、`cron_job_runs.max(executed_at)=2026-06-17T11:01:42.353141+08:00`；因此本轮使用 ACP 事件重构 final。
  - ACP 同窗有 7 个 session、9 次 `stopReason=end_turn`，未见未收口 response。
  - 05:30 CST Feishu scheduler `美股收盘后跨市场复盘`（`session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`）final 开头写出：`已加载市场复盘技能。现在补充跑一轮行情与板块工具...`，尾部来源写出 `Hone data_fetch：SPY、QQQ、SOXX、KWEB、FXI、AMAT、AVGO、MRVL、MU、HOOD、QURE、SPCX 及部分 A/H 代码行情。`
  - 该 final 正常输出美股指数、ETF、宏观驱动、突出公司、A/H 预判、映射代码池、估值分层、风险与证伪条件；没有空回复、投递失败、错投、会话悬挂或链路级数据破坏证据。

## 端到端链路

1. Feishu scheduler 触发观察池、资金流、持仓 / 个股简报等定时任务。
2. runner 尝试使用行情 / 数据工具获取价格、新闻、资金流、财报或行业信息。
3. runner 可能进入行情工具、市场分析流程或公司画像读写 / 更新流程。
4. 最终回复正常收口，但把内部工具名 `data_fetch`、内部流程加载状态或本地画像读写动作作为用户可见正文 / 来源说明发出。

## 期望效果

- 用户可见文本可以说明“本轮主行情源未返回可用结果，已改用公开页面交叉校验”。
- 不应暴露 `data_fetch` 这类内部工具名、工具编排、流程加载状态、本地画像读取 / 写入动作或执行进度。
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
- 2026-06-13 23:04 CST 样本继续证明同一“工具名作为来源背书”句型会进入 scheduler final；23:00 同会话已出现产品化替代表达，说明问题可能有路径/上下文差异，但不能视为已修复。状态保持 `New`，仍按质量性 `P3` 处理。
- 2026-06-14 11:03 CST 样本又回到“当前未取得新的 data_fetch 返回”的失败 / 降级解释句型，说明现有净化或 prompt guard 同时未稳定覆盖“失败降级说明”和“工具名作为来源背书”两类表达。状态保持 `New`，仍按质量性 `P3` 处理。
- 2026-06-14 23:02 CST 样本继续同时覆盖两类表达：21:35 CST 为“专用 data_fetch 本轮未返回可用结果”的失败降级说明，23:00 CST 为“本轮 data_fetch 已返回最新可得 quote”的来源背书。两轮都正常完成并送达，说明主功能链路未受阻；状态保持 `New`，仍按质量性 `P3` 处理。
- 2026-06-15 23:04 CST 样本继续覆盖“失败降级说明”表达：21:35 CST 写出 `专用 data_fetch 未返回可用结果`，23:00 CST 写出 `data_fetch 本轮未返回可用结果`。两轮都正常完成并送达，观察池列表、击球区、价格和财报日期可用；状态保持 `New`，仍按质量性 `P3` 处理。
- 2026-06-16 11:01 CST 样本晚于 2026-06-16 04:08 CST 再次修复记录，仍写出 `data_fetch 未取得可用返回，价格改用 StockAnalysis 校验`。当前证据来自真实 scheduler final 与 cron 台账，因此把状态从代码级 `Fixed` 调回 `New`；该任务仍正常完成并送达，问题只影响用户可见文案边界和产品感，仍按质量性 `P3` 处理。
- 2026-06-18 05:30 CST 样本晚于 2026-06-18 03:04 CST 再次修复记录，且不只外露 `data_fetch` 字面量，也外露 `已加载市场复盘技能`、`行情与板块工具` 等内部技能 / 工具执行口径。当前证据来自 ACP final，说明共享净化层仍未覆盖 scheduler 开头执行进度句和尾部来源背书句。状态从代码级 `Fixed` 调回 `New`；该任务仍正常完成并送达，问题只影响用户可见文案边界和产品感，仍按质量性 `P3` 处理。
- 2026-06-18 08:30 / 09:30 CST 样本继续晚于 2026-06-18 03:04 CST 再次修复记录：08:30 CST `闪迪(SNDK)每日行情与行业简报` 外露 `本地 SNDK 画像` 与 `更新 SNDK 长期画像`，09:30 CST `美股收盘资金流复盘` 外露 `已加载市场分析流程` 和 `Hone data_fetch` 来源背书。当前证据来自 ACP final，说明同一 scheduler 出站净化缺口不只覆盖行情降级说明，也覆盖画像存储 / 内部流程进度句。两个任务仍正常完成并收口，问题只影响用户可见文案边界和产品感，仍按质量性 `P3 / New` 处理。
- 现有 `web_direct_internal_skill_and_local_store_terms_exposed.md` 覆盖 Web direct 的 `skill` / `data/portfolio` / 本地 json 口径；本轮是 Feishu 普通 scheduler 的行情工具降级说明，链路和触发位置不同。
- 现有 `feishu_scheduler_stale_price_fallback_after_data_fetch_failure.md` 覆盖关键行情失败后旧价格 fallback 被记成功；本轮证据不足以判断旧价成功态复发，只确认内部工具名外露。

## 下一步建议

- 扩展共享出站净化或 scheduler prompt guard，将 `data_fetch 本轮未返回可用结果` 等内部工具名口径改写为“主行情源本轮未返回可用结果”。
- 对 Feishu scheduler final 增加回归样本：当内部行情工具失败但有公开来源补充校验时，用户可见文本不得出现 `data_fetch`、tool 名称或内部执行状态。
- 后续巡检继续区分两类证据：若同时复用旧价格并记成功，应回看 stale-price fallback 缺陷；若只是工具名进入最终回复，则按本单跟踪。
- 扩展 sanitizer / prompt guard 时应按语义覆盖 `data_fetch` + `StockAnalysis` 降级句族、`已加载...流程 / 技能` 进度句、以及 `本地...画像 / 更新...画像 / 沉淀...画像` 这类 scheduler 画像读写句，而不是只匹配单个固定句。

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
- 2026-06-13 23:04 CST 补充同根复发证据：
  - 21:35 CST `科技核心股池 · 晚间击球区快报` final 开头写出 `本轮使用 data_fetch quote 校验`。
  - 对应 `cron_job_runs.run_id=41857` 为 `completed + sent + delivered=1`，观察池列表、击球区和财报日期正常输出；没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 同窗 23:00 CST `核心观察股池晚间快报` final 已改写为 `价格用可检索市场源校验`，对应 `cron_job_runs.run_id=41892` 也为 `completed + sent + delivered=1`，可作为正向对照，但不能抵消 21:35 可见复发样本。
  - 本轮 19:01-23:04 CST `data/sessions.sqlite3` 有 19 个 user turn 与 19 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 18 条均为 `completed + sent + delivered=1`。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-06-14 11:03 CST 补充同根复发证据：
  - 09:02 CST `核心观察池早间简报` final 写出 `当前未取得新的 data_fetch 返回，价格沿用本会话 6月13日已校验口径`。
  - 对应 `cron_job_runs.run_id=42167` 为 `completed + sent + delivered=1`，观察池列表、击球区和财报日期正常输出；没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 本轮 07:02-11:03 CST `data/sessions.sqlite3` 有 15 个 user turn 与 15 个 assistant turn，最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口；普通 scheduler 12 条均为 `completed + sent + delivered=1`。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-06-14 23:02 CST 补充同根复发证据：
  - 21:35 CST `科技核心股池 · 晚间击球区快报` final 写出 `专用 data_fetch 本轮未返回可用结果，以下价格与财报日期改用 StockAnalysis 页面校验`。
  - 对应 `cron_job_runs.run_id=42518` 为 `completed + sent + delivered=1`，观察池列表、击球区、价格和财报日期正常输出；没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 23:00 CST `核心观察股池晚间快报` final 写出 `本轮 data_fetch 已返回最新可得 quote，口径为 2026-06-12 美股收盘附近行情`；对应 `cron_job_runs.run_id=42559` 同样为 `completed + sent + delivered=1`。
  - 本轮 19:02-23:02 CST `data/sessions.sqlite3` 有 18 个 user turn 与 18 个 assistant turn，最近 Feishu scheduler 会话均以 assistant 收口；普通 scheduler 18 条均为 `completed + sent + delivered=1`。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-06-15 11:01 CST 补充同根复发证据：
  - 09:03 CST `核心观察池早间简报` final 写出 `本轮未取得 data_fetch 返回，价格用 StockAnalysis 页面校验；财报日期优先沿用最近一次已校验结果，页面仍显示已过日期的标的标注为待确认`。
  - 对应 `cron_job_runs.run_id=42838` 为 `completed + sent + delivered=1`，观察池列表、击球区、价格和财报日期正常输出；没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 本轮 07:02-11:01 CST `data/sessions.sqlite3` 有 20 个 user turn 与 21 个 assistant turn，其中 1 条 assistant 是 07:00 scheduler 结果落在窗口内；最近 Feishu direct / scheduler 与 Discord scheduler 会话均以 assistant 收口，无 user-only 残留。
  - 普通 scheduler 19 条均为 `completed + sent + delivered=1`，未命中 `commodity_causality_guarded=true`、send_failed 或空回复；assistant final 污染扫描只命中本条 `data_fetch` 外露样本，未命中本机路径、raw tool 字段、思维痕迹、provider 原始错误、`open_id / chat_id`、SQLite 或技能状态外露。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-06-15 23:04 CST 补充同根复发证据：
  - 21:35 CST `科技核心股池 · 晚间击球区快报` final 写出 `专用 data_fetch 未返回可用结果，以下改用 StockAnalysis 校验`。
  - 23:00 CST `核心观察股池晚间快报` final 写出 `data_fetch 本轮未返回可用结果，价格改用公开行情页校验`。
  - 对应 `cron_job_runs.run_id=43191/43244` 均为 `completed + sent + delivered=1`；观察池列表、击球区、价格和财报日期正常输出。
  - 本轮 19:03-23:04 CST `data/sessions.sqlite3` 有 45 个 user turn 与 45 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 34 条均为 `completed + sent + delivered=1`。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-06-16 11:01 CST 重新打开：
  - 09:03 CST `核心观察池早间简报` final 写出 `data_fetch 未取得可用返回，价格改用 StockAnalysis 校验，击球区沿用你的本地固定区间`。
  - 对应 `cron_job_runs.run_id=43527` 为 `completed + sent + delivered=1`，观察池列表、击球区、价格和财报日期正常输出；没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 该样本晚于 2026-06-16 04:08 CST 再次修复记录，说明修复仍未覆盖 `data_fetch 未取得可用返回` 句型或 live 出站路径未稳定经过净化；状态从代码级 `Fixed` 调回 `New`。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3`；非 P1，不创建 GitHub Issue。
- 2026-06-18 07:02 CST 重新打开：
  - 05:30 CST `美股收盘后跨市场复盘` final 外露 `已加载市场复盘技能`、`行情与板块工具` 和 `Hone data_fetch`。
  - 同窗 ACP 事件显示该 session 以 `stopReason=end_turn` 正常收口；报告主体、来源和 A/H 映射均完成，没有投递失败、空回复、错投或链路级数据破坏证据。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3`；非 P1，不创建 GitHub Issue。
- 2026-06-18 11:04 CST 补充同根复发证据：
  - 08:30 CST `闪迪(SNDK)每日行情与行业简报` final 外露 `本地 SNDK 画像`、`更新 SNDK 长期画像和今日简报事件`。
  - 09:30 CST `美股收盘资金流复盘` final 外露 `已加载市场分析流程` 和 `Hone data_fetch` 来源背书。
  - 两个 session 均以 `stopReason=end_turn` 正常收口；报告主体、来源和风险结论均完成，没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3 / New`；非 P1，不创建 GitHub Issue。
- 2026-07-04 23:02 CST 补充同根复发证据：
  - 21:35 CST `科技核心股池 · 晚间击球区快报` final 写出 `quote_short 口径未单独覆盖盘前/盘后实时价`。
  - 23:00 CST `核心观察股池晚间快报` final 写出 `本轮 data_fetch 返回的最新可用价格仍是最近一次已校验 quote_short 口径`、`价格来自本轮 data_fetch quote_short`。
  - 本轮 `sessions.sqlite3` 中 3 组 user / assistant 均成对收口；ACP 同窗 20 个 prompt 均 `stopReason=end_turn`，没有投递失败、空回复、错投、会话悬挂或链路级数据破坏证据。
  - 因问题只影响用户可见文案边界和产品感，不阻断 scheduler 主功能链路，仍为质量性 `P3 / New`；非 P1，不创建 GitHub Issue。

## 修复记录

- 2026-06-16 04:08 CST 再次修复并收敛更多 live 句型：
  - `crates/hone-channels/src/runtime.rs` 的共享 `sanitize_user_visible_output(...)` 继续扩展整句级行情降级改写，新增覆盖 `专用 data_fetch 未返回可用结果`、`本轮未取得 data_fetch 返回`、`未能取得新的 data_fetch / 网页行情返回`、`data_fetch quote 校验` 等 6 月 13-15 日真实复发表达。
  - 改写策略保持“去内部实现词、保留业务边界”：用户可继续看到“主行情源本轮未返回可用结果，已改用公开页面补充校验”或保留必要的交易时段/时间口径，但不再看到 `data_fetch`、`StockAnalysis`、`quote` 这类内部工具名和站点名。
  - 新增 / 扩展回归 `sanitize_user_visible_output_rewrites_market_data_fallback_variants`，直接锁住 6 月 15 日 `专用 data_fetch 未返回可用结果...`、6 月 14 日 `本轮未取得 data_fetch 返回...`、6 月 10 日 `未能取得新的 data_fetch / 网页行情返回...` 与 6 月 13 日 `data_fetch quote 校验` 等真实样本。
  - 验证通过：`cargo test -p hone-channels sanitize_user_visible_output_rewrites_market_data_fallback_variants --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests`。
  - 本轮未重启当前 Feishu 服务，也不把当前机器 live 运行态当作恢复证据；当时状态更新为代码级 `Fixed`，但 2026-06-16 09:03 CST 真实 scheduler final 已复发，当前状态重新打开为 `New`。

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
