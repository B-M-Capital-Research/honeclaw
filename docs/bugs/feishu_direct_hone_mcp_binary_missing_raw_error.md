# Bug: Feishu 直聊批量返回 `hone-mcp binary not found` 内部错误

- 发现时间：2026-06-03 11:02 CST
- Bug Type：System Error
- 严重等级：P1
- 状态：New
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
- 回复内容暴露内部可执行文件布局、平台变体名和 `HONE_MCP_BIN` 配置入口；虽然绝对路径已被 `<absolute-path>` 替换，但仍是面向工程实现的原始错误。
- 本窗前段同渠道可正常产出投研回复，后段连续失败，说明这是当前运行态刚发生变化的功能性缺陷，不是历史静态文档问题。

## 用户影响

- 这是功能性缺陷，不是 P3 质量问题：真实用户的投研分析、文章解读、定时任务创建和图片附件处理都没有得到可消费结果。
- 影响 Feishu direct 主链路，且在 25 分钟内跨 5 个 direct session 连续复现 7 次，具备批量影响特征。
- 用户可见回复暴露内部运行依赖和环境变量名称，会降低可信度，并可能误导用户认为需要自行处理服务端部署问题。
- 严重等级定为 P1：核心直聊能力在当前运行态批量不可用，同时存在内部错误外露。

## 根因判断

- 直接根因是当前 Feishu direct 运行环境找不到 `hone-mcp` 运行依赖，或者部署包 / 进程工作目录 / `HONE_MCP_BIN` 配置与 runner 查找规则不一致。
- 错误净化层没有覆盖 `hone-mcp binary not found near current executable` 这类 dependency startup failure，导致原始工程错误进入用户可见 assistant final。
- 这不同于已修复的 `Codex version probe 资源耗尽`：本轮不是瞬时资源限制或 `codex` version probe，而是 `hone-mcp` 二进制缺失 / 路径解析失败。

## 下一步建议

- 修复 Feishu direct runtime 的 `hone-mcp` 二进制打包 / 部署 / 查找路径，确认生产启动方式下 `HONE_MCP_BIN` 或邻近二进制存在。
- 在启动健康检查中验证 `hone-mcp` 可执行，失败时不要接收 Feishu direct 流量。
- 扩展共享用户可见错误净化，把 dependency startup failure 映射为脱敏系统不可用文案，并在日志 / metadata 中保留结构化失败原因。
- 修复后用 Feishu direct 的真实或本地 contract smoke 覆盖：普通文本投研、定时任务创建和图片附件请求至少各一条能正常进入 agent 或返回脱敏失败。
