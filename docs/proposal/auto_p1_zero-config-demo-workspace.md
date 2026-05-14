# Proposal: Zero-Config Demo Workspace for First-Run Product Evaluation

status: proposed
priority: P1
created_at: 2026-05-14 14:03:21 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `config.example.yaml`
- `bins/hone-cli/src/onboard.rs`
- `bins/hone-cli/src/start.rs`
- `crates/hone-channels/src/runners/hone_cloud.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/public-home.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/lib/public-chat.ts`
- `memory/src/portfolio.rs`
- `memory/src/company_profile/types.rs`
- `memory/src/cron_job/types.rs`
- `skills/company_portrait/SKILL.md`
- `skills/scheduled_task/SKILL.md`

## 背景与现状

Honeclaw 已经具备相当完整的真实产品链路：

- README 给出的启动路径包括 `hone-cli onboard`、`hone-cli start`、admin UI 和 public user UI。
- `config.example.yaml` 保留多个真实外部依赖入口：Hone Cloud / OpenRouter / OpenCode / FMP / Tavily / Feishu / Telegram / Discord / Aliyun SMS。
- Public Web 已拆成独立 surface，`packages/app/src/app.tsx` 暴露 `/`、`/roadmap`、`/chat`、`/me`、`/portfolio` 等用户端路径。
- Public `/chat` 当前通过 `getPublicAuthMe()` 恢复登录态；未登录时展示 `PublicLoginForm`，登录后再调用 `/api/public/chat` 和 `/api/public/history`。
- `crates/hone-web-api/src/routes/public.rs` 要求手机号、白名单 invite、TOS、SMS / captcha 校验和 HttpOnly session，适合真实试用用户，但不适合开源访问者无账号快速体验。
- 本地 runner 默认可走 `hone_cloud`，但 `crates/hone-channels/src/runners/hone_cloud.rs` 在 API key 为空时直接失败并提示联系获取 key。
- Public `/portfolio` 和 `public_digest` 已能展示持仓、投资主线、公司画像和画像 Markdown，但这些都依赖当前 actor 已经有 portfolio、company profiles、prefs 和可用 LLM 蒸馏。
- `memory/src/portfolio.rs`、`memory/src/company_profile/types.rs` 和 `memory/src/cron_job/types.rs` 已经有足够清晰的数据形状，可以构造一个代表性投资工作区样本。

也就是说，Hone 的真实能力已经很强，但第一次接触产品的人通常要先跨过至少一个外部门槛：

- public 端需要邀请码、手机号、短信和白名单；
- 本地端需要配置 runner 或 Hone Cloud API key；
- portfolio / company portrait / scheduled task 的核心价值需要先有数据；
- 真实多渠道、事件引擎和市场数据需要额外凭证。

这会让产品评估路径偏重“配置成功”，而不是先理解 Hone 与普通聊天工具的差异：投资上下文、长期公司画像、自动化提醒、多渠道工作台和纪律化研究流程。

## 问题或机会

这是 P1 机会，因为它直接影响开源仓库转化、public trial 转化、桌面首次体验和贡献者效率，而且可以作为只读 / 沙盒能力落地，不要求先接支付、身份合并或真实券商。

### 问题

1. **首次体验被外部凭证阻塞。**  
   开源用户即使成功安装，也可能因为没有 Hone Cloud key、OpenCode 配置、FMP/Tavily key 或 public invite 而无法看到核心产品形态。

2. **空状态无法展示长期价值。**  
   `/portfolio` 在没有 holdings/profile/prefs 时只能引导用户去 chat 补信息；这对真实用户合理，但对评估者来说，他们还没有理解“补完之后会长什么样”。

3. **视频和截图不能替代可操作体验。**  
   `public-home.tsx` 已经嵌入 demo video 和案例 carousel，但用户无法在本机点击一个完整样例，查看 portfolio、画像、自动化任务、会话和消息渲染如何联动。

4. **开发者难以稳定复现产品状态。**  
   前端、Web API、公司画像、cron、event digest、public chat 的联动依赖多份 actor 数据。没有标准 demo fixture 时，开发者修 UI 或写 proposal 往往要手工造数据。

5. **现有 activation / entitlement 提案仍以真实用户为中心。**  
   `Invite Activation Funnel` 回答真实 invite 用户卡在哪个里程碑；`Usage Entitlement Ledger` 回答真实权益和成本；它们不负责“未登录、无 key、无数据也能体验产品”。

### 机会

新增 **Zero-Config Demo Workspace**：一个内置、可重置、明确标记为示例的 actor-scoped 沙盒工作区，让用户不配置外部模型和市场数据也能浏览 Hone 的核心信息架构，并在可选情况下用 stub runner 体验有限对话。

目标不是伪造真实投资能力，而是把产品价值展示从“看视频 / 读 README / 先配 key”提前到“打开一个可交互样例工作台”。

## 方案概述

增加一个明确的 demo mode，包含三层能力：

1. **Demo Data Pack**  
   仓库内保留一组无隐私、无真实用户、可重置的样本资产：
   - 一个 demo actor，例如 `channel=demo,user_id=sample-investor`。
   - 3 到 5 个 holdings / watchlist 条目，覆盖真实持仓、关注标的、长期 / 短期 horizon、策略 notes。
   - 2 到 3 个 company profiles，包含 `profile.md`、dated events、风险、证伪条件、investment mainline。
   - 2 个 scheduled tasks，例如“盘前摘要”和“财报后复盘”，但默认不真实投递。
   - 一小段只读 chat history，展示问题、工具/技能步骤摘要、最终回答和 chart/image marker。
   - 一份静态 digest context，展示 mainline_by_ticker、skipped ticker 和更新时间。

2. **Demo Workspace API**  
   Web API 增加只读 demo endpoint 或 demo actor resolution。它从 fixture/data pack 读取，不写入真实用户数据，不进入正常 quota、SMS、API key、channel delivery。

3. **Demo UI Entry**  
   Public home、public chat logged-out 状态、desktop first-run 和 CLI onboarding 都可以提供“Explore demo workspace”入口。用户进入后能查看完整工作台，所有关键位置都标注 `Demo data`，并提供清晰的“开始真实配置 / 登录 / 运行 onboard”下一步。

第一版以只读体验为主：portfolio、画像、任务、研究主线、历史会话、渠道预览都可浏览；对话可以先用 deterministic stub runner 返回少量固定场景，不允许写 portfolio、画像或真实 cron。

## 用户体验变化

### 用户端

- Public home 的主 CTA 旁新增一个低承诺入口：`View demo workspace`。
- 未登录的 `/chat` 页面除了手机号登录，也可进入 demo chat。
- Demo `/portfolio` 不再是空态，而是展示一组完整样例：
  - 持仓与关注列表；
  - 每个标的的投资主线；
  - 公司画像 read-only modal；
  - 待复盘事件；
  - scheduled task 示例。
- Demo chat 顶部固定显示“示例工作区，不保存到你的账户”。用户可以点击几条推荐问题，例如：
  - “为什么这个组合需要区分长期主线和短期噪音？”
  - “MU 的画像里有哪些证伪条件？”
  - “这个盘前摘要会如何选择推送内容？”
- 用户随时可以从 demo 跳到真实登录、申请 invite、运行本地 `hone-cli onboard` 或下载桌面端。

### 管理端

- Admin dashboard 可提供 `Open demo actor`，用于排查 UI、演示产品和培训运营，不混入真实 invite 用户。
- `/users` 可以只在显式 demo mode 下展示 demo actor，且 badge 标明不是可运营用户。
- Settings 中可显示 demo pack 版本和 fixture 自检状态，方便开发者确认样本数据是否仍和当前 schema 兼容。

### 桌面端

- Desktop bundled 首次启动时，如果 runner 或 Hone Cloud key 未配置，可以先进入 demo workspace，而不是只显示配置缺口。
- Remote mode 可以由远端 backend 返回 `demo_workspace` capability；本地 desktop 不伪造远端状态。
- Demo 入口应明确不会启动真实渠道进程、不会投递 IM、不会写用户 config。

### 多渠道

- 第一版不需要在 Feishu / Telegram / Discord 中开放 demo conversation，避免把示例数据误发到真实群聊。
- 管理端可以提供 multichannel render preview 的 demo 内容，供配置渠道前预览消息在不同平台的呈现。
- 未来如要在 IM 中支持 demo，应仅允许管理员私聊触发，并带清晰示例标识。

## 技术方案

### 1. Demo data pack

建议新增目录：

```text
demo/
  workspace/
    manifest.json
    portfolio.json
    digest_context.json
    sessions.json
    cron_jobs.json
    company_profiles/
      mu/profile.md
      mu/events/2026-04-earnings.md
      tsm/profile.md
      nvda/profile.md
```

`manifest.json` 至少记录：

- `version`
- `updated_at`
- `actor`
- `fixtures`
- `compatible_schema`
- `disclaimer`

Demo data 不进入 `data/` 默认 runtime 目录，避免被误认为用户资产。启动时只读加载，或通过显式 `hone-cli demo seed --data-root <tmp>` 复制到临时 actor sandbox。

### 2. Demo actor 与访问边界

定义一个明确的 demo subject：

```text
ActorIdentity {
  channel: "demo",
  user_id: "sample-investor",
  channel_scope: None
}
```

边界规则：

- Demo actor 不允许进入真实 quota、entitlement、SMS auth、public invite、API key、channel delivery。
- Demo actor 不允许写 `storage.portfolio_dir`、真实 actor sandbox、cron store 或 notification prefs。
- Demo actor 的路径必须来自 `demo/workspace` 或启动时创建的临时目录，并在 API payload 中标记 `demo: true`。
- 若某个 API 不能保证只读，第一版不要接 demo。

### 3. Web API

建议新增只读 routes，例如：

- `GET /api/public/demo/workspace`
- `GET /api/public/demo/history`
- `GET /api/public/demo/digest-context`
- `GET /api/public/demo/company-profile?ticker=MU`
- `GET /api/public/demo/tasks`
- `POST /api/public/demo/chat`

`POST /api/public/demo/chat` 第一版可以只支持 deterministic replies：

- 根据用户选择的 sample prompt 返回固定 Markdown；
- 返回同样的 SSE event shape：`run_started / assistant_delta / run_finished`；
- 不调用真实 runner，不消耗 quota，不保存长期 session。

这样 public chat UI 可以复用大部分渲染逻辑，同时不会把 demo 误包装成真实 AI 能力。

### 4. Frontend

Public surface 增加 demo route：

- `/demo`
- `/demo/chat`
- `/demo/portfolio`

实现上优先复用现有组件和类型：

- `packages/app/src/lib/public-chat.ts` 增加 demo history transform，仍输出 `PublicChatMessage`。
- `public-portfolio.tsx` 抽出 portfolio/digest/profile 渲染子组件，让真实 `/portfolio` 和 demo `/demo/portfolio` 共用。
- `PublicLoginForm` 附近加 demo 入口，但不要让 demo 看起来像已经登录。
- 所有 demo 页面固定显示 `Demo workspace` badge。

### 5. CLI / Desktop

CLI 可新增轻量入口：

- `hone-cli demo`：启动 local backend + public UI，并打开 demo route。
- `hone-cli onboard` 结束时，如果用户跳过 runner 或 provider key，提示可以先运行 `hone-cli demo` 查看产品样例。
- `hone-cli doctor` 可检查 demo data pack 是否可解析，用于贡献者本地验证。

Desktop：

- 如果 runtime readiness 阻塞真实 chat，显示 `Explore demo workspace`。
- Demo mode 只访问 backend demo endpoints，不改 desktop sidecar settings。

### 6. 测试与兼容

新增 CI-safe 验证：

- Rust：解析 `demo/workspace/portfolio.json` 为 `Portfolio`；解析 `company_profiles/*/profile.md` 为 company profile reader 可接受格式；解析 `cron_jobs.json` 为 `CronJobData`。
- Rust API：demo endpoints 无需 cookie，返回 `demo: true`，且不写真实 storage。
- Frontend：demo chat history 和 real history 都能通过同一 message transform；demo portfolio 空态 / 有数据态稳定。
- Regression：`tests/regression/ci/test_demo_workspace.sh` 启动最小 backend 或调用 fixture validator，确保文件存在、schema 可解析、route 返回只读 payload。

## 实施步骤

1. **定义 demo pack schema**
   - 先写 `demo/workspace/manifest.json` 和最小 fixtures。
   - 明确 fixture 只使用公开示例，不含真实用户持仓、聊天、API key、内部 prompt。

2. **实现 fixture validator**
   - 在 Rust 或脚本中校验 portfolio、cron、company profiles、digest context。
   - 加入 CI-safe regression，防止后续 schema 演进打破 demo。

3. **增加只读 demo Web API**
   - 新增 demo routes。
   - 确保所有返回都带 `demo: true`。
   - 禁止 demo route 写入真实 actor storage。

4. **复用前端渲染组件**
   - 抽出 public portfolio 的数据渲染部分。
   - 新增 `/demo`、`/demo/chat`、`/demo/portfolio`。
   - Logged-out chat 页面增加 demo 入口。

5. **加入 CLI / Desktop 入口**
   - `hone-cli demo` 作为本地评估快捷入口。
   - Desktop readiness blocked 时显示 demo CTA。

6. **灰度与文档**
   - README 增加“无 key 体验 demo workspace”。
   - Public home 增加 demo CTA。
   - 保留 demo disclaimer，避免被误认为投资建议或真实行情。

## 验证方式

- `bash tests/regression/ci/test_demo_workspace.sh`
  - demo manifest 存在；
  - 文件名和 schema 可解析；
  - company profile Markdown reader 不报错；
  - cron/portfolio fixture 可 deserialize；
  - demo API 不需要 public cookie；
  - demo API 返回 `demo: true`。
- `bun run test:web`
  - demo history transform 与真实 public history transform 共用测试；
  - demo portfolio 在有 profile、缺 profile、skipped mainline 三种状态下渲染稳定。
- 手工验收：
  - 无 `config.yaml` API key、无 SMS、无 invite 时，用户能从 public home 进入 demo；
  - demo chat 可流式展示固定回复；
  - demo portfolio 可打开画像；
  - demo 页面所有 CTA 都能引导到真实登录 / onboard / GitHub；
  - demo 不会创建真实 cron、不会写真实 portfolio、不会触发 channel delivery。
- 指标：
  - public home -> demo 点击率；
  - demo -> login / invite / GitHub / install 的转化；
  - demo 页面平均停留；
  - demo 入口后真实 onboarding 成功率。

## 风险与取舍

- 风险：用户误以为 demo 是真实投资建议。  
  取舍：所有 demo 页面和回复都标记为示例，样本数据使用公开虚构持仓组合，不展示实时价格，不输出“应买/应卖”。

- 风险：引入 mock runner 后掩盖真实配置问题。  
  取舍：demo runner 只在 `/demo/*` 或 `channel=demo` 下工作；真实 `/chat` 仍走 runtime readiness 和真实 runner。

- 风险：fixture 跟 schema 演进漂移。  
  取舍：把 demo fixture validator 纳入 CI-safe regression；schema 改动必须同步更新 demo pack。

- 风险：demo data 被误写入用户存储。  
  取舍：demo data 默认只读加载；如需 seed，也只写临时 data root，并把 actor channel 固定为 `demo`。

- 风险：增加前端路由和组件复杂度。  
  取舍：优先抽出公共渲染组件，避免复制一套 demo UI；第一版不追求所有 admin 页面都支持 demo。

- 不做边界：
  - 不接真实行情；
  - 不投递真实 IM；
  - 不创建真实 invite user；
  - 不绕过 public auth 访问真实用户数据；
  - 不把 demo metrics 当成真实投资使用指标。

## 与已有提案的差异

查重范围：已检查 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案标题，并全文检索 `demo`、`sample`、`synthetic`、`fixture`、`trial`、`空态`、`示例工作区`、`zero-config` 等关键词。

- 与 `auto_p1_invite_activation_funnel.md` 不重复：该提案服务真实 invite 用户的阶段化激活与运营跟进；本提案服务未登录、无 invite、无模型 key 的评估者和贡献者。
- 与 `auto_p1_investment_context_intake.md` 不重复：该提案帮助真实用户补齐自己的投资上下文；本提案提供不可写的样本上下文，用于先理解产品。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：该提案处理权益、成本、trial plan 和用量；本提案的 demo route 不计费、不消耗额度、不进入 entitlement。
- 与 `auto_p1_runtime_readiness_matrix.md` 不重复：该提案判断真实部署是否 ready；本提案在真实部署未 ready 时提供安全的样例体验，但不隐藏 readiness 阻塞。
- 与 `auto_p1_investment_playbook_launcher.md` 不重复：该提案把真实研究工作流模板化启动；本提案展示一组已完成/可浏览的样例工作区。
- 与 `auto_p1_research_artifact_library.md` 不重复：该提案持久化真实研究交付物；本提案只提供演示用 artifact-like fixture。
- 与 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 不重复：该历史提案聚焦桌面 bundled startup；本提案覆盖 public Web、CLI、desktop 和开发者 fixture。

结论：现有提案覆盖真实用户激活、权益、投资上下文、运行可用性、研究交付物和桌面启动体验，但没有覆盖“零配置、无登录、无外部凭证的可交互产品样例工作区”。因此本提案是新的、可落地的 P1 产品/架构提案。
