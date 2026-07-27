# Bug: Feishu 消息向用户外泄 raw table 组件代码

## 发现时间

2026-07-19 11:01 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

Fixed (code-level; live recheck pending)

## GitHub Issue

无，非 P1

## 证据来源

- 2026-07-27 用户再次反馈并提供飞书截图：
  - “四、机构评级与目标价”与“五、本周关键日历”下方直接显示 `<table columns={[...]}` / `data={[...]}/>` 源码。
  - 本次标签结构完整，列名为 `dataIndex` / `title`，行数据为 `col0`、`col1` 等，和 `bins/hone-feishu/src/markdown.rs` 把标准 Markdown 表格转换成 raw table 字符串的输出完全一致。
  - 这说明 2026-07-19 的共享输出清洗只覆盖“模型正文已经含 raw table”的情况，无法覆盖清洗之后由飞书出站层新生成的 raw table。
- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-07-19 07:01-11:01 CST。
  - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`
  - user `ordinal=13`，`timestamp=2026-07-19T09:13:29.190793+08:00`，用户明确反馈“更新过后每次发过来都有这些代码”，并贴出以 `<table columns={[...]}` / `data={[...]}` / `dataIndex` 组成的 POET 心跳检查表格组件代码。
  - assistant `ordinal=14`，`timestamp=2026-07-19T09:13:45.986652+08:00`，承认这是系统后端把结构化数据或表格定义当作文本发出，并提示检查消息渲染层。
- `data/runtime/logs/web.log.2026-07-19`
  - `2026-07-19 09:13:29` Feishu `MsgFlow` 记录同一用户输入，`input.preview` 中可见 raw table 组件开头。
  - `2026-07-19 09:13:46` 同一会话 `success=true`、`reply.send segments.sent=1/1`，说明用户反馈被正常收口，不是未回复或投递失败。
- 本轮去重检查：
  - 已搜索 `docs/bugs/*.md` 中的 `table`、`columns=`、`dataIndex`、`raw table`、`结构化数据直接泄漏` 等关键词；未发现同一 Feishu raw table 组件外泄链路的独立活跃文档。
  - 既有 `scheduler_heartbeat_trigger_json_payload_leak.md` 覆盖 heartbeat JSON / 字段残片，`feishu_scheduler_data_fetch_tool_name_exposed.md` 覆盖内部工具名外露；本缺陷的用户可见形态是 Feishu 私有 table 组件代码原样出现在消息正文，属于新的独立格式渲染链路。

## 端到端链路

1. Scheduler / heartbeat 或 Feishu 出站链路生成包含表格的用户可见消息。
2. 中间渲染层没有把内部 table 组件转换为 Feishu 客户端可稳定显示的结构，也没有降级为普通文本列表。
3. 用户在 Feishu 侧看到 raw `<table columns=... data=.../>` 风格代码，并在直聊里反馈。
4. 直聊 assistant 正常回复了反馈，但没有自动登记或修复出站渲染问题；本轮由 `bug` 巡检建档。

## 期望效果

- 用户侧只应看到可读的普通文本、Markdown 或 Feishu 客户端可正确渲染的内容。
- 内部组件代码、`columns` / `data` / `dataIndex` 字段和 JSON 风格结构不应进入用户可见正文。
- 当表格结构无法稳定渲染时，应统一降级为分行纯文本或清晰的省略提示，而不是原样输出组件源码。

## 当前实现效果

- 标准 Markdown 表格现在会被解析成飞书卡片 JSON 2.0 根节点的原生 `table` 元素，不再先改写成 Markdown 正文中的 `<table .../>` 字符串。
- 历史完整 raw table 标签会迁移成相同的原生表格；损坏标签会替换为可读错误提示，不再显示组件源码。
- 单卡超过飞书上限的表格会按完整表格边界拆到后续卡片；Markdown-only / 旧流式路径会降级成可读列表。
- direct、scheduler 与 placeholder 最终更新路径共用同一原生卡片渲染入口。
- 本轮只完成代码级修复与自动化验证，未重启线上进程，也未向真实用户发送测试消息。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 主消息生成、会话落库和 Feishu 直聊收口仍然完成；用户仍能从 raw table 代码中大致读到表格字段和部分内容。
- 但用户看到内部组件代码会显著降低可读性和产品可信度，也暴露了不该出现的结构化渲染细节。
- 由于当前证据没有显示错投、未回复、数据破坏、全渠道不可用或敏感凭据泄漏，因此不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 2026-07-27 已确认直接根因：`bins/hone-feishu/src/markdown.rs::convert_table_to_feishu` 把标准 Markdown 表格序列化成 `<table columns={...} data={...}/>` 字符串，随后 `render_outbound_messages` 把该字符串放进 JSON 2.0 卡片的 `markdown.content`。飞书原生表格要求使用卡片根节点的 `{"tag":"table","columns":[...],"rows":[...]}` 元素，内联 raw table 字符串不会被 Markdown 元素解释，因而原样展示。
- 既有 `normalize_raw_feishu_table_tag` 还把结构完整的 raw tag 当作合法结果保留，导致护栏只能转义损坏标签，不能阻止完整标签在飞书端显示源码。
- 该问题可能发生在 scheduler / heartbeat 消息先生成 table 组件、再被 Feishu 普通文本或卡片正文承载的路径上。
- 用户反馈“更新过后每次发过来都有这些代码”，说明问题可能不是单次模型输出，而是某次表格投递策略或渲染代码变更后的稳定退化。

## 下一步建议

1. 新版本部署后，用一条 direct 标准 Markdown 表格和一条 scheduler 表格做真实 Feishu 复核，确认客户端显示原生表格且不再出现 raw `<table .../>`。
2. 若真实客户端仍存在兼容性问题，保留当前结构化数据解析，只把原生表格输出切换为已经覆盖测试的可读列表降级，不恢复内联组件源码。

## 修复记录

- 2026-07-27 根因级修复：
  - `bins/hone-feishu/src/markdown.rs` 在消息拆分前解析标准 Markdown 表格和历史 raw table，生成卡片根节点 JSON 2.0 `table` 元素；列使用兼容性更高的纯文本类型。
  - 单卡遵守最多 5 个原生表格、最多 50 列的协议边界；表格按原子块拆分，损坏或 Markdown-only 场景降级为可读文本，不再向用户展示 raw tag。
  - `bins/hone-feishu/src/outbound.rs` 的 placeholder 最终更新复用同一渲染器，避免更新路径重新把表格塞回 `markdown.content`。
  - `crates/hone-channels/src/prompt.rs` 明确要求模型只输出标准 Markdown 表格，由运行时转换成飞书 JSON 2.0 原生表格组件。
  - 回归覆盖用户截图中的 `dataIndex` / `col0` 结构、标准 Markdown 表格、历史 raw table、损坏标签、单卡 5 表上限与 placeholder 更新。
- 2026-07-19 代码级修复：
  - `crates/hone-channels/src/runtime.rs` 新增 raw table 组件识别与统一降级；当用户可见正文出现 `<table .../>` 且含 `columns=` / `dataIndex` / `data={` 这类内部组件字段时，统一替换为 `表格内容展示异常，请稍后重试。`，避免把 Feishu table 组件源码直接投给用户。
  - 该修复走共享 `sanitize_user_visible_output(...)`，因此 direct reply 与 scheduler delivery 共用同一边界，不需要分别加一套 Feishu 专属清洗逻辑。
  - 新增回归：
    - `sanitize_user_visible_output_rewrites_raw_table_component_copy`
    - `scheduler_delivery_text_rewrites_raw_table_component_copy`

## 验证

- 通过：`cargo test -p hone-feishu markdown -- --nocapture`（18 个相关测试）
- 通过：`cargo test -p hone-feishu outbound -- --nocapture`（6 个相关测试）
- 通过：`cargo test -p hone-feishu -- --nocapture`（69 个测试）
- 通过：`cargo test -p hone-channels prompt --lib -- --nocapture`（55 个相关测试）
- 通过：`cargo check -p hone-feishu --tests`
- 通过：`cargo check -p hone-channels --tests`
- 通过：既有 `sanitize_user_visible_output_rewrites_raw_table_component_copy` 与 `scheduler_delivery_text_rewrites_raw_table_component_copy` 回归。
- 通过：本次 Rust 文件的 scoped `rustfmt --check` 与相关文件 `git diff --check`。
- 仓库包装命令 `bash scripts/ci/check_fmt_changed.sh` 仍会检查 `HEAD^...HEAD` 中与本修复无关的既有 Rust 文件，并因那些基线格式差异失败；它没有报告本次新增的 `markdown.rs` / `outbound.rs` 工作树改动。未为此改写用户的无关变更。
- 未执行：真实 Feishu 实发、运行时重启与部署。
