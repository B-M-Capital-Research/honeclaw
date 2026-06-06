# Bug: Web direct 图片附件未进入可读/OCR 链路且回复外露内部排障口径

- **发现时间**: 2026-06-06 19:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无；本单不是 P1，暂不创建。

## 证据来源

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

- 真实 Web direct 图片会话正常收口，但图片本体/OCR 没有进入可消费上下文，assistant 无法基于截图完成用户原始任务。
- 用户需要改为手工粘贴持仓文字，才能让后续组合分析继续。
- final 没有原样外泄 `image_understanding` 错误 JSON，但外露了内部附件落地、目录扫描和数据库排查口径。

## 用户影响

- 这是功能性 bug，不是单纯表达质量问题。
- 用户上传截图后，核心任务是“读取图片并据此分析/更新”；当前链路要求用户重新粘贴文字，等于图片附件理解功能不可用。
- 定级为 `P2`：它阻断 Web direct 图片附件理解链路并迫使用户绕路，但没有跨用户数据错投、数据破坏、批量投递失败、系统级未回复或 P1 级安全事件证据。

## 根因判断

- 初步判断是 Web direct 附件引用与 runner 可读本地文件/OCR 输入之间缺少稳定桥接，导致 prompt 中只有远端/OSS 线索而没有可直接读取的图片内容。
- `image_understanding` skill 在该运行上下文不可用，且失败没有被产品层识别成统一附件读取失败。
- 出站净化层当前能阻止 raw JSON 进入 final，但没有剥离自然语言中的“根目录 / uploads / /tmp / 会话数据库 / OSS 引用 / 当前工具链”这类内部排障短语。

## 下一步建议

- 检查 Web direct 附件进入 ACP prompt 的路径：上传后应传入可访问图片 URL、文件路径或 OCR 文本，且与 actor sandbox 权限一致。
- 为图片附件读取失败增加统一用户态错误文案，并避免模型自行列举目录、数据库、OSS 引用等实现细节。
- 增加回归：Web direct 带图片附件时，runner 可获得可读 image input；当 image/OCR 不可用时，final 只给产品化重试提示，不包含内部排障词。
