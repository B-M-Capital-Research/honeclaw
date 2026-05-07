# Proposal: Investment Playbook Launcher for Repeatable Research Workflows

status: proposed
priority: P1
created_at: 2026-05-07 17:03:40 +0800
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `packages/app/src/app.tsx`
- `packages/app/src/context/tasks.tsx`
- `packages/app/src/components/task-detail.tsx`
- `packages/app/src/components/skill-detail.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-home.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `crates/hone-web-api/src/routes/cron.rs`
- `crates/hone-web-api/src/routes/skills.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-channels/src/turn_builder.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/agent_session.rs`
- `memory/src/cron_job/storage.rs`
- `memory/src/company_profile/storage.rs`
- `skills/scheduled_task/SKILL.md`
- `skills/company_portrait/SKILL.md`
- `skills/stock_research/SKILL.md`
- `skills/portfolio_management/SKILL.md`
- `skills/position_advice/SKILL.md`
- `skills/notification_preferences/SKILL.md`

## 背景与现状

Hone 当前已经具备一批强能力，但入口仍以“用户知道自己要怎么问”为默认前提：

- Public 端在 `packages/app/src/app.tsx` 中只暴露官网页与 `/chat`，新用户进入后主要面对聊天框；产品承诺中的投资研究、组合跟踪、定时任务和长期画像，需要用户自行把需求表达成 prompt。
- 管理端已有 `/skills`、`/tasks`、`/users`、`/research`、`/notifications`、`/schedule` 等页面，但技能页主要是注册/启停/查看 `SKILL.md`，任务页主要是低层字段编辑：用户 ID、渠道、hour/minute、repeat、task_prompt。
- Skill runtime 已经做成两阶段披露：`turn_builder.rs` 负责 listing、slash skill 与 full `SKILL.md` 注入，`skills/scheduled_task`、`skills/company_portrait`、`skills/stock_research` 等都已经有明确工作流描述。
- 定时任务存储和 API 已经成熟：`memory/src/cron_job/storage.rs` 支持 actor 维度、repeat、tags、执行历史、heartbeat；`crates/hone-web-api/src/routes/cron.rs` 提供 CRUD 与执行记录查询。
- 长期研究资产已有文档型公司画像：`memory/src/company_profile/storage.rs` 与 `skills/company_portrait` 约定 actor sandbox 下的 `company_profiles/<ticker>/profile.md` 和 `events/*.md`，并保持 UI 只读/导入导出边界。
- README 面向外部用户强调 Hone 是投资纪律与长期研究助手，不只是 chat toy；但仓库公开版也说明部分专业 workflow 未开源。这使得开源产品面更需要一层“可用、可解释、可扩展”的标准 workflow 入口，避免核心能力只留在 prompt 和分散页面里。

这不是缺少单个工具的问题，而是产品架构层缺少“把能力组织成可重复工作流”的中间层。

## 问题或机会

### 问题

1. **新用户 activation 路径过长**
   用户要从“我想持续跟踪 NVDA 财报/组合风险/每日盘前信息”走到真实可运行状态，需要知道 skill、任务 prompt、渠道 target、画像沉淀、通知偏好等多个概念。当前 UI 给的是能力清单，不是完成路径。

2. **同类投资任务难以复用**
   定时任务 `task_prompt` 是自由文本。不同用户或不同渠道可能反复创建“财报前准备”“持仓异动复盘”“周末 thesis review”等任务，但系统没有模板版本、默认字段、预期产物、验收指标，也无法比较同一 playbook 在不同 actor 上的效果。

3. **技能是 runtime 能力，不是产品化任务入口**
   `skills/*/SKILL.md` 很适合告诉 agent 怎么做，但不适合直接作为用户端/管理端的交互脚本。比如 `company_portrait` 默认自动建档，`scheduled_task` 要调用 `cron_job`，`stock_research` 要按模式取数；这些组合起来才是用户理解的“研究工作流”。

4. **商业化与增长面缺少可展示的成功路径**
   Public 端可以卖“专业投资助手”，但用户很难在首次使用时快速看到“我已经启用了一个可持续工作流”。相比单次聊天，标准 playbook 更适合形成 trial、升级权益、团队服务、模板市场或咨询交付。

### 机会

行业里的 AI agent 产品正在从“聊天入口”转向“可检查、可复用、可授权的任务流”：用户不只想问一次问题，而是希望 agent 记住目标、定期执行、把证据沉淀到资产，并在关键时刻推送。Hone 已经有 agent runtime、skills、cron、actor sandbox、company portraits 和多渠道投递，差的是把它们收束为一组可启动的投资 playbook。

## 方案概述

新增 **Investment Playbook Launcher**：一层轻量的产品/架构中间层，用结构化 playbook 模板把现有 skill、cron、company portrait、portfolio、notification preferences 组合成可启动、可预览、可复用的投资工作流。

第一批只做 4 个高价值 playbook：

1. **Company Thesis Starter**
   输入 ticker 和研究目标，启动 `stock_research` + `company_portrait`，产出首版画像或更新已有画像。

2. **Earnings Prep and Follow-up**
   输入 ticker、财报日期或让系统查询，创建财报前提醒、财报后复盘任务，并把后续结论沉淀到事件时间线。

3. **Portfolio Daily Guardrail**
   基于已有 portfolio，创建交易日摘要/重大事件/风险提醒任务，并明确遵守通知偏好与勿扰。

4. **Weekly Thesis Review**
   选择一个或多个已有公司画像，创建每周复盘任务，检查投资主线是否仍成立、证伪条件是否触发、是否需要更新画像。

Playbook 不是替换 skill，也不是引入重量级 workflow engine。它先作为声明式模板和 preview/launch API 存在，把最终执行仍交给 `AgentSession`、`skill_tool`、`cron_job` 和 actor sandbox 原生文件能力。

## 用户体验变化

### 用户端

- Public `/chat` 旁边新增“开始一个研究工作流”的轻量入口，展示 3-4 个 playbook 卡片，而不是要求用户自己写复杂 prompt。
- 用户选择 playbook 后只填最少字段：ticker、组合范围、提醒时间、目标渠道、研究语言/深度。
- 提交前展示 preview：将创建什么任务、会使用哪些技能、会在哪个 actor workspace 里写入画像、预计什么时候推送。
- 启动成功后给用户一个可继续追问的 thread，例如“已创建 NVDA Earnings Prep，财报前一天 20:30 推送，财报后自动生成复盘草稿”。

### 管理端

- 新增 `/playbooks` 或在 `/tasks` 上方增加 Playbook Launcher；管理员可以用 playbook 创建任务，而不必手写 `task_prompt`。
- 每个 playbook 有版本、默认 prompt、所需字段、所需能力、当前启用状态，以及最近 launch 成功/失败记录。
- 管理员可以从某个任务详情回看它来自哪个 playbook 版本，便于调试和优化。

### 桌面端

- Desktop 首次启动或 settings 完成 runner/channel 配置后，展示本地可运行 playbook：例如“建立第一个公司画像”“创建每日持仓守门任务”。
- 若 channel 未配置，只允许创建本地/网页可见工作流，并提示多渠道推送需要先完成 Telegram/Discord/Feishu 配置。
- Desktop 不需要引入新的 sidecar；playbook preview/launch 走现有 console backend。

### 多渠道

- Telegram/Discord/Feishu 仍以自然语言为主，但可以支持 `/playbook` 或“创建财报复盘工作流”这类触发句，由 agent 返回可确认的 playbook preview。
- 多渠道 launch 必须保留 `ActorIdentity` 与 `SessionIdentity` 分离：playbook 归属 actor，群聊只在明确触发且确认后创建共享或个人任务。
- 推送内容仍由现有 outbound 和 scheduler 发送，不新增渠道专属模板系统。

## 技术方案

### 1. 新增声明式 playbook manifest

建议新增 `playbooks/` 或 `skills/playbooks/` 目录，第一版使用 Markdown/YAML 声明，不引入数据库：

```yaml
id: earnings_prep_followup
display_name: Earnings Prep and Follow-up
version: 1
priority: P1
required_capabilities:
  - cron_jobs
  - skills
required_skills:
  - stock_research
  - company_portrait
  - scheduled_task
inputs:
  - name: ticker
    type: ticker
    required: true
  - name: earnings_date
    type: date
    required: false
  - name: reminder_time
    type: time
    default: "20:30"
launch:
  mode: agent_prompt
  prompt_template: |
    Use the scheduled_task and company_portrait workflows to create an earnings prep and follow-up loop for {{ticker}}.
```

第一版 manifest 只表达：

- 展示信息
- 输入 schema
- 所需 skill / capability / channel 状态
- preview 模板
- launch prompt 模板
- 可选 cron default

不在 manifest 中编码业务逻辑，避免变成第二套 skill runtime。

### 2. 后端增加只读 listing 与 preview/launch API

在 `crates/hone-web-api` 增加轻量 routes：

- `GET /api/playbooks`
- `GET /api/playbooks/:id`
- `POST /api/playbooks/:id/preview`
- `POST /api/playbooks/:id/launch`

Preview 只做确定性检查：

- 当前 backend 是否有 `cron_jobs`、`skills`、`company_profiles` 等 capability
- required skills 是否启用
- actor 是否明确
- channel target 是否可用
- 输入字段是否通过基本校验
- 将要创建/触发的动作列表

Launch 则构造一个普通 `AgentSession::run()` 请求，由现有 agent runtime 调用 skill/tool 完成任务创建与画像沉淀。这样可以复用 `crates/hone-channels/src/execution.rs` 的 runner、tool registry、actor sandbox 和 prompt-audit 路径。

### 3. 保持 cron 与画像真相源不变

Playbook 不直接写公司画像正文，也不绕过 `cron_job` 工具私自写 cron JSON：

- scheduled task 创建仍通过 `scheduled_task` skill 调用 `cron_job(action="add")`，保持成功字段检查和任务数限制。
- 公司画像写入仍通过 runner 原生文件能力，在 actor sandbox 下维护 `company_profiles/`。
- 如果第一版 launch 需要更强可判定性，可以只直接调用后端 cron API 创建“外壳任务”，但画像更新仍交给 agent。该路径必须在 proposal 执行阶段另行权衡。

### 4. 前端以现有 Provider 组合落地

管理端已有 `TasksProvider`、`SkillsProvider`、`CompanyProfilesProvider` 和 backend capability 判断。新增 `PlaybooksProvider` 后：

- `/playbooks` 页面展示 playbook 列表、状态和 launch form。
- `/tasks` 新建时提供“从 playbook 创建”入口，填完后生成 preview，再 launch。
- `/skills` 中可显示“被哪些 playbook 使用”，帮助管理员理解禁用某个 skill 的影响。
- Public `/chat` 只暴露精选 playbook，不暴露全部管理能力。

### 5. Launch 结果关联与后续指标

第一版可在 cron job `tags` 中写入：

- `playbook:<id>`
- `playbook_version:<version>`
- `ticker:<ticker>`

后续如果需要更完整审计，再新增 `memory/src/playbook_launch.rs` SQLite 表，记录 actor、input hash、created task ids、session id、status、error。第一版不必先上表，除非指标和回溯需求很快成为硬需求。

## 实施步骤

### Phase 1: 静态 manifest 与管理端只读展示

- 新增 `playbooks/` 目录和 4 个内置 manifest。
- 在 `hone-tools` 或 `hone-web-api` 增加 manifest loader，读取、校验、排序。
- 暴露 `GET /api/playbooks` 与 `GET /api/playbooks/:id`。
- 管理端新增只读 Playbooks 页面，显示 required skills/capabilities 当前是否满足。

### Phase 2: Preview 与 Launch 骨架

- 增加 preview API，返回字段校验、所需能力、将触发的 skill、可能创建的 cron job 草案。
- 增加 launch API，把 playbook input 渲染为 agent prompt，并通过 `AgentSession::run()` 执行。
- Launch 后返回 session id、created/updated artifacts 的可读摘要；如果任务创建失败，返回具体失败阶段。

### Phase 3: Public/desktop activation

- Public `/chat` 增加精选 playbook 入口，只提供低风险、易解释的工作流。
- Desktop settings 或 dashboard 在 runner/channel 就绪后展示本地 first-run playbook。
- 多渠道先不做复杂交互，只支持 agent 在会话中返回 preview 并要求用户明确确认。

### Phase 4: 指标、版本和运营闭环

- 为 cron tags 或新增 SQLite launch 表补上 playbook id/version。
- 管理端显示 launch 成功率、最近失败原因、各 playbook 创建的任务数和后续活跃率。
- 将高失败率 playbook 从 public 精选中自动隐藏或标为需要配置。

## 验证方式

- Manifest loader 单元测试：
  - 有效 manifest 能被加载并按优先级/名称排序。
  - 缺少 `id`、重复 `id`、未知 input type、缺少 required skill 时返回明确错误。

- Preview API 测试：
  - skill 被禁用时 preview 显示 blocked，不允许 launch。
  - 缺少 actor、ticker 格式不合法、channel target 缺失时返回 400 或结构化 validation error。
  - backend 缺少 `cron_jobs` capability 时，scheduled 类 playbook 标为不可启动。

- Launch 回归：
  - 使用 mock runner 或 test runner 验证 launch prompt 包含 playbook id/version、actor 信息、用户输入和 required skill 指令。
  - 对创建任务类 playbook，验证最终 cron job 带上 playbook tags，且任务详情能正常读取执行记录。
  - 对 company portrait 类 playbook，在临时 actor sandbox 中验证 `company_profiles/<ticker>/profile.md` 被创建或保留原有文件不被覆盖。

- 前端测试：
  - Playbook 列表能展示 blocked/ready 状态。
  - Launch form 的 required input、preview、submit disabled 状态可测试。
  - Public surface 只显示允许 public 暴露的 playbook。

- 手工验收：
  - 新用户从 Public chat 启动 “Company Thesis Starter”，5 分钟内完成首个研究工作流入口体验。
  - 管理员从 `/tasks` 使用 “Portfolio Daily Guardrail” 创建任务，不需要手写完整 prompt。
  - Desktop 本地无渠道配置时，playbook 明确提示只能先做本地/网页工作流。

## 风险与取舍

- **风险：与 skill runtime 重叠。** Playbook 必须只做产品化编排和输入 schema，不复制 `SKILL.md` 的工作流细节；具体执行仍由 skill 和 runner 完成。
- **风险：launch 结果不够确定。** 如果完全依赖 agent prompt，可能出现未实际调用 `cron_job` 却回复成功的问题。缓解方式是要求 playbook launch 检查工具结果或对 cron 创建使用后端 API 外壳，但第一版应保守限制在低风险流程。
- **风险：模板过早泛化。** 不应一开始做用户自定义 playbook 市场。先用内置 4 个高频投资场景证明 activation 和留存价值。
- **风险：Public 端能力暴露过多。** Public playbook 必须经过 allowlist，不展示管理端、深度研报或需要外部密钥的高风险工作流。
- **取舍：不引入完整 workflow engine。** 仓库已有 `/report` 本地 workflow runner bridge，但本提案不把 Hone 主产品迁到外部工作流系统；先保持轻量 manifest + agent runtime。
- **取舍：不直接改画像 UI 编辑边界。** Company portrait 的 UI 仍只读/导入导出，playbook 通过 agent 原生文件能力沉淀，符合现有 invariants。

## 与已有提案的差异

已检查 `docs/proposal/` 与 `docs/proposals/`：

- 不重复 `auto_p1_automation_intent_control_plane.md`：该提案关注用户自然语言创建自动化任务时的意图确认、preview 和回滚；本提案关注产品层的标准投资工作流模板，把 skill、cron、画像和 public/desktop activation 串成可启动路径。
- 不重复 `auto_p1_research_artifact_library.md`：该提案关注研究产物的存储、引用、交接和复盘；本提案关注启动和编排工作流，产物仍可由未来 artifact library 承接。
- 不重复 `auto_p1_investment_context_intake.md`：该提案关注补齐用户投资背景缺口；本提案只要求 playbook 启动所需的最小输入 schema。
- 不重复 `auto_p1_investment_document_inbox.md`：该提案关注用户上传材料的收件箱和路由；本提案可以在后续把文档作为 playbook 输入，但不是文档摄取层。
- 不重复 `auto_p1_linked-user-workspace.md`：该提案关注跨渠道身份/资产连续性；本提案默认仍按当前 ActorIdentity 归属运行，后续可在 workspace 成熟后支持跨 actor playbook。
- 不重复 `auto_p1_run_trace_workbench.md`：该提案关注运行观测和排障；本提案只在后续阶段记录 playbook launch 结果，不建设 trace workbench。
- 不重复 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该提案关注 skill runtime 与 multi-agent 执行语义；本提案把已存在的 skill 能力包装为用户可理解的投资 workflow 入口。
- 不重复 `docs/proposals/desktop-bundled-runtime-startup-ux.md`：该提案关注 desktop bundled runtime 启动和进程恢复；本提案只在 desktop 就绪后提供可启动的业务工作流入口。

本提案的独立主题是：**把 Hone 从“有很多强能力的聊天/管理台”推进到“用户可以一键启动标准投资研究流程”的产品架构层**。
