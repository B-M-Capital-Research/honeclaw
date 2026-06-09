# Bug: Feishu 直聊图片附件未稳定进入可读链路且外露内部技能状态

## 发现时间

- 2026-06-09 19:03 CST

## Bug Type

- System Error

## 严重等级

- P2

## 状态

- Fixed

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

- 2026-06-10 00:11 CST 已修复。
- Feishu 图片下载在上游未返回 `content_type` 时会兜底标记为 `image/unknown`，避免 `image_<key>.bin` 被共享附件分类误判为普通文件，导致图片附件策略和相关图片处理能力不稳定。
- 共享附件 prompt 明确要求：附件分类为图片时，即使文件名后缀是 `.bin` 也应按图片处理；读取失败时不得列举目录、OSS、数据库、工具链或技能加载状态，只能给产品化重传/粘贴文字提示。
- 共享用户可见净化层会剥离“图片理解工具没有成功激活”“图片分析技能没成功加载”等中文内部能力状态，防止同类 runner 输出继续外露给 Feishu / Web / scheduler 用户。

## 用户影响

- 这是功能性 bug，不是单纯表达质量问题。
- 用户上传/引用图片后，核心任务是读取图片并分析；前两轮没有完成，迫使用户重复发送或改为粘贴文字。
- 同时暴露内部技能状态，降低用户对附件能力的信任。
- 定级为 `P2`：影响 Feishu direct 图片附件理解链路，但本窗只有一个 Feishu direct session 的两次失败后续成功恢复；没有跨用户大面积不可用、错投、数据破坏、敏感路径外泄或系统级未回复证据，因此不是 `P1`。

## 根因判断

- 直接证据只能确认：Feishu prompt metadata 已带 `attachments=1`，但 answer 阶段没有拿到可解析图片内容，并把内部图片工具/技能状态写入 final。
- 代码复核发现 Feishu 图片消息的 fallback 文件名为 `image_<key>.bin`，图片分类依赖下载返回的 content-type；当 Feishu 下载响应缺少或返回空 content-type 时，图片会以 `.bin + None` 进入共享附件 ingest，被归为 `Other`，从而绕过图片专用附件策略。
- 共享用户可见净化层原先只覆盖 `stock_research` / 英文 `skill` 等内部 skill 状态，未覆盖“图片理解工具 / 图片分析技能”这类中文内部能力状态。
- 与 `web_direct_image_attachment_not_readable_internal_debug_leak.md` 相似，都是图片附件未进入可读/OCR链路并外露内部排障口径；但该旧文档覆盖 Web public direct 上传链路，且已在 Web API / shared attachment ingest 上修复。本轮受影响链路是 Feishu direct，不能复用 Web-only 修复结论。
- 与 `web_scheduler_skill_load_failure_phrase_exposed.md` 不同：本轮不是 Web scheduler 回复前言污染，而是 Feishu direct 图片附件主链路未完成。

## 修复情况

- `bins/hone-feishu/src/handler.rs`
  - 新增 `normalize_downloaded_content_type(...)`：Feishu `resource_type=image` 且下载未带 content-type 时，兜底写入 `image/unknown`。
  - 保留既有基于真实 content-type 修正扩展名的逻辑；非图片文件不做伪装。
- `crates/hone-channels/src/attachments/ingest.rs`
  - 图片附件策略补充 `.bin` 图片处理约束和用户态失败边界。
- `crates/hone-channels/src/runtime.rs`
  - 内部 skill/tool 文案净化覆盖 `image_understanding`、`pdf_understanding`、图片理解、图片分析、附件处理、OCR、技能、工具等中文/英文能力状态。
- GitHub Issue：无，原缺陷定级为 P2，未创建 issue。

## 验证

- `cargo test -p hone-channels build_user_input_keeps_unknown_type_feishu_image_readable --lib -- --nocapture`
- `CARGO_TARGET_DIR=/tmp/honeclaw-bug2-check CARGO_INCREMENTAL=0 cargo test -p hone-channels sanitize_user_visible_output_strips_image_skill_state_copy --lib -- --nocapture`
- `cargo test -p hone-feishu image_download_without_content_type_still_enters_image_pipeline -- --nocapture`
- `cargo check -p hone-channels --tests`
- `CARGO_TARGET_DIR=/tmp/honeclaw-bug2-check CARGO_INCREMENTAL=0 cargo check -p hone-feishu --tests`
- 未重启本地服务，未依赖当前机器生产日志、线上渠道状态或真实投递状态判定修复。
