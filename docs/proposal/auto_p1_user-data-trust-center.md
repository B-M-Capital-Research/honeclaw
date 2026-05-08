# Proposal: User Data Trust Center for Privacy, Export, and Deletion

status: proposed
priority: P1
created_at: 2026-05-09 05:03:23 +0800
owner: automation

related_files:

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `crates/hone-web-api/src/routes/users.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/llm_audit.rs`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `memory/src/web_auth.rs`
- `memory/src/portfolio.rs`
- `memory/src/cron_job/storage.rs`
- `memory/src/llm_audit.rs`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-privacy.tsx`
- `packages/app/src/lib/public-content.ts`
- `packages/app/src/pages/users.tsx`

## 背景与现状

Honeclaw 的产品定位已经从本地聊天工具演进成多入口投资研究助理：README 明确展示 Web、Mac App、iMessage、Feishu、Telegram、Discord 等入口，以及公司画像、持仓监控、定时任务和长期记忆。当前仓库也已经把用户数据按 `ActorIdentity(channel, user_id, channel_scope)` 隔离，`docs/invariants.md` 明确要求 `ActorIdentity` 和 `SessionIdentity` 分离，actor sandbox 是用户私有长期资料的边界。

代码层面已经具备不少数据治理基础：

- `memory/src/web_auth.rs` 用 SQLite 保存公开 Web 邀请用户、手机号、密码 hash、服务条款接受版本、API key hash 和 cookie session。
- `crates/hone-web-api/src/routes/public.rs` 提供公开端登录、`/auth/me`、聊天历史、附件上传、公开文件/图片读取和 OpenAI-compatible API key 调用。
- `packages/app/src/pages/public-me.tsx` 展示公开用户的额度、账号信息和密码修改，但目前没有数据导出、删除请求、登录设备或数据范围说明。
- `packages/app/src/pages/public-privacy.tsx` 和 `packages/app/src/lib/public-content.ts` 已经声明用户有访问、更正、删除、数据可携带等权利。
- `memory/src/session.rs` / `memory/src/session_sqlite.rs` 保存会话；JSON 仍可作为权威或回滚镜像，SQLite 可作为读路径或 mirror。
- `memory/src/portfolio.rs`、`memory/src/cron_job/storage.rs`、company profile storage、public uploads、LLM audit SQLite 分别保存持仓、定时任务、公司画像、上传附件和模型调用审计。
- 管理端已有 `/api/users`、`/api/history`、`/api/llm-audit`、`/api/company-profiles/export`、`/api/company-profiles/import/*` 等散点能力，能查看或迁移局部数据。

但是，Hone 还没有一个“用户数据信任中心”：用户无法在一个地方看见 Hone 保存了哪些类别的数据、哪些会被发给第三方模型/数据源、如何导出全部个人资料、如何请求删除账号与关联数据；管理员也没有一个可审计的、按 actor 执行的导出/删除工作台。隐私政策承诺与产品执行面之间存在缺口。

## 问题或机会

这是 P1 级机会。Hone 的核心资产是用户的投资上下文：持仓、研究主线、公司画像、上传文件、定时任务、聊天记录和模型调用轨迹。用户越认真使用，沉淀的数据越敏感；如果用户不能清楚控制这些数据，信任和转化会受到影响。

主要问题：

1. **公开端隐私承诺缺少产品化执行入口。**
   隐私政策已经写到访问、更正、删除、数据可携带等权利，但公开端账号页只展示额度、账号信息和密码操作。用户如果想导出或删除资料，只能走人工联系，体验与政策承诺不一致。

2. **用户数据分散在多个存储域，缺少统一清单。**
   同一个 actor 的数据可能同时存在 session JSON/SQLite、portfolio JSON、cron jobs JSON、cron execution SQLite、company profile Markdown bundle、public uploads、LLM audit SQLite、notification prefs、research outputs、event-engine 记录。现有管理端按功能页查看，缺少“这个用户到底有哪些数据”的横向视图。

3. **删除不是一个单点操作。**
   `web_users` 可以停用邀请码并清 session，但这不是删除账号，也不覆盖历史 session、持仓、画像、上传文件、定时任务、审计记录和事件记录。公司画像支持单个 profile 删除和 bundle 导出，但不能代表全用户删除。

4. **导出能力局部存在但缺少可携带 package。**
   公司画像已有 zip 导出；历史、持仓、任务、上传、审计等还没有面向用户或管理员的一体化导出。用户换设备、从 cloud 迁回 self-host、从公开 Web 迁到桌面时，会遇到资料迁移断层。

5. **商业化与企业/高净值用户转化需要可解释的数据边界。**
   投资助理比普通聊天机器人更接近个人投研工作台。未来如果做团队版、付费版或 Hone Cloud，数据保留、导出、删除、审计与第三方调用透明度会成为付费前的信任门槛。

## 方案概述

新增一个 User Data Trust Center，分两层实现：

1. **Data Inventory Registry**
   在后端定义每类用户数据的统一目录：数据域、存储位置、owner key、是否可导出、是否可删除、默认保留期、是否包含第三方模型请求内容、是否需要管理员确认。

2. **User Data Package**
   为单个 `ActorIdentity` 生成可携带 zip 包，包含 manifest、可读 Markdown/JSON 摘要和原始数据副本。第一版优先覆盖公开 Web 用户，随后扩展到 Feishu/Telegram/Discord/iMessage actor。

3. **Deletion Request Workflow**
   公开端用户可以提交删除请求；管理员工作台看到待处理请求、影响清单和执行按钮。第一版不做即时硬删除，而是采用两阶段：先 revoke 登录/API key/停止任务，再生成备份 manifest，最后按数据域执行删除或 tombstone。

4. **User-facing Trust Surface**
   在公开端 `/me` 增加“Data & Privacy”区域：显示数据类别、最近更新时间、可导出按钮、删除请求入口、第三方服务说明、当前条款版本和联系方式。

5. **Admin Data Operations**
   在管理端用户详情页增加 Data tab：按 actor 展示 sessions、portfolio、cron jobs、company profiles、uploads、audit records、notification prefs、research artifacts 的数量、大小和保留状态；提供导出、删除预演、执行记录。

本提案不要求一开始满足完整 GDPR/CCPA 自动化合规，也不引入外部合规平台。目标是先把 Hone 内部数据边界做成可执行、可验证、可迁移的产品能力。

## 用户体验变化

### 公开 Web 用户

- `/me` 页面新增数据概览：聊天记录、上传附件、公司画像、持仓、定时任务、模型调用日志摘要、API key 状态。
- 用户可以下载个人数据包。下载前显示包内容、大小估算和“可能包含投资备注、上传文件、聊天记录”的确认说明。
- 用户可以提交删除账号与数据请求。提交后立即撤销公开登录 session 和 API key 的选项应可配置；默认先生成管理员待办，避免误删高价值投研资产。
- 隐私政策中的“访问/删除/可携带”不再只是法律文本，而有明确入口。

### 管理端

- `/users/:actorKey/data` 或用户详情页新增 Data tab。
- 管理员可以查看某 actor 的数据清单和存储健康：哪些数据在 JSON、哪些在 SQLite、哪些是 actor sandbox 文件、哪些是上传附件。
- 删除前显示 dry-run：将删除多少 session、多少条 cron、多少个 profile、多少 MB 上传、多少 audit 记录；无法删除或仅能 tombstone 的项必须明确标注。
- 每次导出/删除写一条操作审计：操作者、actor、范围、时间、结果、失败项、导出包 hash。

### 桌面端

- 桌面 bundled 模式可以复用同一 Data tab，重点展示“本机数据目录”和“可打开备份位置”。
- 迁移场景里，桌面可以导出一个 actor 的完整资料，再导入到另一个 self-host 或新电脑。公司画像已有 bundle 导入导出，可作为首个子包复用。

### 多渠道

- 第一版不要求 IM 端直接发起删除，但 IM 用户应能通过 `/me` 或 `/privacy` 得到数据说明链接。
- 对 Feishu/Telegram/Discord/iMessage actor，管理员可以从 Data tab 导出/删除对应 actor 数据，避免只能处理公开 Web 用户。

## 技术方案

### 1. 数据清单 registry

新增后端内部 registry，例如 `crates/hone-web-api/src/data_inventory.rs` 或更通用的 `memory/src/data_inventory.rs`：

```rust
pub struct DataDomainDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub owner: DataOwnerKind,
    pub exportable: bool,
    pub deletable: DeletionMode,
    pub retention: RetentionPolicy,
    pub third_party_payload_risk: bool,
}
```

第一版 domain：

- `web_auth_account`：`memory/src/web_auth.rs` SQLite user/session/API key metadata。
- `sessions`：`memory/src/session.rs` JSON 和 SQLite mirror/runtime backend。
- `portfolio`：`memory/src/portfolio.rs` actor JSON。
- `cron_jobs`：`memory/src/cron_job/storage.rs` actor JSON。
- `cron_runs`：cron execution SQLite。
- `company_profiles`：actor sandbox 下 `company_profiles/`，复用现有 bundle export。
- `public_uploads`：`sessions_dir/public-uploads/<user_id>/`。
- `llm_audit`：`memory/src/llm_audit.rs` SQLite request/response/error/metadata。
- `notification_prefs`：notification preferences store。
- `research_artifacts`：research task outputs / PDF artifacts。
- `event_engine`：event digest / notification rows 中与 actor 相关的部分。

每个 domain 实现：

- `scan(actor) -> DataDomainInventory`
- `export(actor, writer) -> DataExportSection`
- `delete_preview(actor) -> DeletionPreview`
- `delete(actor, mode) -> DeletionResult`

### 2. 导出包格式

生成 zip：

```text
hone-user-data-export/
  manifest.json
  README.md
  account/web_auth.json
  sessions/sessions.jsonl
  sessions/raw/*.json
  portfolio/portfolio.json
  cron/jobs.json
  cron/runs.jsonl
  company_profiles/bundle.zip
  uploads/files/...
  llm_audit/audit.jsonl
  notification_prefs/prefs.json
  research/artifacts/...
```

`manifest.json` 记录：

- export version
- created_at
- actor identity
- included domains
- omitted domains and reasons
- per-domain counts/bytes
- source storage paths as relative labels, not host absolute paths
- sha256 for each included file

注意：导出包不得暴露服务器绝对路径；上传附件和本地图表 marker 中的路径需改写为包内相对路径。

### 3. 删除工作流

新增 API：

- `GET /api/data-inventory?channel=&user_id=&channel_scope=`
- `POST /api/data-export`
- `POST /api/data-deletion/preview`
- `POST /api/data-deletion/requests`
- `POST /api/data-deletion/requests/{id}/approve`
- `POST /api/data-deletion/requests/{id}/execute`
- `GET /api/data-deletion/requests`

公开端：

- `GET /api/public/data-inventory`
- `POST /api/public/data-export`
- `POST /api/public/data-deletion/request`

删除执行策略：

- Web account：清除 active sessions、API key hash、revoked_at 或 tombstone，手机号可按策略 hash 化保留以防滥用重复注册。
- Sessions：删除 actor/session identity 对应 JSON；SQLite backend/mirror 删除对应 session rows。若当前 backend 是 JSON，SQLite mirror 删除失败不应阻断 JSON 删除，但要进入失败项。
- Portfolio / cron jobs / notification prefs：按 actor file 删除或 tombstone。
- Company profiles：删除 actor sandbox 下对应 `company_profiles/`，执行前可生成内部备份包。
- Uploads：删除 `public-uploads/<user_id>/` 或 actor sandbox `uploads/<session_id>/` 中归属于该 actor 的文件。
- LLM audit：考虑合规和排障需要，第一版提供 `redact` 而非硬删：保留 record id、时间、provider、token、success、error class，清空 request_json / response_json / error_text 中可识别内容。若用户明确要求硬删且策略允许，再删除。
- Cron execution / event engine rows：保留匿名化运行指标，清除 actor user id 或按 tombstone actor 替换。

### 4. 权限与安全

- 公开端导出必须要求已登录 cookie；删除请求必须二次确认密码，避免被窃取 cookie 直接删除资料。
- 管理端操作继续走现有 Bearer auth，并增加操作审计。
- 下载包设置短 TTL，导出文件默认落在 runtime temp/export 目录，过期清理。
- 导出和删除都不能接受任意 path 参数，只能通过 registry domain 扫描 actor-owned 数据。
- 删除前强制 dry-run；preview 与 execute 使用相同 domain resolver，避免 preview 和实际执行范围漂移。

### 5. 与现有能力复用

- 复用 `require_actor` / `ActorIdentity` 解析，避免重新定义用户边界。
- 复用 company profiles 的 `export_bundle()` 作为 data package 的子包。
- 复用 `/api/users` 的 session identity 解析逻辑，修补历史 session 没有 actor 字段但 session id 可解析的情况。
- 复用 LLM audit filter 中的 actor fields 作为 audit 导出/脱敏条件。
- 复用公开端 `/auth/me` 的 `to_public_auth_user` 作为账号摘要来源。

## 实施步骤

### Phase 1: Inventory-only MVP

- 增加 data inventory domain trait 和 registry。
- 实现 `web_auth_account`、`sessions`、`portfolio`、`cron_jobs`、`company_profiles`、`public_uploads`、`llm_audit` 的 `scan`。
- 新增 admin `GET /api/data-inventory`。
- 在管理端用户详情页增加只读 Data tab。
- 验证目标：管理员能看到同一 actor 的各类数据数量、最近更新时间、估算大小和导出/删除支持状态。

### Phase 2: Export package

- 实现 zip export，先覆盖 Web actor 和手动指定 actor。
- 公司画像作为嵌套 bundle；session/portfolio/cron/audit 输出 JSON/JSONL。
- 公开端 `/me` 增加下载数据按钮。
- 管理端 Data tab 增加导出按钮和导出结果 hash。
- 验证目标：导出包可离线检查，manifest 与包内文件一致，不含服务器绝对路径。

### Phase 3: Deletion request and dry-run

- 新增公开端删除请求 API，要求密码二次确认。
- 管理端展示 pending requests 和 dry-run 影响范围。
- 第一版 execute 只允许管理员执行，不自动触发。
- 验证目标：停用登录/API key、停止任务、删除或 tombstone 各 domain，并生成操作审计。

### Phase 4: Actor migration and desktop support

- 把 data package import 与 company profile import 对齐，支持从 public Web actor 迁移到 desktop/self-host actor。
- 桌面端显示本地数据目录、备份位置和导出状态。
- 对多渠道 actor 增加 `/privacy` 或 `/me` 入口指引。

## 验证方式

自动化测试：

- Unit tests：每个 domain 的 actor path 解析、scan count、delete preview、export manifest。
- Regression test：构造 fixture actor，写入 session JSON、SQLite mirror、portfolio、cron、company profile、upload、audit，执行 inventory/export，断言 manifest 数量和 zip 文件存在。
- Deletion dry-run test：preview 与 execute 的 domain 列表必须一致；execute 后 scan 结果符合删除策略。
- Public auth test：未登录不能导出；删除请求需要有效 session 和密码确认。
- Path safety test：导出包不得包含 host absolute path；public upload 不能越权引用其它用户目录。

手工验收：

- 公开 Web 用户登录后能在 `/me` 下载数据包，并看到清晰的数据类别说明。
- 管理员能从用户详情页看到某 actor 的数据清单。
- 对测试用户提交删除请求后，管理员可预览影响并执行；执行后公开端无法登录/API key 无效，用户历史、持仓、任务、画像和上传不再可见。
- LLM audit 脱敏后仍保留排障所需的非内容字段。

指标：

- 数据导出成功率。
- 删除请求从提交到完成的耗时。
- 导出包 manifest 与实际文件一致率。
- 数据删除失败项数量。
- 用户关于“数据在哪里/如何删除”的支持问题减少。

## 风险与取舍

- **误删风险高。** 投资研究资产有长期价值，第一版必须采用 request + dry-run + admin approve，不做一键即时硬删除。
- **跨存储一致性复杂。** Session 处于 JSON/SQLite mirror/cutover 过渡期，删除要同时处理两边，并明确某一边失败时的恢复策略。
- **LLM audit 不能简单硬删。** 审计记录对排障和滥用调查有价值；第一版推荐内容脱敏 + 元数据保留，除非策略明确要求硬删。
- **导出包可能含敏感投资资料。** 下载链接必须短期有效，公开端下载需登录，管理员导出需审计。
- **不是通用合规平台。** 本提案不解决所有司法辖区的法务流程，只把 Hone 的数据域、导出、删除和保留策略变成可执行能力。
- **不应扩大跨 actor 读取权限。** Data tab 必须严格按 actor 查询，不能为了方便把不同 channel 的用户自动合并；跨渠道合并属于 Linked User Workspace 的范围。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案。结论：本提案不重复，重点差异如下：

- 与 `auto_p1_linked-user-workspace.md` 不重复：linked workspace 解决跨渠道身份关联和连续上下文；本提案解决单个 actor 或已确认用户的数据清单、导出、删除和隐私执行面。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：entitlement 解决额度、成本、付费和权益；本提案不定义套餐，只处理用户数据控制权。
- 与 `auto_p1_runtime_readiness_matrix.md` 不重复：readiness 解决配置、模型和渠道运行可用性；本提案解决个人数据可见性和生命周期。
- 与 `auto_p1_run_trace_workbench.md` 不重复：run trace 解决单次 agent run 的排障；本提案只把 LLM audit 作为用户数据域之一，重点是导出/脱敏/删除。
- 与 `auto_p1_research_artifact_library.md` 不重复：artifact library 解决研究交付物沉淀和复用；本提案覆盖所有用户数据域，并要求可携带 package 与删除流程。
- 与 `auto_p1_investment_document_inbox.md` 不重复：document inbox 解决用户输入资料的摄取和归档；本提案处理资料进入系统之后的清单、导出和删除。
- 与 `auto_p1_response-feedback-learning-loop.md` 不重复：feedback loop 解决回复质量反馈；本提案不评价回答质量，只管理用户数据生命周期。
- 与 `auto_p1_automation_intent_control_plane.md`、`auto_p1_temporal-operations-calendar.md`、`auto_p1_delivery_decision_loop.md` 不重复：这些提案处理自动化创建、可见性和通知决策；本提案只把 cron job 和 execution history 纳入用户数据 governance。
- 与 `auto_p0_investment_output_safety_gate.md` 不重复：output safety 解决投资输出可信度；本提案解决用户对数据保存、迁移和删除的信任。
- 与 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 不重复：desktop startup 解决 sidecar 启动冲突；本提案只在桌面复用数据导出/迁移视图。
- 与 `docs/proposals/skill-runtime-multi-agent-alignment.md` 不重复：skill runtime 提案解决 skill disclosure 和 multi-agent 执行语义；本提案不改 skill 调用路径。

差异结论：现有提案大多围绕 agent 运行质量、投资研究工作流、通知、自动化和商业权益；本提案填补的是“用户数据权利与信任产品面”这一独立架构层。它能把隐私政策、actor-scoped 存储和多端产品体验连接起来，为公开 Web、Hone Cloud、桌面迁移和高信任投资助理体验提供基础。
