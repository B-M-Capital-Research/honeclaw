# Proposal: Workspace Command Palette and Asset Search

status: proposed
priority: P1
created_at: 2026-05-20 02:03:46 +0800
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
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p2_surface-design-contract.md`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/layout.tsx`
- `packages/app/src/components/sidebar-nav.tsx`
- `packages/app/src/context/sessions.tsx`
- `packages/app/src/pages/users.tsx`
- `packages/app/src/context/research.tsx`
- `packages/app/src/context/tasks.tsx`
- `packages/app/src/context/company-profiles.tsx`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/routes/users.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/cron.rs`
- `crates/hone-web-api/src/routes/research.rs`
- `memory/src/session.rs`
- `memory/src/company_profile/storage.rs`
- `memory/src/cron_job/mod.rs`
- `memory/src/portfolio.rs`

## 背景与现状

Honeclaw 已经从单一聊天入口演进成一个多资产投资工作台。当前仓库里已经有这些相对成熟的对象和页面：

- `packages/app/src/app.tsx` 把 admin surface 拆成 dashboard、sessions、skills、tasks、users、research、llm-audit、logs、task-health、notifications、schedule、settings；public surface 拆成 home、chat、me、portfolio、roadmap 和 legal pages。
- `packages/app/src/pages/layout.tsx` 在 admin console 中根据当前模块切换左侧列表：session list、skill list、task list、actor list、research list。主内容区再进入对应页面。
- `packages/app/src/components/sidebar-nav.tsx` 把导航分成用户视图、研究、系统等模块，但导航仍是固定菜单，不承担搜索或命令分发。
- `crates/hone-web-api/src/routes/users.rs` 可以从 session storage 派生所有会话用户列表，并生成最后消息、最后时间和消息数。
- `crates/hone-web-api/src/routes/history.rs` 可以按 session/actor 读取最近历史，并抽取附件和本地图片 marker。
- `crates/hone-web-api/src/routes/company_profiles.rs`、`portfolio.rs`、`cron.rs`、`research.rs`、`notifications.rs`、`task_runs.rs` 分别暴露公司画像、持仓、任务、深度研究、通知和任务运行记录。
- `memory/src/session.rs` 已经把 session 规范化到 version 4，并支持 SQLite 索引；company portraits、portfolio、cron jobs 也都有 actor-scoped 存储。

这些能力说明 Hone 已经有很多“可被找回和继续操作”的资产：会话、用户 actor、持仓、关注标的、公司画像、画像事件、定时任务、研究报告、通知记录、模型审计、日志、技能和设置项。

但产品入口仍然是页面优先、列表优先、模块优先。用户或管理员必须先知道某个对象属于哪个页面，再进入页面筛选或点击。例如：想找 “MU 相关的会话、画像、持仓、研究任务和提醒” 时，需要在 sessions、users/portfolio、users/memory、research、tasks、notifications 中反复切换。桌面端承载同一套 Web console，这个问题会被放大，因为桌面工作台的高频动作更接近 “Cmd+K 找对象/执行动作”，而不是完整浏览每个页面。

现有提案已经覆盖了很多资产本身的建设：Linked User Workspace 解决跨渠道身份与资产归属，Research Artifact Library 解决深度研究报告留存，User Data Trust Center 解决用户数据盘点与导出删除，Run Trace Workbench 解决单次运行可观测，Investment Playbook Launcher 解决标准工作流启动。本提案关注的不是新增某一类资产，而是为已经存在和未来新增的资产建立一个跨 surface 的查找与命令层。

## 问题或机会

这是 P1 级机会，因为 Hone 的核心体验正在从“问一次问题”转向“维护长期投资工作区”。当资产数量增长后，找不到、想不起、无法从一个对象跳到下一步，都会直接削弱留存、运维效率和商业化体验。

主要问题：

1. **资产分散，用户必须理解内部模块边界。**  
   公司画像在 `/users/:actor/memory`，持仓在 `/users/:actor/portfolio`，会话在 `/sessions`，研究任务在 `/research`，自动化在 `/tasks` 和 `/schedule`，通知在 `/notifications`。这些边界对实现合理，但不应该成为用户查找信息的前置知识。

2. **同一投资对象缺少横向入口。**  
   一个 ticker 可能同时出现在 portfolio、company profile、session 文本、research task、notification record 和 cron prompt 中。当前没有一个统一的 search result 把这些证据按 ticker、actor、时间和类型聚合起来。

3. **管理员高频操作路径太长。**  
   运营常见问题是“这个用户最近有没有收到通知”“为什么这个 symbol 没有画像”“某个任务是不是还在跑”“这次错误在哪个 session”。现在需要在多个页面之间手工拼接。

4. **桌面端缺少本地工作台心智。**  
   Desktop bundled/remote 已经承担进程、设置、渠道和 Web console，但没有符合桌面产品预期的全局命令入口。用户安装桌面端后仍像在浏览管理后台，而不是一个个人投研工作台。

5. **未来资产越多，导航债越重。**  
   Research Artifact、Document Inbox、Trade Discipline Journal、Evidence Queue、Scenario Rehearsal 等提案一旦陆续落地，若每个资产只新增一个页面或 tab，信息架构会越来越难扫描。

机会是：先做一个轻量的 **Workspace Command Palette and Asset Search**，把现有只读索引和少量安全动作聚合起来。它不需要重写存储，不需要破坏 actor 隔离，也不需要等待 workspace-level 存储迁移。第一版可以以 admin/desktop 为主，public 端只暴露当前登录用户自己的安全子集。

## 方案概述

新增一个跨资产的搜索与命令层：

- `WorkspaceSearchIndex`：从 sessions、actors、portfolio、company profiles、cron jobs、research tasks、notifications、skills 和 settings capabilities 派生轻量 search documents。
- `WorkspaceSearchResult`：统一返回 `kind`、`title`、`subtitle`、`actor`、`symbol`、`matched_fields`、`updated_at`、`target_url` 和可用 actions。
- `CommandPalette`：前端全局入口，支持关键词搜索、类型过滤、键盘导航、最近访问、命令执行和深链跳转。
- `WorkspaceCommand`：对搜索结果可执行的安全动作，例如打开会话、查看画像、切到用户 portfolio、打开任务详情、启动研究、复制 actor key、查看通知记录、跳到设置项。

第一版目标不是全文搜索所有历史，也不是做 AI 自动操作中心。它应该先回答三个高频问题：

1. 我在哪里能找到这个用户、ticker、任务、报告或会话？
2. 这个对象和哪些其它 Hone 资产相关？
3. 我能从这里安全地执行哪一个下一步？

## 用户体验变化

### 用户端

- Public `/me` 或 `/portfolio` 可提供一个受限搜索入口，只搜索当前 Web actor 的 portfolio、company portraits、public chat history 摘要、研究报告和启用任务。
- 用户输入 ticker、公司名或任务名，可以直接跳到对应公司画像、持仓、研究报告或最近会话。
- 搜索结果必须只显示当前用户可访问的数据；不显示其它 actor、admin logs、LLM audit 或系统设置。
- 当用户没有任何资产时，搜索入口可以转为快捷动作：补充持仓、创建第一家公司画像、启动一次研究、打开聊天。

### 管理端

- Admin console 顶部增加 `Cmd/Ctrl+K` command palette，独立于当前页面。
- 支持按类型搜索：users、sessions、symbols、portfolio holdings、company profiles、cron jobs、research tasks、notifications、skills、settings。
- 搜索结果展示对象来源和状态，例如：
  - `MU` company profile，actor=`web/u_...`，最近更新 2 小时前。
  - `MU post-market digest` cron job，enabled，最近一次 `sent`。
  - `Feishu group ...` session，最后消息包含 MU。
  - `research task: Micron`，running 37%。
- 结果行提供直接动作：打开、复制 actor key、跳到任务运行记录、打开通知详情、以该 symbol 启动 research、进入用户 mainline。
- Palette 记住最近打开对象，帮助管理员在多个用户或任务之间快速来回切换。

### 桌面端

- Desktop bundled/remote 复用同一个 command palette。桌面用户可以用键盘从任意页面跳到本地设置、渠道状态、最近会话、任务、画像或日志。
- 当 backend disconnected 时，palette 仍能展示少量本地 shell 命令：打开设置、重新连接、查看日志页面；在线 search result 标记为不可用。
- 对 desktop remote 模式，palette 可以把 “当前后端能力缺失” 展示为 disabled command，而不是让用户进入页面后才发现按钮不可用。

### 多渠道

- IM 不需要完整 palette，但可以复用后端搜索能力提供受控命令：
  - `/find MU` 返回当前 actor 或 workspace 范围内的画像、持仓、任务和最近研究入口。
  - `/open tasks` 或 `/open portfolio` 返回 Web/desktop 深链提示。
- 群聊默认只搜索群 session 与群 actor 可见资产；不自动泄露个人 Web actor 的画像或持仓。
- 后续 Linked User Workspace 落地后，可把搜索范围从 actor 扩展到已授权 workspace。

## 技术方案

### 1. 定义统一 search document

在 `crates/hone-web-api` 或 `memory` helper 层新增轻量类型：

```rust
pub struct WorkspaceSearchDocument {
    pub id: String,
    pub kind: WorkspaceSearchKind,
    pub actor: Option<ActorIdentity>,
    pub symbol: Option<String>,
    pub title: String,
    pub subtitle: String,
    pub body_preview: String,
    pub keywords: Vec<String>,
    pub updated_at: String,
    pub target_url: String,
    pub actions: Vec<WorkspaceSearchAction>,
}
```

第一版 `kind` 建议覆盖：

- `session`
- `actor`
- `portfolio_holding`
- `watchlist_item`
- `company_profile`
- `cron_job`
- `research_task`
- `notification`
- `skill`
- `setting`

索引可以请求时动态生成，不必一开始落 SQLite。已有存储规模在个人/早期 public trial 阶段可接受；当 session 和 notification 规模上升，再把 document 投影到 SQLite FTS5。

### 2. 新增搜索 API

新增 admin API：

- `GET /api/search?q=&kind=&actor=&symbol=&limit=`
- `GET /api/search/recent`
- `POST /api/search/recent`

Public API：

- `GET /api/public/search?q=&kind=&limit=`

权限边界：

- Admin search 仍走当前 admin auth；后续 operator access proposal 落地后按 role 过滤 kinds/actions。
- Public search 从 `hone_web_session` 推导 actor，不接受任意 actor query。
- Channel command search 默认只用当前 `ActorIdentity`；workspace search 必须等待显式授权。

### 3. 分阶段接入数据源

Phase 1 只接入最有价值的轻量索引：

- sessions：复用 `SessionStorage::list_sessions()` 与 `session_message_text()`，只索引最近一条 user/assistant preview、session label、actor 和 last time。
- users/actors：复用 `/api/users` 的派生逻辑，索引 channel、user_id、scope 和 session label。
- company profiles：复用 company profile actor spaces/listing，索引 company name、ticker、profile id、updated_at、section headings。
- portfolio：索引 holdings/watchlist symbol、company name、position tags 和 actor。
- cron jobs：索引 task name、prompt preview、enabled、schedule、channel target。
- research tasks：第一版 admin 端可索引当前前端 context 中的 local tasks；等 Research Artifact Library 落地后切到服务端 artifact index。

Phase 2 再接入 notifications、task-runs、skills、settings、llm-audit 和 logs。日志全文不进入默认 search，只提供明确的 `logs` kind 或跳转命令，避免噪音。

### 4. 前端 command palette

新增：

- `packages/app/src/components/command-palette.tsx`
- `packages/app/src/context/workspace-search.tsx`
- `packages/app/src/lib/workspace-search.ts`

行为要求：

- `Cmd/Ctrl+K` 打开；`Esc` 关闭；上下键选择；Enter 执行默认动作。
- 搜索输入 debounce 150-250ms；空输入展示最近对象和常用命令。
- 结果按 `exact symbol match > title match > recent > body preview match` 排序。
- 结果行必须显示 kind badge、actor/channel、updated_at 和状态，避免同名对象误点。
- 所有 actions 走路由深链或现有 API，不在 palette 内做复杂编辑。

### 5. Action 与安全边界

第一版 actions 应保持低副作用：

- `open`: 跳转到已有页面。
- `copy_ref`: 复制 actor key、session id、task id 或 profile id。
- `start_research_for_symbol`: 跳转到 `/research?symbol=...`，不自动启动外部任务。
- `open_chat_prefilled`: 跳到 chat 并预填一句上下文，发送仍由用户确认。
- `open_settings_section`: 跳到 settings 对应区块。

不要在第一版做这些动作：

- 不直接删除、启停或重置任务。
- 不直接修改 portfolio / company profile / notification prefs。
- 不跨 actor 读取全文私有内容。
- 不让 palette 绕过已有 auth 或 future operator role。

### 6. 与 agent runtime 的关系

Palette search 是产品导航层，不是模型工具层。它可以后续作为 agent 可调用的只读 tool，但第一版应保持 UI/API 明确边界：

- Agent 不自动使用 admin-wide search。
- Public/IM agent 如果需要 “find my MU profile”，只能在当前 actor 或授权 workspace 范围内调用受限 search。
- 搜索结果可以给 prompt 提供 asset references，但具体读取详情仍走已有受控 API/tool。

## 实施步骤

1. **Search type 与 API skeleton**
   - 定义 search result/action 类型。
   - 新增 `/api/search`，先聚合 sessions、actors、portfolio、company profiles、cron jobs。
   - 为 query normalization、symbol exact match、actor filtering 增加单元测试。

2. **Admin command palette**
   - 在 `ConsoleLayout` 顶部挂载 `CommandPalette`。
   - 支持 keyboard shortcut、recent items、结果跳转。
   - 保持现有 sidebar/list 不变，palette 作为加速入口。

3. **Public safe subset**
   - 增加 `/api/public/search`，只查当前 Web actor 的 portfolio、profiles、sessions 和 tasks。
   - 在 `/me` 或 `/portfolio` 提供轻量搜索入口。

4. **Actions 与 deep links**
   - 给 sessions、users、profiles、tasks、research、settings 补齐稳定 URL builder。
   - 为 copy/open/prefill 类动作加前端测试。

5. **索引扩展与质量门槛**
   - 接入 notifications、task-runs、skills、research artifacts。
   - 观察响应耗时和结果质量；超过阈值后再引入 SQLite FTS 投影。

## 验证方式

- Rust 单元测试：
  - query normalization：大小写、ticker exact match、中文公司名、actor key。
  - permission filtering：public search 不能返回其它 actor；admin actor filter 生效。
  - result URL builder：sessions/users/tasks/research/settings deep link 稳定。
- Web 单元测试：
  - command palette keyboard flow：open、close、arrow、enter、empty state、recent items。
  - result ranking：exact symbol match 排在 body preview match 前。
  - disabled capability：backend 缺少 research/cron_jobs 时对应 action 不可执行。
- 手工验收：
  - 在 admin desktop/web 中按 `Cmd/Ctrl+K` 搜索一个 ticker，能看到相关 session、portfolio、profile、task。
  - 在 public `/portfolio` 搜索同一 ticker，只看到当前登录用户自己的资产。
  - 搜索不存在对象时展示可执行下一步，而不是空白列表。
- 指标：
  - command palette 打开次数、搜索成功点击率、零结果率。
  - 从 palette 到目标页面的平均跳转时间。
  - 管理员排障路径中跨页面点击次数是否下降。

## 风险与取舍

- **风险：搜索结果泄露跨 actor 数据。**  
  取舍：public/channel search 必须从认证上下文推导 actor，不接受任意 actor 参数；workspace 范围必须等待显式授权。

- **风险：第一版动态聚合变慢。**  
  取舍：先限制 limit、字段和数据源；只索引 preview 与 metadata，不扫完整附件和日志。规模上来后再加 SQLite FTS5 投影。

- **风险：palette 变成高副作用控制台。**  
  取舍：第一版只做 open/copy/prefill/start-draft 等低副作用动作；启停、删除、重置、密钥、导入等动作必须留在原页面并保留确认。

- **风险：和 sidebar / 各页面列表重复。**  
  取舍：sidebar 是结构化导航，页面列表是对象管理，palette 是快速查找和跳转。三者保留不同心智，不用 palette 替代已有页面。

- **风险：搜索质量不稳定导致用户不信任。**  
  取舍：结果行展示 matched field、kind、actor、updated_at；零结果时给出明确范围说明，例如“只搜索当前用户资产”。

## 与已有提案的差异

- 与 `auto_p1_linked-user-workspace.md` 不重复：Linked Workspace 解决真实用户跨渠道身份与资产归属；本提案在现有 actor 边界内先做查找、跳转和低副作用命令层，未来可把 workspace 作为搜索范围。
- 与 `auto_p1_research_artifact_library.md` 不重复：Research Artifact Library 让深度研究报告成为资产；本提案只把报告作为一种可搜索对象和跳转目标。
- 与 `auto_p1_user-data-trust-center.md` 不重复：User Data Trust Center 面向隐私、导出、删除和数据盘点；本提案面向日常工作流中的快速查找和操作。
- 与 `auto_p1_run_trace_workbench.md` 不重复：Run Trace 解释一次 agent run 的内部过程；本提案只帮助用户找到 run/session/task 并进入对应详情。
- 与 `auto_p1_investment_playbook_launcher.md` 不重复：Playbook Launcher 启动结构化研究流程；本提案可以把 playbook 作为 command action，但不定义投研流程本身。
- 与 `auto_p2_surface-design-contract.md` 不重复：Surface Design Contract 治理视觉和交互一致性；本提案定义一个具体跨 surface 产品能力，后续实现应遵守该设计契约。
