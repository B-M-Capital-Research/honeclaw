# Proposal: Artifact Access Boundary for Local and Cloud Files

status: proposed
priority: P0
created_at: 2026-05-27T20:04:53+08:00
owner: automation

related_files:
- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/current-plans/cloud-pg-oss-runtime-migration.md`
- `config.example.yaml`
- `crates/hone-core/src/actor.rs`
- `crates/hone-core/src/config/server.rs`
- `crates/hone-web-api/src/routes/files.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/cloud_oss.rs`
- `crates/hone-web-api/src/state.rs`
- `crates/hone-channels/src/outbound.rs`
- `crates/hone-channels/src/sandbox.rs`
- `crates/hone-channels/src/attachments/ingest.rs`
- `memory/src/session_sqlite.rs`
- `memory/src/web_auth.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/lib/messages.ts`

## 背景与现状

Honeclaw 已经从本地投资助理扩展为多入口产品：管理端 Web、公共 Web/SMS 登录、OpenAI-compatible Hone Cloud API、桌面 bundled/remote 模式、Feishu/Telegram/Discord/iMessage 多渠道，以及 actor sandbox 中的公司画像、上传附件和生成图片。

当前系统在身份层已经形成几个重要约束：

- `ActorIdentity` 负责权限、quota、sandbox、私有数据隔离；`SessionIdentity` 负责会话历史归属。
- public Web 用户通过 `memory/src/web_auth.rs` 中的 invite-list、SMS 登录、HttpOnly session cookie 和 API key 访问 `channel=web` actor。
- channel attachments 写入 actor sandbox 的 `uploads/<session_id>/`；company profiles 位于 actor sandbox 的 `company_profiles/`。
- Web 端图片/附件通过 assistant 文本中的 `file://` marker 和 `/api/image`、`/api/file` 代理读取；公共端通过 `/api/public/image`、`/api/public/file` 包一层登录校验后复用同一代理。
- Cloud PG / OSS 迁移已开始：`cloud.oss` 可配置后，public uploads 会进入 `oss://bucket/key`，文件代理也能从 OSS 读取托管对象。

这些能力的组合意味着 Hone 已经不只是“本机读一个文件”的产品，而是一个会把用户上传、agent 生成、长期记忆、研究附件、导出包和云对象在多个 surface 之间流转的 artifact 系统。但代码里还没有一个显式的 `Artifact` / `ArtifactAccess` 真相源；很多路径仍以“该文件是否在允许 root 下”或“该用户是否已登录”作为读取判断。

## 问题或机会

这是 P0，因为 artifact 读取边界直接影响核心数据安全、公共端可信度、桌面/远端 backend 的安全体验，以及 PG/OSS 云化后的多用户隔离根基。

主要问题：

1. **读取授权粒度不够表达真实 owner**
   `routes/files.rs` 的代理基于 storage roots、sandbox base dir 和 OSS bucket 判断可读性；`routes/public.rs` 的 public wrapper 先要求登录，再复用同一代理。登录态能证明“这是一个 public 用户”，但还不能单独证明“这个 path/object 属于当前用户、当前 session、当前 actor，或是被授予可读”。

2. **local path 和 OSS URI 的安全语义不一致**
   public upload 写 OSS 时使用 `public_upload_prefix/user_id/day/name`，chat 提交附件时有 `is_public_upload_uri_for_user` 校验。但通用文件代理只要识别为托管 bucket 内对象即可读取，没有统一的 owner manifest 来区分 public upload、agent generated image、support bundle、company-profile export、未来 research artifact。

3. **artifact 生命周期与访问控制被分散在多个模块**
   上传限制在 public route；图片 marker contract 在 outbound/Web parser；actor sandbox 负责路径布局；storage budget、document inbox、backup restore 等未来能力会继续增加 artifact 类型。缺少统一 handle 会让每个新功能重复实现路径清洗、owner 判断、MIME、过期、下载名和审计。

4. **云化会放大本地假设**
   本地 owner mode 下“文件在本机 root 内”通常足够直观；远端 public Web 或 Hone Cloud API 中，用户无法理解一个 `file:///abs/path` 或裸 `oss://bucket/key` 为什么可见。产品需要把附件呈现为可解释的“我的上传”“本轮生成图表”“公司画像导出”，而不是暴露底层路径。

5. **增长与协作能力需要可分享但不越权的 artifact**
   Shareable briefs、expert review、support bundle、collaborative rooms、public portfolio 都会需要“短期可访问链接”或“按角色可读附件”。如果没有统一边界，后续要么过度封闭影响体验，要么通过临时 URL 绕过审计。

## 方案概述

引入统一 Artifact Access Boundary：新增一个轻量的 artifact registry 和 file proxy access layer，把所有用户可见或跨 surface 流转的本地文件/OSS 对象包装成稳定 `artifact_id` 或短期 `artifact_token`，读取时基于 owner、scope、purpose、expiry 和 backend location 做授权，而不是直接信任 path。

第一版目标不是重写全部存储，而是建立一个最小可落地的读取边界：

- 所有 public upload 新写入 `ArtifactRecord`。
- assistant final 文本中的 `file://` / `oss://` 在进入 Web history projection 时尽量转换成 `artifact_id` metadata。
- `/api/public/file`、`/api/public/image` 改为优先接受 `artifact_id` 或 signed `artifact_token`；保留 path 兼容但收窄到当前用户历史中已出现或 registry 可证明 owner 的对象。
- `/api/file`、`/api/image` 在 admin/local owner mode 下继续支持调试路径，但远端 deployment mode 要求 operator scope 或 artifact grant。
- OSS 只通过 registry 中的 managed key 读取；裸 `oss://bucket/key` 不作为长期公共 API。

## 用户体验变化

### 用户端

- 聊天上传后，前端拿到的不再只是 `{ path, name, kind, size }`，而是 `{ artifact_id, name, kind, size, preview_url, download_url }`。
- 历史消息里的图片/附件可以稳定打开，不依赖底层本地绝对路径是否还可见。
- 如果附件过期、被删除、迁移中或权限不足，显示明确状态：`已过期`、`已删除`、`迁移中`、`无权访问`，而不是普通 404。
- 未来分享 brief 或导出画像时，可以生成有限期链接；用户能看到该链接何时过期、是否可撤销。

### 管理端

- 用户详情页增加 artifact tab 或嵌入现有 data trust/storage surfaces：按 actor/session/domain 展示上传、生成图表、导出包、support bundle、company profile bundle。
- 管理员读取用户 artifact 时走 operator scope 和 audit；local owner mode 可以默认放行但 UI 标注本地 owner 访问。
- 运维排障时可从 artifact id 追到 storage backend、MIME、大小、created_at、owner、引用它的 session/message。

### 桌面端

- bundled 模式继续使用本机文件，但 UI 不展示 host absolute path；桌面可以通过 artifact id 调用内嵌 backend。
- remote backend 模式下，桌面不会尝试打开远端 `file://`；所有附件走 backend proxy URL 或 signed download URL。
- 本地生成图片、图表、快速捕获材料未来可以统一进入同一个 artifact registry，避免桌面独立维护一套附件路径规则。

### 多渠道

- Feishu/Telegram/Discord 的图片投递仍可消费 final 文本中的 marker，但 outbound 层可先解析为 artifact，再根据 channel capability 下载 bytes 或上传平台素材。
- 对不支持附件下载的渠道，返回可撤销短期链接，而不是原始本地路径或永久 OSS URI。
- channel 侧回传的用户附件可以登记到 actor-scoped artifact registry，和 public Web 上传保持一致 owner 语义。

## 技术方案

### 1. ArtifactRecord 数据模型

建议在 `memory` 或后续 PG-backed repository 中新增表/trait：

```rust
pub struct ArtifactRecord {
    pub artifact_id: String,
    pub owner: ArtifactOwner,
    pub source: ArtifactSource,
    pub purpose: ArtifactPurpose,
    pub backend: ArtifactBackend,
    pub location: ArtifactLocation,
    pub display_name: String,
    pub content_type: String,
    pub size_bytes: Option<u64>,
    pub sha256: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub deleted_at: Option<String>,
    pub metadata_json: serde_json::Value,
}

pub enum ArtifactOwner {
    Actor(ActorIdentity),
    WebUser { user_id: String },
    Session(SessionIdentity),
    OperatorGenerated,
}

pub enum ArtifactSource {
    PublicUpload,
    ChannelUpload,
    AgentGenerated,
    CompanyProfileExport,
    SupportBundle,
    ResearchArtifact,
}

pub enum ArtifactBackend {
    LocalFile,
    Oss,
}
```

第一版可以 SQLite/local JSON 起步，云化时换成 PG repository；接口从一开始按 trait 设计，避免把 PG 迁移写死在 Web route。

### 2. 访问判断

新增 `ArtifactAccessContext`：

- `PublicUser { user_id }`
- `PublicApiKey { user_id, key_prefix }`
- `Actor { actor }`
- `Operator { operator_id, scopes }`
- `LocalOwner`
- `ChannelRuntime { actor, channel }`

读取规则：

- public user 只能读取 owner 为同一 `web_user` / `Actor(web,user)` / 当前用户 session 引用过的 artifact。
- public API key 默认只能读同 owner 的 artifact；是否允许生成下载链接由 API contract 决定。
- operator 需要 `artifacts.read`，读取敏感类型如 support bundle/LLM prompt attachment 需要更细 scope。
- local owner mode 可读本机 roots，但仍记录 audit 和返回 artifact envelope。
- channel runtime 只能读当前 actor/session 相关 artifact，除非未来 collaborative room 明确授予。

### 3. URL 与兼容策略

新接口：

- `POST /api/public/uploads` 返回 `artifact_id`、`preview_url`、`download_url`。
- `GET /api/public/artifacts/:id` 基于当前 cookie/API key 授权读取。
- `GET /api/artifacts/:id` 基于 admin/local owner/operator 授权读取。
- `POST /api/artifacts/:id/grants` 生成短期 token，用于分享、support、channel fallback。

兼容期：

- 保留 `path` 参数读取，但 public route 必须把 path 反查为 registry record 或验证它属于当前用户 upload root/OSS prefix。
- 管理端 path 读取仅在 local deployment 或 operator scope 下开放；远端 deployment 给出迁移提示。
- assistant 文本中的 `file://` marker 暂时保留，Web history projection 增加“marker -> artifact candidate”的转换，避免一次性改动 runner contract。

### 4. OSS 与本地文件统一

- `OssClient::public_upload_key` 继续生成分层 key，但 registry 是读取授权真相源。
- `parse_managed_uri` 不再意味着可读，只意味着 location 可解析；授权必须经过 registry。
- 本地文件登记时保存 canonical path，但响应中不返回绝对路径。
- 对临时生成图、chart、PDF preview 等，可设置默认 retention；对 company profile export/support bundle 可设置短期过期。

### 5. Session 与消息引用

在 session message metadata 或 history projection 中保留：

```json
{
  "attachments": [
    {
      "artifact_id": "art_...",
      "name": "chart.png",
      "kind": "image",
      "size": 12345
    }
  ]
}
```

历史 JSON/SQLite 不需要立即迁移全文；读取旧消息时可以继续解析 `[附件: path]`，但如果能反查 registry，就补充 artifact metadata。新消息优先写 artifact metadata。

## 实施步骤

### Phase 1: Registry 与 public upload 写入

- 新增 `ArtifactStore` trait 和本地 SQLite 实现。
- public upload 成功后创建 `ArtifactRecord`，返回 `artifact_id` 和兼容 `path`。
- 为 `LocalFile`、`Oss` 两种 backend 写单元测试：owner、content_type、size、location、deleted/expired 状态。
- 增加 `artifact_id` 格式、owner 序列化、`ActorIdentity` roundtrip 测试。

### Phase 2: File proxy 授权收口

- 新增 `/api/public/artifacts/:id` 和 `/api/artifacts/:id`。
- public `/api/public/file` / `/api/public/image` 优先支持 `artifact_id`，path fallback 必须证明属于当前用户。
- admin `/api/file` / `/api/image` 在远端 deployment mode 中要求 bearer/operator/local owner 明确 scope。
- OSS read path 改为 registry-driven，裸 `oss://` 只作为兼容输入。

### Phase 3: History 与 renderer 接入

- `history_from_messages` 和前端 public chat attachment 模型支持 `artifact_id`。
- `packages/app/src/lib/messages.ts` 在解析 assistant inline image 时优先生成 artifact URL。
- `crates/hone-channels/src/outbound.rs` 支持 marker -> artifact resolution hook；外部通道按 bytes 上传或短期链接发送。
- 保留 `file://` final text contract，但用户可见层不再暴露绝对路径。

### Phase 4: Audit、grant、迁移工具

- 对 artifact read/write/delete/grant 写轻量 audit event。
- 支持短期 `artifact_token`，用于分享 brief、support bundle、专家 review 和无登录 channel fallback。
- 写一次性 scanner，把现有 public upload 目录、`gen_images`、actor sandbox uploads 中近期文件登记为 registry records；旧文件找不到 owner 时标为 `legacy_unowned`，只在 local owner/admin debug 下可读。

## 验证方式

verification:
- Unit tests：`ArtifactStore` create/get/list/delete、owner serialization、expiry/deleted 状态、OSS/local location parsing。
- Route tests：public user A 不能读取 public user B 的 artifact；登录但无 owner 的 `path` / `oss://` 返回 403；当前用户上传后可通过 `artifact_id` 读取。
- Compatibility tests：旧 `path` 格式在当前用户 upload root 内仍可用；旧 assistant `file://` marker 可渲染；远端 deployment 下 unauthorized path fallback 被拒绝。
- Security regression：路径穿越、OSS bucket/key 伪造、URL encoded `..`、不同 user_id prefix 混淆、deleted/expired artifact read。
- Frontend tests：public chat 上传、历史附件、图片 lightbox、附件过期态、download URL 生成。
- Manual smoke：public Web 上传 PDF/图片并聊天引用；桌面 bundled 生成 chart；remote backend 打开历史附件；Feishu/Telegram 发送生成图。
- Metrics：artifact read 403 rate、legacy path fallback rate、unowned legacy artifact count、OSS read errors、artifact grant count。

## 风险与取舍

risks:
- **兼容复杂度**：历史消息里已有裸 path/marker。取舍：先做 projection/fallback，不立即重写所有 session。
- **数据库迁移时机**：Cloud PG 迁移仍在进行。取舍：store trait 先落地本地 SQLite，schema 设计贴近 PG，后续切 repository。
- **性能开销**：每次附件读取多一次 registry lookup。取舍：artifact metadata 小，可按 id 缓存；读取 bytes 仍是主要成本。
- **误伤本地开发体验**：本地调试经常直接打开绝对路径。取舍：local owner mode 保留 path fallback，但远端/public 默认收紧。
- **artifact owner 推断不完整**：旧文件可能无法确定 owner。取舍：标记 `legacy_unowned`，只给 admin/local owner debug，不自动暴露给 public。
- **不要在第一版做通用网盘**：本提案只解决 Hone 生成/上传/导出 artifact 的读取边界，不做任意文件管理、同步盘、永久公网 CDN 或复杂 DRM。

## 与已有提案的差异

- 与 `auto_p1_user-data-trust-center.md` 不重复：该提案关注用户数据清单、导出、删除、隐私解释；本提案关注每次 artifact 读取的 owner/access enforcement 和 URL contract。
- 与 `auto_p1_storage-budget-artifact-lifecycle.md` 不重复：该提案关注容量、retention、清理和预算；本提案关注读取授权、artifact id、signed grant 和 local/OSS 统一访问边界。
- 与 `auto_p1_research_artifact_library.md` 不重复：该提案把外部深度研究结果沉淀成研究库；本提案是跨 public upload、agent generated image、OSS object、channel attachment 和 export bundle 的通用读取边界。
- 与 `auto_p0_public-edge-abuse-guard.md` 不重复：该提案关注 public SMS/chat/upload/API 的滥用、频控和成本保护；本提案关注已上传或已生成 artifact 的跨用户/跨 surface 访问边界。
- 与 `auto_p1_investment_document_inbox.md` 不重复：该提案把用户上传材料升级为投资证据 inbox；本提案是底层 artifact access layer，可服务 document inbox，但不定义文档理解流程。
- 与 `auto_p1_external-egress-ledger.md` 不重复：该提案记录第三方出站边界；本提案处理 Hone 内部 local/OSS artifact 的入站读取与分享授权。
- 与 `auto_p1_multichannel-render-preview.md` 不重复：该提案验证多渠道渲染效果；本提案保证渲染所需文件在各渠道读取时有统一授权。
- 与 `auto_p1_local-backup-restore-vault.md` 不重复：该提案关注备份/恢复集合；本提案定义备份中每个 artifact 平时如何被引用、读取、撤销和审计。
- 与 `auto_p1_hone-cloud-api-contract.md` 不重复：该提案定义公开 API/developer console；本提案补齐 API 返回或读取附件时必须依赖的 artifact access primitive。

