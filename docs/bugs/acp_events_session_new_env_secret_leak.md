# Bug: ACP session/new 原始事件日志记录 MCP 环境凭据

## 发现时间

2026-07-01 23:02 CST

## Bug Type

System Error

## 严重等级

P1

## 状态

New

## GitHub Issue

[Issue #51](https://github.com/B-M-Capital-Research/honeclaw/issues/51)

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 2026-07-01 19:35-23:01 CST 窗口内检出 48 条 `session/new` ACP 事件。
  - 每条事件的 MCP server `env` payload 都包含云数据库与对象存储相关敏感字段名，覆盖 Web 与 Feishu actor。
  - 本文档只记录字段类别和计数，不记录任何真实值、手机号、账号、token、绝对本机路径或日志原文。
- 查重：
  - `docs/bugs/code-quality-patrol.md` 仅登记过 ACP parse-error raw protocol line 的日志质量风险；本轮证据是正常 `session/new` audit payload 系统性记录 MCP env，不属于同一具体坏点。
  - 既有用户可见 prompt / rawOutput 泄漏 P1 多数聚焦渠道出站；本轮没有证据显示这些凭据进入用户可见回复，但已经进入本地持久化运行日志。

## 端到端链路

1. Channel runtime 为每轮 Codex ACP 会话构造 `session/new` 请求。
2. 请求内包含 MCP server 启动配置，其中 `env` 承载云数据库、对象存储、运行模式和 actor scope 等环境变量。
3. ACP audit logger 将原始 `session/new` payload 写入 `acp-events.log`。
4. 日志成为后续巡检、排障、自动化 agent 和人工可读取的持久化材料。

## 期望效果

- ACP audit 日志不应持久化任何凭据值或可直接复用的访问密钥。
- 对 `session/new.params.mcpServers[].env` 应只保留安全字段、字段名白名单或脱敏后的摘要。
- 调试需要时，应通过受控开关写入受限 artifact，并默认关闭。

## 当前实现效果

- 最近四小时真实运行日志中，`session/new` 原始 payload 批量保留 MCP server env。
- 日志中可见的敏感字段类别包括云数据库连接凭据和对象存储访问凭据。
- 该问题不是单次模型输出质量波动，而是 audit logger 对正常 ACP 控制面事件的结构化记录策略问题。

## 用户影响

- 任何能读取 runtime 日志的本机进程、自动化 agent 或排障人员，都可能接触到可复用的后端访问凭据。
- 若日志被打包、上传、贴入 issue、交给外部 agent 或用于问题复现，凭据泄露面会从本机扩大到协作链路。
- 当前未见凭据进入最终用户回复或跨用户投递，但凭据进入持久化日志已经构成数据安全边界缺陷，因此定级为 P1。

## 根因判断

- ACP raw event audit 以完整原始 payload 为真相源持久化，缺少字段级 redaction。
- `session/new` payload 中 MCP server env 的敏感性高于普通 protocol metadata，但当前日志路径没有区分。
- 需要在写入 `acp-events.log` 前做结构化脱敏，而不是依赖后续巡检或人工不复制日志原文。

## 下一步建议

1. 在 ACP event logger 写入前，对 `session/new.params.mcpServers[].env[].value` 做默认脱敏，敏感字段只保留 `<redacted>`。
2. 对所有 env 字段应用 denylist + allowlist 双层策略：`PASSWORD`、`SECRET`、`KEY`、`TOKEN`、数据库连接、对象存储连接等默认不落明文。
3. 增加回归测试：构造带数据库和对象存储 env 的 `session/new` 事件，断言日志 JSON 中没有真实值。
4. 评估历史 `acp-events.log` 的轮转、删除或访问限制；如果这些凭据仍有效，应按内部流程轮换。
