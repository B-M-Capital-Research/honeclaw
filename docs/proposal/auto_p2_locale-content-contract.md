# Proposal: Locale Content Contract for Public and Operator Trust Surfaces

status: proposed
priority: P2
created_at: 2026-05-10 23:04:49 +0800
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `packages/app/src/lib/i18n.ts`
- `packages/app/src/lib/public-content.ts`
- `packages/app/src/lib/admin-content/shared.ts`
- `packages/app/src/lib/admin-content/structure.test.ts`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/public-home.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/components/notification-preferences-card.tsx`
- `crates/hone-web-api/src/routes/meta.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `bins/hone-cli/src/i18n.rs`
- `bins/hone-cli/src/onboard.rs`

## 背景与现状

Honeclaw 已经从单机投研助手扩展成公开网站、公共 Web Chat、管理端、桌面端、本地 CLI、多渠道消息和主动推送组合起来的产品。当前仓库已经有几块很有价值的本地化基础：

- `packages/app/src/lib/i18n.ts` 提供 `useLocale()`、`setLocale()`、`makeContentProxy()`、`tpl()`、`plural()`、`formatDate()`、`formatNumber()`，并说明 public surface 默认跟随浏览器语言，admin surface 可从 `/api/meta.language` 引导。
- `packages/app/src/lib/public-content.ts` 将公开站点文案放进中英文平行树，法律条款、导航、首页、路线图、登录等页面已经大量复用 `CONTENT`。
- `packages/app/src/lib/admin-content/*` 将管理端页面文案拆成平行内容树，`structure.test.ts` 已经开始做 shape parity 检查。
- `bins/hone-cli/src/i18n.rs` 和 `hone-cli onboard` 已经让首次安装选择 `config.yaml.language`，并把 CLI / console 默认语言纳入配置。
- `crates/hone-web-api/src/routes/meta.rs` 暴露 `/api/meta` 与 `/api/language`，说明语言已经被视为 runtime product setting，而不是纯前端状态。

但当前实现仍存在明显断层：

- `packages/app/src/pages/public-portfolio.tsx` 是登录用户长期投资上下文入口，却硬编码了大量中文 UI、错误、按钮、空状态和解释文案，例如“投资上下文”“立即刷新”“加载失败”“公司画像 (read-only)”等。
- `packages/app/src/pages/chat.tsx` 的聊天加载状态仍有硬编码中文，例如“上滑加载更早消息”，而 public chat 是新用户最先体验的核心产品面。
- `packages/app/src/pages/users.tsx`、`packages/app/src/components/symbol-drawer.tsx` 等页面仍直接使用 `zh-CN` 日期格式，绕过了 `useLocale()` / `formatDate()`。
- 后端 public / admin API 的用户可见错误多数是中文字符串，例如 `public.rs`、`public_digest.rs`、`notification_prefs.rs` 中的登录、密码、画像、prefs 校验错误。前端直接展示这些错误时，英文用户会看到中文故障。
- `README.md` 面向英文用户展示开源项目，但运行后的 public product 仍可能在关键路径混用中英文，削弱国际开源转化、商业试用和客服可信度。

这说明 Hone 已经有本地化工具，却还没有一个跨 public/admin/desktop/CLI/API 的内容契约来决定：哪些字符串必须进入内容树，哪些后端错误必须结构化，如何验证新增页面不会退回硬编码中文，以及如何让 agent / automation 在改 UI 时不破坏双语体验。

## 问题或机会

这不是“把中文翻成英文”的小修补，而是一个产品架构问题。

1. **公开产品信任受损。** Hone 的 public surface 承担获客、邀请登录、首次 chat、投资上下文和用户账号管理。用户在英文站点进入 `/portfolio` 或遇到 API 错误后看到中文，会直接降低产品完成度和付费/留存信任。

2. **桌面和本地安装体验不连续。** CLI onboarding 已经能选择语言，admin console 也能保存语言，但用户进入部分页面时仍回到硬编码中文。语言设置无法成为稳定的产品承诺。

3. **客服和协作成本升高。** 后端错误直接返回自然语言中文，前端没有稳定 `code + localized message` 映射。后续若要做客服面板、公开 API、Hone Cloud SDK 或诊断包，很难按错误类别统计、搜索和本地化。

4. **自动化改 UI 容易回归。** 仓库已有内容树 shape test，但没有扫描硬编码用户可见文案，也没有约束新 public/admin 页面必须通过 `CONTENT` / `admin-content`。自动化持续产出页面或功能时，很容易继续把“临时中文”写进产品层。

5. **开源增长机会被浪费。** Hone 的 README、网站和安装路径都面向中英文用户。完善双语内容契约可以让国际用户更容易自助安装、理解错误、配置渠道和体验长期投资记忆，不必先加入中文社群才能排障。

## 方案概述

新增一个轻量但强约束的 **Locale Content Contract**，把语言支持从“前端工具函数”提升为 public/admin/desktop/CLI/API 共享的产品契约。

核心做法：

1. 建立内容域边界：
   - public product copy 继续归 `packages/app/src/lib/public-content.ts`。
   - admin/operator copy 继续归 `packages/app/src/lib/admin-content/*`。
   - shared formatting / plural / date / number 统一走 `packages/app/src/lib/i18n.ts`。
   - CLI copy 继续归 `bins/hone-cli/src/i18n.rs`，但和 Web 共享命名规范与错误 code。
   - API 不再把所有用户可见错误只作为中文文本返回；新增稳定 `error_code`，前端按 locale 映射展示。

2. 给用户可见页面定义 hardcoded-copy budget：
   - public surface：除品牌名、ticker、协议字段、ARIA technical labels 外，不允许新增裸中文/英文长句。
   - admin surface：新增页面必须放入对应 `admin-content/<page>.ts`，并通过 shape parity test。
   - debug/log/raw backend messages 可以保留原文，但 UI 必须明确标为 raw detail，不作为主要用户文案。

3. 将后端错误分层：
   - `message` 保留当前兼容字段，短期不破坏现有前端。
   - 新增 `code` / `params` / `severity` / `docs_hint` 可选字段。
   - 前端优先用 `code` 映射本地化文案；没有映射时才展示 `message`。

4. 增加自动化验证：
   - 扩展 `packages/app/src/lib/admin-content/structure.test.ts` 的思路，为 `public-content` 也做 shape parity。
   - 增加 CI-safe 文案扫描脚本，扫描 public/admin TSX 中明显的中文裸字符串、`zh-CN` 直接调用、`Loading…` / error hardcode 等。
   - 脚本允许注释白名单，避免误伤代码注释、测试 fixture、品牌名和 raw log viewer。

5. 先迁移最高信任面，不追求一次性全仓清零：
   - 第一阶段只覆盖 public `/chat`、`/me`、`/portfolio`、public auth errors、public digest errors。
   - 第二阶段覆盖 admin `/users`、`/notifications`、`/schedule`、settings 中仍残留的 inline 文案。
   - 第三阶段再把 channel / CLI / desktop 的用户可见错误 code 对齐。

## 用户体验变化

### 用户端

- 公开网站、登录、Chat、个人页、投资上下文页在中英文切换后保持一致语言，不再在英文模式中突然出现中文提示。
- `/portfolio` 的画像缺口、蒸馏失败、刷新成功、空状态和只读说明都会使用用户当前 locale。
- 后端错误会显示用户能理解的文案，例如密码错误、邀请码失效、画像不存在、portfolio 为空、digest refresh 失败，而不是直接暴露中文 Rust error。
- 用户可以更清楚地区分“产品提示”和“原始诊断 detail”；debug detail 可以折叠展示。

### 管理端

- 管理端继续保留 bilingual operator console，但新增页面必须有内容树和 parity test。
- `/users`、`/notifications`、`/schedule` 这类客服/运营页面在英文环境下更适合给海外用户排障截图，不需要人工翻译每个状态。
- API error code 让管理端可以聚合“常见失败原因”，例如 `public_portfolio_empty`、`profile_not_found`、`prefs_invalid_timezone`。

### 桌面端

- 桌面 bundled 模式继承 `config.yaml.language` 后，Web shell、settings、public-like account views 和错误提示不再混合语言。
- 本地用户导出诊断或查看错误时，UI 用本地化摘要，保留 raw error 给高级排障。

### 多渠道

- Feishu / Telegram / Discord 等渠道不必第一阶段全面改造，但工具和 API 可逐步返回 `error_code`，让 channel adapter 以后按用户/actor language 渲染。
- `notification_preferences`、`portfolio_management` 等 skill 的工具错误可以先保留自然语言，再逐步加 code，以免一次性改动 runner contract。

## 技术方案

### 1. 内容树与页面迁移

- 在 `packages/app/src/lib/public-content.ts` 新增 `portfolio`, `chat_runtime`, `digest_context`, `public_errors` 等节点。
- 将 `packages/app/src/pages/public-portfolio.tsx` 的硬编码中文迁到 `CONTENT.portfolio`：
  - 页面标题、说明、按钮、刷新状态、profile modal、空状态、错误 fallback。
  - 日期格式改用 `formatDate()` 或 `Intl.DateTimeFormat` 包装函数。
- 将 `packages/app/src/pages/chat.tsx` 中的聊天 runtime 文案迁到 `CONTENT.chat` 或 `CONTENT.chat_runtime`，避免 public chat 在英文 locale 下出现中文加载提示。
- 对 admin 侧零散 `zh-CN` 日期调用，改用 `useLocale()` 或 `formatDate()`，但保留 raw log timestamp 的机器格式。

### 2. 后端错误 code 兼容层

新增一个小型错误响应 helper，不立刻重写所有 route：

```rust
pub struct ApiErrorBody {
    pub error: String,
    pub code: Option<String>,
    pub params: Option<serde_json::Value>,
}
```

兼容策略：

- `json_error(status, message)` 保持可用。
- 新增 `json_error_code(status, code, fallback_message, params)`。
- public auth / public digest / notification prefs 优先迁移：
  - `invite_missing`
  - `invite_invalid_or_expired`
  - `password_too_weak`
  - `public_portfolio_missing`
  - `public_portfolio_empty`
  - `company_profile_not_found`
  - `prefs_invalid_timezone`
  - `prefs_invalid_kind_tag`
  - `digest_refresh_failed`
- 前端 `apiFetch` / `parseJson` 捕获错误时保留 `code` 和 `params`，页面通过内容树映射。

### 3. 前端错误映射

在 `packages/app/src/lib/api.ts` 或新增 `packages/app/src/lib/api-errors.ts` 中定义：

```ts
export type ApiErrorCode =
  | "invite_missing"
  | "invite_invalid_or_expired"
  | "password_too_weak"
  | "public_portfolio_missing"
  | "public_portfolio_empty"
  | "company_profile_not_found"
  | "prefs_invalid_timezone"
  | "prefs_invalid_kind_tag"
  | "digest_refresh_failed"
```

页面展示优先级：

1. `CONTENT.errors[code]` + `params`
2. route-specific fallback
3. backend `message`
4. generic localized unknown error

### 4. CI-safe 文案扫描

新增脚本建议：

- `tests/regression/ci/test_locale_content_contract.sh`
- 或 `packages/app/src/lib/content-contract.test.ts`

扫描规则第一版保持保守：

- public TSX 中出现中文字符且不在注释、测试、导入的 content 文件内，报错。
- public/admin TSX 中直接 `toLocaleString("zh-CN"` 或 `toLocaleDateString("zh-CN"`，报错。
- 新增 `Loading…`、`Error:`、`加载失败` 等裸字符串时提示迁移到内容树。
- 允许白名单文件：
  - `public-content.ts`
  - `admin-content/*.ts`
  - `*.test.ts`
  - markdown / docs / comments

这类脚本不需要外部账号，适合进入 `tests/regression/run_ci.sh`。

### 5. 文档与 agent 协作规则

实现阶段应更新：

- `docs/repo-map.md`：补充 Web i18n / content contract 入口与 common coupled changes。
- `docs/invariants.md`：新增“public/admin 用户可见文案必须走内容树或 error code 映射”的长期约束。
- 可选新增 `docs/runbooks/localization-content.md`：说明如何给页面加双语文案、如何新增 API error code、如何跑扫描脚本。

本提案本身只创建 proposal，不修改上述文档；真正实施时再同步。

## 实施步骤

### Phase 1: Contract and Public Trust Surfaces

1. 为 `public-content.ts` 增加 `portfolio`、`chat_runtime`、`errors` 节点，并补齐中英文 shape。
2. 迁移 `public-portfolio.tsx` 的所有用户可见硬编码文案。
3. 迁移 `chat.tsx` 中 public chat runtime 的明显硬编码加载/错误文案。
4. 为 `public-content` 增加 shape parity test，和 admin-content 一样防止中英文树漂移。
5. 为 public pages 增加保守 hardcoded-copy 扫描。

### Phase 2: API Error Codes

1. 新增兼容的 `json_error_code` helper。
2. 迁移 `public.rs`、`public_digest.rs`、`notification_prefs.rs` 中会直接展示给用户的错误。
3. 前端 `api.ts` 保留 structured error，public pages 使用 `CONTENT.errors` 渲染。
4. 添加 route-level unit tests，确保 code、fallback message 和 HTTP status 都存在。

### Phase 3: Operator and Desktop Consistency

1. 扫描 admin TSX 中直接 `zh-CN` 格式化与裸字符串，按页面迁入 `admin-content`。
2. Settings 中将 `config.yaml.language`、localStorage override、public surface locale 的关系解释清楚。
3. 桌面 bundled 模式验证：首次启动语言、切换语言、刷新页面、重启 sidecar 后一致。

### Phase 4: Channel and Tool Gradual Alignment

1. 给高频工具错误增加 code，但不强制一次性改 runner contract。
2. 对 channel adapter 设计 `actor_locale` 查询入口，后续让 IM 错误按用户语言输出。
3. 将本地化错误 code 纳入诊断包和客服排障面板。

## 验证方式

- `bun run test:web`：
  - `public-content` / `admin-content` shape parity。
  - API error parser 能保留 `code` / `params` / fallback message。
  - `public-portfolio` 在 zh/en locale 下关键标签不同且不丢字段。
- `bash tests/regression/run_ci.sh`：
  - 新增 locale content contract 脚本，扫描 public/admin TSX 的硬编码文案回归。
- Rust unit tests：
  - `json_error_code` 输出兼容旧 `error` 字段。
  - public auth / digest / prefs 关键错误返回稳定 code。
- 手工验收：
  - 浏览器 locale 为英文时打开 `/`, `/chat`, `/me`, `/portfolio`，不出现中文主要文案。
  - 设置 admin language 为 English 后打开 `/users`, `/notifications`, `/schedule`, `/settings`，主要控件和日期跟随英文。
  - 构造画像不存在、portfolio 为空、邀请码错误、密码弱等失败路径，前端展示英文文案，raw detail 可追踪。
- 指标：
  - public hardcoded-copy scanner warning 数量持续下降。
  - 英文用户首轮 chat / portfolio 页面错误退出率下降。
  - support 截图中混合语言问题减少。

## 风险与取舍

- **风险：迁移范围容易膨胀。** 取舍是先迁 public trust surfaces 和高频错误，不追求全仓一次性无裸字符串。
- **风险：error code 设计过早固化。** 第一版只给 public/auth/digest/prefs 的稳定错误加 code，内部 debug error 继续走 raw message。
- **风险：内容树变大后维护成本上升。** 继续用 shape parity test，按页面拆分 admin content；public content 若继续膨胀，可拆成 `public-content/*.ts`。
- **风险：扫描脚本误报。** 第一版保守，只扫 public/admin TSX 的明显中文和直接 locale 格式化；允许注释白名单，并把 raw log/detail 页面排除。
- **风险：后端 fallback message 仍是中文。** 为兼容旧前端可以保留，但新前端优先 code 映射；后续再逐步让 fallback message 也按 `Accept-Language` 或 config language 生成。
- **不做的边界：** 不在本提案里重写所有 skill prompt、模型回答语言策略或多渠道消息语言检测；LLM 回答语言仍由用户输入和 prompt 约束处理。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 与历史 `docs/proposals/`：

- 不重复 `auto_p1_invite_activation_funnel.md`：该提案关注邀请用户从注册到首个价值行为的 milestone；本提案关注所有 public/admin/desktop 信任面的语言一致性和错误契约。
- 不重复 `auto_p1_user-data-trust-center.md`：该提案关注用户数据 inventory、导出、删除和隐私信任；本提案关注用户可见文案、错误 code 和本地化验证。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：该提案关注模型、渠道、provider、sidecar 是否 ready；本提案关注 ready / not ready 这些状态如何被双语、结构化地解释给用户和 operator。
- 不重复 `auto_p1_delivery_decision_loop.md`：该提案聚焦通知为什么发送/不发送以及 prefs 调整闭环；本提案只把通知偏好相关错误和设置文案纳入统一内容契约，不改变 delivery decision。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：该提案关注 answer quality feedback 和偏好学习；本提案关注固定产品 UI / API error 的语言契约，不学习用户投资或回答偏好。
- 不重复 `desktop-bundled-runtime-startup-ux.md`：该历史提案关注桌面 sidecar 启动、锁、接管和状态；本提案只要求桌面表层继承同一 locale contract。
- 不重复 `skill-runtime-multi-agent-alignment.md`：该历史提案关注 skill runtime 和 multi-agent 执行；本提案不改 skill 注入模型，只给工具/API错误未来 code 化留入口。

## 文档同步说明

本轮只创建 proposal，不开始实施，不修改 `docs/current-plan.md`，也不归档任何活跃任务。若后续执行本提案，应按影响范围更新 `docs/repo-map.md`、`docs/invariants.md`，并视需要新增 localization runbook。
