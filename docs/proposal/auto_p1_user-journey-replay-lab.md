# Proposal: User Journey Replay Lab for Release Confidence

status: proposed
priority: P1
created_at: 2026-05-15 20:03:36 +0800
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
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `docs/proposal/auto_p1_interrupted-run-recovery-inbox.md`
- `tests/regression/run_ci.sh`
- `tests/regression/ci/`
- `tests/regression/manual/`
- `tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`
- `packages/app/package.json`
- `packages/app/src/pages/chat.test.ts`
- `packages/app/src/pages/public-portfolio-model.test.ts`
- `packages/app/src/pages/settings-model.test.ts`
- `packages/app/src/pages/task-health-model.test.ts`
- `packages/app/src/pages/users-model.test.ts`
- `packages/app/src/lib/messages.test.ts`
- `packages/app/src/lib/backend.test.ts`
- `packages/app/src/lib/log-refs.test.ts`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/ingress.rs`
- `crates/hone-channels/src/run_event.rs`
- `crates/hone-channels/src/outbound.rs`
- `crates/hone-event-engine/src/router/tests.rs`
- `crates/hone-web-api/src/routes/{chat,public,history,cron,task_runs,notifications}.rs`
- `memory/src/{session,session_sqlite,cron_job,portfolio,web_auth}.rs`

## 背景与现状

Honeclaw 已经从一个本地投资聊天助手演进成多入口、多运行形态的 agent 产品。README 展示了 Web、Mac App、iMessage、Feishu、Telegram、Discord、公司画像、持仓监控和定时任务；`docs/repo-map.md` 进一步说明了管理端与公开端分离、桌面 bundled/remote、channel sidecar、actor sandbox、统一 runner、技能两阶段披露、event-engine、公司画像迁移、public chat 和 Hone Cloud API 等结构。

仓库当前的测试与回归体系已经覆盖不少底层事实：

- `AGENTS.md` 和 `docs/invariants.md` 明确默认 CI 包含 Rust check/test、前端单元测试、CI-safe 回归脚本，并把外部账号依赖检查放入 `tests/regression/manual/`。
- `tests/regression/ci/` 里已有 session migration、legacy data migration、skill runtime stage consistency、Tauri sidecar wrapper args、source CLI start contract、finance automation contracts 等脚本。
- `tests/regression/manual/` 里保留了真实渠道、ACP runner、install bundle/brew、event-engine news classifier baseline、chart visualization 多渠道投递等手工回归。
- `packages/app/src` 有较多 model/lib 层测试，例如 chat、settings、users、task-health、public portfolio、backend、messages、log refs、share modal 和 notification preferences。
- `crates/hone-event-engine/src/router/tests.rs` 对事件路由、cap/cooldown、digest、价格 band 等规则已有系统化 Rust 单元测试。
- `tests/regression/manual/test_event_engine_news_classifier_baseline.sh` 已经把真实新闻分类基线沉淀为 fixture，说明仓库接受“代表性真实样本 + 可回归判断”的质量策略。

这些测试很有价值，但它们仍主要按模块或单一能力组织。Hone 现在真正容易出事故的地方，是跨产品表面的用户旅程：公开用户从 SMS 登录进入 `/chat`，回答生成并持久化；管理员在 `/users` 里查看同一 actor 的持仓和画像；桌面 bundled 模式启动 backend 和 channel；一个 cron job 创建后通过 event-engine 或 channel 投递；一次带本地图表的回答在 Web 留 marker、在 IM 变成图片。现有测试能证明很多零件正确，却还没有一个一等的“用户旅程回放实验室”来证明这些零件组合后仍然完成用户可理解的结果。

活跃计划里还有 ACP runtime、canonical config、skill runtime、Feishu placeholder、chart visualization 和 active bug burn-down 等高耦合改造。后续如果继续推进 Run Trace、Safety Gate、Update Compatibility、Linked Workspace、Research Artifact Library 等提案，单靠模块测试和人工 smoke 会越来越难回答一个关键问题：这次改动有没有破坏 Hone 的核心产品路径？

## 问题或机会

这是 P1，因为它直接影响发布信心、用户留存、桌面与安装版可靠性、自动化质量和维护效率。Hone 的用户体验不是某个函数返回值，而是一串跨端状态变化：登录、发问、runner 执行、工具调用、session 持久化、画像/持仓/任务读取、推送投递、错误恢复、管理端诊断。当前缺口集中在六类：

1. **模块测试不能代表产品旅程。**
   `packages/app` 的 model tests 能证明前端转换逻辑，Rust 单测能证明 router 或 outbound helper，但它们不能证明一个公开用户从登录到聊天再到历史恢复、管理员从用户列表跳到同一 actor、cron job 从创建到执行记录再到通知页面可见。

2. **手工回归覆盖真实链路，但不可持续。**
   `tests/regression/manual/` 对 Feishu、Discord、Telegram、ACP runner、install smoke 很重要；但它们依赖外部账号、模型或本机状态，无法成为每次 PR / release 的稳定门禁。结果是高风险跨端链路常常只能在发布前靠人工记忆挑几条跑。

3. **真实 bug 样本没有统一回放格式。**
   event-engine news classifier 有 baseline fixture，active bug burn-down 和 handoff 中也积累了真实样本；但聊天、public auth、cron、channel outbound、desktop startup、company profile import 等链路没有统一的 `journey fixture`。每个问题修复后都倾向于补局部测试，而不是形成可复用的端到端场景。

4. **发布前缺少“核心体验仍可用”的证据。**
   CI 契约能保证编译、单元测试和 CI-safe 脚本通过，但不能生成面向维护者的 release confidence 报告：哪些用户旅程被覆盖、哪些被跳过、哪些需要人工账号、哪些因为模型 nondeterminism 只做结构断言。

5. **AI runner 不确定性让端到端测试难落地。**
   真实 runner、外部数据源和 IM 平台有成本、延迟和不确定性。如果没有 fake runner、fake source、fake channel sink 和 fixture 状态包，团队会在“完全真实但不稳定”和“完全单元但不代表用户”之间摇摆。

6. **后续提案缺少共同验证底座。**
   Safety Gate、Run Trace、Feedback、Runtime Readiness、Delivery Decision、Interrupted Recovery、Multichannel Render Preview 都需要验证跨模块效果。每个提案单独发明验证脚本，会造成碎片化；User Journey Replay Lab 可以成为这些提案共享的回放基座。

机会是：Hone 已经有统一 `AgentSession`、`execution`、canonical run events、session storage、frontend model tests、regression script 目录和手工 baseline 经验。第一版不需要真实模型或真实 IM 账号，只要建立一套可离线运行的 journey fixture + fake adapters + 断言 DSL，就能显著提升重构与发版质量。

## 方案概述

新增 **User Journey Replay Lab**：一个面向产品旅程的回放与验证层，把关键用户路径抽象为可版本化 fixture，并用 fake runner / fake data source / fake channel sink 在 CI 或 release 前稳定复现。

核心对象：

1. `JourneyFixture`
   描述初始状态、输入动作、fake 外部响应、期望可见结果和需要检查的存储/API/UI 投影。fixture 存在 `tests/fixtures/journeys/`。

2. `ReplayHarness`
   在临时 data root、临时 config、fake runner、fake source 和 fake channel sink 下启动必要后端组件，按 fixture 执行动作，并收集 session、cron history、delivery rows、API responses、frontend model output 或 screenshot artifact。

3. `JourneyAssertion`
   使用稳定的结构断言，而不是比较完整 LLM 文本。例如：assistant turn 持久化、history 含附件 marker、cron row 状态为 failed/sent、delivery reason code 存在、public user 只能读自己的 actor、channel sink 收到 text/image 顺序。

4. `ReplayReport`
   输出一份机器可读 JSON 和简短 Markdown 摘要，列出通过、失败、跳过原因、依赖真实账号的 manual journey、覆盖的产品表面和相关提案。

第一版目标不是替代所有 E2E，也不是把真实浏览器、真实模型和真实 IM 都放进 CI。它应优先补“可确定、可离线、能代表核心产品路径”的场景。

## 用户体验变化

### 用户端

- 用户不会直接看到 Replay Lab，但会间接受益：public `/chat`、`/me`、`/portfolio`、history restore、附件/图片渲染和错误恢复在发布前有更稳定的产品级证明。
- 当 public chat、Hone Cloud API 或 scheduled task 出现回归时，维护者可以把真实失败最小化为 journey fixture，后续不再依赖用户重复复现。

### 管理端

- 管理端可以新增一个只面向 maintainer 的 `Release Confidence` 或 `Diagnostics` 区块，展示最近一次 replay report：核心旅程通过率、跳过原因、失败链接、manual-only 项。
- `/logs`、`/task-health`、`/sessions` 后续可链接到对应 journey fixture，用于解释“这个问题已有回归覆盖 / 尚无覆盖”。
- 管理员在准备正式 release notes 时，可以引用 replay report 作为最小质量证据之一。

### 桌面端

- Desktop bundled 的关键旅程可以被 fake sidecar / fake backend 状态回放：启动后 meta 可读、channel status 聚合、logs 合并、settings 保存后 effective config 生成。
- 真实 Tauri packaging 仍属于 release lane；Replay Lab 先覆盖桌面产品语义，不要求 CI 在所有平台完整打包 DMG。

### 多渠道

- Feishu / Telegram / Discord / iMessage 的真实账号检查继续留在 `tests/regression/manual/`。
- CI replay 使用 fake channel sink 验证共同契约：ingress actor scope、group pretrigger、placeholder lifecycle、outbound chunking、local image marker 转 text/image、失败时的用户可见错误。
- 当真实渠道 bug 修复后，应尽量把协议无关部分沉淀为 fake sink journey，真实账号脚本只验证平台连接与上传限制。

## 技术方案

### 1. Journey fixture 格式

新增目录：

```text
tests/fixtures/journeys/
  public_chat_basic.yaml
  public_chat_attachment_marker.yaml
  admin_user_portfolio_projection.yaml
  cron_create_run_history.yaml
  channel_image_outbound_order.yaml
  interrupted_session_recovery.yaml
```

建议字段：

```yaml
id: public_chat_basic
priority: p1
surfaces: [public_web, session_storage, web_api]
requires: [fake_runner, sqlite_sessions]
initial_state:
  config: fixtures/config/public_chat.yaml
  web_users:
    - phone: "+15550000001"
      user_id: "web-demo"
      quota: 5
actions:
  - kind: public_login
    user_id: "web-demo"
  - kind: public_chat
    message: "请解释 MU 的长期投资主线需要看什么"
fake_runner:
  assistant_text: "我会先区分长期主线和短期噪音。"
assertions:
  - kind: api_status
    path: /api/public/history
    status: 200
  - kind: session_tail
    roles: [user, assistant]
  - kind: text_contains
    source: assistant
    value: "长期主线"
```

关键原则：

- fixture 不存真实密钥、真实手机号或真实用户数据。
- LLM 输出默认由 fake runner 控制；少量 live-model journey 只能进入 manual lane。
- 断言尽量结构化，避免整段文本 golden snapshot。
- 每个 fixture 声明 surfaces 和 related proposal，便于 report 聚合覆盖范围。

### 2. Fake adapters

为回放提供确定性外部边界：

- `FakeAgentRunner`
  实现 `AgentRunner` trait，按 fixture 输出 run events：delta、tool call、error、timeout、local image marker、empty success。
- `FakeChannelSink`
  收集 outbound segment，验证 chunk、placeholder、image upload surrogate、reply target、group/direct 语义。
- `FakeMarketDataSource`
  提供 quote/news/SEC/search 固定响应和失败模式，后续可与 Source Provenance 提案共用 observation metadata。
- `FakeClock`
  固定 Asia/Shanghai 与 UTC 时间，验证 temporal prompt、cron occurrence、quiet hours 和 freshness。
- `TempRuntimeRoot`
  每次 replay 使用隔离 data root、config、sessions、SQLite、actor sandbox 和 uploads，避免污染开发者本机数据。

这些 fake adapter 应尽量挂在现有边界上：`AgentSession` / `execution` / `outbound` / Web API route / memory storage，而不是绕过产品主链路直接调用内部函数。

### 3. Replay CLI 与 CI 分层

新增命令可以先用脚本包装，稳定后进入 `hone-cli`：

```shell
bash tests/regression/ci/test_user_journey_replay.sh
cargo run -p hone-cli -- dev replay-journey --fixture tests/fixtures/journeys/public_chat_basic.yaml
```

分层建议：

- `ci-fast`
  纯 fake、无网络、无外部账号、总时长控制在几分钟内，进入 `tests/regression/ci/`。
- `ci-release`
  稍慢但仍无外部账号，可在 tag release 或 nightly 跑，覆盖更多 frontend/server integration。
- `manual-live`
  真实 IM、真实 ACP、真实模型、真实安装包，继续放在 `tests/regression/manual/`，但 report 里明确标注未进入默认门禁。

### 4. 前端验证方式

第一版不必强制全浏览器 E2E。可以先复用 `packages/app` 的 model/lib 测试策略：

- 对 API responses 运行 frontend projection helper，验证 public portfolio、task health、users、settings、messages 能解析 replay 产生的数据。
- 对关键页面保留少量 Playwright smoke，例如 public chat history restore、admin user detail、notifications/task health 列表。Playwright 只在 `ci-release` 或手动 release preflight 跑，避免拖慢常规 PR。
- 对分享图、图表、local image marker 等视觉路径，继续与 `auto_p1_multichannel-render-preview.md` 分工：Replay Lab 验证旅程中媒体契约存在，Render Preview 专门验证跨渠道渲染质量。

### 5. 失败样本沉淀流程

为 bugfix 建立最小规则：

1. 从真实问题中提取不含密钥和隐私的输入、状态和期望结果。
2. 先判断是否能成为 fake journey；如果依赖真实平台，则拆成 fake journey + manual live script 两层。
3. 把 fixture 加到 `tests/fixtures/journeys/`，并在对应 bug/handoff/proposal 中链接。
4. CI report 显示该 fixture 覆盖的 regression id。

这能把活跃 bug burn-down、event-engine baseline、channel smoke 和 release validation 串成长期资产。

## 实施步骤

### Phase 1: 最小回放骨架

- 定义 `JourneyFixture` schema 和 JSON/YAML parser。
- 实现 `TempRuntimeRoot`、`FakeClock`、`FakeAgentRunner` 和最小 `ReplayHarness`。
- 添加 3 条 CI-safe journey：
  - public chat basic：登录后发问、fake assistant、history 可读。
  - admin user projection：session/portfolio/company profile fixture 能出现在 `/users` 聚合视图。
  - outbound local image marker：assistant 文本含 `file://`，Web history 保留 marker，fake channel sink 收到 text/image/text。
- 新增 `tests/regression/ci/test_user_journey_replay.sh`，只跑上述 fake journey。

### Phase 2: 自动化与事件链路

- 增加 cron create/run/history journey，覆盖 `cron_job` storage、task run、notifications/task-health projection。
- 增加 event-engine delivery journey，复用 fake market event 和 fake sink，断言 delivery decision、digest queue 或 failed sink 状态。
- 增加 interrupted session journey，模拟 user 已落盘但 assistant 未终态，断言恢复 item 或用户可见失败提示。
- 让 replay report 输出覆盖矩阵：public/admin/desktop/channel/cron/event-engine/session/storage。

### Phase 3: Release Confidence 报告

- 在 release preflight 增加 `ci-release` journey 集合和 Playwright smoke。
- 管理端 diagnostics 或静态 artifact 展示最近 replay report。
- 将真实 manual scripts 注册进同一 report schema，哪怕它们没有在 CI 执行，也能显示“需要人工验证”的剩余风险。

### Phase 4: 与其它提案集成

- Run Trace Workbench：replay 运行自动生成 trace fixture，用来测试 trace 聚合。
- Investment Output Safety Gate：加入危险输出 fake runner journey，验证 gate verdict 和降级投递。
- Response Feedback：加入 answer id + feedback submission journey。
- Runtime Readiness：加入 runner blocked/config degraded journey，验证用户态和管理态文案。
- Delivery Decision Loop：加入 filtered/queued/sent journey，验证解释和偏好 patch 建议。

## 验证方式

- 静态验证：新增 fixture 必须通过 schema lint；禁止包含密钥、绝对个人路径、真实手机号或真实用户标识。
- CI 验证：`bash tests/regression/ci/test_user_journey_replay.sh` 在无网络、无外部账号条件下通过。
- Rust 验证：fake runner、fake sink、fixture parser、assertion evaluator 有单元测试；关键 storage/API 边界用现有 crate tests 覆盖。
- 前端验证：`bun run test:web` 覆盖 replay response 到前端 model 的转换；必要时在 release lane 跑 `bun run test:e2e:web` 的少量 smoke。
- 报告验证：Replay report 必须列出执行 fixture、跳过 fixture、失败原因、覆盖 surfaces、耗时和相关 regression id。
- 人工验收：选择一条历史真实 bug，将其转成 fake journey，确认本地能先失败后通过，并且不会调用真实模型或 IM 平台。

## 风险与取舍

- 风险：回放系统本身变成另一套复杂框架。取舍：第一版只支持少量 action/assertion，不追求通用浏览器自动化平台；优先覆盖最常坏的产品路径。
- 风险：fake runner 掩盖真实模型问题。取舍：Replay Lab 只证明产品状态机和接口契约；模型质量继续由 Safety Gate、Feedback Loop、event-engine baseline 和 manual live tests 覆盖。
- 风险：fixture snapshot 太脆，导致维护成本高。取舍：默认做结构断言和 reason code 断言，不比较完整 Markdown、完整 HTML 或完整日志。
- 风险：CI 时间增加。取舍：分 `ci-fast`、`ci-release`、`manual-live` 三层；默认 PR 只跑少量 fake journey。
- 风险：真实用户样本脱敏不彻底。取舍：fixture schema lint 拒绝密钥、绝对路径、真实手机号格式和大段原文；导入真实 bug 时先最小化。
- 边界：本提案不替代单元测试、Rust 集成测试、前端 model tests、真实 IM 手工回归或正式桌面打包验证；它提供跨模块产品旅程的稳定回放层。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 下全部自动提案和历史 `docs/proposals/`：

- 不重复 `auto_p1_run_trace_workbench.md`：Run Trace 解释一次已经发生的运行，Replay Lab 在发布前构造并回放可控用户旅程；两者可通过 trace fixture 集成。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：Feedback 采集用户对回答的质量评价，Replay Lab 验证产品路径是否按预期完成。
- 不重复 `auto_p0_investment_output_safety_gate.md`：Safety Gate 判断投资敏感输出能否送达，Replay Lab 可把 gate 场景作为 journey 断言，但不定义投资安全策略。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：Readiness 评估当前部署配置是否可用，Replay Lab 在隔离 fake runtime 中执行具体旅程。
- 不重复 `auto_p1_multichannel-render-preview.md`：Render Preview 专注跨渠道渲染质量，Replay Lab 只验证旅程中媒体契约和 segment 顺序。
- 不重复 `auto_p1_interrupted-run-recovery-inbox.md`：Recovery Inbox 处理真实中断项，Replay Lab 提供可重复的中断场景测试。
- 不重复 `docs/proposals/desktop-bundled-runtime-startup-ux.md`：该提案改善桌面启动冲突体验，Replay Lab 可验证桌面语义但不重新设计启动策略。

本提案的独立主题是：把 Hone 的核心用户旅程变成可版本化、可离线、可报告的发布质量资产，降低跨模块重构和 release 的产品回归风险。
