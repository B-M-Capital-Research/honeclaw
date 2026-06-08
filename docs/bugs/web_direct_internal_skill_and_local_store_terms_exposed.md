# Bug: Web direct 投研回复外露内部 skill 与本地存储口径

- **发现时间**: 2026-06-08 23:04 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-08 20:11-20:30 CST
  - session_id: `Actor_web__direct__web-user-879a3b18fce2`
    - 用户消息摘要：用户询问 KRMN / RKLB / MRVL 在 6、7、8 月的走势与判断。
    - ACP 事件显示该轮最终 `response stopReason=end_turn`，说明 Web direct 回复链路已收口。
    - assistant final 在业务分析前写出本地画像缺失、`Hone 的 stock_research 技能名当前没有激活`、改用其它技能框架，以及财报日历工具返回全市场列表等内部执行说明。
  - session_id: `Actor_web__direct__web-user-f40ae1caa720`
    - 用户消息摘要：用户要求按北京时间 20:30 核对持仓过去 24 小时新闻、行情和风险。
    - ACP 事件显示该轮最终 `response stopReason=end_turn`，说明 Web direct 回复链路已收口。
    - assistant final 在最终回复中写出 `账本文件已定位到本地 data/portfolio 下`、`本地文件仍只显示...`、`本地json文件仍只显示...`，随后才说明以 Hone 持仓工具为准。
- 本轮 2026-06-08 19:03-23:03 CST 复核：
  - `data/sessions.sqlite3` 按真实消息时间有 11 个 Feishu user turn 与 11 个 assistant final，均成对收口；SQLite 当前没有 Web direct final 镜像，Web direct 证据来自 ACP 日志。
  - `acp-events.log` 同窗 Web / Feishu direct 均有 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、quota、panic 或 provider 原始错误。
  - assistant final 污染扫描未命中空回复、`/Users/` 绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误或 panic。
  - `cron_job_runs` 同窗无新增记录；`data/runtime/task_runs.2026-06-08.jsonl` 中 `poller.fmp.price` 48 次、`poller.fmp.news` 16 次、`poller.fmp.extended_hours` 8 次均为 `ok + items=0`。

## 端到端链路

1. Web direct 用户发起投研 / 持仓复盘请求。
2. runner 调用行情、新闻、技能和持仓相关工具，部分内部能力不可用或本地存储与权威持仓工具口径不一致。
3. assistant 最终回复正常输出业务分析，并以 ACP `end_turn` 收口。
4. 最终用户可见文本同时暴露内部 skill 名称、skill 激活状态、`data/portfolio` 本地存储口径、`json` 文件口径和工具过滤异常说明。

## 期望效果

- Web direct 最终回复应只暴露用户可理解的业务口径，例如“本轮以权威持仓工具为准”或“改用行情与新闻数据完成分析”。
- 内部 skill 名称、skill 激活状态、本地目录名、文件格式、工具返回异常和执行过程应留在日志或被改写成产品化说明。
- 当本地文件与权威工具不一致时，用户态文案应强调最终采用的权威数据源，不应列出内部文件位置或 `json` 存储细节。

## 当前实现效果

- 回复完成了投研分析、持仓复盘和风险提示，用户主要问题被回答。
- 但最终可见文本包含内部能力编排与存储细节，包括 skill 名未激活、本地账本目录、本地 json 文件和工具过滤异常。
- 这类文本没有泄露绝对路径或原始 token，但仍把内部运行机制当作用户态解释，影响产品专业度。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 本轮 Web direct 均以 `stopReason=end_turn` 收口，没有未回复、空回复、投递失败、错投、会话状态错乱或系统链路中断证据。
- 用户仍获得了主要投研结论和风险分析，因此不影响主功能链路，按规则定级为 P3，而不是 P1/P2。
- 影响主要是内部实现细节外露、用户对数据权威口径产生疑惑，以及回复显得像调试过程而不是成品投研答复。

## 根因判断

- 直接证据只能证明 Web direct answer 阶段把内部执行状态和本地存储口径写入最终用户可见文本。
- 初步判断是共享用户可见输出净化已覆盖部分 scheduler skill 降级前言和公司画像路径，但 Web direct 对自然语言形式的 `skill 未激活`、`本地 data/portfolio`、`本地 json 文件` 等口径缺少足够过滤或改写。
- 该问题不同于 `web_scheduler_skill_load_failure_phrase_exposed.md`：本轮是 Web direct 直聊最终回复，且同时包含本地存储口径外露；旧缺陷只覆盖 Web scheduler 的“技能未加载 / 当前运行器”降级措辞。
- 该问题也不同于 raw tool output 外泄：本轮没有原始 JSON、工具日志、绝对路径、provider 报错或 `<think>` 进入 final，而是模型自然语言层面复述内部执行过程。

## 下一步建议

- 扩展共享用户可见输出净化或 Web direct final guidance，过滤 / 改写以下自然语言内部口径：
  - `技能名当前没有激活`、`某 skill/tool 未激活`、`改用某技能框架`
  - `本地 data/...`、`本地 json 文件`、`账本文件已定位到...`
  - `工具返回了全市场列表而不是按标的过滤`
- 对 Web direct 增加回归样本：内部 skill 不可用、持仓本地文件与权威工具不一致时，最终回复应只保留业务化数据口径，不出现内部目录、文件格式或 skill 激活状态。
- 后续巡检若仅在 `tool_call_update.rawOutput` 内看到这类信息，但最终用户可见 final 已自然化，不应补充为本缺陷复发。

## 修复记录

- 2026-06-09 已修复：
  - 共享 `sanitize_user_visible_output(...)` 新增内部执行说明剥离规则：会过滤 `stock_research` / `skill` 未激活、改用其它技能框架、`data/portfolio` / 本地 `json` 文件口径，以及“返回全市场列表而不是按标的过滤”等自然语言内部说明。
  - 保留最终业务结论与“以权威持仓工具为准”这类用户态口径，不再把 Web direct 的内部排障过程当成 final 正文。

## 验证

- `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 文档同步

- 已同步更新 `docs/bugs/README.md` 活跃表与已修复表。
- 本修复只收紧共享用户态文案净化边界，不改变模块边界、长期约束或运行工作流，无需更新 `docs/repo-map.md`、`docs/invariants.md` 或新增 handoff。
