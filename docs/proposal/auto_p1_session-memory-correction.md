# Proposal: Session Memory Correction Workbench

status: proposed
priority: P1
created_at: 2026-05-16 02:02:00 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_cross-company-thesis-map.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `crates/hone-channels/src/session_compactor.rs`
- `crates/hone-channels/src/agent_session/restore.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `packages/app/src/lib/messages.ts`
- `packages/app/src/lib/types.ts`
- `packages/app/src/components/chat-view.tsx`
- `packages/app/src/pages/sessions.tsx`
- `packages/app/src/pages/chat.tsx`

## 背景与现状

Hone 已经不是一次性问答工具。当前运行时会把跨渠道消息、工具结果、slash skill 状态、ACP runner transcript、manual `/compact` 与自动 compaction 统一写进本地 session：

- `memory/src/session.rs` 使用 versioned session JSON，并可通过 SQLite mirror/index 加速读取；`Session` 显式保存 `actor`、`session_identity`、`messages`、`metadata`、`runtime.prompt` 和 `summary`。
- `crates/hone-channels/src/session_compactor.rs` 会在长会话中生成 `Conversation compacted` boundary、`【Compact Summary】`、compact skill snapshot，并把 `SessionSummary` 写回 session。
- `crates/hone-channels/src/agent_session/restore.rs` 从最近 compact boundary 之后恢复上下文，同时会把 compact summary 和 invoked skill snapshot 重新注入给 runner。
- `crates/hone-web-api/src/routes/history.rs` 已经识别 `compact_boundary`、`compact_summary`、`compact_skill_snapshot`，并把它们标成 `synthetic` / `transcript_only` 给前端。
- `packages/app/src/lib/messages.ts` 会在 timeline 中过滤 transcript-only message，`packages/app/src/components/chat-view.tsx` 只把 compact boundary 显示成“会话已压缩”。
- README 和 public 产品页强调“长期记忆”“投资纪律”“跨平台访问”，但当前用户可直接查看和维护的长期资产主要是 company portraits 与 portfolio；会话级压缩记忆更像隐藏的 runtime 机制。

这套实现对长会话性能和 ACP runner 兼容很关键，但它也引入一个产品风险：Hone 后续回答可能依赖一段用户没有明确看过、不能纠正、不能失效的 compact summary。投资助理场景里，错误记忆不是普通聊天瑕疵；它可能影响后续公司研究、持仓解释、定时任务、通知相关性和用户信任。

## 问题或机会

这是 P1 级提案，因为它显著提升核心体验与系统可信度，但不要求推翻现有 session/compaction 架构。

主要问题：

1. **会话记忆的语义状态不可见。**
   当前 history API 能返回 compact summary，但 public chat timeline 默认隐藏 transcript-only 内容；管理端 session 页面也更像历史浏览器，而不是“这段会话当前会让 Hone 记住什么”的工作台。用户不知道 Hone 是依据原始近端消息、compact summary、skill snapshot，还是 company profile 回答。

2. **错误 compact summary 缺少纠错路径。**
   `SessionCompactor` 的 prompt 已经尽力约束“不生成新的投研结论”，但 summary 仍由 LLM 生成。若它错误提取了用户观点、股票关注表、未决问题或群聊结论，后续 restore 会继续携带错误。现在用户只能继续聊天说“你记错了”，系统没有把这类 correction 结构化地作用到 summary。

3. **短期会话记忆与长期画像边界容易混淆。**
   公司画像是长期投资记忆真相源；session summary 是会话恢复材料。当前 UI 没有把两者清楚分层：用户可能以为“聊天里纠正过”就会更新画像，也可能以为“画像里改过”会自动清除旧 session summary。

4. **管理员排障要靠 trace 和原始历史拼接。**
   Run Trace Workbench 能解释一次 run 的技术链路，但当用户问“为什么 Hone 一直以为我看多 X / 持有 Y / 已经要求 Z”时，排障者需要手动读 session JSON、compact summary、metadata、company profile 和最近对话，缺少一个按 session 聚合的 memory view。

5. **多渠道和群聊放大误记忆影响。**
   `SessionIdentity` 允许群聊共享上下文，group compaction 会记录“进行中议题 / 已形成结论 / 未决问题 / 群约定”。如果群摘要误把某个成员观点当作群共识，后续触发会把错误带入共享 session。现有机制没有让群主或触发者审阅和修正群摘要。

机会是：在不改变 session 存储权威、不开放任意编辑历史的前提下，新增一个轻量的 **Session Memory Correction Workbench**。它把 compact summary、session metadata、skill snapshot、最近原文窗口和长期画像引用合并成可读视图，并提供受控 correction overlay，让用户和管理员能让错误记忆失效、补充更正、触发重新 compact。

## 方案概述

新增会话级 memory review/correction 层，包含四个对象：

1. `SessionMemoryView`
   从现有 session 派生的只读视图，展示当前 active context 的来源：最近原始消息、最近 compact summary、compact skill snapshot、invoked skills、session identity、actor、是否 group session、最近 compaction 时间。

2. `MemoryClaim`
   从 compact summary 和 session metadata 中抽取的可审阅条目，例如“用户关注 NVDA”“用户观点是 X”“群已形成结论 Y”“未决问题 Z”“本会话已调用 company_portrait”。第一版可以先用规则和 Markdown section parser 抽取，不需要新 LLM。

3. `MemoryCorrection`
   受控 correction overlay，不直接改历史消息。字段包含 session_id、claim_id 或 summary_section、correction text、action (`invalidate` / `replace` / `append_note` / `recompact_with_instruction`)、created_by、created_at、status。

4. `MemoryPolicy`
   规定 correction 如何进入下一轮 prompt：失效的 claim 不再注入；替换或补充说明作为 session-fixed correction block 注入；用户要求“重新整理记忆”时触发 manual compact，并把 correction 作为 compactor 的 `user_instructions`。

核心原则：

- 不让用户或管理员随意改原始 transcript；原始历史仍是审计材料。
- 不把 session summary 升级为长期公司画像；长期投资主线仍以 `company_profiles/` 为准。
- 不把 correction 当成投资事实来源；它只描述“用户认为当前会话记忆哪里错了/需要忽略”。
- 第一版只覆盖 session-level memory，不做跨 actor workspace 合并，也不替代 User Data Trust Center 的导出/删除能力。

## 用户体验变化

### 用户端

- Public `/chat` 或 `/me` 增加“当前记忆”入口，用户可以看到 Hone 当前会从本会话中带入的简短摘要。
- 每条 memory claim 旁边提供轻量动作：
  - `这不是我的观点`
  - `已经过期`
  - `改成...`
  - `不要在本会话继续使用`
- 用户提交 correction 后，下一次对话中 Hone 应能明确避免旧说法，例如“我不会再把 NVDA 当成你的持仓，只保留为关注标的”。
- 当 correction 涉及长期画像时，UI 不直接改画像，而是引导“是否让 Hone 更新公司画像”，交给 `company_portrait` skill。

### 管理端

- `/sessions` 或用户详情页新增 Memory tab：
  - 当前 session 的 compact boundary 列表、summary、skill snapshot。
  - active context 由哪些消息恢复而来。
  - 用户/管理员提交过哪些 correction。
  - correction 是否已进入下一次 prompt 或已触发 recompact。
- 排障时管理员能回答“本次错误来自旧 summary、旧 company profile、最近原文，还是 runner 自己发挥”。
- 管理员可以对明显错误的 compact summary 发起 `recompact_with_instruction`，但不能编辑原始消息。

### 桌面端

- Desktop bundled/remote 模式复用同一 API；本地用户在长会话变慢或回答串题时，可以点击“整理/修正当前记忆”。
- 如果 auxiliary LLM 未配置，显示“可提交 correction，但暂不能重新 compact”，并指向 settings 的 auxiliary/profile 配置状态。

### 多渠道

- IM 端不需要完整工作台。用户可以用文本指令：
  - `/memory`：返回最多 5 条当前记忆摘要。
  - `/memory forget NVDA 是我的持仓`：创建 invalidate correction。
  - `/compact 以后只保留我们讨论过的长期 thesis，不要保留价格噪音`：继续复用已有 manual compact，但把结果写入 workbench。
- 群聊中 correction 默认只允许触发者修正自己观点；对“群共识 / 群约定”的 correction 应要求管理员或群主配置后再开放。

## 技术方案

### 1. 派生 SessionMemoryView API

在 `crates/hone-web-api` 增加 `routes/session_memory.rs`，只读 API 先不引入新存储：

- `GET /api/session-memory?session_id=...`
- `GET /api/public/session-memory`

返回示例：

```json
{
  "session_id": "web__direct__user",
  "actor": { "channel": "web", "user_id": "user" },
  "session_identity": { "channel": "web", "user_id": "user" },
  "is_group": false,
  "summary": {
    "content": "...",
    "updated_at": "2026-05-16T02:02:00+08:00",
    "source": "compact_summary",
    "trigger": "auto"
  },
  "active_window": {
    "messages_after_boundary": 8,
    "last_message_at": "..."
  },
  "skill_snapshots": [
    { "skill": "company_portrait", "chars": 4200 }
  ],
  "claims": [
    {
      "id": "claim_1",
      "kind": "user_view",
      "text": "用户关注 NVDA，但未确认持仓。",
      "source": "compact_summary",
      "status": "active"
    }
  ],
  "corrections": []
}
```

实现可以复用：

- `select_messages_after_compact_boundary`
- `message_is_compact_summary`
- `message_is_compact_skill_snapshot`
- `invoked_skills_from_metadata`
- `session_message_text`

第一版 claim parser 只处理当前 compactor prompt 的稳定结构：

- 个人会话：`股票关注表`、`【历史对话总结】`
- 群会话：`进行中议题`、`已形成结论`、`未决问题`、`群约定 / 待办`

### 2. Correction overlay 存储

在 `memory` 增加 `session_memory_correction` 模块。第一版可用 SQLite，便于按 session / actor / status 查询：

```sql
CREATE TABLE session_memory_corrections (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  claim_id TEXT,
  summary_section TEXT,
  action TEXT NOT NULL,
  correction_text TEXT NOT NULL,
  status TEXT NOT NULL,
  created_by TEXT NOT NULL,
  created_at TEXT NOT NULL,
  applied_at TEXT,
  superseded_by TEXT
);
```

状态：

- `pending`: 已提交，尚未进入下一轮 prompt。
- `active`: 已被下一轮 prompt 或 recompact 消费。
- `superseded`: 被新的 correction 替代。
- `dismissed`: 用户或管理员撤回。

不建议把 correction 写进 `Session.metadata`，因为 metadata 已被 ACP compact/reseed、invoked skills 等 runtime 状态使用；独立表更适合查询和审计。

### 3. Prompt 注入与 restore 策略

在 `crates/hone-channels/src/agent_session/restore.rs` 或更靠近 prompt assembly 的层加入一个小的 correction block：

```text
[Session Memory Corrections]
- Ignore the old compact-summary claim: "用户持有 NVDA".
- Treat the user's latest correction as: "NVDA is watchlist-only, not a holding."
```

约束：

- correction block 属于 session-fixed/dynamic context，不写入 static system prefix，避免破坏 prompt cache。
- 只注入 `active` 或刚提交的 `pending` correction；注入成功后把 pending 标记 active。
- 当用户执行 `/compact` 且存在 correction 时，把 correction 摘要传入 `SessionCompactor::compact_session(..., user_instructions)`，让新的 compact summary 吸收 correction。
- 重新 compact 成功后，可把旧 correction 标记 `superseded` 或保留为已应用记录。

### 4. Web API 与权限

新增 API：

- `POST /api/session-memory/corrections`
- `POST /api/session-memory/corrections/{id}/dismiss`
- `POST /api/session-memory/recompact`
- `POST /api/public/session-memory/corrections`
- `POST /api/public/session-memory/recompact`

权限规则：

- Public 端只能访问当前 web session 对应的 actor/session。
- Admin 端可按 `session_id` 或 actor 查询，但要遵守管理端 Bearer token。
- 群聊 correction 第一版只读展示，不开放 public Web 修改，除非能证明当前用户是该 group session 的授权操作者。

### 5. 前端落点

建议先做三处轻量 UI：

- `packages/app/src/pages/chat.tsx`
  - 在 chat header 增加 memory popover。
  - 展示 summary 与 claim 列表；每条提供“过期/错误/改写”表单。
- `packages/app/src/pages/sessions.tsx` / `AdminChatShell`
  - 增加 Memory tab 或侧栏，供管理员排障。
- `packages/app/src/components/chat-view.tsx`
  - 保留“会话已压缩”的简洁提示，但给管理员模式增加查看 summary 的入口。

不建议第一版在消息流里直接展示完整 compact summary，避免普通用户把运行时摘要误认为正式投资报告。

### 6. 与长期画像和反馈系统的关系

- 若 correction 指向 company profile 内容，例如“长期 thesis 写错了”，UI 只生成一个 `company_portrait` handoff prompt，不直接改 `profile.md`。
- 若 correction 指向回答质量，例如“上一条回答引用了错误记忆”，可以同时创建 response feedback，但 feedback 是质量评价，correction 是后续上下文约束。
- 若 correction 指向用户数据删除，例如“忘掉我上传的文件”，应转到 User Data Trust Center，而不是只在 session prompt 层忽略。

## 实施步骤

### Phase 1: 只读 Memory View

- 新增 `SessionMemoryView` 类型和只读 API。
- 从现有 session messages 派生 summary、active window、skill snapshots 和 claims。
- Admin session 页面增加 Memory tab；public chat header 增加只读 memory popover。
- 添加单元测试覆盖个人/群聊 compact summary parser。

### Phase 2: Correction Overlay

- 新增 correction 存储模块和 API。
- Public 端支持对当前 session 提交 `invalidate` / `replace` correction。
- Prompt restore 阶段注入 correction block，pending 成功消费后标记 active。
- 添加 regression：旧 summary 中的错误持仓 claim 被 correction 覆盖，下一轮 runtime input 不再包含未修正 claim。

### Phase 3: Recompact 与多渠道指令

- `POST /api/session-memory/recompact` 复用现有 manual compact pipeline。
- `/memory` 和 `/memory forget ...` 指令走同一 correction API。
- 群聊先只支持查看 memory view；群 correction 需要明确权限后再放开。

### Phase 4: 与长期资产协作

- correction UI 识别“应更新画像 / portfolio / notification prefs / cron”的场景，跳转到对应 agent-mediated workflow。
- 与 response feedback、mutation ledger、data trust center 对接，形成“回答错了 -> 标注原因 -> 修正会话记忆 -> 必要时修正长期资产”的闭环。

## 验证方式

- 单元测试：
  - 个人 compact summary 的股票关注表和历史总结可抽取为稳定 claims。
  - 群 compact summary 的四段标题可抽取为 group claims。
  - malformed summary 不 panic，只返回 `parse_status=degraded`。
- API 测试：
  - public session memory API 只能返回当前 cookie 用户的 session。
  - admin API 能按 `session_id` 查询，并正确标记 synthetic/transcript-only sources。
  - correction 创建、dismiss、active 状态流转稳定。
- Runtime 回归：
  - 构造 session：compact summary 里写“NVDA 是用户持仓”，correction 写“NVDA 只是关注”；下一轮 restored runtime input 不应把 NVDA 当持仓。
  - manual recompact 带 correction instruction 后，新 summary 不再保留被 invalidate 的 claim。
  - correction block 不进入 static system prefix。
- 前端测试：
  - chat memory popover 能展示空态、degraded parser、claims、corrections。
  - mobile narrow viewport 下 correction 表单不遮挡 composer。
- 手工验收：
  - 长会话触发 `/compact` 后，用户能查看摘要并提交纠错。
  - 管理员能从 session 页面判断一次串题是旧 compact summary、旧画像还是最近消息造成。

## 风险与取舍

- 风险：把 compact summary 产品化展示后，用户可能误以为它是正式研究结论。
  取舍：默认展示“当前会话记忆”，不使用“报告”“画像”“投资结论”等词；完整 summary 仅在展开后显示。
- 风险：correction overlay 可能与原始 transcript 冲突。
  取舍：不改 transcript；prompt 明确 correction 是用户后续声明，优先于旧 summary，但不优先于结构化真相源如 portfolio/company profile。
- 风险：额外 prompt block 增加 token 和 cache miss。
  取舍：只注入有 active/pending correction 的 session，且使用短句；不放入 static system prefix。
- 风险：claim parser 对自由 Markdown 脆弱。
  取舍：第一版只解析 compactor 自己生成的稳定标题；解析失败时仍能展示 raw summary 和手动 correction。
- 风险：群聊 correction 权限复杂。
  取舍：第一版群聊只读或管理员修改；个人私聊先落地完整闭环。
- 不做：不重写 compaction 算法，不替代 run trace，不做全用户数据删除，不开放 UI 直接编辑 company portrait 正文。

## 与已有提案的差异

- 不重复 `auto_p1_user-data-trust-center.md`：Trust Center 解决隐私、导出、删除和数据范围透明；本提案解决“会话恢复时 Hone 当前会记住什么，以及用户如何纠正错误 compact memory”。
- 不重复 `auto_p1_run_trace_workbench.md`：Run Trace 面向工程排障的一次 run 时间线；本提案面向 session-level semantic memory，重点是用户可见与可纠错。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：Feedback 记录某条回答是否好/坏；correction overlay 会改变后续上下文恢复，是 runtime 行为输入。
- 不重复 `auto_p1_cross-company-thesis-map.md`：Thesis map 是公司画像之间的派生一致性图；本提案不生成跨公司长期观点，只修正会话摘要和短期恢复材料。
- 不重复 `auto_p1_investment_context_intake.md`：Context intake 补齐 portfolio/profile/prefs/cron 的初始化缺口；本提案处理长会话中自动压缩出来的短期记忆错误。
- 不重复 `skill-runtime-multi-agent-alignment.md`：该历史提案关注 skill state 与 runner/stage policy；本提案只把已有 compact summary/skill snapshot 做成可审阅的用户产品面。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，因此不更新 `docs/current-plan.md`、`docs/repo-map.md` 或 `docs/invariants.md`。如果后续实际落地该提案，才需要新增 current plan，并在改变 session memory API、prompt restore 或前端会话入口时同步更新 repo map 与相关长期约束。
