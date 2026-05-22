# Proposal: Storage Budget and Artifact Lifecycle Control Plane

status: proposed
priority: P1
created_at: 2026-05-22 14:04:10 +0800
owner: automation
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_local-backup-restore-vault.md`
- `docs/proposal/auto_p1_redacted-support-bundle.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/files.rs`
- `crates/hone-channels/src/sandbox.rs`
- `crates/hone-channels/src/response_finalizer.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `memory/src/llm_audit.rs`
- `crates/hone-core/src/task_observer.rs`
- `crates/hone-web-api/src/routes/task_runs.rs`
- `bins/hone-cli/src/cleanup.rs`
- `packages/app/src/pages/logs.tsx`
- `packages/app/src/pages/llm-audit.tsx`
- `packages/app/src/pages/task-health.tsx`

## 背景与现状

Hone 已经从聊天入口演进成长期运行的投资研究工作台：public Web 支持附件上传，IM 渠道把附件落到 actor sandbox，`chart_visualization` 等 skill 会生成 PNG，Web/外部渠道通过 `file://` marker 渲染或投递本地图片，LLM audit 用 SQLite 保存请求/响应/错误，task observer 写 `task_runs.YYYY-MM-DD.jsonl`，desktop/backend/channel 又会持续写 runtime logs。

当前代码里已经有若干局部保护：

- `crates/hone-web-api/src/routes/public.rs` 对 public chat 单次上传限制为最多 4 个文件、每个文件 10 MB，并通过 `validate_public_upload_path` 限制只能引用当前用户上传根目录下的文件。
- `crates/hone-channels/src/sandbox.rs` 把 channel 附件写入 actor sandbox 的 `uploads/<session_id>/`，符合 actor 隔离。
- `crates/hone-tools/src/skill_tool.rs` 校验 skill script artifact 必须是允许目录内的图片；`response_finalizer.rs` 会把本地图片稳定到 `storage.gen_images_dir/<session_id>/`。
- `memory/src/llm_audit.rs` 有 `llm_audit_retention_days`，启动和每 100 次写入后会清理过期审计记录。
- `crates/hone-core/src/task_observer.rs` 把 task run JSONL 默认保留 14 天。
- `bins/hone-cli/src/cleanup.rs` 可以交互式删除安装目录下的 runtime data、release bundles、config 和 profile。

这些都是必要底座，但它们仍是分散的局部限制，不构成一个面向长期运行的容量治理产品层。Hone 的高价值功能越多，运行产物越多：上传文档、PDF preview、图表 PNG、support bundles、LLM audit、task runs、logs、actor sandbox 临时文件、未来 document inbox extraction cache 和备份 restore points 都会增长。当前系统还不能回答：每个数据域占多少空间、哪些文件可安全清理、哪些属于长期研究资产、哪些快过期、清理会影响哪些 session/history/附件链接、某个用户或本机是否接近容量预算。

## 问题或机会

1. **单次上传限制不能解决长期堆积。**  
   Public upload 和 channel upload 都限制了单次风险，但没有统一的 per-actor / per-install storage budget。一个活跃用户反复上传研报、截图、CSV、PDF preview 和图表后，磁盘会持续增长，直到系统或用户手工发现。

2. **运行产物和长期资产边界不清。**  
   公司画像、portfolio、cron jobs、session history 是长期资产；`gen_images`、support bundle、runtime logs、task run JSONL、临时 attachment preview 和 extraction cache 更接近运行产物。缺少生命周期元数据时，清理只能靠路径猜测，容易误删仍被 session/history 引用的附件，也容易把可清理缓存永久保留。

3. **桌面和 self-host 用户缺少容量可见性。**  
   Desktop packaged 模式把数据、日志和 actor sandboxes 放在应用数据目录；普通用户不会知道哪个目录在膨胀。管理端虽有 logs、LLM audit、task-health 页面，但没有一个地方展示“本机 Hone 数据占用 3.2 GB，其中 1.8 GB 是可清理 artifacts”。

4. **Hosted/public 场景会被磁盘成本和滥用拖累。**  
   Public Web 已经有 invite、SMS、API key 和额度雏形。若没有 storage quota、上传 retention 和清理审计，单个高频用户或异常客户端可以制造大量本地文件和审计记录，影响同机其它用户。

5. **未来提案会继续增加文件域。**  
   Investment Document Inbox 会登记文档和 extraction cache；Redacted Support Bundle 会生成短 TTL 诊断包；Backup Vault 会生成 restore points；Research Artifact Library 会保存交付物。没有统一 lifecycle registry，这些能力会各自实现 retention，最终出现配置漂移和清理盲区。

这值得列为 P1：它直接影响核心可用性、桌面/自托管稳定性、public 服务成本控制和用户对数据安全的信任。它不要求重写业务链路，可以先以扫描、预算、标记和 dry-run 清理落地。

## 方案概述

新增 **Storage Budget and Artifact Lifecycle Control Plane**：为 Hone 的文件和审计产物建立统一生命周期 registry、容量扫描、预算告警、清理预演和可审计删除。

核心原则：

- 不把长期研究资产当缓存清理。
- 所有清理先 dry-run，显示影响和可恢复性。
- 清理只基于注册过的 storage domain，不接受任意路径删除。
- 对 public/hosted 用户使用更严格默认预算；本地 desktop/self-host 可以放宽但必须可见。
- 清理结果写入本地 audit/task run 记录，便于排障。

核心对象：

- `StorageDomainDescriptor`：描述一个数据域的根目录、归属、是否长期资产、默认 retention、是否可自动清理、是否包含敏感内容。
- `ArtifactRecord`：对可清理产物建立最小元数据，包含 actor、session_id、source、path、size、created_at、last_referenced_at、retention_class、reference_count。
- `StorageBudgetPolicy`：定义 per-install、per-actor、per-domain 的 soft/hard limit。
- `CleanupPreview`：列出候选删除项、预计释放空间、风险等级、需要确认的长期资产、会失效的链接。
- `CleanupRun`：记录执行人、范围、释放空间、失败项和校验结果。

## 用户体验变化

### 用户端

- Public `/me` 增加轻量数据占用摘要：上传文件数量、占用、保留期限和删除入口。
- Public `/chat` 上传达到 soft limit 时提前提示：“你上传的研究材料接近本周期上限，可先删除不再需要的附件。”
- 如果某个历史附件已因 retention 清理，聊天历史中显示“附件已过期”，而不是保留一个打不开的链接或本地路径。

### 管理端

- Settings 或新增 `/storage` 页面展示全局存储面板：
  - 总占用、按 domain 分组、按 actor 排序、增长趋势。
  - 可清理空间：expired uploads、orphaned generated images、expired support bundles、old logs、old task runs、已过 retention 的 LLM audit。
  - 高风险空间：长期画像、portfolio、cron、session SQLite，不默认清理。
- 用户详情页可跳到该 actor 的 storage view，支持 export/delete 前的范围预览。
- 清理动作必须先展示 preview，再执行；执行后给出释放空间和失败原因。

### 桌面端

- Desktop dashboard 显示本机 Hone 数据占用和“可安全清理”按钮。
- 打包桌面在升级、session SQLite cutover、生成大量图表或 support bundle 后，可提示清理临时产物。
- `hone-cli cleanup` 保持破坏性全清理语义；新增更温和的 `hone-cli storage scan/cleanup`，优先用于日常维护。

### 多渠道

- IM 用户不直接触发全局清理。
- 当用户通过 Feishu/Telegram/Discord 上传附件超过个人预算时，回复短提示并给出 Web 入口或管理员处理说明。
- Agent 在回答历史附件时能区分“文件存在”“已过期清理”“从未登记”，减少幻觉式引用旧本地路径。

## 技术方案

### 1. Storage domain registry

建议在 `hone-core` 或 `memory` 新增只含数据和扫描接口的 registry：

```rust
pub struct StorageDomainDescriptor {
    pub id: &'static str,
    pub root: StorageRoot,
    pub owner_kind: StorageOwnerKind,
    pub retention_class: RetentionClass,
    pub default_retention_days: Option<u32>,
    pub auto_cleanup: AutoCleanupMode,
    pub contains_user_content: bool,
    pub contains_secrets: bool,
}
```

首批 domain：

- `public_uploads`: `public-uploads/<user_id>/`，actor-owned，默认 hosted 30/90 天，本地可配置。
- `actor_uploads`: actor sandbox `uploads/<session_id>/`，actor-owned，跟 session 引用和 document inbox 登记状态联动。
- `generated_images`: `storage.gen_images_dir/<session_id>/`，session-owned，默认可按引用和时间清理。
- `support_bundles`: runtime support bundle 目录，短 TTL。
- `task_runs`: `task_runs.YYYY-MM-DD.jsonl`，已有 14 天保留，纳入统一面板。
- `llm_audit`: SQLite audit，已有 retention，纳入容量和 vacuum/checkpoint 建议。
- `runtime_logs`: backend/channel/desktop logs，按大小和时间滚动。
- `document_cache`: 未来 document inbox extraction/OCR/vector cache。
- `backup_restore_points`: 未来 backup vault 的 restore points，只显示和按策略清理，不和普通缓存混在一起。

长期资产如 `company_profiles`、`portfolio`、`cron_jobs`、`web_auth`、`sessions` 也应被扫描展示，但默认 `auto_cleanup=never`。

### 2. Artifact metadata and reference scan

第一版不要求所有文件写入时都登记，可以先做 hybrid：

- 对新产物写 `ArtifactRecord`，例如 public upload、stable generated image、support bundle。
- 对历史目录做 best-effort scanner，按路径、mtime、大小和 session id 推断。
- 对 session history、public history 和 message attachment metadata 做引用扫描，给候选文件标注 `referenced_by_session=true`。
- 对公司画像和 document inbox 注册的文件禁止进入自动清理候选。

`generated_images` 特别需要引用保护：`response_finalizer.rs` 会把图片路径稳定到 session 目录，Web 历史和外部通道可能还引用该文件。清理策略应优先删除未被任何 session/history 引用的 orphan，再处理过期且可降级为“附件已过期”的文件。

### 3. Budget policy

新增配置项可先放在 `server.storage_budget` 或 `storage.lifecycle`：

```yaml
storage:
  lifecycle:
    enabled: true
    public_upload_retention_days: 90
    generated_image_retention_days: 30
    support_bundle_retention_hours: 24
    runtime_log_retention_days: 14
    per_actor_soft_limit_mb: 512
    per_actor_hard_limit_mb: 1024
    install_soft_limit_mb: 10240
```

策略：

- Soft limit：允许继续使用，但 UI/API 返回 warning。
- Hard limit：拒绝新增上传或 artifact 生成，给出清理入口。
- Admin/local actor 可以配置 bypass，但仍记录占用。
- Existing `llm_audit_retention_days` 和 `TASK_RUNS_RETENTION_DAYS` 先保持原 source of truth，Storage 面板读取并展示，不重复定义。

### 4. API / CLI

新增 admin/local-only API：

- `GET /api/storage/summary`
- `GET /api/storage/domains`
- `GET /api/storage/actors/:actor`
- `POST /api/storage/cleanup-preview`
- `POST /api/storage/cleanup`

Public API：

- `GET /api/public/storage/summary`
- `POST /api/public/storage/delete-upload`

CLI：

```shell
hone-cli storage scan --json
hone-cli storage cleanup --dry-run --domain generated_images --older-than 30d
hone-cli storage cleanup --expired
hone-cli storage quota --actor web::<user_id>
```

`hone-cli cleanup` 仍是卸载/重置工具；`hone-cli storage cleanup` 是日常安全清理工具。

### 5. Cleanup execution

清理流程：

1. 解析 scope：domain、actor、older-than、expired-only、orphan-only。
2. 生成 preview，列出每个候选项和风险。
3. 校验路径必须在注册 domain 根目录下，且 canonical path 不越界。
4. 对 SQLite domain 只调用对应 storage 的 prune/vacuum/checkpoint 方法，不直接删数据库文件。
5. 对文件 domain 执行 move-to-trash 或 quarantine，再最终删除；本地 desktop 可保留短期 undo，server 可直接删除但写审计。
6. 清理后重新扫描，写 `CleanupRun` 和 `task_observer` 记录。

### 6. 与现有提案的衔接

- User Data Trust Center 解决“某个用户有哪些数据、如何导出/删除”的隐私权利；Storage Lifecycle 解决“整个安装和每个 actor 的容量预算、运行产物生命周期和安全清理”。
- Investment Document Inbox 定义文档身份、解析和长期引用；Storage Lifecycle 为它提供预算、缓存清理和过期策略执行。
- Local Backup and Restore Vault 解决可恢复快照；Storage Lifecycle 不创建备份包，只在清理前提示最近备份状态或要求先备份。
- Redacted Support Bundle 生成短 TTL 诊断包；Storage Lifecycle 负责统一扫描和清理这些短期 bundle。
- Usage Entitlement Ledger 解决权益和成本；Storage Lifecycle 可向其提供 storage usage event，但不定义商业 plan。

## 实施步骤

### Phase 1: 只读扫描与容量面板

- 新增 storage domain registry 和 scanner。
- 覆盖 public uploads、actor uploads、generated images、task runs、LLM audit、runtime logs。
- 新增 admin `/api/storage/summary` 和 `hone-cli storage scan --json`。
- 管理端展示总量、domain 分布、top actors 和可清理估算。

### Phase 2: Cleanup preview

- 实现 `CleanupPreview`，支持 expired/orphan generated images、old task runs、expired support bundles、old runtime logs。
- 对 session/history 引用做保守保护。
- Public upload 先只支持用户自己删除未被 document inbox 标记为长期材料的文件。
- 增加路径越界、symlink、canonical root 校验测试。

### Phase 3: Safe cleanup execution

- 实现 admin/local cleanup API 和 `hone-cli storage cleanup --expired`。
- SQLite domain 通过已有 prune 或专用 API 清理，不直接删文件。
- 记录 `CleanupRun`，在 task-health 或 storage 页面展示最近清理结果。
- 达到 hard limit 时阻止新增 public upload / artifact generation，并返回稳定 reason code。

### Phase 4: Product integration

- Public `/me` 增加个人存储摘要。
- Desktop dashboard 增加本机可清理空间提示。
- Chat/history 对已过期附件显示稳定占位。
- 与未来 Document Inbox、Support Bundle、Backup Vault 对接各自 domain descriptor。

## 验证方式

- Rust 单元测试：
  - domain scanner 正确统计 public uploads、actor uploads、generated images、task runs。
  - symlink 和 `..` 路径无法进入 cleanup candidate。
  - referenced generated image 不会被 orphan cleanup 选中。
  - expired support bundle / old log / old task run 进入 preview。
  - SQLite domain cleanup 只调用 storage prune，不删除数据库文件。
- Web API 测试：
  - admin 可以查看全局 summary；public 只能查看自己的 storage summary。
  - actor A 不能删除 actor B 的 upload。
  - cleanup preview 与 cleanup apply 的 candidate 集合一致；apply 后重新扫描释放空间。
- 前端测试：
  - storage summary 数据转换、容量格式化、soft/hard limit 状态。
  - 历史附件已过期时显示稳定文案，不泄露本地绝对路径。
- 手工验收：
  - 上传多份 public 附件后，storage summary 显示该 actor 占用。
  - 生成 chart PNG 后，generated images domain 增长；删除未引用图片后历史仍正常。
  - 设置小 hard limit 后，新增上传被拒绝并提示清理入口。
  - `hone-cli storage cleanup --dry-run` 不修改文件；`--expired` 只删除 preview 中确认的候选。

## 风险与取舍

- 风险：误删仍被历史引用的文件。取舍：第一版只自动清理高置信 orphan/expired artifacts；引用不明的用户内容只展示，不自动删。
- 风险：扫描大目录会拖慢后台。取舍：扫描按 domain 分页和缓存，UI 显示 `generated_at`，后台清理走低频任务或用户触发。
- 风险：storage budget 可能被误解成商业权益。取舍：配置层先只做技术容量预算；商业 plan 由 Usage Entitlement Ledger 统一解释。
- 风险：本地 desktop 用户不希望自动删除研究资料。取舍：长期资产默认 never auto-clean；本地默认只提示，不强制。
- 风险：多套 retention 配置漂移。取舍：已有 `llm_audit_retention_days`、`TASK_RUNS_RETENTION_DAYS` 先保留为 domain source of truth，Storage 面板只读展示；后续再统一配置迁移。
- 不做：不实现完整备份/恢复，不做文档语义解析，不做第三方云存储，不删除 company portraits / portfolio / cron / sessions 等长期资产。

## 与已有提案的差异

查重范围：

- `docs/proposal/` 全部现有 `auto_p*.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 不重复 `auto_p1_user-data-trust-center.md`：该提案关注用户数据权利、actor 级导出/删除和隐私说明；本提案关注安装级和 actor 级容量预算、运行产物 lifecycle、清理预演与磁盘稳定性。
- 不重复 `auto_p1_investment_document_inbox.md`：该提案给上传文档建立长期文档身份、解析和 review 流；本提案提供跨 domain 的预算与缓存/产物清理，不负责文档语义。
- 不重复 `auto_p1_local-backup-restore-vault.md`：Backup Vault 负责创建可恢复快照；本提案负责日常存储扫描和安全清理，并可在清理前提示备份状态。
- 不重复 `auto_p1_redacted-support-bundle.md`：Support Bundle 负责生成诊断包；本提案把 support bundles 纳入短 TTL domain 并统一清理。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：Entitlement 处理商业权益和用量扣减；本提案处理磁盘占用和 artifact 生命周期，可把结果作为 usage signal 但不定义 plan。
- 不重复 `auto_p1_multichannel-render-preview.md`：Render Preview 保障消息/图片跨渠道显示正确；本提案解决生成图片长期堆积、引用保护和过期清理。

本轮只新增 proposal，不开始实现，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/storage-budget-artifact-lifecycle.md`，并在新增 storage registry、CLI/API、清理策略或配置项时同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
