# Proposal: Ingress Replay Lab for Channel Trigger and Session Attribution

status: proposed
priority: P1
created_at: 2026-05-28 02:04:29 +0800
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
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `docs/proposal/auto_p1_channel-activation-proof.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_privacy-preserving-product-events.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_interrupted-run-recovery-inbox.md`
- `crates/hone-channels/src/ingress.rs`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-channels/src/core/intercept.rs`
- `crates/hone-channels/src/attachments.rs`
- `crates/hone-channels/src/attachments/ingest.rs`
- `bins/hone-feishu/src/handler.rs`
- `bins/hone-telegram/src/handler.rs`
- `bins/hone-discord/src/handlers.rs`
- `bins/hone-discord/src/utils.rs`
- `memory/src/session.rs`
- `memory/src/cron_job/mod.rs`
- `tests/regression/ci/`
- `tests/regression/manual/`

## 背景与现状

Honeclaw 的多渠道能力已经从“每个 bot 各自处理消息”逐步收敛到共享 ingress / session / outbound 基础设施。当前结构里，Web、Feishu、Telegram、Discord、iMessage 和桌面 runtime 最终都服务于同一个投资研究助手，但一条消息能否进入 agent 主链路，取决于进入 `AgentSession::run()` 之前的一组高风险决策。

当前仓库里已有几个关键事实：

- `crates/hone-channels/src/ingress.rs` 定义了 `ChatMode`、`IncomingEnvelope`、`GroupTrigger`、`GroupPretriggerWindowRegistry`、`MessageDeduplicator`、`SessionLockRegistry`、`ActorScopeResolver` 和 `persist_buffered_group_messages`。
- `docs/invariants.md` 明确要求 `ActorIdentity` 负责权限、quota、sandbox、私有数据隔离，`SessionIdentity` 负责会话历史归属；`ChatMode` 只描述 direct/group 消息形态，不能作为 session ownership 的真相源。
- Feishu、Telegram、Discord 的 handler 都在本地实现平台事件解析、chat scope 过滤、allowlist / 权限检查、dedup、群聊 trigger、pretrigger buffer、busy guard、intercept command、附件 ingest、`IncomingEnvelope` 组装和 `AgentSession` 调用。
- 群聊现在有短窗口 pretrigger 语义：未显式触发的文本可能先进入 buffer，等后续 `@bot` 或 reply-to-bot 触发时再作为上下文写入共享 group session。
- 群聊并发有 busy lifecycle：同一个 group session 正在处理时，新的显式触发不会启动第二个 run，而是提示等待，并把文本重新放回 pretrigger window。
- Telegram 还有 media group 聚合和同发言者最近往返 continuation anchor；Feishu 有 open_id/email/mobile/contact target 分支；Discord 同时支持 message mention 和 `/skill` slash command。
- `core/intercept.rs` 已经把 `/register-admin` 和 `/report` 这类 pre-session command bridge 从 LLM 链路分离出去，不写入 session transcript。

这些机制是正确方向，但它们现在主要靠各 channel handler 的局部代码、局部单元测试和运行日志维持。Hone 已有 outbound 渲染提案、channel activation proof、run trace、user journey replay 等方向，但还缺一个专门面向 **入站消息触发和会话归因** 的产品/工程契约：给定一组真实或合成平台事件，系统应该能解释它们为什么被忽略、缓冲、拦截、进入哪个 actor/session、带上哪些附件和上下文，以及最后是否允许启动 agent run。

## 问题或机会

这是 P1 问题，因为入站归因错误会直接破坏核心体验和数据边界，而且通常比模型回答错误更隐蔽。

1. **群聊触发语义容易回归。**  
   Feishu、Telegram、Discord 的群聊入口都要判断 direct mention、reply-to-bot、question signal、chat scope、pretrigger window 和 busy state。任一分支漂移，都可能出现“明明 @ 了却没回复”“没 @ 却偷听进上下文”“上一条还在跑又启动第二条”的用户体验问题。

2. **Actor / Session 归因一旦错就是数据安全问题。**  
   direct actor、group actor、触发者 actor、共享 `SessionIdentity`、`channel_target` 和 `channel_scope` 不能混用。错误归因会让个人持仓、公司画像、quota、cron target 或群聊历史落到错误边界，后续再靠 answer finalizer 很难补救。

3. **pre-session command 与普通聊天的边界不够可视。**  
   `/register-admin`、`/report`、未来 `/link`、`/feedback`、`/workspace` 等命令都应该在进入 runner 前被拦截、校验、回复，并明确是否写 session、是否消耗 quota、是否允许群聊。现在这些规则没有一个统一 replay 面可以证明。

4. **附件与媒体组进入 prompt 前的组合行为复杂。**  
   `attachments/ingest.rs` 已经集中大小、图片尺寸、PDF preview 和 vector store 等 gate，但平台 handler 仍负责收集 raw attachments、构造用户输入、处理 Telegram media group、生成 attachment ack。用户实际看到的“这条图片/文件是否被理解”取决于 handler 与共享 ingest 的组合。

5. **现有观测多偏运行后，缺少运行前判定证据。**  
   Run Trace 可以串起一次 run，但很多 ingress 决策发生在 run 之前：blocked by chat scope、duplicate ignored、pretrigger buffered、busy deferred、intercept handled、unparsed ignored。没有专门的 ingress decision record，维护者只能从散落日志推断。

6. **外部真实账号回归成本高。**  
   真实 Feishu/Telegram/Discord 手工回归必须保留，但入站触发规则中的大多数可以用平台事件 fixture 离线回放。若不沉淀离线 replay，每次 channel handler 修复都会重新依赖人工记忆。

机会是：Hone 已经有共享 ingress 类型、session identity 模型、attachment ingest、CI-safe regression 目录和丰富的历史 bug/handoff。第一版不需要重构所有 channel，只要把“平台事件 -> ingress decision -> envelope/run/no-run”的路径抽象出来，就能显著提高多渠道可信度。

## 方案概述

新增 **Ingress Replay Lab**：一个用于回放、解释和验证多渠道入站消息决策的离线实验室。它的核心不是替代真实平台 SDK，而是把平台 handler 中最容易出错的决策转化为可版本化 fixture 和可审计 decision record。

核心能力：

- `IngressDecision`
  - 描述一条平台事件经过 Hone 后的判定结果：`ignored`、`buffered`、`intercepted`、`busy_deferred`、`rejected_attachment`、`enveloped`、`run_started`。
  - 包含 reason code、channel、raw message id、actor、session identity、chat mode、trigger、buffered count、attachment summary、quota/run eligibility、是否写 session。

- `IngressFixture`
  - 存放在 `tests/fixtures/ingress/`，用平台无关 JSON/YAML 表示 direct、group、mention、reply、duplicate、busy、media group、slash command、attachment rejection 等样本。
  - 可选择 channel-specific raw payload 片段，但断言目标必须落到共享 decision / envelope 语义。

- `IngressReplayHarness`
  - 用 fake core、fake session storage、fake attachment blobs、fake platform metadata 回放 fixture。
  - 不启动真实 runner，不调用外部 IM API，不需要 token。
  - 输出 machine-readable report，说明每条事件最终有没有进入 `AgentSession`，如果进入，`IncomingEnvelope` 长什么样。

- `Ingress Decision Log`
  - 运行时轻量记录关键入站判定的结构化 reason code。第一版可以只写日志和 future product events，不新增重型数据库。
  - 让 `/logs`、未来 `/traces`、`product-events` 能区分“用户消息没回”到底是 ignored、buffered、busy、intercepted 还是 run failed。

第一版目标很克制：把入站链路中最核心、最容易破坏数据边界的规则可回放化，而不是做完整平台模拟器。

## 用户体验变化

### 用户端

- 群聊里 Hone 的触发规则更稳定：什么情况下需要 @、reply 或私聊，哪些未触发文本会作为上下文进入下一次回复，都能被 fixture 固化。
- busy 提示更一致：同一 group session 正在处理时，新触发者收到明确等待提示，而不是看起来消息丢失或启动假 placeholder。
- 附件失败更可解释：图片过大、PDF 失败、media group 未聚合完成、文件被 gate 拒绝时，用户得到一致的可见反馈。
- pre-session command 不再随机进入投资聊天上下文；用户发 `/report` 或未来 `/link` 时，系统明确把它当命令处理。

### 管理端

- 管理员查看日志或未来 trace 时，可以看到 ingress decision reason，例如 `group_pretrigger_buffered`、`duplicate_message`、`busy_deferred`、`chat_scope_blocked`、`intercept_report_start`、`attachment_rejected_size`。
- 支持把一次真实用户反馈最小化为 ingress fixture，后续在 CI 中回放，避免同类 channel bug 反复出现。
- 对群聊问题，管理端能看到触发 actor 与共享 session identity 的区别，减少误判“用户 A 的数据为什么出现在群 session”。

### 桌面端

- Desktop bundled 的 channel status 可以补充最近 ingress 决策摘要：最近是否大量消息被 chat scope 阻断、是否重复 busy、是否附件持续被拒。
- 本地用户调试 Feishu/Telegram/Discord 时，不必只看滚动日志；可导出一个脱敏 ingress replay report 给维护者。

### 多渠道

- Feishu：覆盖 p2p/open_chat、mention、open_id/email/mobile speaker label、target resolution 相关 metadata、`/report` intercept 和 group busy。
- Telegram：覆盖 private/chat、reply-to-bot、media group 聚合、同发言者 continuation anchor、HTML/attachment 输入组合。
- Discord：覆盖 DM/guild、direct mention、slash `/skill`、channel id scoped group session、author allowlist。
- iMessage：第一版只定义 fixture 形态和 direct ingress 断言，真实 macOS 权限检查继续留在 manual regression。

## 技术方案

### 1. 定义稳定 ingress decision 类型

在 `crates/hone-channels/src/ingress.rs` 附近新增类型，或拆成 `crates/hone-channels/src/ingress_decision.rs`：

```rust
pub enum IngressDecisionKind {
    Ignored,
    Buffered,
    Intercepted,
    BusyDeferred,
    Rejected,
    Enveloped,
    RunStarted,
}

pub struct IngressDecision {
    pub kind: IngressDecisionKind,
    pub reason_code: String,
    pub channel: String,
    pub raw_message_id: Option<String>,
    pub actor: Option<ActorIdentity>,
    pub session_identity: Option<SessionIdentity>,
    pub session_id: Option<String>,
    pub channel_target_redacted: Option<String>,
    pub chat_mode: Option<ChatMode>,
    pub trigger: Option<GroupTrigger>,
    pub buffered_count: usize,
    pub attachment_count: usize,
    pub will_start_agent_run: bool,
}
```

原则：

- 不记录完整消息正文；fixture 可保存合成文本，运行时 decision log 默认只记录长度、hash 或截断摘要。
- target 要脱敏，避免日志泄露手机号、open_id、chat id。
- `ActorIdentity` / `SessionIdentity` 直接引用现有类型，不引入第三套身份模型。
- reason code 必须稳定，供测试、日志和产品视图复用。

### 2. 抽取 handler 中的纯决策层

不要求一次性重写平台 handler，但应逐步把以下决策抽成可测试函数：

- chat scope 是否允许 direct/group。
- allowlist / bot self / duplicate 是否阻断。
- group trigger 是否满足 `GroupTriggerMode`。
- 未触发 group 文本是否进入 pretrigger。
- busy 状态下是否 defer 到 pretrigger，并返回哪种用户可见提示。
- intercept command 是否处理、是否禁止进入 session。
- actor / session identity / channel target 如何解析。
- attachment ingest 结果如何拼成 user input 和 ack。

每个函数返回 `IngressDecision` 或 `Decision + NextAction`，channel handler 继续负责平台 SDK 调用和真实 reply。

### 3. 建立 ingress fixture 格式

新增目录：

```text
tests/fixtures/ingress/
  feishu_group_mention_with_pretrigger.yaml
  feishu_direct_report_intercept.yaml
  telegram_media_group_direct.yaml
  telegram_group_busy_deferred.yaml
  discord_guild_mention_session_scope.yaml
  discord_duplicate_ignored.yaml
  attachment_image_too_large_rejected.yaml
```

建议字段：

```yaml
id: telegram_group_busy_deferred
channel: telegram
initial_state:
  chat_scope: all
  pretrigger_enabled: true
  active_session:
    session_identity: "Session_telegram__group__chat_3a-100123"
    speaker_label: "Alice"
events:
  - message_id: "tg-2"
    chat: { kind: group, id: "-100123" }
    from: { id: "bob", label: "Bob" }
    text: "@Hone 帮我接着看 NVDA"
    trigger: { direct_mention: true }
expected:
  decisions:
    - kind: busy_deferred
      reason_code: group_session_busy
      actor: "Actor_telegram__bob__chat_3a-100123"
      session_identity: "Session_telegram__group__chat_3a-100123"
      buffered_count: 1
      will_start_agent_run: false
```

第一版 fixture 可以只覆盖共享决策结果，不需要完整平台 SDK payload。等决策层稳定后，再补 channel-specific parser fixture。

### 4. Replay harness 与 CI-safe 脚本

新增 CI-safe 回归脚本：

```shell
bash tests/regression/ci/test_ingress_replay.sh
```

执行内容：

- 构造临时 data root 和 session storage。
- 加载 `tests/fixtures/ingress/*.yaml`。
- 对每个 fixture 调用对应 channel 的 replay adapter。
- 断言 decision sequence、envelope actor/session、buffered messages、intercept result、attachment summaries。
- 生成 `target/ingress-replay/report.json` 和简短 Markdown 摘要。

Rust 单元测试继续覆盖小函数，regression 脚本覆盖跨模块组合。真实账号联通性仍留在 `tests/regression/manual/`，不进入默认 CI。

### 5. 运行时观测接入

短期：

- 在 Feishu / Telegram / Discord handler 的关键 early return 前写结构化日志，带 `ingress_decision.reason_code`。
- 对进入 `IncomingEnvelope` 的消息，记录 actor/session/channel target 的脱敏摘要。
- 对 pretrigger buffer、busy defer、intercept command、attachment rejection 给出统一 reason code。

中期：

- 与 `auto_p1_run_trace_workbench.md` 协作：如果消息进入 agent run，trace timeline 第一段展示 ingress decision。
- 与 `auto_p1_privacy-preserving-product-events.md` 协作：只上报 reason code 和聚合计数，不上报正文。
- 与 `auto_p1_channel-activation-proof.md` 协作：activation proof 可以复用最近 inbound decision 捕获 target，但不混同“可投递证明”和“入站触发证明”。

## 实施步骤

### Phase 1: Decision code inventory

- 列出 Feishu、Telegram、Discord handler 现有 early return / buffer / intercept / run-start 分支。
- 定义稳定 reason code 枚举和 redaction helper。
- 在不改行为的前提下，为关键分支补 `IngressDecision` 生成函数和日志。
- 补最小单元测试：dedup、chat scope、group trigger、busy、pretrigger、intercept。

### Phase 2: Replay fixture MVP

- 新增 `tests/fixtures/ingress/` 和 replay runner。
- 先覆盖 8 到 12 个最关键样本：
  - direct message starts run
  - duplicate ignored
  - group text buffered without trigger
  - group mention flushes pretrigger
  - group busy defers new trigger
  - `/report` intercepted
  - attachment rejected before prompt
  - Discord group session scoped by channel id
  - Telegram media group combined into one user input
  - Feishu group actor/session split
- 新增 CI-safe regression script。

### Phase 3: Admin and trace integration

- 在 `/logs` 或未来 `/traces` 中识别 ingress reason code，展示为可过滤字段。
- 对 failed/ignored 用户反馈，支持导出脱敏 ingress event skeleton，便于新增 fixture。
- 将 ingress decision 作为 Run Trace 第一阶段，而不是让维护者从原始日志中猜。

### Phase 4: Channel-specific parser fixtures

- 对 Feishu event、Telegram update、Discord message/slash interaction 增加少量 raw payload parser fixtures。
- 只保留协议关键字段，不保存真实用户消息或平台敏感 id。
- 将 platform parser 与 shared decision 分层验证，避免一个 parser bug 被误判为 agent runtime bug。

## 验证方式

- Rust 单元测试：
  - `ActorScopeResolver` direct/group 解析不改变现有 actor/session 语义。
  - `GroupPretriggerWindowRegistry` 继续验证 max messages、max age、dedup。
  - `SessionLockRegistry` busy guard 返回当前 active speaker，并在 drop 后释放。
  - intercept command 返回 `intercepted` decision 且不写 session、不启动 runner。
  - attachment rejection 返回稳定 reason code，且不会进入 prompt。
- CI-safe regression：
  - `bash tests/regression/ci/test_ingress_replay.sh` 对所有 fixture 输出 deterministic report。
  - fixture 断言 `will_start_agent_run`、actor、session identity、buffered count、reason code。
  - 无外部账号、无网络、无真实 IM API。
- 手工验收：
  - 在 Feishu / Telegram / Discord 各跑一条 direct、一条 group mention、一条未触发 group 文本、一条 busy trigger，确认运行日志 reason code 与 fixture 语义一致。
  - 对 `/report` 或 `/register-admin` 验证 pre-session intercept 不进入 chat transcript。
- 产品指标：
  - 多渠道“消息没回”类反馈能被归因到 ignored/buffered/busy/intercept/run_failed/outbound_failed 中的一类。
  - 每次修复 channel ingress bug 后，能新增一个 fixture，避免只靠手工复现。

## 风险与取舍

- 风险：抽取决策层可能触碰 channel handler 的真实平台细节。取舍：第一阶段只加 decision 记录和测试，不改真实发送/回复行为。
- 风险：fixture 过度模拟平台 SDK，维护成本高。取舍：MVP 用平台无关 fixture，只覆盖共享语义；raw payload fixture 放到后续少量关键样本。
- 风险：运行时 decision log 泄露用户消息或平台 id。取舍：默认只记录 reason code、长度、hash 和脱敏 target；完整正文只允许合成 fixture。
- 风险：与 User Journey Replay Lab 重叠。取舍：本提案只验证入站触发和 session 归因，不覆盖完整产品旅程、前端 UI、runner 输出或 outbound 渲染。
- 风险：reason code 过早固化导致后续难调整。取舍：先稳定核心 code，增加新 code 允许，删除/重命名必须通过 fixture 更新显式发生。
- 不做：不替代真实 IM live smoke，不改变 `ActorIdentity` / `SessionIdentity` 模型，不把群聊未触发文本长期保存为个人记忆，不让 test harness 调用真实 runner 或外部账号。

## 与已有提案的差异

- 不重复 `auto_p1_multichannel-render-preview.md`：该提案关注 final answer 到多渠道可见输出的 outbound 渲染；本提案关注平台消息进入 Hone 之前的 inbound trigger、buffer、intercept 和 session attribution。
- 不重复 `auto_p1_channel-activation-proof.md`：该提案证明某个 channel target 能被发送测试消息；本提案证明真实/合成入站消息会不会进入正确 actor/session 和 agent run。
- 不重复 `auto_p1_user-journey-replay-lab.md`：该提案覆盖跨产品旅程的端到端 release confidence；本提案是更底层、更聚焦的 ingress fixture 和 decision contract，可作为 Journey Replay 的一个组件。
- 不重复 `auto_p1_run_trace_workbench.md`：Run Trace 聚合一次已发生运行的证据；本提案覆盖很多不会进入 run 的入站判定，并为 trace 第一阶段提供结构化事实。
- 不重复 `auto_p1_privacy-preserving-product-events.md`：Product events 关注隐私友好的采纳与行为指标；本提案关注可回放的 correctness fixture 和 reason-code 验收。
- 不重复 `auto_p1_linked-user-workspace.md`：Linked Workspace 解决跨渠道身份绑定；本提案不合并身份，只验证当前 `ActorIdentity` / `SessionIdentity` 边界是否被 channel ingress 正确使用。
- 不重复 `auto_p1_interrupted-run-recovery-inbox.md`：Interrupted Run 处理进入 run 后被中断的恢复；本提案处理 run 前的 ignored/buffered/busy/intercept/envelope 判定。
- 不重复历史 `docs/archive/plans/group-shared-session.md`、`group-reply-append-chain.md` 和 `attachment-ingest-unify.md`：这些是已完成实现/重构计划；本提案新增的是长期可运行的 replay lab 与产品化验收面。
