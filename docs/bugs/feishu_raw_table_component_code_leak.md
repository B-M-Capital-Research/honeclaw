# Bug: Feishu 消息向用户外泄 raw table 组件代码

## 发现时间

2026-07-19 11:01 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

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

- 用户真实收到过 raw table 组件代码，并明确感知为“代码”污染。
- 同一用户在 09:13 CST 反馈后，assistant 仅说明这是后端渲染管道问题，并建议找管理员。
- 本轮窗口其他 Feishu/Web direct 会话均有 assistant 收口；assistant final 污染扫描未命中空回复、`<think>`、本机路径、`data_fetch`、`cron_job`、tool 字段或 provider 原始错误。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 主消息生成、会话落库和 Feishu 直聊收口仍然完成；用户仍能从 raw table 代码中大致读到表格字段和部分内容。
- 但用户看到内部组件代码会显著降低可读性和产品可信度，也暴露了不该出现的结构化渲染细节。
- 由于当前证据没有显示错投、未回复、数据破坏、全渠道不可用或敏感凭据泄漏，因此不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 初步判断出站渲染层对 raw table 组件缺少统一净化 / 降级边界，或表格组件生成与 Feishu 客户端支持能力之间存在协议不匹配。
- 该问题可能发生在 scheduler / heartbeat 消息先生成 table 组件、再被 Feishu 普通文本或卡片正文承载的路径上。
- 用户反馈“更新过后每次发过来都有这些代码”，说明问题可能不是单次模型输出，而是某次表格投递策略或渲染代码变更后的稳定退化。

## 下一步建议

1. 在 Feishu 出站最低发送边界统一拦截 `<table`、`columns=`、`dataIndex`、`data={` 等 table 组件源码，并转换为分行纯文本。
2. 检查 scheduler / heartbeat 与 direct reply 是否共用同一 Markdown / Feishu 卡片预处理路径，避免某些 fallback 或 direct text 分支绕过净化。
3. 为用户贴出的 POET / SIVE table payload 增加回归样本，覆盖 raw table、Markdown 表格、截断 table 和长表格分段。
4. 修复后用真实 Feishu 消息复核用户侧不再看到 raw component code，再把状态更新为 `Fixed`。
