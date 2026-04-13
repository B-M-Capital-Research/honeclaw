# Proposal: Skill Runtime Alignment with Claude Code and Multi-Agent Optimization

日期：2026-04-13
状态：草案

## 背景

Hone 当前的 skill runtime 已经完成了几个关键收敛：

- 以 `SKILL.md` 作为真相源
- 先暴露 compact listing，再在调用时注入完整 skill 内容
- 支持 `context`、`agent`、`model`、`effort`、`paths`、`script`、`shell` 等 frontmatter 字段解析
- 支持 slash skill、`discover_skills`、`skill_tool`、session restore

但它与 Claude Code 官方 skill 模型仍存在几类关键差异，尤其是在以下两点上：

1. “skill 被激活后，运行时该如何严格影响工具、模型、上下文和恢复语义”
2. `multi-agent` runner 下，skill 是否仍然按“可发现 -> 可调用 -> 可持续影响当前任务”的方式工作

本 proposal 目标不是立即改代码，而是先给出一个对齐 Claude Code 官方模型的优化方向，并明确 `multi-agent` 路径下的 skill 语义应该如何收口。

## 官方基线

对比基线采用 Claude Code 官方文档，参考：

- [Skills](https://code.claude.com/docs/en/skills)
- [Extend Claude Code](https://code.claude.com/docs/en/features-overview)
- [Subagents](https://code.claude.com/docs/en/sub-agents)
- [Commands](https://code.claude.com/docs/en/commands)
- [Tools reference](https://code.claude.com/docs/en/tools-reference)

基于上述官方资料，可抽象出 Claude Code 的“标准 skill 实现”要点：

1. Skill 是 prompt-based workflow，而不是硬编码命令逻辑。
2. 描述信息常驻上下文，完整 skill body 只在实际调用时加载。
3. `disable-model-invocation` 与 `user-invocable` 是两套不同控制面：
   - 前者控制 Claude 能否自动调用
   - 后者控制用户菜单可见性
4. `allowed-tools` 是“技能激活期间的额外预批准权限”，不是“技能可见工具全集”。
5. `context: fork` + `agent` 会把 skill 作为一个真正的子代理任务在隔离上下文中运行。
6. skill 可以引用 supporting files，并按需加载，而不是只能依赖单个 `SKILL.md`。
7. skill 调用后的内容会作为单条消息留在会话里，并在 compaction 后被重新附着。
8. subagent 可以通过 `skills:` 预加载 skill；skill 也可以反过来通过 `context: fork` 驱动 subagent。
9. 工具、skill、subagent 是不同层次的能力：
   - tool 提供动作能力
   - skill 提供知识和 workflow
   - subagent 提供隔离上下文和专用执行环境

## Hone 当前实现

### 已对齐的部分

截至 2026-04-13，Hone 当前已经和 Claude Code 模型对齐的部分包括：

1. 以 `SKILL.md` frontmatter + body 作为 skill 真相源，见 `crates/hone-tools/src/skill_runtime.rs`。
2. 运行时聚合多层目录，并支持近处覆盖远处，见 `SkillRuntime::load_all_skills()`。
3. 默认暴露 compact skill listing，只在 `skill_tool(...)` 或 slash invoke 时展开完整 prompt，见 `crates/hone-channels/src/agent_session.rs`。
4. skill prompt 可以在 session metadata 中持久化并参与 restore，见 `memory/src/session.rs` 与 `AgentSession::restore_context()`。
5. 支持 `context` / `agent` / `paths` / `script` / `shell` / `arguments` 等字段解析并透出给上层。
6. `load_skill` 已经退化为兼容层，主路径为 `discover_skills` + `skill_tool`。

### 与 Claude Code 的关键差异

| 维度 | Claude Code 官方模型 | Hone 当前状态 | 主要问题 |
| --- | --- | --- | --- |
| 自动调用控制 | `disable-model-invocation` 与 `user-invocable` 分离 | 仅有 `user-invocable`；不存在“对模型隐藏但对用户可见”的控制面 | 无法把高副作用 skill 从模型自动发现面摘掉 |
| tool 权限语义 | `allowed-tools` 是 skill 激活后的权限增量 | `allowed-tools` 目前主要是解析与展示字段；默认对话链路没有严格 runner-side enforcement | 文档宣告与实际执行语义不一致 |
| `context: fork` | 真正驱动 subagent / isolated context | 字段可解析，但默认运行时还没有稳定落到 runner 级隔离执行 | skill 无法真正选择 inline vs fork 执行语义 |
| supporting files | `SKILL.md` 可引用额外文件并按需加载 | 目前只有主 `SKILL.md` + 可选 `script`；没有 supporting files / 按需引用机制 | 大型 skill 只能堆在单文件正文中 |
| 动态上下文注入 | 官方支持 `!cmd` 预处理注入 | Hone 只有 `${HONE_SESSION_ID}` / `${HONE_SKILL_DIR}` / `${ARGUMENTS}` 占位替换 | skill 对实时上下文的表达力偏弱 |
| skill 生命周期 | skill 激活后作为一条明确的会话上下文存在，并在 compaction 后按预算重新附着 | slash 路径可恢复；模型通过 `skill_tool` 的自动激活并不总是具备相同恢复语义 | “显式 slash skill”和“模型自动 skill”不是同一套持久化模型 |
| subagent 协作 | subagent 可预载 skills，skill 可驱动 subagent | Hone 有 `context` / `agent` 字段，但没有形成与 runner 选择联动的统一语义 | skill 和 multi-agent / runner 配置各自为政 |
| 热重载 | 官方近期已支持 live reload | Hone 当前计划中仍把 watcher hot reload 视为 gap | 改 skill 后需要更重的刷新路径 |

### 额外实现偏差

还有两个当前实现中的工程偏差值得单独指出：

1. `skills/skill_manager/SKILL.md` 已推荐新 schema `allowed-tools`，但 `skills/skill_manager/create_skill.sh` 生成的仍是旧 `tools:` 字段。
2. Web API / Web UI 目前只支持查看与启停 skill，不支持以“官方 skill schema”为中心的创建、校验和 lint 流程。

## Multi-Agent Runner 现状

`multi-agent` runner 目前是一个“两阶段混合执行器”：

1. search 阶段：
   - `FunctionCallingAgent`
   - 共享主会话 `system_prompt`
   - 使用完整 `ToolRegistry`
2. answer 阶段：
   - `OpencodeAcpRunner`
   - 使用 handoff text 接收 search 结果
   - 只限制 `max_tool_calls`

关键实现位于：

- `crates/hone-channels/src/runners/multi_agent.rs`
- `crates/hone-channels/src/core.rs`
- `crates/hone-channels/src/runners/opencode_acp.rs`
- `crates/hone-channels/src/mcp_bridge.rs`

### 当前 skill 使用模式

#### 1. Search 阶段继承完整 skill discovery 语义

`MultiAgentRunner` 的 search 阶段直接复用主 `system_prompt` 和完整 `ToolRegistry`。

这意味着：

- search 阶段看得到 turn-0 skill listing
- search 阶段看得到 “相关 skills” 提示
- search 阶段理论上可调用 `discover_skills`
- search 阶段理论上可调用 `skill_tool`
- search 阶段并不只拥有搜索工具，而是拥有整个注册表

虽然 `build_search_input()` 只显式鼓励 `web_search` / `data_fetch`，但这是软提示，不是硬限制。

结果是：search 阶段在架构上并不是“搜索代理”，而是“带搜索倾向的全能代理”。

#### 2. Search 阶段的自动 skill 调用不是一等状态

search 阶段若调用 `skill_tool`，调用结果会进入 `tool_calls_made`，但 skill 激活并没有被提升为 runner 级显式状态。

更具体地说：

- `skill_tool` 的 metadata 持久化依赖 `HONE_MCP_SESSION_ID`
- 该环境变量来自 Hone MCP bridge
- `FunctionCallingAgent` 直接在进程内执行 `ToolRegistry`，并不会经过 Hone MCP

因此，search 阶段自动调用 `skill_tool` 时：

- tool 结果会存在
- 但“skill 已激活”的状态不一定会被统一写入 session metadata
- restore / compaction 也就不能稳定地把它当作长期 skill context 处理

这导致 Hone 当前存在两种 skill 激活路径：

1. slash skill：由 `AgentSession` 显式持久化
2. 模型自动 `skill_tool`：更像一次普通工具调用

这与 “user slash skills and model skill_tool calls must share the same prompt-expansion source of truth” 的目标并不完全一致。

#### 3. Search direct-return 会绕过 answer 阶段

当前 `MultiAgentRunner` 只要 search 阶段没有发生 tool call，就会直接返回 search 阶段答案，不进入 answer 阶段。

这会带来两个模式分叉：

1. 对于普通问答，这是合理的成本优化。
2. 对于显式 slash skill 或隐式自动 skill，它会导致 skill 任务有时只经过 search 模型，有时又进入 answer 模型。

换句话说，当前 multi-agent 的“技能任务是否走第二阶段”由“search 阶段是否碰巧调用了工具”决定，而不是由“当前任务是否处于 skill execution mode”决定。

这会让 skill 的执行稳定性、输出风格、恢复语义都变得不确定。

#### 4. Answer 阶段没有继承 active skill state

当 search 阶段完成后，answer 阶段接收的是：

- 原始请求文本
- search 阶段最终说明
- tool transcript 的 JSON 文本

但 answer 阶段没有显式继承 “active skills” 集合，也没有继承 search 阶段的 skill-expanded prompt 作为独立上下文层。

当前 handoff 的本质是“把上阶段结果序列化成文本喂给下阶段”，而不是“把上阶段 skill state 迁移给下阶段”。

因此如果 search 阶段调用了 `skill_tool`：

- answer 阶段最多只能从 JSON transcript 里间接看见 skill prompt
- 但它并不真正处于“该 skill 已激活”的运行时状态

#### 5. Answer 阶段只限制次数，不限制工具范围

在 `multi-agent` 模式下，answer 阶段会设置 `max_tool_calls`，但没有设置 `allowed_tools`。

这意味着 Hone MCP bridge 会限制“还能调用几次”，却不会限制“能调用哪些工具”。

从结果上看，answer 阶段仍然拥有整个 MCP 暴露的工具表，只是被提示“最好只补一次工具调用”。

这会让 multi-agent answer 阶段与 skill frontmatter 的 `allowed-tools` / `context` / `agent` 语义完全脱节。

## 诊断结论

当前 Hone skill runtime 的主要问题，不是 skill 解析层太弱，而是“skill 状态还没有成为 runner architecture 的一等概念”。

这具体表现为：

1. skill 解析与 skill 执行是有的，但 skill 激活后的运行时后果还不稳定。
2. `context` / `agent` / `model` / `effort` / `allowed-tools` 等字段已经存在，但还没形成统一执行契约。
3. `multi-agent` 路径进一步放大了这个问题，因为它把执行切成两个阶段，却没有把 skill state 一起切过去。

因此，优化重点不应先放在“再加几个 frontmatter 字段”，而应放在“把 skill activation 变成跨 runner、跨 stage 的统一状态机”。

## 优化目标

建议把目标收口为以下五条：

1. 让 Hone skill runtime 在语义上对齐 Claude Code 官方 skill 模型。
2. 让 slash skill 与模型自动 `skill_tool` 调用共享同一套持久化与恢复语义。
3. 让 `context: fork`、`agent`、`model`、`effort`、`allowed-tools` 真正影响执行，而不是只作为展示字段。
4. 让 `multi-agent` 两阶段执行能够传递 active skill state，而不是只传文本摘要。
5. 让高副作用 skill 可以从模型自动调用面安全摘除。

## 建议方案

### Proposal 1：把 active skill state 提升为 `ExecutionRequest` / `AgentRunnerRequest` 的显式字段

当前 skill 激活主要依赖：

- slash invoke 时 `AgentSession` 的特殊持久化
- `skill_tool` 工具执行时的副作用写回

建议改为：

- 在 `AgentSession` 内统一解析“本 turn 激活了哪些 skills”
- 在 `ExecutionRequest` / `AgentRunnerRequest` 中显式携带：
  - `active_skills`
  - `active_skill_prompts`
  - `active_skill_overrides`

这样做的收益：

1. slash skill 与自动 `skill_tool` 调用可以共用同一套状态模型。
2. multi-agent search -> answer handoff 可以显式传递 active skill state。
3. session restore 不必再依赖工具实现内部是否正确写 metadata。

### Proposal 2：补上 `disable-model-invocation`，形成双控制面

建议在 Hone frontmatter 中补齐 Claude Code 风格的：

- `disable-model-invocation: true`
- 保留 `user-invocable`

目标语义：

- `disable-model-invocation: true`
  - 模型 discovery 面完全不可见
  - 只能用户显式 slash 或 UI 显式触发
- `user-invocable: false`
  - 用户菜单不可见
  - 模型仍可在相关任务中自动使用

这会比现在单靠 `user-invocable` 更接近官方模型，也更适合处理：

- deploy / commit / release / production side-effect skills
- 只做背景知识注入的 hidden helper skills

### Proposal 3：把 `allowed-tools` 从“展示字段”升级为“stage-aware policy overlay”

建议把 `allowed-tools` 的目标语义改成：

- 与 Claude Code 一致：它不是“唯一可用工具表”，而是“skill 激活期间的额外许可 / 首选许可”
- 在 Hone 中具体实现为：
  - runner policy overlay
  - MCP bridge allowlist overlay
  - 可选的 in-process tool registry overlay

对 multi-agent 的具体建议：

1. search 阶段默认白名单应极小化：
   - `discover_skills`
   - `skill_tool`
   - `web_search`
   - `data_fetch`
   - 以及少数确认需要的只读工具
2. answer 阶段默认白名单应再小一层：
   - 仅允许补充性工具
   - 对 skill 激活场景按 `allowed-tools` 叠加许可
3. skill 若未声明 `allowed-tools`，runner 不应自动放大权限

这样可避免 search 阶段和 answer 阶段在没有明确需求时仍能碰到全量工具。

### Proposal 4：让 `context: fork` 真正落到 runner 级分流

建议明确 skill 的两种执行模式：

1. `context: inline`
   - skill prompt 注入当前会话
   - 持续影响本会话直至被替换或压缩掉
2. `context: fork`
   - skill 不直接污染主会话上下文
   - 由专用 runner / subagent 执行
   - 主会话只接收结构化结果与必要摘要

建议不要再把 `context: fork` 仅仅当作未来字段，而要把它做成真正的执行分流点。

对 multi-agent 的推荐落法：

- `context: fork` skill 优先绕过普通 search-direct-return 逻辑
- 明确走“forked skill runner -> summarized result -> parent session”
- `agent` 字段决定 fork runner 的 profile：
  - 只读研究
  - 计划型
  - 通用执行

这样会更接近 Claude Code 的 skill/subagent 关系，也能减少大型 workflow skill 对主上下文的污染。

### Proposal 5：把 active skill handoff 从“JSON transcript”升级为“结构化 handoff”

当前 multi-agent 的 stage handoff 更像“文本拼接”。

建议改为显式 handoff 结构：

- `original_request`
- `search_findings`
- `search_tool_transcript`
- `active_skills`
- `active_skill_prompts`
- `skill_execution_mode`
- `supplemental_tool_budget`

这样 answer 阶段可以：

1. 明确知道哪些 skill 已经激活
2. 明确知道这些 skill 是 inline 还是 fork
3. 根据 handoff state 继续遵守 skill 的工具 / 模型 / effort 约束

### Proposal 6：区分“搜索代理”和“执行代理”

当前 multi-agent 的 search 阶段既承担“搜索”又承担“可能的 skill orchestration”。

建议后续将 stage 角色明确拆开：

1. Search stage
   - 目标：判断是否需要外部新鲜信息
   - 默认工具：`web_search` / `data_fetch` / 少量只读辅助
2. Skill stage（可选）
   - 目标：当任务明显匹配 skill 时，激活 skill 并决定 inline / fork
3. Answer stage
   - 目标：综合 search / skill 结果，完成最终响应

这意味着 multi-agent 不一定永远是“两阶段”，而应允许在 skill-heavy 任务上走“三阶段或分支化”流程。

### Proposal 7：补齐 supporting files 与 richer skill packaging，但放到后续阶段

与 Claude Code 相比，Hone 当前最明显的作者体验缺口之一，是 skill 只能依赖单文件正文。

建议后续分阶段补齐：

- supporting files 引用
- 按需加载 reference/examples/templates
- 更完整的 skill authoring / lint / validation
- `skill_manager` 脚本升级到新 schema

但这部分应排在 runtime semantics 之后，因为当前最大问题是“skill 激活后如何影响执行”，不是“skill 文件如何更优雅地组织”。

## 推荐落地顺序

### Phase 0：文档与语义收口

目标：

- 明确 Hone skill contract 与 Claude Code baseline 的差异
- 冻结“我们要对齐到什么语义”

建议输出：

- 更新 `docs/current-plans/skill-runtime-align-claude-code.md`
- 在 `docs/technical-spec.md` 中区分“已实现语义”和“已解析但未强执行语义”

### Phase 1：把 active skill state 提升为显式执行状态

目标：

- slash / auto skill 统一
- restore / compaction / multi-agent handoff 统一

建议改动范围：

- `AgentSession`
- `ExecutionRequest`
- `AgentRunnerRequest`
- `memory::InvokedSkillRecord`

### Phase 2：Runner 级 policy overlay

目标：

- `allowed-tools`、`model`、`effort` 开始真正影响执行

建议改动范围：

- `ExecutionService`
- `hone_mcp_servers()`
- in-process function-calling tool filtering
- `multi-agent` search / answer stage policy

### Phase 3：真正落地 `context: fork`

目标：

- skill 能显式驱动隔离代理执行

建议改动范围：

- `SkillRuntime`
- `AgentSession`
- `multi-agent`
- 新的 skill execution coordinator

### Phase 4：Authoring / supporting files / live reload

目标：

- 提升 skill 编写体验
- 对齐 Claude Code 的 supporting files 与 hot reload 能力

建议改动范围：

- `skill_manager`
- Web skill management UI
- file watcher / cache invalidation

## 非目标

本 proposal 明确不建议在第一阶段就做以下事情：

1. 立刻把所有 skill 改写成 Claude Code 全量兼容格式
2. 一次性实现 supporting files、shell injection、hooks lifecycle、subagent preload 全套功能
3. 在没有统一 active skill state 之前，直接给某个 runner 打补丁式地强行处理 `allowed-tools`

这些都容易把系统继续推向“字段越来越多，但执行语义仍然分裂”的状态。

## 风险

1. skill state 提升为 runner 一等概念后，会涉及 session restore、compaction、runner handoff、prompt audit 等多处链路。
2. `allowed-tools` 一旦开始强执行，可能暴露现有 skills 对工具权限声明不完整的问题。
3. `context: fork` 落地后，部分当前依赖“skill 直接污染主会话”的 workflow 会出现行为变化，需要回归脚本覆盖。
4. multi-agent 如果引入第三阶段或 skill stage，成本与延迟模型也要一起重算。

## 建议结论

建议把 Hone skill runtime 的下一阶段重心放在“执行语义统一”而不是“继续补字段”：

1. 先把 active skill state 做成显式运行时状态。
2. 再把 `allowed-tools` / `model` / `effort` / `context` 真正接到 runner。
3. 最后再补 supporting files、hot reload 和 authoring UX。

对 `multi-agent` 来说，最关键的不是“让它也能看到 skills”，而是“让 skill 在多阶段执行里保持同一语义”。否则 multi-agent 只会把当前 skill runtime 的不一致放大。
