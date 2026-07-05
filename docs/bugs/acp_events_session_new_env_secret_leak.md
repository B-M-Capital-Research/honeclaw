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

## 最新进展（2026-07-05 11:01 CST）

- 本轮按 `data/runtime/logs/acp-events.log` 事件时间复核 2026-07-05 07:02-11:01 CST live `session/new`：
  - 同窗 18 条 `session/new` 均携带 `mcpServers` 结构，共 210 个 env 条目。
  - 187 个 env value 已写成 `<redacted>`；未红掉条目只命中当前低敏布尔 / 开关字段，例如 `HONE_MCP_ALLOW_CRON` 与 `HONE_CLOUD_MODE`。
  - 未检出非白名单凭据值回退，也未发现该问题进入用户可见 assistant final。
- 状态继续维持 `Fixed`；历史日志清理 / 凭据轮换风险仍由 Issue #51 跟踪，本轮不重复创建 GitHub Issue。

## 最新进展（2026-07-05 08:27 CST）

- 本轮按 `tail -n 4000 data/runtime/logs/acp-events.log` 只读复核最新 live `session/new`：
  - 最近 9 条 `session/new` 共 98 个 env 条目，其中 87 个已写成 `<redacted>`。
  - 敏感条目未再检出未脱敏值；保留明文的仅是当前白名单内低敏字段。
  - 说明当前代码与 live 日志行为一致，旧巡检里把“仍保留 `mcpServers` / `env` 结构”直接等同于“值未脱敏”的结论不再成立。
- 本轮未重启服务；状态继续维持 `Fixed`，后续风险收敛为历史日志清理与凭据轮换，而不是当前 live 脱敏失效。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 2026-07-04 23:02-2026-07-05 03:02 CST 窗口按事件时间复核 44 条 `session/new` ACP 事件，覆盖 Feishu / Web / Discord actor；44 条仍保留 `mcpServers` / `env` 结构。
  - 本轮只记录结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径：44 条事件共 379 个 env 条目，其中 333 个非白名单值为 `<redacted>`，剩余 46 个未红掉条目均属于已允许低敏白名单（`HONE_CLOUD_MODE`、`HONE_CLOUD_ENABLED`、`HONE_CLOUD_STRICT_NO_LOCAL_STORAGE`、`HONE_MCP_ALLOW_CRON`、`HONE_MCP_MAX_TOOL_CALLS`、`HONE_MCP_ALLOWED_TOOLS`），未检出非白名单未红掉值。
  - 同窗 44 个 `session/prompt` 对应 43 个 `stopReason=end_turn` 与 1 个产品化 scheduler timeout 失败；未见 ACP response error，也未见该问题进入用户可见 assistant final。
  - 结论：当前 live `session/new` 日志路径已加载脱敏逻辑，不再持久化非白名单 env 明文值；状态从 `P1 / New` 调整为 `P1 / Fixed`。历史 `acp-events.log*` 已落盘旧凭据仍需单独清理 / 轮换，已有 Issue #51，本轮不重复创建。
- `data/runtime/logs/acp-events.log`
  - 2026-07-04 15:02-19:04 CST 窗口按事件时间继续检出 6 条 `session/new` ACP 事件，覆盖 Feishu direct 与 Web direct actor。
  - 本轮只做结构化计数与字段类别判断，不复制日志原文：6 条事件均仍携带 `mcpServers` / `env` 结构，`<redacted>` 计数为 0。
  - 同窗 6 个 prompt 中 5 个已有 `stopReason=end_turn`；另 1 个 Feishu direct 会话仍在持续输出 chunk，尚未收口。风险继续集中在 ACP audit 持久化边界，不是用户可见回复外泄。
  - 状态维持 `P1 / New`；已有 GitHub Issue #51，本轮不重复创建。
- `data/runtime/logs/acp-events.log`
  - 2026-07-03 19:02-23:02 CST 窗口按事件时间继续检出 23 条 `session/new` ACP 事件，全部仍携带 `session/new.params.mcpServers[].env` 字段。
  - 本轮只做结构化计数与字段类别判断，不复制日志原文：23 条事件累计 483 个 env entry，`<redacted>` 计数为 0。
  - 同窗 `data/sessions.sqlite3` 有 20:00 Web scheduler 与 21:35 / 23:00 Feishu scheduler 共 3 组 user / assistant final，均成对收口；assistant final 未命中 env 字段、raw tool 输出、provider 原始错误、panic 或本机绝对路径。风险继续集中在 ACP audit 持久化边界，不是用户可见回复外泄。
  - 状态维持 `P1 / New`；已有 GitHub Issue #51，本轮不重复创建。
- `data/runtime/logs/acp-events.log`
  - 2026-07-03 15:10-19:01 CST 窗口按事件时间继续检出 16 条 `session/new` ACP 事件，全部仍携带 `session/new.params.mcpServers[].env` 字段。
  - 本轮只做结构化计数与字段类别判断，不复制日志原文：`<redacted>` 计数为 0；同窗另有 16 次 `session/prompt` 与 16 个 `stopReason=end_turn`，未见 ACP response error、runner error、stream disconnect、quota、panic 或资源耗尽。
  - 同窗 `data/sessions.sqlite3` 只有 18:00 Web scheduler 1 个 user turn 与 1 条 assistant final，成对收口；风险继续集中在 ACP audit 持久化边界，不是用户可见回复外泄。
  - 状态维持 `P1 / New`；已有 GitHub Issue #51，本轮不重复创建。
- `data/runtime/logs/acp-events.log`
  - 2026-07-03 11:00-15:10 CST 窗口内继续检出 15 条 `session/new` ACP 事件，全部仍携带 `session/new.params.mcpServers[].env` 字段。
  - 本轮只做结构化计数与字段类别判断，不复制日志原文：`<redacted>` 计数为 0，按敏感字段名估算仍有 225 次敏感 env 字段命中。
  - 同窗 `data/sessions.sqlite3` 有 3 个 Feishu direct user turn 与 3 条 assistant final，均成对收口；风险继续集中在 ACP audit 持久化边界，不是用户可见回复外泄。
  - 状态维持 `P1 / New`；已有 GitHub Issue #51，本轮不重复创建。
- `data/runtime/logs/acp-events.log`
  - 2026-07-02 23:02-2026-07-03 03:03 CST 窗口内继续检出 9 条 `session/new` ACP 事件，全部晚于修复提交 `f4dc305d`。
  - 本轮只做结构化计数与字段类别判断，不复制日志原文：9 条事件累计 189 个 env entry，其中按敏感字段名估算至少 108 个云数据库 / 对象存储 / 凭据相关 env value 仍未红掉，`<redacted>` 计数为 0。
  - 同窗 `data/sessions.sqlite3` 有 1 个真实 Feishu direct user turn 与 1 条 assistant final，成对收口；风险继续集中在 ACP audit 持久化边界，不是用户可见回复外泄。
  - 状态维持 `New`；已有 GitHub Issue #51，本轮不重复创建。
- `data/runtime/logs/acp-events.log`
  - 2026-07-02 03:04 CST 代码修复提交 `f4dc305d fix: redact mcp env values in acp event logs` 之后，2026-07-02 03:04-07:02 CST 窗口仍检出 13 条 `session/new` ACP 事件。
  - 本轮只做结构化计数与字段类别判断，不复制日志原文：13 条事件累计 273 个 env entry，其中 247 个非低敏白名单 env entry 仍以未红掉值进入持久化事件日志，`<redacted>` 计数为 0。
  - 同窗可见 14 次 `session/prompt`、14 个 `stopReason=end_turn`，未见 ACP response error、runner error、stream disconnect、panic 或 context-window response error；风险继续集中在日志持久化边界，不是用户可见回复外泄。
  - 这批样本全部晚于代码修复提交，说明当前 live runtime 仍未加载修复，或修复未覆盖当前 `acp-events.log` 写入路径；状态维持 `New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建。
- `data/runtime/logs/acp-events.log`
  - 2026-07-01 23:01-2026-07-02 03:02 CST 窗口内再次检出 17 条 `session/new` ACP 事件。
  - 本轮只做结构化计数与字段类别判断，不复制日志原文：17 条事件均包含 MCP server `env` payload，累计 357 个 env entry；除低敏白名单外，仍有 323 个非白名单 env entry 以未红掉值进入持久化事件日志。
  - 同窗可见 17 次 `session/prompt`、18 个 `stopReason=end_turn`，未见 response error、runner error、stream disconnect、panic、quota 或 context-window ACP response error；风险集中在日志持久化边界，不是用户可见回复外泄。
  - 该样本晚于 2026-07-02 03:03 CST 代码级修复记录，说明当前 live runtime 仍未加载修复或修复未覆盖当前事件日志路径；状态从 `Fixed` 回退为 `New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建。
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

- 2026-07-02 03:02 CST 运行态复核显示，当时 `acp-events.log` 仍会持久化 `session/new.params.mcpServers[].env` 的非白名单未红掉值。
- 2026-07-03 07:00 CST 运行态复核显示，当前 live `session/new` 事件仍包含 `mcpServers[].env` 字段，但本窗未检出 env entry 明文值。
- 2026-07-03 11:05 CST 运行态复核显示，07:00-11:05 CST 新增 `session/new` 事件又出现未红掉的敏感 env value，说明旧修复只覆盖了单一 `env` 结构假设。
- 2026-07-04 03:04 CST 代码层继续收紧 `acp-events.log` 净化：`session/new` 现在会同时覆盖 `mcpServers` 数组与对象映射两种形状，并对 `env` 的数组条目 / 对象映射统一做白名单外值脱敏。
- 2026-07-04 07:03 CST 运行态复核显示，03:08 CST 加固提交之后的 live `session/new` 事件仍有 Feishu 样本携带 `mcpServers[].env`，且 `<redacted>` 计数为 0；说明当时运行进程仍未加载修复，或仍存在未覆盖的日志写入路径。
- 2026-07-05 03:02 CST 运行态复核显示，最新 live `session/new` 事件仍保留 `mcpServers/env` 结构，但非白名单 env value 已统一脱敏，未检出非白名单明文值；当前按运行态 `Fixed` 记录。
- `params.mcpServers[].env` 现在默认不保留未知 env 明文值；仅 `HONE_CLOUD_MODE`、`HONE_CLOUD_ENABLED`、`HONE_CLOUD_STRICT_NO_LOCAL_STORAGE`、`HONE_MCP_ALLOW_CRON`、`HONE_MCP_MAX_TOOL_CALLS`、`HONE_MCP_ALLOWED_TOOLS` 保留原值，其余统一写成 `<redacted>`。
- 新增日志回归，分别覆盖数组条目形状与对象映射形状的云数据库凭据、对象存储凭据和本地数据目录路径，断言不会进入持久化 JSONL。

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
4. 2026-07-04 03:04 CST 进一步把净化逻辑扩到 `mcpServers` 对象映射与 `env` 对象映射，避免不同 runner / ACP 序列化形状绕过已有数组分支。
5. 新增 `log_acp_payload_redacts_session_new_mcp_env_object_map_values` 回归，锁住对象映射形状同样不会落盘敏感 env value。

## 验证

- `cargo test -p hone-channels log_acp_payload_redacts_session_new_mcp_env_values -- --nocapture`
- `cargo test -p hone-channels log_acp_payload_redacts_session_new_mcp_env_object_map_values -- --nocapture`
- `cargo test -p hone-channels parse_error_log_records_bounded_redacted_raw_line_preview -- --nocapture`
- `cargo check -p hone-channels --tests`
- `rustfmt --edition 2024 --check crates/hone-channels/src/runners/acp_common/log.rs`
- `git diff --check`

## 后续风险

1. 历史 `data/runtime/logs/acp-events.log*` 里已落盘的旧凭据不会被代码修复自动清除；如这些凭据仍有效，仍需按内部流程轮换并清理旧日志。
2. 2026-07-05 03:02 CST 当前 live 路径已确认非白名单 env value 被红掉；若后续真实窗口再次出现 `<redacted>` 计数为 0 或非白名单未红掉值，应从 `Fixed` 回退为 `New` 并继续沿 Issue #51 跟进。

## 最新运行态复核（2026-07-02 11:01 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-02 07:01-11:01 CST。
  - 代码修复提交 `f4dc305d fix: redact mcp env values in acp event logs` 之后，当前 live 日志继续检出 31 条 `session/new` ACP 事件，覆盖 Feishu 22 条、Web 8 条、Discord 1 条。
  - 本轮只记录结构化计数，不复制日志原文或任何 env 值：31 条事件累计 652 个 env entry，其中 590 个非低敏白名单 env entry 仍未红掉，`<redacted>` 计数为 0。
  - 同窗可见 31 次 `session/prompt`、32 个 `stopReason=end_turn`、0 个 ACP response error；风险仍集中在日志持久化边界，不是用户可见回复外泄。
- 本轮判断
  - live runtime 仍未加载修复，或修复仍未覆盖当前 `acp-events.log` 写入路径；状态维持 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建。

## 最新运行态复核（2026-07-02 15:01 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-02 11:01-15:01 CST。
  - 代码修复提交 `f4dc305d fix: redact mcp env values in acp event logs` 之后，当前 live 日志继续检出 9 条 `session/new` ACP 事件。
  - 本轮只记录结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径：9 条事件累计 189 个 env entry，189 个非低敏白名单 env entry 仍未红掉，`<redacted>` 计数为 0。
  - 同窗直聊 / scheduler ACP prompt 均能继续收口，未见该问题进入用户可见回复；风险仍集中在持久化日志边界。
- 本轮判断
  - live runtime 仍未加载修复，或修复仍未覆盖当前 `acp-events.log` 写入路径；状态维持 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建。

## 最新运行态复核（2026-07-02 19:03 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-02 15:01-19:03 CST。
  - 代码修复提交 `f4dc305d fix: redact mcp env values in acp event logs` 之后，当前 live 日志继续检出 7 条 `session/new` ACP 事件。
  - 本轮只记录结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径：7 条事件累计 147 个 env entry，其中 91 个非低敏白名单 env entry 仍未红掉，`<redacted>` 计数为 0。
  - 同窗可见 7 次 `session/prompt`、28 个 ACP response，未见 response error、runner error、stream disconnect、panic、quota 或 context-window ACP response error；风险仍集中在持久化日志边界，不是用户可见回复外泄。
- 本轮判断
  - live runtime 仍未加载修复，或修复仍未覆盖当前 `acp-events.log` 写入路径；状态维持 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建。

## 最新运行态复核（2026-07-02 23:03 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-02 19:02-23:02 CST。
  - 代码修复提交 `f4dc305d fix: redact mcp env values in acp event logs` 之后，当前 live 日志继续检出 51 条 `session/new` ACP 事件。
  - 本轮只记录结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径：51 条事件累计 1071 个 env entry；按当前低敏白名单估算，969 个非低敏 env entry 仍未红掉，`<redacted>` 计数为 0。
  - 同窗 direct / scheduler ACP 会话仍有 `stopReason=end_turn` 收口样本，未见该问题进入用户可见回复；风险仍集中在持久化日志边界。
- 本轮判断
  - live runtime 仍未加载修复，或修复仍未覆盖当前 `acp-events.log` 写入路径；状态维持 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建。

## 最新运行态复核（2026-07-03 07:00 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-03 03:00-07:00 CST。
  - 本窗检出 5 条 `session/new` ACP 事件，覆盖 Web 3 条、Feishu 2 条；`params.mcpServers` 仍存在且每条包含 `env` 字段。
  - 本轮只做结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径：5 条事件未再检出可枚举 env entry 明文值，敏感字段名命中为 0，`<redacted>` 计数为 0。
  - 同窗可见 5 次 `session/prompt`，4 个 `stopReason=end_turn`，另有 1 个 scheduler runner timeout；未见该问题进入用户可见回复。
- 本轮判断
  - 与 2026-07-02 多个窗口持续检出未红掉 env value 相比，本窗 `session/new` 不再暴露 env 明文值，说明 `f4dc305d` 的日志净化已在当前 live 路径生效。
  - 状态从 `P1 / New` 调整为 `Fixed`；历史 `acp-events.log*` 已落盘凭据仍需单独清理 / 轮换，因此暂不关闭。
  - 已有关联 GitHub Issue #51，本轮不重复创建。

## 最新运行态复核（2026-07-03 11:05 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-03 07:00-11:05 CST。
  - 本窗检出 22 条 `session/new` ACP 事件，覆盖 Web 5 条、Feishu 16 条、Discord 1 条；每条仍包含 MCP server `env` 字段。
  - 本轮只记录结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径：22 条事件累计 463 个 env entry，`<redacted>` 计数为 0；按敏感字段名估算 286 个敏感字段名命中，其中 264 个非白名单敏感 env value 仍未红掉。
  - 同窗 `data/sessions.sqlite3` 有 7 个 user turn 与 7 条 assistant final，均成对收口；assistant final 污染扫描未命中 env 字段、raw tool 输出、provider 原始错误、panic 或本机绝对路径。风险仍集中在 ACP audit 持久化边界，不是用户可见回复外泄。
- 本轮判断
  - 07:00 CST 将该缺陷调整为 `Fixed` 的结论被新窗口运行态推翻；当前 live 日志路径仍会持久化未红掉的敏感 env value，状态从 `P1 / Fixed` 回退为 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建；issue 内容已覆盖该 P1，后续修复应优先确认 live runtime 与实际写入路径是否加载同一净化逻辑。

## 最新运行态复核（2026-07-04 03:05 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-03 23:02-2026-07-04 03:05 CST。
  - 本窗检出 5 条 `session/new` ACP 事件，5 条仍包含 MCP server `env` 字段。
  - 本轮只记录结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径：5 条事件估算 84 个 env key，`<redacted>` 计数为 0。
  - 同窗 `data/sessions.sqlite3` 只有 3 个 Feishu scheduler user turn 与 3 条 assistant final，均成对收口；assistant final 污染扫描未命中 env 字段、raw tool 输出、provider 原始错误、panic、本机绝对路径或 `mcpServers`。
- 本轮判断
  - 最新 live 日志仍显示 `session/new` 会持久化 MCP env 结构，且没有红线替换命中；这批样本发生在本轮 03:04 代码修复前的旧运行进程上，说明当时 live 尚未加载更宽的 payload-shape 脱敏。
  - 本轮已把代码级状态更新为 `P1 / Fixed`；风险仍集中在 ACP audit 持久化边界，不是用户可见回复外泄。
  - 已有关联 GitHub Issue #51，本轮不重复创建；待后续巡检在新代码实际加载后确认不再出现未红掉 env value。

## 最新运行态复核（2026-07-04 07:03 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-04 03:02-07:03 CST，重点复核 03:08 CST 加固提交 `d05fc8db fix: harden acp session-new log redaction` 之后的真实运行态。
  - 03:08 CST 之后仍检出 3 条新的 Feishu `session/new` ACP 事件，均包含 `mcpServers[].env`；每条结构化计数为 1 个 MCP server、1 个 env 容器、21 个 env name，`<redacted>` 为 false。
  - 本轮只记录结构化计数与字段类别，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径；字段类别覆盖 actor / channel scope、cloud runtime、数据库连接、对象存储和本地数据目录相关 env。
  - 同窗 `data/sessions.sqlite3` 只有 06:00-06:01 CST Feishu scheduler 1 组 user / assistant final，成对收口；assistant final 未命中 env 字段、raw tool 输出、provider 原始错误、panic、本机绝对路径或 `mcpServers`。风险仍集中在 ACP audit 持久化边界，不是用户可见回复外泄。
- 本轮判断
  - 03:06 CST 代码级 `Fixed` 结论被 03:08 之后的新运行态样本推翻；当前 live 日志路径仍会持久化未红掉的 MCP env value，状态从 `P1 / Fixed` 回退为 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建；issue 内容已覆盖该 P1，后续修复应优先确认 live runtime 是否已重启到 `d05fc8db`，以及是否还有绕过 `sanitize_acp_payload_for_log(...)` 的写入路径。

## 最新运行态复核（2026-07-04 11:01 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-04 07:01-11:01 CST。
  - 本窗检出 23 条 `session/new` ACP 事件，23 条均仍包含 `mcpServers[].env` 字段；结构化计数约 507 个 env name-like 字段，`<redacted>` 计数为 0。
  - 本轮只记录结构化计数与字段类别，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径；字段类别继续覆盖 actor / channel scope、cloud runtime、数据库连接、对象存储和本地数据目录相关 env。
  - 同窗 `data/sessions.sqlite3` 有 10:00 / 10:30 Feishu scheduler 与 10:29 Feishu direct 共 3 组 user / assistant final，均成对收口；assistant final 未命中 env 字段、raw tool 输出、provider 原始错误、panic、本机绝对路径或 `mcpServers`。风险仍集中在 ACP audit 持久化边界，不是用户可见回复外泄。
- 本轮判断
  - 最新 live 日志仍会持久化未红掉的 MCP env 结构，说明 03:08 CST 加固提交之后真实运行态仍未加载修复，或仍存在绕过净化函数的写入路径；状态维持 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建；后续修复应优先确认 live runtime 版本、`session/new` 写入路径与历史日志清理 / 凭据轮换。

## 最新运行态复核（2026-07-04 15:02 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-04 11:02-15:02 CST。
  - 本窗检出 2 条 `session/new` ACP 事件，2 条均仍包含 `mcpServers[].env[]`；结构化计数为每条 1 个 MCP server、21 个 env 条目，`<redacted>` 计数为 0。
  - 本轮只记录结构化计数与字段类别，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径；字段类别仍覆盖 actor / channel scope、cloud runtime、数据库连接、对象存储和本地数据目录相关 env。
  - 同窗 `data/sessions.sqlite3` 只有 13:35 Feishu direct 1 组 user / assistant final，成对收口；assistant final 未命中 env 字段、raw tool 输出、provider 原始错误、panic、本机绝对路径或 `mcpServers`。风险仍集中在 ACP audit 持久化边界，不是用户可见回复外泄。
- 本轮判断
  - 最新 live 日志仍会持久化未红掉的 MCP env 结构；状态维持 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建；后续修复仍应优先确认 live runtime 版本、`session/new` 写入路径与历史日志清理 / 凭据轮换。

## 最新运行态复核（2026-07-04 23:02 CST）

- `data/runtime/logs/acp-events.log`
  - 巡检窗口：2026-07-04 19:01-23:02 CST。
  - 本窗检出 20 条 `session/new` ACP 事件，20 条均仍包含 `mcpServers` / `env` 结构，`<redacted>` 计数为 0。
  - 本轮只记录结构化计数，不复制日志原文、env 值、账号、手机号、token 或绝对本机路径；同窗 20 个 prompt 均以 `stopReason=end_turn` 收口，未见该问题进入用户可见 assistant final。
- 本轮判断
  - 最新 live 日志仍会持久化未红掉的 MCP env 结构；状态维持 `P1 / New`。
  - 已有关联 GitHub Issue #51，本轮不重复创建；后续修复仍应优先确认 live runtime 版本、`session/new` 写入路径与历史日志清理 / 凭据轮换。
