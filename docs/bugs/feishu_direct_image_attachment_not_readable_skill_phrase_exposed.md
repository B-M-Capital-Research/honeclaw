# Bug: Feishu 直聊图片附件未稳定进入可读链路且外露内部技能状态

## 发现时间

- 2026-06-09 19:03 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- New

## GitHub Issue

- 无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 巡检窗口：2026-06-09 15:03-19:03 CST。
  - 本窗共有 35 个 user turn 与 35 个 assistant turn；最近活跃 Feishu direct session 均以 assistant final 收口，没有未回复或空回复。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、`company_profiles/...`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、quota、panic、stream disconnect、`enabled=true/false`、`HONE_MCP_BIN` 或 `data/portfolio`。
  - Feishu direct session `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3` 在 2026-06-09 16:41 CST 与 16:47 CST 连续两次收到用户请求摘要：`用大白话分析给我看`。
  - 2026-06-09 16:42 CST assistant final 回复没有拿到可解析附件内容，并写出“图片理解工具也没有成功激活”。
  - 2026-06-09 16:47 CST assistant final 再次回复没有拿到附件可读内容，并写出“图片分析技能也没成功加载”。
  - 2026-06-09 16:53 CST 同一 session 出现一条空文本 user turn，16:55 CST assistant 才成功读取图片并给出大白话分析，说明前两轮没有完成用户请求。
- `data/runtime/logs/acp-events.log`
  - 同一 session 在 16:41、16:47、16:53 CST 三轮 prompt 均以 `stopReason=end_turn` 收口，没有 runner error、stream disconnect、quota、HTTP 400/429、panic 或 provider 原始错误。
  - 结构化抽取显示 16:41 与 16:47 两轮 prompt 都带 `attachments=1 buffered_messages=0`，但最终仍声称没有可解析附件。
  - 16:53 CST 后同一会话成功输出图片内容分析，说明故障不是 Feishu direct 全局不可用，而是同一图片附件上下文在前两轮没有稳定进入可读/理解链路。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 本窗普通 scheduler 仅 1 条 `A股港股收盘后跨市场复盘`，状态为 `completed + sent + delivered=1`。
  - heartbeat 仍有 30 条 `PlainTextSuppressed`、4 条 `ContextOverflowError`、2 条 `JsonMalformed`、2 条 `JsonUnknownStatus` 等旧/已知运行态信号；这些未进入用户可见 assistant final，落在既有 heartbeat 结构化/context overflow 文档范围。
- 最近四小时无非文档代码提交。

## 端到端链路

1. Feishu direct 用户发送带附件上下文的图片分析请求。
2. Feishu ingress / prompt 构造把本轮送入 ACP runner，prompt metadata 显示 `attachments=1`。
3. Runner 正常执行并 `end_turn`，但 answer 阶段没有读取到可解析附件内容。
4. 用户可见回复要求用户重新发图或粘贴文字，并暴露“图片理解工具 / 图片分析技能没有成功激活”。
5. 用户再次发送后，第三轮才成功产出图片内容分析。

## 期望效果

- Feishu direct 图片附件应稳定进入共享附件 ingest / OCR / 图片理解路径，使用户一次发送图片后即可得到图片内容分析。
- 若附件读取失败，用户可见文案应是产品化提示，例如“当前未能读取这张图片，请重新上传或粘贴文字”，不应暴露内部工具、技能激活状态或运行编排。
- 同一附件上下文在短时间内不应前两轮不可读、第三轮才可读，除非有明确、可审计的用户侧重新上传或系统侧附件状态变化。

## 当前实现效果

- 前两轮 prompt 已带 `attachments=1`，但 answer 阶段仍认为没有可解析附件内容。
- 用户可见回复没有完成“分析这张图”的核心任务，只给出绕路建议。
- 回复同时外露内部能力状态：“图片理解工具没有成功激活”“图片分析技能没成功加载”。
- 第三轮同一 session 才成功给出图片分析，说明问题具备短时不稳定性，不是简单的用户问题不清楚。

## 用户影响

- 这是功能性 bug，不是单纯表达质量问题。
- 用户上传/引用图片后，核心任务是读取图片并分析；前两轮没有完成，迫使用户重复发送或改为粘贴文字。
- 同时暴露内部技能状态，降低用户对附件能力的信任。
- 定级为 `P2`：影响 Feishu direct 图片附件理解链路，但本窗只有一个 Feishu direct session 的两次失败后续成功恢复；没有跨用户大面积不可用、错投、数据破坏、敏感路径外泄或系统级未回复证据，因此不是 `P1`。

## 根因判断

- 直接证据只能确认：Feishu prompt metadata 已带 `attachments=1`，但 answer 阶段没有拿到可解析图片内容，并把内部图片工具/技能状态写入 final。
- 与 `web_direct_image_attachment_not_readable_internal_debug_leak.md` 相似，都是图片附件未进入可读/OCR链路并外露内部排障口径；但该旧文档覆盖 Web public direct 上传链路，且已在 Web API / shared attachment ingest 上修复。本轮受影响链路是 Feishu direct，不能复用 Web-only 修复结论。
- 与 `web_scheduler_skill_load_failure_phrase_exposed.md` 不同：本轮不是 Web scheduler 回复前言污染，而是 Feishu direct 图片附件主链路未完成。
- 初步怀疑 Feishu ingress 附件解析、post/image 消息落库、共享附件 ingest 到 runner prompt 的连接，或图片理解 fallback 判断存在不稳定边界。

## 下一步建议

- 先复核 Feishu direct 的 post / image 附件入口：确认 `attachments=1` 时实际附件 bytes、本地路径、MIME、OCR 文本或图片可读路径是否进入 runner。
- 对 Feishu direct 增加附件回归：当 prompt metadata 含图片附件时，最终用户可见回复不得出现“图片理解工具/图片分析技能未激活”等内部能力状态；读取失败时只能返回产品化重传提示。
- 对比 16:41/16:47 失败轮与 16:53 成功轮的附件 metadata 差异，确认是否是首次 post 附件未下载完成、消息类型兼容问题，还是 answer 阶段过早判断。

## 验证

- 本轮是缺陷台账维护任务，未修改业务代码、测试代码或配置代码。
- 已验证范围：SQLite 会话收口、assistant final 污染扫描、同 session prompt `attachments=1` 抽取、ACP `end_turn` 收口、cron_job_runs 普通 scheduler / heartbeat 同窗状态、最近四小时非文档代码提交检查。
