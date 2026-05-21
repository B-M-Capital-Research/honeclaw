# Proposal: Desktop Quick Capture Inbox for Investment Evidence

status: proposed
priority: P2
created_at: 2026-05-21 20:04:34 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_workspace-command-palette.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `bins/hone-desktop/src/{main.rs,commands.rs,tray.rs}`
- `bins/hone-desktop/src/sidecar.rs`
- `packages/app/src/pages/chat.tsx`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
- `crates/hone-channels/src/sandbox.rs`
- `memory/src/company_profile/{storage,transfer,types}.rs`

## 背景与现状

Hone 已经具备跨 Web、桌面和 IM 的投资研究入口，但“用户正在电脑上阅读一段材料时，如何低摩擦交给 Hone”还没有被产品化。

当前真实结构是：

- Desktop Tauri host 主要负责 backend 模式、sidecar 生命周期、渠道设置、agent/model/FMP/Tavily 配置和本地 CLI probe。`bins/hone-desktop/src/tray.rs` 目前只是空的 extension point，`commands.rs` 中也没有剪贴板、选区、截图、网页 URL、PDF 或全局快捷键相关 command。
- Public Web chat 支持附件上传，`packages/app/src/pages/chat.tsx` 有 composer、粘贴/附件处理、历史恢复和分享卡片能力。
- Public upload 由 `crates/hone-web-api/src/routes/public.rs` 写入当前 Web 用户的 upload root，并限制 chat 只能引用该用户的上传路径。
- IM 入口已有共享附件摄取层，`crates/hone-channels/src/attachments/ingest.rs` 能处理文件名清洗、大小限制、PDF preview、压缩包 manifest、图片 gate 和附件 manifest。
- Actor sandbox 是执行期文件边界，channel 附件落在 `uploads/<session_id>/` 下，符合 `ActorIdentity` 隔离约束。
- 已有 `auto_p1_investment_document_inbox.md` 提出了把上传文件实体化为长期文档对象；但它关注的是“文档进入系统之后如何登记、解析、治理和交接”，不是桌面端如何从用户当前工作流捕获材料。

这说明底层能力已经有相当多积木，但入口仍偏被动：用户要打开 Web chat 或 IM，把文件拖进去或粘贴文本。对一个桌面投资助理来说，真正高频的场景往往发生在浏览器、PDF 阅读器、券商页面、研报、邮件、Excel、截图工具和聊天软件之间。

## 问题或机会

Hone 的长期价值依赖证据沉淀：用户看到一条新闻、一段财报说明、一张持仓截图、一个估值表、一个 SEC 片段或一个竞争对手观点时，最好能立刻交给 Hone 归档、解释或排队复盘。现在这条链路有几个缺口：

1. **捕获入口不在用户当前上下文里。**  
   桌面端没有 tray、global shortcut、share extension、clipboard capture 或 quick note。用户必须切回 Web/IM，对高频研究流很重。

2. **捕获意图和文档治理断开。**  
   同样一段材料可能是“马上问一句”“保存到 MU 画像”“加入证据复盘”“稍后读”“提取持仓”。当前上传只表达“附到这一轮 chat”，没有先捕获为一个 lightweight inbox item，再选择后续动作。

3. **截图和剪贴板是投资研究的常见输入，但不是一等入口。**  
   用户常拿到的是网页选中文本、截图、PDF 页面、表格片段或 URL，不一定是一个准备好的文件。Web chat 可以粘贴，但桌面端不能在系统级提供统一体验，也不能提示本地/远端 backend 对文件可见性的差异。

4. **多渠道能回答，却难以沉淀桌面阅读材料。**  
   Feishu/Telegram/Discord 适合移动或协作触达；桌面阅读时，把材料扔给 IM 再回 Web 复盘并不自然。Hone Desktop 应该承担“本机研究入口”的职责，而不仅是 sidecar manager。

5. **这个能力适合增长，但风险需要边界。**  
   一键捕获能显著降低激活成本，也容易误收敏感屏幕内容、个人路径或未授权网页。必须先把权限、预览、脱敏、存储位置和远端上传边界说清楚。

本提案定为 P2：它对桌面体验和证据沉淀有明确价值，但最好依赖 Document Inbox、Permission Broker、Linked Workspace 或至少 actor-scoped document metadata 的一部分先落地；不应抢在 P0/P1 的安全、权限、通知可靠性和核心运行链路之前。

## 方案概述

新增 **Desktop Quick Capture Inbox**：桌面端提供轻量系统入口，把用户当前看到的投资材料捕获为 actor-scoped capture item，再由用户选择进入 chat、document inbox、evidence queue、research artifact 或 company portrait handoff。

第一版建议只做最小闭环：

- Desktop tray 菜单：`Capture Clipboard`、`Capture Screenshot`、`Capture URL / Note`、`Open Capture Inbox`。
- Tauri command：读取剪贴板文本/图片、接收用户选择的本地文件、创建 capture item。
- Web console 页面或抽屉：展示最近 capture items，允许用户选择“继续追问”“保存为文档”“加入证据复盘”“更新画像草稿”“删除”。
- 所有捕获默认先停在 inbox，不自动写入 company profile、portfolio、cron job 或长期 thesis。
- Remote backend 模式下必须明确提示：本机文件/截图需要用户确认上传到远端；未确认前只保留本机临时引用，不把绝对路径发给 backend。

核心对象建议：

- `DesktopCaptureItem`
  - `capture_id`
  - `actor`
  - `source`: `clipboard_text`、`clipboard_image`、`screenshot`、`file`、`url_note`
  - `desktop_mode`: `bundled` / `remote`
  - `content_kind`: `text`、`image`、`pdf`、`table`、`url`、`unknown`
  - `title`
  - `preview`
  - `local_path` 或 `uploaded_document_id`
  - `sha256`
  - `created_at`
  - `status`: `captured`、`needs_upload_confirm`、`saved_to_document_inbox`、`sent_to_chat`、`queued_for_review`、`deleted`
  - `routing_hints`: detected symbols、URL host、selected action

## 用户体验变化

### 用户端

- 桌面用户看到 tray 菜单和可选全局快捷键，例如 `Capture to Hone`。
- 捕获后出现小窗口或 toast，不直接打断工作流：
  - 展示文本前几行、图片缩略图、URL、文件名和大小。
  - 让用户选择：`Ask now`、`Save to inbox`、`Review later`、`Delete`。
- `Ask now` 会打开 `/chat` 并把 capture 作为附件/上下文带入当前消息草稿，而不是直接自动发送。
- `Save to inbox` 会进入文档/证据收件箱，等待后续解析和路由。
- 用户可以在捕获时补一句 note：例如“这段和 MU 的库存周期有关”。
- 对 remote backend 显示明确边界：“这会上传到远端 Hone 服务用于解析”，用户确认后才上传。

### 管理端

- 管理端可在用户详情或未来 Documents/Captures tab 中查看 capture item 状态、来源、是否已保存为文档、是否被删除。
- 支持过滤 `needs_review`、`needs_upload_confirm`、`failed_extract`、`saved_to_document_inbox`。
- 不允许管理员直接读取用户本机未上传的 local-only capture；只能看到 metadata 和用户确认上传后的内容。

### 桌面端

- `bins/hone-desktop/src/tray.rs` 从空 extension point 变成桌面 capture 的入口。
- bundled 模式下，capture 可以写入 app data dir，再由本机 backend 读取。
- remote 模式下，capture 默认是本机临时项；用户选择上传后走 public/admin upload API。
- Desktop settings 增加 capture 权限和 retention：
  - 是否启用全局快捷键。
  - 是否允许截图。
  - local-only capture 保留多久。
  - remote upload 是否每次确认。

### 多渠道

- 多渠道不需要直接实现 desktop capture。
- 用户可以在 IM 中引用最近 capture，例如“把我刚才桌面保存的那段材料总结一下”，但只有在同一 actor/workspace 有权限时才可见。
- 群聊默认不能读取个人 desktop capture，除非后续 Linked Workspace / group workspace 明确授权。

## 技术方案

### 1. Desktop capture command 层

在 `bins/hone-desktop/src/commands.rs` 增加命令，第一版保持小而可测试：

- `capture_clipboard_text()`
- `capture_clipboard_image()`
- `capture_file(path)`
- `list_desktop_captures()`
- `delete_desktop_capture(capture_id)`
- `upload_desktop_capture(capture_id)`

实现上优先使用 Tauri 官方插件或系统能力：

- 剪贴板：文本和图片分开处理，图片写入 desktop data dir 的 capture staging 目录。
- 文件：只保存用户明确选择的路径或复制到 staging；不要扫描目录。
- 截图：第一版可先不做全屏截图，避免权限复杂度；先支持 clipboard image 和 file picker。真正 screenshot 放 Phase 2。

所有 command 返回结构化错误 code，例如 `clipboard_empty`、`image_too_large`、`remote_upload_requires_confirmation`、`permission_denied`。

### 2. Capture staging 与 actor 归属

新增本机 staging 目录：

```text
<desktop_data_dir>/captures/
  capture_index.sqlite3
  blobs/<capture_id>/content.txt
  blobs/<capture_id>/image.png
  blobs/<capture_id>/metadata.json
```

actor 归属策略：

- bundled admin/local 模式：默认归属当前 desktop backend 的 selected actor 或 local admin actor；如果没有明确用户，capture 只能作为 local draft，不进入长期资产。
- public/remote 模式：归属当前登录 Web user；未登录时只能 local draft。
- 后续 Linked Workspace 落地后，可以把 capture 归属 workspace，但执行期仍必须解析成具体 actor 权限。

### 3. 与 Web API 和 Document Inbox 的衔接

第一版不要求 Document Inbox 已完全落地，但接口应预留：

- `POST /api/desktop-captures/import`
  - bundled/backend 同机路径：接收 staging metadata 和受控 local file reference。
  - remote 模式：接收 multipart upload。
- `POST /api/desktop-captures/{id}/send-to-chat`
- `POST /api/desktop-captures/{id}/save-document`
- `POST /api/desktop-captures/{id}/queue-evidence-review`

如果 `auto_p1_investment_document_inbox.md` 已实现，`save-document` 应创建 `InvestmentDocument`，并复用附件解析、PDF preview、image gate 和 retention policy。

如果尚未实现，第一版可以只把 capture 作为 chat attachment 草稿，不声称已经归档为长期文档。

### 4. 安全与权限边界

- 不做静默屏幕读取；截图、剪贴板图片、文件都必须来自用户显式操作或系统授权。
- 不自动抓浏览器当前 URL 或页面 DOM；URL capture 第一版由用户手工粘贴，或从剪贴板 URL 识别。
- 不把本地绝对路径发给 LLM；backend 可读取同机 staging 时，也应在 prompt 中用相对 capture label 或 document id。
- remote upload 必须二次确认，并展示文件名、大小、host/URL、图片缩略图或文本预览。
- 捕获内容默认不进入公司画像、portfolio、scheduled task 或 memory summary；必须由用户选择后续动作。
- 所有删除应删除 blob 和 extraction cache；如果内容已沉淀为画像或报告，删除 capture 只标记来源删除，不自动改写派生成果。

### 5. 前端落点

建议新增或复用：

- `packages/app/src/context/desktop-captures.tsx`
- `packages/app/src/lib/desktop-captures.ts`
- Dashboard capture inbox strip
- Chat composer capture drawer
- 未来 Documents 页面中的 `Captures` filter

UI 上不要做复杂文件管理器。第一版只回答：

- 最近捕获了什么。
- 是否已经上传/保存/发送。
- 下一步动作是什么。
- 为什么某个 capture 不能用于当前 backend mode。

## 实施步骤

### Phase 1: Local clipboard/file capture draft

- 在 desktop 侧新增 capture staging store 和 text/file capture commands。
- Tray 增加 `Capture Clipboard` 和 `Open Capture Inbox`。
- Web console 读取 local capture list，允许删除和发送到 chat 草稿。
- 不做截图，不做 remote upload，不做长期文档归档。

### Phase 2: Remote-safe upload and document handoff

- 增加 remote upload confirmation flow。
- 复用 public/admin upload 或新增 capture import API。
- 与 Investment Document Inbox 对接：capture 可保存为 `InvestmentDocument`。
- 统一解析 PDF/image/table preview，并展示 extraction status。

### Phase 3: Screenshot and URL/note capture

- 增加截图能力和权限说明。
- 支持 URL + note capture，先不抓网页正文；如需网页正文，走明确的 web fetch/search 工具并记录 source provenance。
- 捕获后可以生成 Evidence Review Item 或 Company Portrait handoff 草稿。

### Phase 4: Workspace and growth polish

- 与 Linked Workspace 对接，让同一真实用户在 desktop/public Web 中看到同一 capture inbox。
- 增加轻量 onboarding：首次桌面启动提示“用快捷键保存你正在看的研报或截图”。
- 记录隐私保护的产品事件：capture created、saved、deleted、sent_to_chat，用于判断功能是否降低研究材料录入摩擦。

## 验证方式

- Rust / Tauri 单元测试：
  - capture metadata 序列化、状态转换、retention 清理。
  - remote mode 下未确认上传时不能返回可被 backend/LLM 读取的文件路径。
  - 删除 capture 会删除 blob 并保留最小 tombstone。

- Web API 测试：
  - public 用户只能导入自己的 capture。
  - remote upload 超限、空文本、未知 MIME、路径越界返回稳定错误 code。
  - save-document 与 document inbox 缺失时 graceful degrade，不假装归档成功。

- 前端测试：
  - capture list 状态：local draft、needs upload confirm、uploaded、sent to chat、deleted。
  - chat composer 能接收 capture 草稿但不自动发送。
  - remote mode 文案明确显示上传边界。

- 手工验收：
  - macOS desktop bundled 模式下，复制一段网页文字，tray capture 后能在 chat 草稿引用。
  - remote backend 模式下，复制同一段文字，必须确认上传后才能进入 chat。
  - 图片过大、剪贴板为空、backend disconnected 时都有可理解错误。

- 指标：
  - 从看到材料到发给 Hone 的操作数下降。
  - capture 保存后 7 天内转化为 document/review/chat 的比例。
  - 用户因“找不到上次看到的材料”重复上传的比例下降。

## 风险与取舍

- 风险：桌面截图和剪贴板权限容易引发隐私担忧。取舍：第一版只做显式 clipboard/file capture，截图延后，所有 remote upload 必须确认。
- 风险：capture inbox 与 Document Inbox、Evidence Review Queue、Research Artifact Library 产生概念重叠。取舍：capture 是入口和草稿；document/review/research/profile 才是长期资产或后续 workflow。
- 风险：remote backend 无法读取本地 staging 文件。取舍：remote mode 明确区分 local draft 和 uploaded capture，不走本地路径引用。
- 风险：全局快捷键和 tray 在不同平台行为复杂。取舍：先支持 macOS Tauri tray 菜单，global shortcut 作为可选 Phase 2/3。
- 风险：过度捕获会制造未处理材料堆积。取舍：capture item 默认短 retention，并提供 review later 队列和批量删除。
- 不做：不自动读取浏览器 DOM，不做静默屏幕监控，不把 capture 自动写入公司画像或 portfolio，不替代已有 Web/IM 上传。

## 与已有提案的差异

查重范围包括本轮开始时 `docs/proposal/` 下全部自动提案和历史 `docs/proposals/`：

- 不重复 `auto_p1_investment_document_inbox.md`：该提案关注上传后的文档资产、解析、治理和长期交接；本提案关注桌面捕获入口、local/remote 权限边界、capture draft 和用户当前工作流中的低摩擦入口。
- 不重复 `auto_p1_evidence_review_queue.md`：evidence queue 关注一条证据是否改变投资 thesis；本提案只负责把桌面材料捕获进系统，并可生成 review item。
- 不重复 `auto_p1_research_artifact_library.md`：research artifact 关注深度研究报告交付物；本提案处理剪贴板、截图、URL、文件等原始材料入口。
- 不重复 `auto_p1_agent-permission-broker.md`：permission broker 关注 runtime action approval；本提案会依赖其思想处理 capture/upload 权限，但不设计通用 agent action permission 层。
- 不重复 `auto_p1_workspace-command-palette.md`：command palette 关注资产搜索与命令启动；本提案关注系统级材料捕获和 capture inbox。
- 不重复 `desktop-bundled-runtime-startup-ux.md`：该提案解决 sidecar ownership 和启动恢复；本提案只利用 desktop host/tray 增加研究材料入口。

查重结论：现有 proposal 已覆盖文档资产、证据复盘、研究报告、工作区、权限、桌面启动和命令搜索，但没有覆盖“桌面端从剪贴板/文件/截图/URL 捕获投资材料，并以 local/remote 安全边界进入 Hone workflow”的产品入口。因此本主题是新的、可落地的 P2 提案。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/desktop-quick-capture-inbox.md`，并在新增 Tauri command、tray/global shortcut、capture staging store、capture API、Document Inbox 对接或桌面权限策略后同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
