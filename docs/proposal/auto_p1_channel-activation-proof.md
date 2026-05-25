# Proposal: Channel Activation Proof for Verified Multi-Channel Onboarding

status: proposed
priority: P1
created_at: 2026-05-25 14:04:24 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `config.example.yaml`
- `crates/hone-web-api/src/routes/channel_settings.rs`
- `crates/hone-web-api/src/routes/meta.rs`
- `crates/hone-web-api/src/types.rs`
- `crates/hone-channels/src/bootstrap.rs`
- `crates/hone-channels/src/outbound.rs`
- `bins/hone-desktop/src/sidecar/processes.rs`
- `bins/hone-desktop/src/sidecar/settings.rs`
- `bins/hone-cli/src/reports.rs`
- `bins/hone-cli/src/main.rs`
- `memory/src/cron_job/storage.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/context/console.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/backend.ts`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_end-user-notification-control.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Honeclaw 的核心价值已经不只是 Web chat，而是把同一个投资研究助手接入 Web、桌面、Feishu、Telegram、Discord、iMessage，并让定时任务和事件引擎能主动回到用户所在渠道。仓库当前已经有多块基础能力：

- `crates/hone-web-api/src/routes/channel_settings.rs` 和 `bins/hone-desktop/src/sidecar/settings.rs` 都能读取/保存 Feishu、Telegram、Discord、iMessage 的 enable、token、chat scope、allowlist 和 target handle，并重新生成 `data/runtime/effective-config.yaml`。
- `bins/hone-desktop/src/sidecar/processes.rs` 在 bundled 模式下会根据 canonical config 启动启用的 channel sidecar，并清理 stale heartbeat。
- `crates/hone-channels/src/bootstrap.rs` 为每个 channel binary 统一加载 runtime config、检查 enabled、获取 process lock、写 heartbeat。
- `crates/hone-web-api/src/routes/meta.rs` 通过 heartbeat registry、heartbeat 文件和 OS process scan 汇总 `/api/channels` 状态，能区分 disabled、unsupported、stopped、running、degraded。
- `bins/hone-cli/src/reports.rs` 的 `doctor` 会检查已启用 channel 是否配置了认证字段，`hone-cli channels targets` 通过 `memory/src/cron_job/storage.rs` 汇总 cron job 和 execution history 中出现过的 `channel_target`。
- `packages/app/src/pages/settings.tsx` 的 Channel 设置页可以编辑配置，但只负责保存；`packages/app/src/context/console.tsx` 轮询 channel status badge。

这些能力说明“配置保存”“进程在线”“历史 target 可见”都已经有入口。但从产品角度看，用户真正需要证明的是：我刚配置的某个渠道，是否能向我或目标群发出一条受控测试消息，并且这个 target 后续能被定时任务、事件推送和 agent 回复稳定复用。

当前系统没有一个一等的 **activation proof**。保存 token 后，用户还要自己猜是否需要重启、是否 bot 被拉进群、allowlist 是否命中、Feishu open_id 是否可解析、Telegram chat id 是否正确、Discord bot 是否有发言权限。等到第一条定时任务或事件推送失败时，问题已经发生在核心价值链路上。

## 问题或机会

这是 P1 问题，因为多渠道可达性直接影响首次激活、主动通知可信度和支持成本。

1. **配置成功不等于投递成功。**  
   `channel-auth` 检查只能判断字段是否非空，`/api/channels` 只能判断进程是否在线。平台权限、bot 入群、receive_id/open_id 解析、chat scope、allowlist、目标会话是否存在，都要到真实发送时才暴露。

2. **用户不知道下一步动作。**  
   Web/desktop 设置页保存后只提示配置已保存或需要重启；CLI doctor 只列出配置/二进制/认证状态。用户很难知道“现在去 Telegram 给 bot 发 `/start`”“把 Feishu bot 加到目标群”“在群里 @Hone 触发一次注册”“复制这个 chat id 到 allowlist”。

3. **target discovery 和 target verification 没有闭环。**  
   `CronJobStorage::list_channel_targets()` 能从已有任务和执行历史聚合 target，但首次配置时常常还没有任何 cron target。反过来，如果历史 target 存在，也不代表当前 token、权限和 chat scope 仍然可达。

4. **主动推送失败会被误解为 agent 不可靠。**  
   用户通常不会区分“模型没工作”“事件没触发”“路由过滤”“渠道不可达”。一条关键财报/价格提醒没送达，就会损害对投资助手守护能力的信任。

5. **支持排障缺少可复用证据。**  
   维护者需要串联 channel settings、runtime heartbeat、sidecar logs、delivery logs、cron targets、平台后台配置。缺少一次标准化的 proof run 记录，就很难判断问题发生在 credentials、process、target、permission、render、send 还是 downstream delivery decision。

机会是：Hone 已经具备配置面、进程状态、outbound 抽象、cron target directory、notifications 日志和 desktop sidecar 控制。第一版不需要重构多渠道消息链路，只需要增加一个受控的“验证目标可达”流程，把配置到首条成功投递之间的空白补齐。

## 方案概述

新增 **Channel Activation Proof**：在保存或启用渠道后，引导用户完成一条可审计的 proof run。它不是普通业务消息，也不进入投资分析上下文，而是一个受控的系统验证动作。

核心能力：

- `ChannelActivationProbe`
  - 对指定 channel 执行静态配置检查、进程/heartbeat 检查、target discovery、权限前置提示和可选 test send。
  - 输出稳定 verdict：`not_configured`、`needs_restart`、`process_offline`、`needs_target`、`send_blocked`、`sent`、`confirmed`、`failed`。

- `ChannelActivationProof`
  - 记录一次 proof run 的结果：channel、target、scope、started_at、completed_at、verdict、reason code、redacted detail、message id 或平台回执、next action。
  - 可先存 SQLite 或 JSONL，保留 30 天即可；不需要变成长期会话历史。

- `Test Send`
  - 发送一条明确标记为系统测试的短消息，例如“这是 Hone 的渠道可达性测试。收到后无需回复。”
  - 不调用 agent runner，不消耗 daily conversation quota，不进入 session transcript，不触发投资建议风险。

- `Target Capture`
  - 对 Telegram/Discord/Feishu group 场景，支持从最近 inbound envelope、cron target directory 或手动输入中选择 target。
  - 对 Feishu 支持 email/mobile/open_id 的 target resolution dry-run，明确说明解析失败或歧义。

- `Activation Checklist`
  - 设置页按 channel 显示下一步：配置 token、保存并重启、确认 sidecar online、选择目标、发送测试、查看结果。
  - CLI 提供同等能力，例如 `hone-cli channels prove telegram --target <id>` 和 `hone-cli channels prove --json`。

第一版重点是证明“渠道能向目标投递”，不是证明消息内容渲染质量或事件路由策略。渲染质量由 `auto_p1_multichannel-render-preview.md` 处理；事件投递解释由 `auto_p1_delivery_decision_loop.md` 处理。

## 用户体验变化

### 用户端

- Public/Web 用户在绑定或启用渠道后，可以看到“未验证 / 已验证 / 最近失败”的渠道状态，而不是只看到配置是否存在。
- 对需要用户动作的渠道，页面给出具体下一步，例如“请先在 Telegram 中给 bot 发送任意消息以捕获 chat id”。
- 收到测试消息后，用户能在 Web 上看到“Telegram 私聊已验证，最后验证于 2026-05-25 14:04”。

### 管理端

- Channel 设置页从“表单保存”升级为“保存 + proof checklist”：
  - auth fields configured
  - effective config regenerated
  - process online
  - target selected
  - test send succeeded
  - recent delivery failures
- `/api/channels` 或新的 channel proof API 返回最近 proof verdict，使 dashboard 能区分“进程在线但从未验证投递”和“最近投递失败”。
- 管理员可以为某个 actor/target 发起 proof run，结果可复制到 support bundle 或 issue。

### 桌面端

- Bundled 模式下，保存 channel settings 后可以自动触发本地 sidecar 重启/刷新状态，再引导 test send。
- Remote mode 下不运行本地 sidecar 操作，只展示远端 backend 的 proof API 结果，避免把本机环境误当成远端环境。
- Desktop channel status badge 不再只显示 running/stopped，也能显示“ready for test”“verified”“needs target”。

### 多渠道

- Telegram/Discord：优先从最近 inbound message 捕获 chat id / guild-channel id；手动输入 target 时先 dry-run 格式校验，再 test send。
- Feishu：区分 email/mobile/open_id/chat_id，先做 target resolution dry-run；歧义或无法解析时不发送测试消息。
- iMessage：由于本地权限和 macOS 环境强依赖，第一版只做本机支持性检查、target handle 校验和可选 test send，不把它纳入 CI 或远端证明。
- Web channel：可以把 public chat 本身视为 baseline channel，只记录 API/session ready，不需要外部 test send。

## 技术方案

### 1. 定义 proof 类型

建议在 `crates/hone-core` 或 `crates/hone-web-api` 的共享层增加数据类型，先保持只读/写简单：

```rust
pub struct ChannelActivationProbeRequest {
    pub channel: String,
    pub target: Option<String>,
    pub channel_scope: Option<String>,
    pub dry_run: bool,
}

pub struct ChannelActivationProof {
    pub id: String,
    pub channel: String,
    pub target_redacted: Option<String>,
    pub channel_scope: Option<String>,
    pub verdict: ChannelActivationVerdict,
    pub reason_code: String,
    pub next_action: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub platform_message_id: Option<String>,
    pub detail_json: serde_json::Value,
}
```

`detail_json` 必须脱敏：不记录 bot token、app secret、完整手机号、完整 open_id 或完整 chat id。target 可以保留后 4 位或 hash。

### 2. Probe 分层

每次 probe 分四层执行，任何一层失败都返回明确 reason code：

1. `config`: channel enabled、必填凭证非空、chat scope 合法、allowlist 形态合法。
2. `runtime`: backend connected、effective config 已生成、channel binary 支持当前平台、process heartbeat fresh。
3. `target`: target 来源明确，且符合该平台格式；Feishu 额外做 email/mobile/open_id/chat_id 解析 dry-run。
4. `send`: 通过现有 channel outbound/client 发送一条 test message，记录平台回执或错误类型。

这样可以把“没配置”“没启动”“没目标”“目标不可解析”“平台拒绝发送”分开，而不是全部落成 send failed。

### 3. 复用现有 channel 代码但避免进入 agent 链路

- Feishu 可在 `bins/hone-feishu/src/client.rs` 或 outbound helper 中抽一个可测试的 `send_activation_test_message`，Web API 侧通过共享库或轻量 service 调用。
- Telegram/Discord 同理，把“发送纯文本到指定 target”的最小能力从 listener/outbound 局部代码中抽成可复用函数。
- iMessage 保持平台隔离，先在本机 binary/desktop command 里实现。
- 不经过 `AgentSession::run()`，不写 session，不走 runner，不消耗 quota。
- proof message 内容固定，禁止用户输入任意文本，避免把 test send 变成绕过权限的发信 API。

如果第一版不方便从 Web API 直接调用 bin 内私有 client，可以先实现 `dry_run + next action + target capture`，真实 test send 在各 channel binary 启动时通过内部 admin endpoint 或 command queue 执行。但最终产品目标应是统一 proof API。

### 4. API 与 CLI

新增 admin/desktop-only API：

- `GET /api/channel-proofs?channel=&target=`
- `POST /api/channel-proofs/probe`
- `POST /api/channel-proofs/test-send`

公共用户 API 可以晚一阶段开放，只允许当前 web actor 自己绑定的 channel target，不允许任意发信。

CLI：

- `hone-cli channels prove <channel> --target <target> [--dry-run]`
- `hone-cli channels prove <channel> --from-recent-target`
- `hone-cli channels proofs --json`

`hone-cli doctor` 可附加摘要：`channel-proof:telegram=not_verified`，但不要默认发送消息。

### 5. 前端落点

在 `packages/app/src/pages/settings.tsx` 的 channel tab 增加每个 channel 的 proof panel：

- 当前状态：`Not verified` / `Verified` / `Failed recently` / `Needs restart` / `Needs target`。
- target selector：来自 recent inbound、cron target directory、manual input。
- `Dry run` 和 `Send test` 两个动作。`Send test` 明确提示会向目标发一条系统测试消息。
- 最近 proof history：时间、verdict、reason、next action。

`packages/app/src/context/console.tsx` 轮询 channel status 时，可以保留当前 5s 进程状态；proof history 不需要高频轮询，设置页打开时加载即可。

### 6. 与后续提案协作

- `Runtime Readiness Matrix` 可以读取最新 proof verdict，把 `channel_delivery` 从静态检查升级为“已验证/未验证/最近失败”。
- `Multichannel Render Preview` 负责内容形态；proof 只发送固定短消息，不验证复杂 Markdown/图片。
- `Delivery Decision Loop` 解释事件为何发送或未发送；proof 验证平台目标是否可达。
- `Redacted Support Bundle` 若落地，可包含最近 proof records 的脱敏摘要。

## 实施步骤

### Phase 1: Read-only proof model and dry-run

- 定义 `ChannelActivationProof` / `ChannelActivationVerdict` / reason codes。
- 新增 proof storage，先用 JSONL 或 SQLite 均可，保留 30 天。
- 实现 config/runtime/target 三层 dry-run，不发送真实消息。
- CLI 增加 `hone-cli channels prove <channel> --dry-run --json`。
- Web API 增加 admin-only dry-run route。

### Phase 2: Target capture and settings UI

- 从 `CronJobStorage::list_channel_targets()` 和最近 inbound/session actor 中生成 target candidates。
- Settings channel tab 增加 proof checklist、target selector 和 dry-run 结果。
- `/api/channels` 或独立 API 返回最近 proof summary。
- 保存 channel settings 后提示是否继续完成 proof，不自动发送。

### Phase 3: Controlled test send

- 为 Feishu、Telegram、Discord 抽出固定 test message 发送函数。
- Web API/desktop 触发 test send 时只允许固定文案和已选择 target。
- 记录平台回执、错误类型、redacted target。
- iMessage 作为 macOS-only optional lane，不能进入 CI 默认门禁。

### Phase 4: Product integration

- `hone-cli doctor` 和 Runtime Readiness 引用 proof verdict。
- Task create/detail 页面在目标渠道未验证时提示“可创建，但建议先完成渠道验证”。
- Notification/Delivery failure 抽屉链接到最近 proof run。
- Public user self-service channel binding 后，只暴露当前用户的 proof 状态和 test send。

## 验证方式

- Rust 单元测试：
  - channel disabled => `not_configured`。
  - enabled but auth field empty => `not_configured` with missing field reason。
  - enabled/auth configured but no fresh heartbeat => `process_offline`。
  - missing target => `needs_target`。
  - target redaction never stores full token/open_id/mobile/chat id。
  - dry-run never calls outbound send and never writes session/quota.

- API 测试：
  - admin can run dry-run for supported channels.
  - public route, if added, can only prove current actor target.
  - invalid channel returns stable 400 reason code.
  - proof history respects limit and does not leak secrets.

- Frontend tests:
  - settings model maps verdicts to checklist state and next action.
  - target selector handles empty candidates, manual target and last verified target.
  - send button requires explicit confirmation.

- Manual regression:
  - `tests/regression/manual/test_channel_activation_proof.sh` can cover real Feishu/Telegram/Discord accounts when credentials are present.
  - iMessage proof remains manual macOS-only.

- Product metrics:
  - Ratio of enabled channels with at least one verified proof.
  - Time from channel settings save to first verified proof.
  - Reduction in `target_resolution_failed` and first-notification `send_failed` records.

## 风险与取舍

- 风险：test send 本身可能被滥用成任意发信接口。取舍：固定文案、admin/desktop 权限、rate limit、target allowlist、proof storage audit，禁止自定义消息。
- 风险：平台 API 差异会让第一版实现变复杂。取舍：先做 dry-run 和 target capture，再逐个 channel 接入真实 send。
- 风险：Feishu target resolution 可能需要额外权限或 API 调用。取舍：解析失败时给出 next action，不把它伪装成投递成功。
- 风险：用户可能把“测试消息成功”理解为所有主动推送都会成功。取舍：UI 明确说明 proof 只验证目标可达，不验证事件策略、quiet hours、digest、内容渲染和未来平台状态。
- 风险：proof records 可能含敏感 target。取舍：全链路 redaction，存 hash/尾号，不保留完整 secret 或完整手机号。
- 不做：不新增外部账号依赖到 CI，不改变 agent runner，不改 `AgentSession` 主链路，不把公司画像或投资建议写入测试消息。

## 与已有提案的差异

查重范围：

- `docs/proposal/` 下全部 `auto_p*.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 和 `auto_p1_runtime_readiness_matrix.md` 不重复：readiness 关注部署/模型/能力是否配置可用，本提案关注某个外部渠道 target 是否经过受控 test send 证明可达。Readiness 可以消费 proof 结果，但不替代 proof run。
- 和 `auto_p1_multichannel-render-preview.md` 不重复：render preview 验证复杂 answer 在各平台的格式降级，本提案只发送固定短测试消息，验证 token、进程、目标、权限和平台投递。
- 和 `auto_p1_delivery_decision_loop.md` 不重复：delivery decision 解释事件为何 sent/queued/filtered/failed，本提案在事件发生前验证渠道目标本身是否可达。
- 和 `auto_p1_end-user-notification-control.md` 不重复：notification control 让用户配置接收偏好，本提案让用户/管理员证明接收渠道已经连通。
- 和 `desktop-bundled-runtime-startup-ux.md` 不重复：desktop proposal 聚焦 bundled runtime ownership、进程接管和启动体验；本提案跨 CLI/Web/Desktop/IM，聚焦配置后的渠道可达性证据。
- 和 `skill-runtime-multi-agent-alignment.md` 不重复：本提案不涉及 skill frontmatter、multi-agent 阶段状态或 runner 工具权限。

## 文档同步说明

本轮只新增 proposal，不开始实施，不改变模块边界、长期约束、测试规则或运行配置，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。若后续正式执行，需要按阶段新增动态计划，并在实现 API/CLI/UI 后同步 repo map 与必要的 runbook/verification 文档。
