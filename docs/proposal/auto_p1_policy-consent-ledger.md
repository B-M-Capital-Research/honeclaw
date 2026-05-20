# Proposal: Policy Consent Ledger for Terms, Privacy, and Investment Risk Acknowledgement

status: proposed
priority: P1
created_at: 2026-05-20 08:02:17 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p2_locale-content-contract.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/types.rs`
- `memory/src/web_auth.rs`
- `packages/app/src/lib/tos.ts`
- `packages/app/src/lib/public-content.ts`
- `packages/app/src/components/public-login-form.tsx`
- `packages/app/src/pages/public-terms.tsx`
- `packages/app/src/pages/public-privacy.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/chat.tsx`

## 背景与现状

Hone 已经不只是本地投资研究聊天工具。README 展示了公开 Web、Mac App、iMessage、Feishu、Telegram、Discord、公共 chat、持仓监控、长期公司画像和定时任务；公开端还提供 `/api/public/v1/chat/completions`，让 Hone Cloud 或其它客户端用 API key 访问同一套投资助手能力。

仓库已经具备一层初始政策接受能力：

- `packages/app/src/lib/public-content.ts` 中有中英文 `Terms of Service` 与 `Privacy Policy` 正文，包含投资建议免责声明、第三方 LLM/数据源说明、跨境传输、用户权利、政策更新和联系方式。
- `packages/app/src/lib/tos.ts` 定义 `TOS_VERSION = "2.1"` 与 `TOS_EFFECTIVE_DATE = "2026-05-18"`，并注释说明后端在 `public.rs` 镜像该版本。
- `crates/hone-web-api/src/routes/public.rs` 也定义 `TOS_VERSION = "2.1"`；SMS 登录时要求请求携带相同版本，否则拒绝登录，并在登录成功后调用 `record_tos_acceptance`。
- `memory/src/web_auth.rs` 的 `web_invite_users` 表有 `tos_accepted_at` / `tos_version` 字段，`record_tos_acceptance` 会覆盖当前用户的接受版本和时间。
- `packages/app/src/components/public-login-form.tsx` 在登录表单勾选协议，并把前端版本传给后端。
- `PublicAuthUserInfo` 已把 `tos_accepted_at` / `tos_version` 返回给公开端账号页，说明 UI 已有展示当前接受状态的基础。

这些实现把“登录时勾选协议”做到了可用，但还没有形成跨入口的 **Policy Consent Ledger**。当前真实结构里，政策版本是前后端两个常量手动同步；接受记录只有当前版本和当前时间，没有历史事件、接受来源、IP/session/API key、文本 hash、风险确认项或撤回状态；已有 session 和 API key 在政策升级后是否应该继续使用，也缺少统一判定点。

这对一个投资研究 assistant 尤其重要。Hone 的产品承诺不是泛聊天，而是帮助用户处理持仓、研究主线、定时提醒和多渠道主动推送。用户在 Web 登录时同意条款，不等于在 API key、桌面 remote mode、IM 引导、公开分享、未来付费计划或重大政策更新时都有清晰、可审计、可撤回的同意边界。

## 问题或机会

这是 P1 级问题：它不一定会立刻导致核心链路不可用，但会直接影响公开产品可信度、合规准备、商业化、用户数据治理和投资风险边界。

主要问题：

1. **政策版本源头重复。**
   前端 `packages/app/src/lib/tos.ts` 和后端 `crates/hone-web-api/src/routes/public.rs` 各自定义 `TOS_VERSION`。只要其中一处漏改，用户可能看到 v2.2 协议却向后端提交 v2.1，或后端接受一个前端未展示的版本。

2. **接受记录不可追溯。**
   `web_invite_users.tos_version` 只保留最后一次接受状态。它无法回答：用户在什么入口、哪个 session、哪段文本 hash、哪个语言版本、哪些风险确认项下接受；也无法保留政策升级前后的历史链路。

3. **政策升级后没有统一拦截点。**
   SMS 登录会检查当前版本，但已登录 cookie、公开 chat、公开 OpenAI-compatible API key、桌面 remote backend、未来 share link 或导出操作不一定会重新检查 `tos_version == current_policy_version`。用户可能长期绕过重新确认。

4. **投资风险确认还停留在协议正文里。**
   Terms 写明 Hone 不构成投资建议，Output Safety Gate 会处理输出层风险，但用户层面的风险认知没有结构化记录。例如“我理解 Hone 不替我做买卖决定”“我理解会调用第三方模型和数据源”“我理解跨境处理”的确认不能按功能、版本或重大变更单独审计。

5. **数据权利和同意撤回缺少闭环。**
   Privacy 文本已经写到用户可撤回同意；User Data Trust Center 提案覆盖导出/删除，但没有一个同意事件账本来记录撤回触发了哪些能力暂停、API key revoke、session 失效或删除请求。

机会是：Hone 已经有公开端条款正文、Web auth SQLite、cookie session、API key、public `/me`、账号页和 admin invite users。第一版不需要引入外部合规平台，只要把政策版本、同意事件、重新确认和能力拦截做成一个小而硬的产品/架构层，就能补齐商业化和多入口运行的信任基座。

## 方案概述

新增 **Policy Consent Ledger**：把 Terms、Privacy、投资风险确认、第三方处理确认、重大政策更新和撤回动作抽象成带版本的政策包与同意事件，而不是继续只在用户表上保留最后一次 `tos_version`。

核心对象：

- `PolicyDocument`：一份可发布的政策文档，例如 `terms`, `privacy`, `investment_risk_ack`, `third_party_processing`。
- `PolicyBundle`：一次用户必须确认的政策组合，例如 `public_web_v2_2`，包含文档版本、effective date、material change 标记、内容 hash 和适用 surface。
- `ConsentEvent`：用户对某个 bundle 的一次动作，包含 actor/web user、action、surface、session/API key、locale、IP hash、user agent hash、accepted fields、created_at。
- `ConsentRequirement`：某个入口或能力要求的最低政策状态，例如 public chat 需要 `terms/privacy current`，API key 调用还需要 `api_terms current`，数据导出需要 `privacy current`。
- `ConsentGate`：在 public auth、public chat、API key、账号高风险动作和未来多渠道引导前统一判断是否需要重新确认。

一期目标：

- 把当前 `TOS_VERSION` 升级为后端可读的 policy bundle source of truth，并让前端通过 `/api/public/policy/current` 获取当前版本。
- 新增 consent ledger 表，保留每次接受、重新接受、撤回、管理员强制重置的事件。
- 对 public cookie session、public chat、OpenAI-compatible API key 调用和 `/me` 高风险动作增加 current policy check。
- 在 `/me` 增加当前政策状态、最近接受时间、需要重新确认的原因和撤回入口说明。
- 保持旧 `tos_version` 字段作为兼容缓存，直到公开端和管理端都迁到 ledger。

## 用户体验变化

### 用户端

- 登录表单不再只显示一个静态 v2.1；它从后端读取当前政策包，展示版本、生效日期和是否包含重大变更。
- 已登录用户在政策有重大更新后访问 `/chat` 或 `/me`，会看到简短的重新确认面板，而不是莫名被登出。
- `/me` 显示当前已接受的 Terms/Privacy/投资风险确认状态、接受时间、版本和“为什么需要重新确认”。
- 用户撤回必要同意时，页面明确说明哪些能力会暂停：public chat、API key、定时任务触发、数据导出/删除流程等。
- 投资风险确认可以用独立、短句、可审计的 checkbox 表达：
  - Hone 只提供研究辅助，不构成投资建议。
  - 我会自行判断并承担投资决策结果。
  - 我理解服务可能调用第三方 LLM、市场数据和搜索服务。

### 管理端

- 管理员在用户详情中看到 consent 状态：当前版本、最近确认时间、过期原因、撤回状态、API key 是否因政策失效而受限。
- 对“用户说没有同意过新协议”“为什么 API key 变 403”“为什么 chat 要重新确认”等问题，管理员能查到事件账本，而不是只看最后一个 `tos_version` 字段。
- 未来 Operator Access Audit 落地后，管理员强制重置用户 consent requirement 的动作应写 operator audit；本提案只定义 consent 事件本身。

### 桌面端

- Remote backend 模式下，桌面 shell 不能假设本地已同意；它应展示后端返回的 policy required 状态，并跳到同一个确认面板。
- Bundled 本地模式可以默认使用本地政策包；如果公开 Web 能力或 Hone Cloud API key 被启用，也应显示对应政策状态。
- 不要求第一版修改 Tauri sidecar 生命周期，只复用 Web app 的 policy status API 与前端状态。

### 多渠道

- Feishu / Telegram / Discord / iMessage 第一版不需要直接展示完整法律文本，但当某个 actor 需要通过 Web 完成重新确认时，channel reply 可以给出短链接或“请到 Web/桌面确认后继续使用”的稳定错误。
- 多渠道 actor 与 public Web user 的合并应等 `Linked User Workspace` 类能力落地；第一版先把 Web user/API key 的 consent 做正确，并为 actor-scoped consent 预留 schema。

## 技术方案

### 1. Policy bundle source of truth

新增一个后端可序列化的 policy registry，优先放在 `crates/hone-web-api`，后续如 CLI/desktop 也需要可下沉到 `hone-core`：

```rust
pub struct PolicyBundle {
    pub id: String,
    pub version: String,
    pub effective_date: String,
    pub material_change: bool,
    pub documents: Vec<PolicyDocumentRef>,
    pub required_acknowledgements: Vec<PolicyAcknowledgement>,
}
```

第一版可以用 Rust 常量或 checked-in JSON/YAML，不需要数据库管理后台。关键是只有后端 registry 是当前版本源头；前端 `TOS_VERSION` 变成 fallback/测试 fixture，生产 UI 从 API 获取：

- `GET /api/public/policy/current`
- `GET /api/public/policy/status`
- `POST /api/public/policy/accept`

`packages/app/src/lib/tos.ts` 继续保留 `TOS_EFFECTIVE_DATE` 之类的构建期 fallback，但登录表单应优先使用 API 返回值，避免前后端常量漂移。

### 2. Consent ledger storage

在 `memory/src/web_auth.rs` 的同一 SQLite DB 或新增 `memory/src/policy_consent.rs` 中增加表：

```sql
policy_consent_events (
  event_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL,
  owner_kind TEXT NOT NULL,          -- web_user | actor | api_key
  owner_id TEXT NOT NULL,
  actor_channel TEXT,
  actor_user_id TEXT,
  actor_channel_scope TEXT,
  bundle_id TEXT NOT NULL,
  bundle_version TEXT NOT NULL,
  action TEXT NOT NULL,              -- accept | withdraw | admin_reset | migrate_legacy
  surface TEXT NOT NULL,             -- public_login | public_chat | public_api | public_me | desktop_remote
  locale TEXT,
  session_id TEXT,
  api_key_prefix TEXT,
  ip_hash TEXT,
  user_agent_hash TEXT,
  document_hashes_json TEXT NOT NULL,
  acknowledgements_json TEXT NOT NULL
);

policy_consent_state (
  owner_kind TEXT NOT NULL,
  owner_id TEXT NOT NULL,
  bundle_id TEXT NOT NULL,
  current_action TEXT NOT NULL,
  bundle_version TEXT NOT NULL,
  accepted_at TEXT,
  withdrawn_at TEXT,
  last_event_id TEXT NOT NULL,
  PRIMARY KEY (owner_kind, owner_id, bundle_id)
);
```

兼容策略：

- 迁移时读取 `web_invite_users.tos_version/tos_accepted_at`，为已有用户写一条 `migrate_legacy` 或 `accept` 事件，`surface=legacy_public_login`。
- `web_invite_users.tos_version` 暂时继续更新，作为旧 API 和管理端列表的兼容缓存。
- 不存原始 IP 或完整 UA；只存 hash 或截断摘要，避免 consent ledger 变成新的敏感数据堆。

### 3. Consent gate

新增统一判定函数：

```rust
pub enum ConsentGateDecision {
    Allow,
    RequireAcceptance { bundle: PolicyBundle, reason: ConsentReason },
    BlockWithdrawn { bundle_id: String },
}
```

接入点：

- `handle_sms_login`：继续要求用户提交当前 bundle，但使用 registry 校验，而不是硬编码 `TOS_VERSION`。
- Public cookie auth helper：如果用户 session 有效但 consent 过期，`/api/public/auth/me` 返回 `requires_policy_acceptance=true`，而不是只返回普通用户信息。
- `handle_public_chat` 和 `/api/public/v1/chat/completions`：对过期/撤回 consent 返回 `403 policy_acceptance_required`，避免旧 session/API key 绕过新条款。
- `/api/public/upload`、数据导出、删除请求、API key reset 等高风险动作：要求 privacy/current 和必要风险确认 current。
- 管理端用户页：读取 consent status，但不允许管理员代替用户接受；只能 reset requirement 或 revoke user/API key。

### 4. 前端状态与交互

前端新增 `packages/app/src/lib/policy.ts` 与可复用组件：

- `PolicyAcceptancePanel`
- `PolicyStatusBadge`
- `PolicyChangeSummary`

Public 登录和 `/me` 复用同一组件。`/chat` 的 API client 如果收到 `policy_acceptance_required`，进入确认状态；确认成功后恢复原页面，不丢失用户已经输入但未发送的消息。

内容树：

- 继续使用 `packages/app/src/lib/public-content.ts` 管理中英文 copy。
- required acknowledgements 用稳定 id，例如 `not_investment_advice`, `user_responsibility`, `third_party_processing`, `cross_border_processing`，由内容树映射为本地化短句。
- `Locale Content Contract` 提案落地后，后端错误 code 可共享 `policy_acceptance_required`, `policy_withdrawn`, `policy_bundle_mismatch`。

### 5. API key 与会话兼容

OpenAI-compatible API key 调用不能弹窗确认，因此策略应明确：

- 如果 key owner consent 过期，返回 403 JSON，错误 code 为 `policy_acceptance_required`，附 `policy_url` 和 `required_bundle_version`。
- 用户重新登录 Web 并接受后，旧 API key 可恢复；不必默认 rotate。
- 如果用户撤回必要同意，API key 应进入 suspended 状态，直到重新接受或管理员处理。

Cookie session：

- 过期 consent 不等于 session 失效；保留 session，让用户能进入 `/me` 和确认页面。
- 撤回必要 consent 后，chat/API/upload 等业务能力 blocked，但账号页、导出/删除请求说明仍可访问。

## 实施步骤

### Phase 1: 只读 registry 与状态 API

- 在后端建立当前 policy bundle registry。
- 增加 `GET /api/public/policy/current` 和 `GET /api/public/policy/status`。
- 前端登录表单和 `/me` 改为读取当前 bundle；保留本地常量 fallback。
- 增加测试覆盖前后端版本不一致时的错误状态。

### Phase 2: Consent ledger 与 legacy migration

- 新增 consent events/state 表和 storage API。
- 从 `web_invite_users.tos_version` 迁移已有接受状态。
- 登录成功写 ledger event，并继续更新旧字段。
- `/me` 展示 ledger status，而不是只展示旧字段。

### Phase 3: Enforcement gates

- Public chat、public API key、upload、API key reset、数据导出/删除请求接入 `ConsentGate`。
- 过期 consent 返回结构化错误，前端进入重新确认面板。
- API key 403 响应包含可操作的 Web URL 与 required version。

### Phase 4: 管理端与多渠道提示

- 管理端用户详情展示 consent events 和当前状态。
- Channel adapter 对 `policy_acceptance_required` 使用统一短提示。
- 如果后续 Linked User Workspace 落地，把 owner 从 `web_user` 扩展到 workspace 或 actor，并保留 web_user 兼容映射。

## 验证方式

- 单元测试：
  - policy registry 返回当前 bundle，required acknowledgement id 稳定。
  - legacy `tos_version/tos_accepted_at` 可迁移为 ledger state。
  - accept/withdraw/admin_reset 事件更新 state 且保留历史事件。
  - `ConsentGate` 对 current、outdated、withdrawn、missing state 返回正确 decision。
- Web API 测试：
  - SMS 登录提交旧版本被拒绝，新版本成功并写 ledger。
  - 已登录 cookie 在 policy bump 后 `/auth/me` 返回 `requires_policy_acceptance`。
  - public chat 和 `/api/public/v1/chat/completions` 在 consent 过期时返回 403 + code。
  - 重新接受后旧 session/API key 恢复可用。
- 前端测试：
  - 登录表单使用后端 bundle 版本渲染。
  - `/chat` 收到 `policy_acceptance_required` 后保留草稿并显示确认面板。
  - `/me` 正确展示 current / outdated / withdrawn 三种状态。
- 手工验收：
  - bump registry 版本，刷新已登录 public user，确认可进入 `/me` 重新同意，不能继续 chat 直到同意完成。
  - 使用 API key 调用旧 consent 用户，确认收到可读 403，Web 重新同意后同一 key 恢复。
  - 中英文 locale 下确认 acknowledgement 短句和 policy links 都正确。

## 风险与取舍

- **风险：确认流程过重，打断新用户试用。**
  取舍：只有 material change 或必要 bundle 过期时才重新确认；普通 copy 修订可发布为 non-material 版本，仅在 `/me` 显示更新。

- **风险：政策 registry 变成法律文本 CMS，增加维护负担。**
  取舍：第一版只管理版本、hash、required acknowledgement 和 URL，不在数据库里编辑长文本；长文本继续在现有内容树维护。

- **风险：跨渠道 actor 与 Web user 归属不清。**
  取舍：第一版 owner 以 web_user/API key 为主，schema 预留 actor/workspace；不要提前把所有 IM actor 强行绑定到手机号。

- **风险：误把 consent gate 当成投资输出安全。**
  取舍：本提案只解决用户级政策接受与能力准入；模型输出是否安全仍归 `InvestmentOutputSafetyGate`。

- **风险：历史用户迁移会产生不完整事件。**
  取舍：迁移事件显式标记 `surface=legacy_public_login`，document hash 可为空或为 legacy marker，不伪造当时的完整上下文。

## 与已有提案的差异

- 与 `auto_p0_investment_output_safety_gate.md` 不重复：该提案处理每次投资敏感输出能否送达；本提案处理用户在使用能力前是否已接受当前政策与风险确认。
- 与 `auto_p1_user-data-trust-center.md` 不重复：该提案处理数据清单、导出、删除和隐私执行面；本提案提供 consent events/state，可作为删除、撤回和数据权利流程的前置事实来源。
- 与 `auto_p2_locale-content-contract.md` 不重复：该提案处理中英文文案、API error code 和内容树契约；本提案只复用本地化 copy，核心是政策版本源头、同意账本和能力拦截。
- 与 `auto_p0_operator-access-audit.md` 不重复：该提案审计管理员和自动化访问；本提案审计用户或 actor 对政策包的接受、撤回和重新确认。
- 与 `auto_p1_hone-cloud-api-contract.md` 不重复：该提案定义公开 API 契约和开发者控制台；本提案只规定 API key owner 在政策过期时如何被阻断和恢复。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：entitlement 管权益、额度和成本；consent ledger 管政策接受和风险确认。两者都可影响能力准入，但判定依据不同。

查重结论：`docs/proposal/` 和 `docs/proposals/` 已有输出安全、用户数据、语言内容、权限、商业权益、Hone Cloud API 和 operator audit，但没有覆盖“政策包版本源头 + 用户同意事件账本 + 旧 session/API key 重新确认 gate”的主题。本提案是一个新的 P1 产品/架构改进方向。

## 文档同步说明

本轮只新增 proposal，不开始实现，因此不更新 `docs/current-plan.md`，也不归档计划页。若后续实际落地本提案，应新增或复用 `docs/current-plans/policy-consent-ledger.md`，并在引入 consent storage、public policy API、chat/API key enforcement 或多渠道提示时同步更新 `docs/repo-map.md`、`docs/invariants.md` 和必要 decision。
