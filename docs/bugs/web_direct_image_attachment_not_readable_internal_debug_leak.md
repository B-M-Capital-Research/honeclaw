# Bug: Web direct 图片附件未进入可读/OCR 链路且回复外露内部排障口径

- **发现时间**: 2026-06-06 19:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无；本单不是 P1，暂不创建。

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检时间窗：2026-07-14 19:02-23:02 CST。
  - 21:38 CST Web direct session `Actor_web__direct__web-user-c394f2531362`，用户上传两张持仓截图，附件行显示 `分类=图片`、`类型=image/jpeg`、`下载状态=成功`，且有本地路径。
  - 21:38 CST assistant final 回复 `图片附件已收到，但我这边读取到的持仓数据为空`，要求用户手动提供总资产、现金和各持仓数量 / 价格；没有外露 OSS、数据库或本机路径，但图片理解主链路仍未完成。
  - 21:50 CST 同会话上传 CSV，附件行显示 `分类=表格`、`下载状态=成功`，assistant 首轮仍回复 `无法直接读取这个 CSV 文件内容`；21:52 用户要求“重新读取我的csv文件”后，21:54 CST assistant 才拿到完整 CSV 并完成分析。
  - 判断：图片附件读取/OCR 链路仍会迫使用户绕路；CSV 首轮失败后同会话可恢复，暂作为附件读取不稳定的旁证，不另建生成文件下载缺陷。无错投、敏感信息泄露、全渠道不可用或批量失败证据，维持 `P2 / New`，非 P1。
- `data/sessions.sqlite3` -> `session_messages`
  - 2026-07-14 04:03-04:12 CST，`session_id=Actor_web__direct__web-user-400794904801`。
  - 用户连续 3 次上传同一张图片附件并询问 CALL 盈利概率；附件行显示 `分类=图片`、`类型=image/jpeg`、`下载状态=成功`，且 prompt 中包含本地可读路径。
  - 04:04 / 04:06 CST assistant final 连续两次只返回 `当前信息暂时未完成实时核验，请稍后再试。`，`metadata_json` 标记 `error_kind=AgentFailed` / `run_failed=true`。
  - 04:12 CST assistant final 改为产品化说明 `图片我这边暂时无法稳定读取到内容，OCR 没有成功提取到完整数据`，并要求用户手动提供标的、行权价、到期日、权利金和张数。
  - 本轮回复没有再次外露 OSS、数据库、目录扫描或内部 skill 状态，说明文案净化较历史样本有改善；但图片附件已下载成功后仍无法进入可读/OCR 输入，核心功能链路复发。
- 本轮状态判断：
  - 该样本晚于 2026-06-08 16:11 CST 代码级修复记录，且发生在真实 Web direct 会话中，因此从 `Fixed` 回退为 `New/P2`。
  - 问题阻断 Web direct 图片附件理解链路并迫使用户改用文字绕路，但没有错投、敏感信息泄露、全渠道不可用或批量投递失败证据，因此不升级为 P1，不创建 GitHub Issue。

- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-06 16:41-16:44 CST
  - `session_id=Actor_web__direct__web-user-d53f847825ce`
  - Web direct 同一会话内有 3 个 `session/prompt` 与 3 个 `stopReason=end_turn`，说明 runner 正常收口，不是全局未回复。
  - 其中 16:44 CST tool update 记录一次 `image_understanding` 调用失败，错误为技能未激活；该原始 JSON 没有直接进入 final。
- 用户可见 final 摘要：
  - assistant 先说明无法读取用户上传的 `IMG_0202`，后续又表示只看到 OSS 附件引用、没有图片本体或 OCR 文本，因此不能把图片当作已验证持仓来重算组合。
  - final 多次使用内部排障口径，例如“根目录、uploads、/tmp、会话数据库、OSS 引用、当前工具链”等，而不是稳定的产品化附件失败说明。
  - 同一会话后续在用户粘贴持仓文字后能完成组合分析，说明问题集中在 Web 图片附件读取/OCR 链路，不是金融分析能力整体不可用。
- 同窗对照：
  - 15:02-19:02 CST `data/sessions.sqlite3` 有 3 个 Feishu user turn 与 3 个 assistant final，均成对收口。
  - Feishu assistant final 污染扫描未命中空回复、`company_profiles/...`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400/429`、`Resource temporarily unavailable`、`quota exhausted`、`Param Incorrect`、panic 或 `index out of bounds`。
  - 同窗 `acp-events.log` 有 3 个 Feishu prompt / 3 个 `stopReason=end_turn` 与 4 个 Web prompt / 4 个 `stopReason=end_turn`，未见 `stream disconnected before completion`、runner error、quota、HTTP 400/429 或 panic。
- 去重检查：
  - `web_direct_generated_files_not_downloadable.md` 覆盖 Web 生成文件无法作为附件/下载交付。
  - `web_company_profile_relative_path_exposed.md` 覆盖公司画像相对路径外露。
  - `web_scheduler_skill_load_failure_phrase_exposed.md` 覆盖 Web scheduler 内部 skill 降级措辞外露。
  - 本轮问题是 Web direct 用户上传图片没有落到可读/OCR 输入，且失败解释暴露内部附件/工具链排障口径，属于新的受影响链路。

## 端到端链路

1. Web direct 用户上传图片，希望 assistant 基于截图更新或重算持仓/组合。
2. Web 侧 prompt 进入 Codex ACP runner，并声明支持 image prompt capability。
3. runner 侧没有拿到本轮图片本体或 OCR 文本；一次 `image_understanding` 技能调用失败。
4. assistant 最终只能要求用户粘贴图片里的文字，原始图片任务未完成。
5. final 还把内部目录、附件引用和工具链排障口径写给用户，降低可理解性并暴露实现细节。

## 期望效果

- Web direct 上传图片后，附件应作为可读图片、OCR 文本或结构化 attachment metadata 进入 runner，使 assistant 能直接完成截图理解任务。
- 如果附件读取失败，用户可见回复应是产品化提示，例如“当前未能读取这张图片，请重新上传或粘贴文字”，不应暴露本地目录、会话数据库、OSS 引用或工具链细节。
- 内部 tool / skill 失败应留在日志或台账中，不应转化成面向用户的排障过程。

## 当前实现效果

- 2026-07-14 07:01 CST 运行态回退：
  - Web direct 图片附件仍可能在附件已下载成功、prompt 含本地路径的情况下无法被 runner/OCR 稳定读取。
  - 用户最终收到的是“请粘贴关键字段”的绕路提示；历史内部排障口径本轮未复发，但附件理解主链路仍不可用。
- 2026-06-08 16:11 CST 已修复：
  - Public Web chat 入口不再把附件简化成裸 `[附件: path]` 文本，而是对用户上传路径做作用域校验后复用共享附件 ingest 管线。
  - 本地上传文件会复制到 actor sandbox 的本轮 `uploads/<session_id>/`，云模式 `oss://...` public upload 会先通过 OSS client 读回 bytes，再交给同一 ingest 管线生成可读本地附件。
  - 共享附件层在 cloud authoritative 模式下继续把附件上传到 actor OSS URL，但保留当前轮 runner 需要的 `local_path`，避免 prompt 中只剩 `oss://...`。
  - 图片附件策略不再强制模型调用可能被禁用的 `image_understanding` skill；现在优先基于附件行里的本地可读路径理解图片，只有当前阶段明确暴露该 skill 时才调用。若无法读取图片，也要求给产品化重试提示，不列举目录、OSS、数据库或工具链细节。
- 修复后，Web public direct 图片附件至少会以共享附件上下文进入 runner；当附件被准入策略全部拒绝时，API 直接返回产品化错误，不进入 LLM 自行排障。

## 用户影响

- 这是功能性 bug，不是单纯表达质量问题。
- 用户上传截图后，核心任务是“读取图片并据此分析/更新”；当前链路要求用户重新粘贴文字，等于图片附件理解功能不可用。
- 定级为 `P2`：它阻断 Web direct 图片附件理解链路并迫使用户绕路，但没有跨用户数据错投、数据破坏、批量投递失败、系统级未回复或 P1 级安全事件证据。

## 根因判断

- 根因确认：
  - Public Web chat 上传链路绕过了 `hone-channels::attachments` 共享 ingest，只把上传路径拼进 prompt，云模式下尤其容易只暴露 `oss://...` 引用而没有本地可读文件。
  - 共享 ingest 在 cloud authoritative 模式下上传 actor OSS 后覆盖了 `local_path`，导致当前轮 runner 也只能看到 OSS URI。
  - 图片附件策略把 `image_understanding` 写成必选优先路径，skill 未激活时容易诱发模型围绕目录、OSS、工具链做自然语言排障。

## 下一步建议

- 下一轮只读复核真实 Web 图片上传会话：确认 runner final 不再要求用户绕路粘贴文字，且不再出现“根目录 / uploads / /tmp / 会话数据库 / OSS 引用 / 当前工具链”等用户可见排障口径。
- 如果后续仍出现图片理解失败，应优先检查运行阶段实际可用的本地图片读取能力或 `image_understanding` skill 激活状态；不要回退到让模型自行扫描目录/数据库。

## 修复记录

- `2026-06-08 16:11 CST` 已修复：
  - `crates/hone-web-api/src/routes/public.rs`
    - Public chat 收到附件时构造 `RawAttachment` 并调用 `ingest_raw_attachments(...)`。
    - `oss://` public upload 会经 `OssObjectStore::get_object(...)` 拉回 bytes；读取失败直接返回“附件读取失败，请重新上传后重试，或直接粘贴图片中的文字。”。
    - 全部附件被共享准入策略拒绝时，API 直接返回产品化附件状态，不把空附件 prompt 交给 runner。
  - `crates/hone-channels/src/attachments/ingest.rs`
    - cloud authoritative 模式下保留当前轮本地 `local_path`，同时把 `url` 更新为 actor OSS URI。
    - 图片附件默认策略改为本地可读路径优先，skill 可用时才调用 `image_understanding`，并禁止把目录、OSS、数据库或工具链细节当作用户回复。

## 验证

- `cargo test -p hone-web-api public_chat_user_input_ -- --nocapture`
- `cargo test -p hone-web-api public_attachment_filename_prefers_client_name_for_oss_uri -- --nocapture`
- `cargo test -p hone-channels build_user_input_includes_attachment_notes --lib -- --nocapture`
- `cargo check -p hone-web-api --tests`
- `cargo check -p hone-channels --tests`
