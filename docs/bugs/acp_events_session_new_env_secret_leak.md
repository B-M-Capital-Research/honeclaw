# Bug: ACP session/new 原始事件日志记录 MCP 环境凭据

## 发现时间

2026-07-01 23:02 CST

## Bug Type

System Error

## 严重等级

P1

## 状态

Fixed

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

- 2026-07-02 已在 `acp-events.log` 写入前对 `session/new` payload 做结构化脱敏。
- `params.mcpServers[].env` 现在默认不保留未知 env 明文值；仅 `HONE_CLOUD_MODE`、`HONE_CLOUD_ENABLED`、`HONE_CLOUD_STRICT_NO_LOCAL_STORAGE`、`HONE_MCP_ALLOW_CRON`、`HONE_MCP_MAX_TOOL_CALLS`、`HONE_MCP_ALLOWED_TOOLS` 保留原值，其余统一写成 `<redacted>`。
- 新增日志回归，覆盖云数据库凭据、对象存储凭据和本地数据目录路径三类敏感值，断言不会进入持久化 JSONL。

## 用户影响

- 任何能读取 runtime 日志的本机进程、自动化 agent 或排障人员，都可能接触到可复用的后端访问凭据。
- 若日志被打包、上传、贴入 issue、交给外部 agent 或用于问题复现，凭据泄露面会从本机扩大到协作链路。
- 当前未见凭据进入最终用户回复或跨用户投递，但凭据进入持久化日志已经构成数据安全边界缺陷，因此定级为 P1。

## 根因判断

- ACP raw event audit 以完整原始 payload 为真相源持久化，缺少字段级 redaction。
- `session/new` payload 中 MCP server env 的敏感性高于普通 protocol metadata，但当前日志路径没有区分。
- 需要在写入 `acp-events.log` 前做结构化脱敏，而不是依赖后续巡检或人工不复制日志原文。

## 修复情况

1. `crates/hone-channels/src/runners/acp_common/log.rs` 新增 `sanitize_acp_payload_for_log(...)`，只对 `session/new` 做专项日志净化，避免影响其它 ACP 调试记录。
2. `session/new.params.mcpServers[].env[].value` 改为默认红掉，未知 env 不再以明文进入 `acp-events.log`。
3. 新增 `log_acp_payload_redacts_session_new_mcp_env_values` 回归，验证日志 JSON 中只保留安全白名单值。

## 验证

- `cargo test -p hone-channels log_acp_payload_redacts_session_new_mcp_env_values -- --nocapture`
- `cargo test -p hone-channels parse_error_log_records_bounded_redacted_raw_line_preview -- --nocapture`
- `cargo check -p hone-channels --tests`

## 后续风险

1. 历史 `data/runtime/logs/acp-events.log*` 里已落盘的旧凭据不会被代码修复自动清除；如这些凭据仍有效，仍需按内部流程轮换并清理旧日志。
2. 本轮未重启 live 服务、未对当前旧日志做在线验证，因此先记代码级 `Fixed`；若新窗口仍见明文 env，应重新打开该缺陷。
