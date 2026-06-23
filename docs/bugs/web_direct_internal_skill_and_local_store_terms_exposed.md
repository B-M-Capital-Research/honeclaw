# Bug: Web direct 投研回复外露内部 skill 与本地存储口径

- **发现时间**: 2026-06-08 23:04 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **GitHub Issue**: 无，非 P1

## 修复记录

- 2026-06-24 07:02 CST 继续补充同根复发证据：
  - 03:02-07:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话；本轮以 `data/runtime/logs/acp-events.log` 重构 Web direct 用户可见 final。
  - 06:30 CST Web direct session `Actor_web__direct__web-user-14f4cadb069f` 的组合复盘 final 以 `stopReason=end_turn` 收口，主体完成 2026-06-23 美股常规收盘价格、组合当日贡献、个股归因和风险提示。
  - 但正文前段继续写出“我会用 stock_research 技能处理这次组合复盘”和“优先使用行情工具拿同一口径报价”，把内部 skill / 工具编排作为用户态正文发出。
  - 本窗未见 Web direct response error、stream disconnect、quota、panic、provider 原始错误、绝对路径或 token 外露。问题不影响主功能链路，因此按质量性 `P3 / New` 继续跟踪；非 P1，不创建 GitHub Issue。

- 2026-06-23 23:02 CST 修复结论回退：
  - 19:02-23:02 CST `data/sessions.sqlite3` 仍未追平最近真实会话；本轮以 `data/runtime/logs/acp-events.log` 重构 Web direct 用户可见 final。
  - 20:01 CST Web direct session `Actor_web__direct__web-user-e05f5e5f74a3` 的 NVDA 投研 final 以 `stopReason=end_turn` 收口，主体完成行情、财报、官方消息、估值和风险条件，但正文继续写出 `StockAnalysis` 行情口径，并在结尾写出“已更新 NVDA 公司画像”。
  - 22:36-22:38 CST Web direct session `Actor_web__direct__web-user-7c0d676f10ee` 的 CBRS / Cerebras 财报前瞻 final 正常输出财报时间、市场预期和风险框架，但前段继续写出“我本地没有看到 CBRS/Cerebras 的既有公司画像”和“沉淀成一份简洁画像”。
  - 两个样本晚于 2026-06-23 03:04 CST 共享净化修复记录；本窗未见 Web direct response error、stream disconnect、quota、panic、provider 原始错误、绝对路径或 token 外露。问题不影响主功能链路，因此按质量性 `P3 / New` 重新进入活跃待修复；非 P1，不创建 GitHub Issue。

- 2026-06-23 03:04 CST 再次修复：
  - 共享 `sanitize_user_visible_output(...)` 补齐 `StockAnalysis DRAM holdings` 这类站点名 + 持仓后缀来源口径，并与新增的画像简写/沉淀句式收口共用同一净化规则。
  - 新增回归 `sanitize_user_visible_output_rewrites_hone_market_tool_copy`、`sanitize_user_visible_output_strips_profile_creation_progress_variants`，既有 `RE_STOCKANALYSIS_LABEL` 改写继续覆盖 Web direct 来源段。
  - 验证：`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。
  - 本轮未重启当前 Web 服务；按当前代码与回归验证回写 `Fixed`，后续若确认加载当前代码的新运行态仍复现，再重新打开。

- 2026-06-22 19:00 CST 修复结论回退：
  - 15:04-19:00 CST `data/sessions.sqlite3` 仍未追平最近真实会话；本轮以 `data/runtime/logs/acp-events.log` 重构 Web direct 用户可见 final。
  - 15:06 CST Web direct session `Actor_web__direct__web-user-c3cef1bfa64d` 的 DRAM / Roundhill Memory ETF 投研 final 以 `stopReason=end_turn` 收口，主体完成 ETF 定位、持仓、周期弹性、风险、动作建议和来源。
  - 但 final 来源段继续写出 `StockAnalysis DRAM holdings`，把内部/实现侧行情站点口径作为用户态来源说明发出；这晚于 2026-06-21 19:09 CST 共享净化修复记录。
  - 同窗未见 Web direct response error、stream disconnect、quota、panic、provider 原始错误、绝对路径或 token 外露。问题不影响主功能链路，因此按质量性 `P3 / New` 重新进入活跃待修复；非 P1，不创建 GitHub Issue。

- 2026-06-21 19:09 CST 修复：
  - 共享 `sanitize_user_visible_output(...)` 扩展 Web / Feishu 共用的内部执行进度净化，继续覆盖本机命令、内部工具、画像存在性与画像写入动作等自然语言句式。
  - 新增 runner warning 剥离和 `StockAnalysis` 标签改写，避免 Web direct / scheduler 将执行环境、工具名或站点执行口径暴露为用户态报告正文。
  - 验证：`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-18 19:03-23:03 CST。
  - session_id: `Actor_web__direct__web-user-e05f5e5f74a3`
    - `2026-06-18 20:02 CST` 用户要求分析 NVDA 最新消息与基本面；assistant final 以 `stopReason=end_turn` 收口，完成行情、官方消息、客户线索、资本开支压力和结论。
    - 但 final 前段写出“我会把今天的有效增量写回 NVDA 公司画像”，把内部画像写入动作当作用户态正文。
  - session_id: `Actor_web__direct__web-user-2dca3d98a31b`
    - `2026-06-18 22:04 CST` 用户要求分析 AAOI；assistant final 完成基本面、估值、多空逻辑和结论，但前段写出 `我没有找到本地已有的 AAOI 公司画像` 与“沉淀成 AAOI 画像”。
  - 两个样本均正常完成业务回答并收口，没有空回复、投递失败、错投、原始工具 JSON、token 或绝对路径外露；问题仍只影响 Web direct 用户可见文案边界，保持 `P3 / New`。
- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-18 15:03-19:03 CST。
  - session_id: `Actor_web__direct__web-user-e1ed2ef04d14`
    - `2026-06-18 15:40 CST` 用户上传持仓截图并要求用中文给出持仓建议；ACP chunks 显示该轮最终以 `stopReason=end_turn` 收口。
    - assistant final 成功读出截图里的 10 个仓位，并给出组合暴露、仓位风险和单票建议，说明图片读取和主业务分析链路完成。
    - 但 final 前段继续外露本机执行过程：`本地环境没有 python 命令，我改用可用的 python3 试一次行情读取`，以及“行情库不可用，再用公开报价端点做一次轻量校验”等内部排障口径。
    - 该样本晚于 2026-06-18 03:04 CST 共享净化层修复记录；没有未回复、空回复、错投、投递失败、原始工具 JSON、token 或绝对路径外露，问题仍限定在用户可见文案边界。
- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-17 23:01-2026-06-18 03:01 CST。
  - session_id: `Actor_web__direct__web-user-12b12fcf502c`
    - `2026-06-18 02:51 CST` assistant final 以 `stopReason=end_turn` 收口，业务上回答了 LRCX / 泛林集团在 AI 半导体设备链中的位置，但开头写出“本地没有已有的 LRCX 公司画像”“我会把这轮形成的长期主线沉淀成画像”等本地画像存储与执行过程口径。
    - 同一 web session 在 `2026-06-18 02:51 CST` 的前一轮 final 也把“我先核对泛林集团对应实体、最新财报/指引和近期关于产能与需求的公司表述”这类执行进度拼进最终用户可见正文。
  - 同窗 ACP 重构出 24 条用户可见 assistant final，全部以 `end_turn` 收口；未见 Web direct response error、stream disconnect、quota、panic、provider 原始错误、绝对路径或 token 外露。
  - `data/sessions.sqlite3` 在同窗没有 `session_messages` 新增，Web direct 证据来自 ACP 流式日志重构。
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
- 2026-06-18 19:03 CST 复核显示，Web direct 图片持仓分析链路在读取截图并完成组合建议后，仍把 `python` / `python3` 命令切换和行情读取排障过程写进 final。
- 2026-06-18 03:02 CST 复核显示，修复后仍有 Web direct final 外露更自然语言化的本地画像存储 / 沉淀动作，虽不再出现原始 `skill 未激活` 或 `data/portfolio` 目录名，但仍把内部长期画像写入计划当成用户态正文。
- 这类文本没有泄露绝对路径或原始 token，但仍把内部运行机制当作用户态解释，影响产品专业度。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 本轮 Web direct 均以 `stopReason=end_turn` 收口，没有未回复、空回复、投递失败、错投、会话状态错乱或系统链路中断证据。
- 用户仍获得了主要投研结论和风险分析，因此不影响主功能链路，按规则定级为 P3，而不是 P1/P2。
- 影响主要是内部实现细节外露、用户对数据权威口径产生疑惑，以及回复显得像调试过程而不是成品投研答复。

## 根因判断

- 直接证据只能证明 Web direct answer 阶段把内部执行状态和本地存储口径写入最终用户可见文本。
- 初步判断是共享用户可见输出净化已覆盖部分 scheduler skill 降级前言和公司画像路径，但 Web direct 对自然语言形式的 `skill 未激活`、`本地 data/portfolio`、`本地 json 文件` 等口径缺少足够过滤或改写。
- 2026-06-18 03:02 CST 的复发说明现有净化规则仍偏短语匹配，覆盖了显式 `skill` / 路径 / json 口径，但没有覆盖“本地没有画像 / 沉淀成画像 / 写入长期跟踪框架”这类无路径的内部存储动作。
- 该问题不同于 `web_scheduler_skill_load_failure_phrase_exposed.md`：本轮是 Web direct 直聊最终回复，且同时包含本地存储口径外露；旧缺陷只覆盖 Web scheduler 的“技能未加载 / 当前运行器”降级措辞。
- 该问题也不同于 raw tool output 外泄：本轮没有原始 JSON、工具日志、绝对路径、provider 报错或 `<think>` 进入 final，而是模型自然语言层面复述内部执行过程。

## 下一步建议

- 扩展共享用户可见输出净化或 Web direct final guidance，过滤 / 改写以下自然语言内部口径：
  - `技能名当前没有激活`、`某 skill/tool 未激活`、`改用某技能框架`
  - `本地 data/...`、`本地 json 文件`、`账本文件已定位到...`
  - `工具返回了全市场列表而不是按标的过滤`
  - `本地没有已有公司画像`、`沉淀成画像`、`写入长期跟踪框架`、`记录本轮结论` 等画像存储 / 执行过程句式
- 对 Web direct 增加回归样本：内部 skill 不可用、持仓本地文件与权威工具不一致时，最终回复应只保留业务化数据口径，不出现内部目录、文件格式或 skill 激活状态。
- 增加 Web direct 公司画像沉淀回归：即使内部写入或更新画像，final 也只说“后续可沿用本轮框架”这类产品化口径，不出现本地画像是否存在或写入动作。
- 后续巡检若仅在 `tool_call_update.rawOutput` 内看到这类信息，但最终用户可见 final 已自然化，不应补充为本缺陷复发。

## 修复记录

- 2026-06-18 23:03 CST 补充同根复发证据：
  - 最近四小时 Web direct 投研 final 继续外露“写回 / 沉淀公司画像”“本地已有画像”等自然语言内部存储动作。
  - 代表样本为 20:02 CST NVDA 投研与 22:04 CST AAOI 投研；两个回复主体均完整，ACP `end_turn` 收口，不影响主功能链路。
  - 因此状态保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-06-18 19:03 CST 修复结论回退：
  - 最近四小时 Web direct 真实 final 再次外露本机命令可用性和内部行情读取排障过程，说明 03:04 CST 共享净化层代码级修复未覆盖 Web direct 图片 / 持仓分析场景里的 `本地环境没有 python 命令，我改用 python3` 句式。
  - 主功能链路仍正常：截图被读出，组合分析完成，ACP `end_turn` 收口；因此按质量性 `P3 / New` 重新进入活跃待修复。
  - 非 P1，不创建 GitHub Issue。

- 2026-06-18 03:04 CST 再次修复：
  - 共享 `sanitize_user_visible_output(...)` 继续扩展自然语言内部执行进度净化，新增覆盖 `本地没有已有的 ... 公司画像`、`我先核对...`、`沉淀成画像`、`我会新增...长期画像` 等无路径、无 `skill` 关键词的画像存储 / 调研前言句式。
  - 新增回归 `sanitize_user_visible_output_strips_natural_language_profile_progress_copy`，锁住 2026-06-18 02:51 CST Web direct LRCX 样本；共享 `sanitize_user_visible_output_strips_internal_runtime_progress_copy` 同步覆盖 Feishu direct 的本机命令 / 内部流程前言。
  - 验证通过：`cargo test -p hone-channels sanitize_user_visible_output_strips_natural_language_profile_progress_copy --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_strips_internal_runtime_progress_copy --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`、`cargo check -p hone-channels --tests`。
  - 本轮未重启 live 服务，也不把当前机器运行态当作恢复证据；状态更新为代码级 `Fixed`，后续若部署后仍有新的自然语言内部存储动作进入 final，再基于新样本重新打开。

- 2026-06-18 03:02 CST 修复结论回退：
  - 最近四小时 Web direct 真实 final 再次外露本地画像存在性和沉淀动作，说明 2026-06-09 修复未覆盖无路径、无 `skill` 关键词的自然语言化内部存储过程。
  - 主功能链路仍正常，按质量性 `P3 / New` 重新进入活跃待修复；非 P1，不创建 GitHub Issue。

- 2026-06-09 已修复：
  - 共享 `sanitize_user_visible_output(...)` 新增内部执行说明剥离规则：会过滤 `stock_research` / `skill` 未激活、改用其它技能框架、`data/portfolio` / 本地 `json` 文件口径，以及“返回全市场列表而不是按标的过滤”等自然语言内部说明。
  - 保留最终业务结论与“以权威持仓工具为准”这类用户态口径，不再把 Web direct 的内部排障过程当成 final 正文。

## 验证

- `cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 文档同步

- 已同步更新 `docs/bugs/README.md` 活跃表与已修复表。
- 本修复只收紧共享用户态文案净化边界，不改变模块边界、长期约束或运行工作流，无需更新 `docs/repo-map.md`、`docs/invariants.md` 或新增 handoff。
