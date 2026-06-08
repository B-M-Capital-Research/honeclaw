# Bug: Feishu 直聊批量返回 `hone-mcp binary not found` 内部错误

- 发现时间：2026-06-03 11:02 CST
- Bug Type：System Error
- 严重等级：P1
- 状态：Fixed
- GitHub Issue：[#48](https://github.com/B-M-Capital-Research/honeclaw/issues/48)

## 证据来源

- `data/sessions.sqlite3` 最近四小时真实会话窗口（`2026-06-03 07:01-11:01 CST`）：
  - `session_messages` 共有 14 个 Feishu user turn 与 14 个 Feishu assistant turn。
  - `2026-06-03T07:14-08:33+08:00` 仍有多条正常投研分析回复，说明直聊链路在本窗前段可工作。
  - `2026-06-03T10:32:12+08:00` 起，连续 7 个 Feishu direct assistant final 都返回同一内部错误：`hone-mcp binary not found near current executable; tried: <absolute-path>/hone-mcp, <absolute-path>/hone-mcp-aarch64-apple-darwin (set HONE_MCP_BIN to override)`。
  - 受影响 session：
    - `Actor_feishu__direct__ou_5f9e9e0bfe7deb3f65197e75892a377e21`：`10:32` 与 `10:47` 两次请求均返回该错误，其中一次是创建每日 10 点 IREN/NUAI 数据中心进展整理任务。
    - `Actor_feishu__direct__ou_5fe40dc70caa78ad6cb0185c21b53c4732`：`10:34` 文章深度分析请求与 `10:42` 继续追问均返回该错误。
    - `Actor_feishu__direct__ou_5ff7c950a113ff1bb9ecb1950f3cb1e37c`：`10:39` 港股买点咨询返回该错误。
    - `Actor_feishu__direct__ou_5fb47bd113e7776b05e7a5c2c56e310652`：`10:53` VITL 投资判断请求返回该错误。
    - `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`：`10:57` 图片附件请求返回该错误。
- `docs/bugs/` 去重搜索：
  - 没有 Feishu direct 同根因文档。
  - `docs/bugs/telegram_update_listener_connection_refused.md` 仅在 Telegram 历史证据里提过相同 `hone-mcp binary not found` 字符串，影响渠道和当前表现不同，不能复用为同一缺陷。
- `cron_job_runs.max(executed_at)` 仍停在 `2026-06-01T00:26:00.908925+08:00`；本轮没有新增 scheduler run 证据，因此本缺陷先限定为 Feishu direct 直聊链路。
- 最近四小时无非文档代码提交；本轮只维护缺陷台账。
- `2026-06-03 15:02 CST` 复核最近四小时真实会话窗口（`2026-06-03 11:02-15:02 CST`）：
  - `session_messages` 共有 20 个 Feishu user turn 与 20 个 Feishu assistant turn，整体没有孤立未回复 user turn。
  - `2026-06-03T11:30:18-12:18:23+08:00` 又有 9 个 Feishu direct assistant final 返回同一 `hone-mcp binary not found near current executable...` 原始错误，跨 5 个 direct session。
  - `2026-06-03T13:43-14:50+08:00` 出现 11 条非同类错误的正常 assistant final，说明当前运行态已有部分恢复迹象，但不能证明代码级修复已合入。
  - `cron_job_runs` 在该窗口无新增 run；最近四小时无非文档代码提交。
  - 本轮开始前存在 `bins/hone-cli/src/start.rs` 与 `crates/hone-channels/src/runtime.rs` 的未提交代码改动，内容分别涉及自动传递 `HONE_MCP_BIN` 与净化该原始错误；由于本自动化只允许维护 `docs/bugs/`，这些代码改动已按任务边界清理，不能作为 `Fixed` 依据。

## 端到端链路

1. Feishu 用户在 direct p2p 会话发起投研、文章分析、定时任务创建或图片附件处理请求。
2. channel runtime 接收消息并创建 / 续用 direct session。
3. agent runner 在处理请求前需要找到并启动 `hone-mcp` 或其平台变体。
4. 当前运行环境的可执行文件邻近目录找不到 `hone-mcp` / `hone-mcp-aarch64-apple-darwin`，且未通过 `HONE_MCP_BIN` 指定覆盖路径。
5. 上层将底层二进制定位失败原文作为 assistant final 落库并发给 Feishu 用户。
6. 用户请求没有进入实际工具 / agent 处理，任务正文、投资判断、定时任务创建确认或附件理解均未完成。

## 期望效果

- Feishu direct 运行环境应能稳定找到必要的 `hone-mcp` 二进制，或在启动前健康检查中阻止坏进程接流量。
- 即使本机二进制缺失，也应返回脱敏、稳定、用户可理解的系统不可用提示，而不是暴露内部 binary 名称、探测路径和环境变量提示。
- 失败态应记录为明确的 runner / dependency startup failure，便于值守和修复 agent 快速定位到打包、部署或运行路径问题。

## 当前实现效果

- `2026-06-03 10:32-10:57 CST` 连续 7 个真实 Feishu direct 请求都没有完成用户任务，只返回 `hone-mcp binary not found near current executable...`。
- `2026-06-03 11:30-12:18 CST` 又有 9 个真实 Feishu direct 请求继续只返回同类原始错误；这是本轮确认仍活跃的关键证据。
- 回复内容暴露内部可执行文件布局、平台变体名和 `HONE_MCP_BIN` 配置入口；虽然绝对路径已被 `<absolute-path>` 替换，但仍是面向工程实现的原始错误。
- 本窗前段同渠道可正常产出投研回复，后段连续失败；13:43 CST 之后又出现正常 Feishu direct final，说明当前运行态可能已被临时止血，但底层路径查找与错误净化仍需要代码级修复。

## 用户影响

- 这是功能性缺陷，不是 P3 质量问题：真实用户的投研分析、文章解读、定时任务创建和图片附件处理都没有得到可消费结果。
- 影响 Feishu direct 主链路，且在 25 分钟内跨 5 个 direct session 连续复现 7 次，具备批量影响特征。
- 用户可见回复暴露内部运行依赖和环境变量名称，会降低可信度，并可能误导用户认为需要自行处理服务端部署问题。
- 严重等级定为 P1：核心直聊能力在当前运行态批量不可用，同时存在内部错误外露。

## 根因判断

- 直接根因是当前 Feishu direct 运行环境找不到 `hone-mcp` 运行依赖，或者部署包 / 进程工作目录 / `HONE_MCP_BIN` 配置与 runner 查找规则不一致。
- 错误净化层没有覆盖 `hone-mcp binary not found near current executable` 这类 dependency startup failure，导致原始工程错误进入用户可见 assistant final。
- 这不同于已修复的 `Codex version probe 资源耗尽`：本轮不是瞬时资源限制或 `codex` version probe，而是 `hone-mcp` 二进制缺失 / 路径解析失败。

## 巡检更新

- `2026-06-03 12:31 CST` 有运行态止血尝试：重新构建 `hone-mcp`，并用显式 `HONE_MCP_BIN` 重启本机 `hone-console-page` 与 `hone-feishu`；部分健康检查与 13:43 后 Feishu direct normal final 表明运行态可能已恢复。
- 但本轮巡检确认，支撑“启动路径修复”和“用户可见错误净化”的代码仍只是未提交本地改动，不属于当前 `main`。因此状态不能登记为 `Fixed`，仍保持 `New`。
- 已有脱敏 GitHub Issue [#48](https://github.com/B-M-Capital-Research/honeclaw/issues/48)，本轮不重复创建。

## 修复记录

- `2026-06-08 12:08 CST` 追加错误边界加固：
  - `crates/hone-channels/src/runtime.rs` 的共享用户可见错误边界继续补齐 `hone-mcp` 启动依赖失败分类，新增识别 `binary not found` 与 `not found near current executable` 这类启动依赖缺失信号。
  - `user_visible_error_message` 与 `user_visible_error_message_or_none` 都会把该类错误映射为稳定的本机执行环境不可用文案，避免向 Feishu 用户暴露候选二进制路径、`HONE_MCP_BIN` 细节或其它工程化启动信息。
  - 新增回归 `user_visible_error_message_maps_hone_mcp_startup_errors` 与 `user_visible_error_message_or_none_keeps_hone_mcp_startup_errors_sanitized`，锁住 Feishu direct / scheduler 共用错误净化层的行为。
  - 本轮未重启当前 Feishu 服务，也不把当前机器运行态作为线上恢复证据；状态维持代码级 `Fixed`，建议正常部署/重启后复测。

- `2026-06-04` 已修复：
  - `bins/hone-cli/src/start.rs` 现在会在源码/CLI 启动链路里显式透传 `HONE_MCP_BIN`，优先把当前 root 下已定位到的 `hone-mcp` 二进制传给子进程，避免 Feishu / Web / scheduler runner 只依赖“当前可执行文件附近碰巧有 hone-mcp”。
  - `crates/hone-channels/src/runtime.rs` 现在把 `hone-mcp binary not found near current executable ... (set HONE_MCP_BIN to override)` 这类 dependency startup failure 统一映射为用户态文案 `当前本机执行环境暂时不可用，请稍后再试。`，不再向 Feishu 用户暴露二进制名、探测路径和环境变量。
  - 配合同轮 `execution.rs` / `mcp_bridge.rs` 的绝对配置路径与数据根透传，runner 子进程的启动/依赖环境边界已收口到代码，不再依赖手工脏改或临时 shell 环境。

## 验证

- `cargo test -p hone-cli child_envs_exports_hone_mcp_bin_from_source_root -- --nocapture`
- `cargo test -p hone-channels user_visible_error_message_rewrites_missing_hone_mcp_binary_errors -- --nocapture`
- `cargo check -p hone-channels -p hone-cli --tests`
- `cargo test -p hone-channels user_visible_error_message_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 后续关注

- 本轮没有重启当前 Feishu 服务做 live 复核；若后续仍出现同类错误，应优先核对实际启动入口是否经过 `hone-cli start` / desktop runtime env 物料化链路。
- 若运行态仍偶发 `hone-mcp` 缺失，应继续补启动前健康检查，把坏进程挡在接流量之前。
