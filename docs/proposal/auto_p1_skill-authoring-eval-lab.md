# Proposal: Skill Authoring Eval Lab for Reliable Agent Capabilities

status: proposed
priority: P1
created_at: 2026-05-23 08:04:24 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `docs/proposal/auto_p1_skill-trust-marketplace.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `docs/proposal/auto_p1_prompt-context-budget-inspector.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `crates/hone-tools/src/skill_runtime.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-tools/src/discover_skills.rs`
- `crates/hone-web-api/src/routes/skills.rs`
- `crates/hone-channels/src/turn_builder.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `crates/hone-channels/src/session_compactor.rs`
- `packages/app/src/context/skills.tsx`
- `packages/app/src/pages/skills.tsx`
- `packages/app/src/components/skill-detail.tsx`
- `packages/app/src/components/skill-list.tsx`
- `skills/chart_visualization/SKILL.md`
- `skills/chart_visualization/scripts/render_chart.py`
- `skills/company_portrait/SKILL.md`
- `skills/skill_manager/SKILL.md`
- `tests/regression/manual/test_skill_runtime_cli.sh`
- `tests/regression/manual/test_hone_mcp_skill_dir_env.sh`
- `tests/regression/manual/test_hone_mcp_cron_visibility.sh`
- `tests/regression/manual/test_opencode_acp_skill_toggle.sh`

## 背景与现状

Hone 的 skill runtime 已经成为产品架构里的核心扩展点，而不是简单的 prompt 文件夹。

当前实现已经具备几个重要能力：

- `SkillRuntime` 会从系统目录、自定义目录和动态 `.hone/skills` 发现 `SKILL.md`，解析 `allowed-tools`、`aliases`、`user-invocable`、`model`、`effort`、`context`、`agent`、`paths`、`hooks`、`arguments`、`script`、`shell` 等 frontmatter。
- `SkillStageConstraints` 已经能根据 MCP stage 的 `HONE_MCP_ALLOW_CRON` 和 `HONE_MCP_ALLOWED_TOOLS` 隐藏当前阶段不可用的 skill，避免“看得到但调用失败”。
- `skill_tool` 已经支持显式 `execute_script=true` 执行 skill 声明的脚本，并校验脚本路径不能逃逸 skill 目录、本地图像 artifact 必须在允许目录和扩展名内。
- `turn_builder.rs`、`mcp_bridge.rs` 和 session compaction 已经围绕“两阶段 skill disclosure、slash/direct invoke、invoked skill metadata restore”形成运行时契约。
- 管理端 `/skills` 可以查看注册 skill、详情、来源、启停状态、allowed tools、script/path gate，但它仍主要是 inventory 和开关，不是 authoring / validation / regression surface。
- 现有手工回归脚本覆盖了若干真实链路：CLI slash skill、MCP skills dir env、cron visibility、opencode ACP skill toggle。这些证明重要，但它们依赖本机 runner / 外部 CLI 状态，多数不能成为默认 CI 的快速门禁。
- 内置 skill 已经有较复杂的能力差异：`chart_visualization` 有 Python 脚本和 artifact contract；`company_portrait` 有 references 和长期 Markdown 资产约束；`skill_manager` 面向 authoring；`scheduled_task` 需要 cron 能力。

这说明 Hone 已经把 skill 做成了 agent 产品的“能力包”边界，但 skill 作者和维护者仍缺少一个系统化的反馈回路：一个 skill 是否 schema 合法、在当前 stage 是否可见、slash 是否可解析、脚本是否能在沙箱里返回合法 artifact、compaction 后是否能恢复、不同 runner 是否能稳定选择它，目前需要靠人工读代码、跑散落脚本和真实对话判断。

现有提案已经覆盖相邻层面：

- `skill-runtime-multi-agent-alignment` 关注运行时语义、active skill state、stage policy 和 multi-agent handoff。
- `skill-trust-marketplace` 关注第三方/团队 skill 的安装、权限审批、兼容性、升级和回滚。
- `agent-permission-broker` 关注执行时的 actor/channel/session 权限裁决。
- `model-route-evaluation-lab` 关注模型路由升级的质量评估。

但这些都没有把“skill authoring 的日常质量门禁”作为独立产品面。Hone 需要一个 **Skill Authoring Eval Lab**：让作者在发布或启用 skill 前，用低成本的 lint、dry-run、fixture、runner smoke 和 UI preview 证明它真的能工作。

## 问题或机会

这是 P1，因为 skill 是 Hone 后续产品能力、开源生态、团队 SOP 和商业化 skill pack 的关键扩展面。没有 authoring eval lab，skill 数量越多，越容易出现用户看得见但用不好、脚本 artifact 失效、跨 runner 行为漂移、prompt 体积膨胀、权限声明和真实能力不一致的问题。

主要缺口如下：

1. **schema 合法不等于产品可用。**  
   `SkillRuntime` 能解析 frontmatter，但作者无法在 UI/CLI 中一次性看到：缺 `description`、alias 冲突、`allowed-tools` 拼错、`script` 不存在、`arguments` 与脚本参数不匹配、`paths` glob 永不命中、`context: fork` 当前未完整执行等问题。

2. **stage-aware 可见性缺少预演。**  
   当前 runtime 可以按 constraints 过滤 skill，但作者无法模拟 “Web chat / ACP MCP / cron-enabled stage / restricted stage / public user / admin user” 下的 discover/list/slash 结果。结果是一个 skill 在管理端显示 enabled，真实 runner 阶段却不可用，排障成本高。

3. **script 和 artifact contract 靠真实对话验证。**  
   `chart_visualization` 这类 skill 的价值依赖脚本 stdout JSON、artifact 路径、图片扩展名、`file://` marker 和 outbound 渲染。现在缺少 `skill eval run chart_visualization --case ...` 这样的 dry-run，作者通常要通过真实 agent 回答间接发现脚本 contract 错误。

4. **prompt 质量和上下文预算没有 skill 级基线。**  
   skill body、references、invocation prompt、related hints、compaction snapshot 都会消耗上下文。当前 `prompt-context-budget-inspector` 提案可观测 turn budget，但 skill 作者没有独立指标：skill body 多长、引用文件是否过重、调用后会给 prompt 增加多少字符、是否容易触发截断。

5. **回归脚本分散且多为 manual-only。**  
   `tests/regression/manual/test_skill_runtime_cli.sh` 和 ACP 相关脚本有价值，但默认 CI 不能依赖本机 OpenCode/Codex/Feishu 等外部状态。缺少一套 CI-safe skill fixture，可以至少证明 discovery、schema、stage filtering、script stdout parser、artifact validator 和 slash resolver 不漂移。

6. **管理端技能页不能回答“这个 skill 能否上线”。**  
   `/skills` 能看 Markdown 和开关，却不能显示 lint findings、兼容阶段矩阵、最近 eval 结果、golden case 是否通过、脚本是否可执行、artifact 样例是否可渲染。管理员启用高价值 skill 时仍像是在切一个静态开关。

机会是把 Hone 的 skill 从“可加载”升级为“可验证、可预演、可回归”的能力包。这个方向能直接降低内置 skill 迭代风险，也能为后续 marketplace / commercial skill pack 提供质量门槛。

## 方案概述

新增 **Skill Authoring Eval Lab**，覆盖 CLI、Web 管理端和 CI-safe fixture 三个层面。第一版不需要改 agent runtime 主链路，也不需要引入真实 LLM judge；先把确定性验证和轻量 dry-run 做扎实。

核心对象：

1. `SkillLintReport`  
   对单个 skill 或 skill 目录执行静态检查，输出 finding code、severity、位置、建议修复方式。

2. `SkillStagePreview`  
   在给定 constraints 下展示该 skill 是否 visible / invocable / executable，以及不可用原因，例如 `missing_tool: cron_job`、`disabled_by_registry`、`path_gate_not_matched`。

3. `SkillEvalCase`  
   checked-in 或自定义的测试用例，包含 skill id、输入 args、file paths、stage constraints、是否执行 script、期望 stdout/artifact/prompt 片段和 expected invocation metadata。

4. `SkillEvalRun`  
   一次执行结果，记录 lint、stage preview、prompt expansion size、script result、artifact validation、warnings、duration、runner smoke 状态和输出摘要。

5. `SkillGoldenFixture`  
   CI-safe fixture，优先验证确定性契约，不依赖外部账号。对需要真实 runner 的部分保留 manual eval profile。

第一版目标：

- 作者能在本地运行 `hone-cli skills lint` 和 `hone-cli skills eval`。
- 管理端 `/skills` 能显示 lint 与 stage preview。
- CI 能跑一组内置 skill fixtures，防止 schema/parser/stage/artifact contract 回归。
- 手工 runner smoke 继续存在，但从散落脚本收敛成 eval profile 的一部分。

## 用户体验变化

### 用户端

- 普通 public 用户不直接看到 Eval Lab。
- 当用户调用一个不可用 skill 时，提示从泛泛的“不存在或未激活”升级为更具体但不泄露内部路径的原因，例如“当前渠道不支持定时任务工具”或“该技能需要管理员完成脚本验证后启用”。
- 如果一个 skill 的 eval 状态是 failing，终端用户不会被引导去使用它，减少“菜单里有但体验坏”的情况。

### 管理端

- `/skills` 增加 Quality panel：
  - `Lint`: pass/warn/fail，列出 top findings。
  - `Stage matrix`: Web chat、ACP MCP、cron-enabled、restricted stage 下是否可见和可调用。
  - `Prompt budget`: body、references、invocation prompt、compact snapshot 估算字符数。
  - `Script dry-run`: 是否有 script、最近 dry-run 是否成功、artifact 样例是否通过验证。
  - `Golden cases`: 最近一次 eval 时间、通过率、失败 case。
- 启用带 script、cron、admin 或宽 path gate 的 skill 前，如果 lint/eval 失败，UI 显示 blocking reason。第一阶段可以只警告，不强制阻断；后续和 permission broker / marketplace 合流。
- 管理员可在 skill 详情页点击 “Run dry eval”，选择一个 fixture 或输入 args，结果只写 eval run，不进入真实 session。

### 桌面端

- Desktop bundled 模式可以运行本机 skill eval，适合作者调试自定义 skill、Python 脚本、图表 artifact 和本地路径。
- Remote backend 模式只显示远端 eval 状态，不允许桌面壳直接扫描或执行本机 skill 文件，避免用户误以为本机文件会影响远端服务。
- 打包版可在 Settings 或 Skills 页显示“当前安装内置 skill 自检状态”，帮助用户区分 runner 配置问题和 skill 本身问题。

### 多渠道

- Feishu/Telegram/Discord/iMessage 不提供 authoring UI。
- 多渠道 runtime 复用 stage preview 结果：如果 slash skill 在该 channel stage 不可用，返回短原因码映射后的用户态文案。
- 对定时任务或图表类 skill，eval lab 可提供多渠道 render preview 的输入样本，但真实渲染预览仍归 `auto_p1_multichannel-render-preview.md`。

## 技术方案

### 1. Skill lint service

在 `hone-tools` 中新增轻量 lint 模块，尽量复用 `SkillRuntime` 已有 parser：

```text
crates/hone-tools/src/skill_lint.rs
```

建议第一版 finding code：

- `missing_name_or_description`
- `duplicate_alias`
- `unknown_allowed_tool`
- `script_missing`
- `script_path_escape`
- `script_not_executable_or_not_file`
- `arguments_declared_but_unused`
- `path_glob_never_matches_fixture`
- `frontmatter_context_not_enforced`
- `hook_declared_but_not_enforced`
- `body_too_large`
- `references_too_large`
- `user_invocable_without_alias_or_clear_name`
- `admin_or_cron_skill_without_risk_note`

severity 分为 `info`、`warning`、`error`、`blocking_for_enable`。其中 `hook_declared_but_not_enforced` 不是要求立刻实现 hooks，而是诚实暴露当前 runtime gap，符合 `docs/invariants.md` 对 frontmatter gap 的约束。

### 2. Stage preview

复用现有 `SkillStageConstraints`，新增纯函数：

```rust
pub fn preview_skill_for_stage(
    runtime: &SkillRuntime,
    skill_id: &str,
    file_paths: &[String],
    constraints: &SkillStageConstraints,
) -> SkillStagePreview
```

输出：

- `registered`
- `enabled`
- `path_matched`
- `visible_in_listing`
- `discoverable`
- `slash_invocable`
- `skill_tool_loadable`
- `script_executable`
- `missing_tools`
- `blocked_reasons`

这个 preview 不调用 LLM，不启动 runner，只验证 runtime 能确定的事实。

### 3. Eval case format

新增 fixture 目录：

```text
tests/fixtures/skills/
  manifest.json
  chart_visualization/basic_line.json
  scheduled_task/restricted_stage.json
  company_portrait/prompt_budget.json
  skill_manager/slash_resolution.json
```

case 示例：

```json
{
  "skill_id": "chart_visualization",
  "stage": {
    "allow_cron": false,
    "allowed_tools": ["skill_tool"]
  },
  "file_paths": [],
  "execute_script": true,
  "script_arguments": {
    "title": "Revenue trend",
    "series": [1, 2, 3]
  },
  "expect": {
    "lint_max_severity": "warning",
    "visible": true,
    "script_success": true,
    "artifact_extensions": ["png"],
    "prompt_contains": ["chart spec", "file:///"]
  }
}
```

对无法 CI-safe 执行的 case 加 `profile: manual`，例如需要真实 ACP runner 的 auto skill selection。

### 4. CLI commands

在 `hone-cli` 增加命令：

```shell
hone-cli skills lint --all --json
hone-cli skills lint chart_visualization
hone-cli skills preview scheduled_task --stage restricted
hone-cli skills eval --fixture tests/fixtures/skills/manifest.json
hone-cli skills eval chart_visualization --case basic_line --json
```

命令默认不修改 skill registry，不写 session，不执行未显式请求的脚本。执行脚本时使用临时 eval session id 和临时 artifact root；输出只保存 eval summary。

### 5. Web API and admin UI

扩展 `crates/hone-web-api/src/routes/skills.rs`：

- `GET /api/skills/:id/lint`
- `GET /api/skills/:id/stage-preview`
- `POST /api/skills/:id/eval`
- `GET /api/skills/eval-runs?skill_id=...`

第一版 eval run 可写本地 SQLite 或 JSONL：

```text
data/runtime/skill_eval_runs.jsonl
```

字段包含 skill id、source、checksum/body hash、started_at、status、finding counts、stage preview summary、script result summary、artifact count、duration、error code。不要记录完整 prompt 或用户私有输入；fixture case 可以记录路径和 case id。

前端在 `SkillDetail` 加 Quality panel，不需要新增大型页面。

### 6. CI-safe regression

新增 CI-safe 脚本：

```text
tests/regression/ci/test_skill_eval_lab.sh
```

覆盖：

- 内置 `SKILL.md` 都能被 parser 和 lint service 读取。
- disabled registry 不影响 registered lint，但影响 stage preview 的 active/visible 结果。
- `scheduled_task` 在 `allow_cron=false` stage 下不可见或明确 blocked。
- `chart_visualization` script dry-run 返回合法 JSON 和 artifact。
- 恶意 fixture 的 `script: ../escape.sh` 必须被拒绝。
- Web slash resolver fixture 和 Rust stage preview 对 enabled/user_invocable 的判断一致。

现有 manual scripts 保留，但可以逐步调用同一 eval case manifest，减少重复维护。

### 7. 兼容策略

- 不改变 `SkillRuntime` 当前加载顺序和 registry 语义。
- 不要求所有 skill 第一阶段都有 golden case；先覆盖内置高价值 skill 和带 script/cron/admin 权限的 skill。
- eval lab 的结果默认 advisory，不直接阻断 runtime；等 marketplace / permission broker 落地后，再把 blocking finding 接入 enable/install gate。
- 对 `hooks`、`context: fork` 等未完全执行的字段，lint 只提示 runtime gap，不要求作者移除字段。
- 自定义 skill 目录可能包含私有 SOP，eval run 默认只保存 metadata，不上传、不进入 support bundle，除非用户显式导出。

## 实施步骤

### Phase 1: Deterministic lint and stage preview

- 在 `hone-tools` 增加 `skill_lint` 与 `preview_skill_for_stage`。
- 为 frontmatter、alias、allowed-tools、script path、stage constraints 添加单元测试。
- CLI 增加 `skills lint` 和 `skills preview`。
- `/skills` Quality panel 先显示静态 lint 和 stage matrix，不执行脚本。

### Phase 2: CI-safe eval fixtures

- 新增 `tests/fixtures/skills/manifest.json` 和 4 个内置 skill case。
- 增加 `tests/regression/ci/test_skill_eval_lab.sh`。
- 将 `chart_visualization` 脚本 dry-run 纳入 CI-safe fixture；若 Python/matplotlib 环境不可稳定依赖，则先拆成 stdout contract unit test，真实渲染保留 manual profile。
- 把 manual-only ACP skill scripts 的 case 元数据迁到同一 manifest，但标注 `profile=manual`。

### Phase 3: Admin eval runs

- 增加 `/api/skills/:id/eval` 和 eval run JSONL/SQLite。
- `SkillDetail` 支持选择 fixture dry-run，展示 findings、stage result、prompt budget、script summary 和 artifact warnings。
- 带 script/cron/admin 的 skill 启用前展示最近 eval 状态。

### Phase 4: Runner smoke and marketplace handoff

- 为 `codex_acp` / `opencode_acp` 增加 optional manual smoke profile，验证模型是否会在指定 prompt 下调用目标 skill。
- 将 eval status 暴露给 future `Skill Trust Marketplace`，作为安装/升级后的 compatibility gate。
- 将 prompt size 和 warnings 接入 future `Prompt Context Budget Inspector`，作为 turn budget 的 skill-level 输入。

## 验证方式

- Rust unit tests:
  - `SkillLintReport` 能识别缺字段、重复 alias、未知 allowed tool、script path escape、缺失 script 文件。
  - `preview_skill_for_stage` 对 enabled/disabled、path gate、cron blocked、tool allowlist 产生稳定 blocked reasons。
  - lint 不因为 `hooks` / `context: fork` 这类当前 gap 字段崩溃，而是输出 warning。
- Frontend unit tests:
  - `SkillDetail` Quality panel 能渲染 pass/warn/fail。
  - Stage matrix 对 blocked reason 和 enabled state 显示一致。
  - Eval run 失败时不会切换 registry enabled 状态。
- CI-safe regression:
  - `bash tests/regression/ci/test_skill_eval_lab.sh` 在无外部账号环境下运行。
  - 内置 skill fixture 可被读取，恶意路径 fixture 被拒绝。
  - `scheduled_task` 在 restricted stage 的可见性结果与 `SkillStageConstraints` 一致。
- Manual smoke:
  - `bash tests/regression/manual/test_skill_runtime_cli.sh` 逐步改为读取同一 manifest 的 manual profile。
  - 在桌面 bundled 模式打开 `/skills`，对 `chart_visualization` 跑 dry eval，确认 artifact warning 可读。
  - 禁用 `skill_manager` 后，stage preview 和真实 `skill_tool` 调用都显示 disabled。
- 指标:
  - 内置 skill lint warning 数量随版本下降。
  - 带 script skill 的 dry-run 成功率。
  - 因 skill unavailable 导致的用户失败回复减少。
  - 新增/修改 skill 的 PR 是否附带 fixture 或明确豁免。

## 风险与取舍

- **风险：Eval Lab 变成另一个复杂测试框架。**  
  取舍：第一版只做 skill runtime 已有事实的确定性验证和少量 script dry-run，不引入 LLM judge，不要求覆盖所有 agent 行为。

- **风险：作者误以为 lint pass 等于回答质量 pass。**  
  取舍：UI 和文档明确区分 deterministic contract、runner smoke 和 answer quality。投资输出质量仍由 output safety、response feedback 和 model eval 覆盖。

- **风险：执行自定义 script dry-run 有安全面。**  
  取舍：默认 lint 不执行脚本；dry-run 必须显式触发，只在 skill 目录内运行，使用临时 artifact root，不传真实用户 session，stderr 脱敏继续复用 `skill_tool` 策略。

- **风险：CI 环境缺 Python/matplotlib 导致 chart fixture 不稳定。**  
  取舍：先把脚本 stdout/parser/artifact path contract 做成稳定测试，真实 matplotlib 渲染可以保留 manual 或环境检测跳过，但必须在报告中显式标注未验证。

- **风险：与 marketplace / permission broker 边界重叠。**  
  取舍：Eval Lab 只证明 skill 包“写得对、当前环境能跑”；是否允许安装、启用、执行副作用，仍交给 marketplace 和 permission broker。

- **不做的边界。**  
  第一版不做中心化 skill catalog、不做评分评论、不做自动修复 skill、不做真实 LLM 回答打分、不把 eval 结果自动上传，也不要求所有 custom skill 必须通过 eval 才能存在。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 下全部 `auto_p*.md` 与历史 `docs/proposals/`：

- 不重复 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该提案处理运行时语义、active skill state、multi-agent handoff、stage policy 和 future authoring 方向；本提案把 authoring lint、stage preview、fixture、dry-run 和 CI-safe regression 具体化为独立产品/架构面。
- 不重复 `docs/proposal/auto_p1_skill-trust-marketplace.md`：marketplace 解决 skill 包来源、安装审批、权限 diff、版本、升级和回滚；本提案解决安装或启用前后如何证明 skill 自身 contract 正确。
- 不重复 `docs/proposal/auto_p1_agent-permission-broker.md`：permission broker 处理执行时 actor/channel/session 是否允许某动作；本提案只做 authoring/eval 层的可用性和质量验证。
- 不重复 `docs/proposal/auto_p1_model-route-evaluation-lab.md`：model lab 比较不同模型/路由在业务样本上的输出质量；本提案验证 skill schema、prompt、stage、script 和 artifact contract。
- 不重复 `docs/proposal/auto_p1_prompt-context-budget-inspector.md`：prompt budget inspector 观察完整 turn 的上下文来源和裁剪；本提案提供 skill-level prompt size 和 warning，作为其未来输入之一。
- 不重复 `docs/proposal/auto_p1_multichannel-render-preview.md`：render preview 验证最终消息跨渠道呈现；本提案只生成 skill eval 的 artifact/message 样本，可供 render preview 消费。

查重结论：现有提案覆盖 skill runtime 对齐、skill 分发信任、执行权限、模型评估和渲染预览，但没有覆盖“skill 作者如何在发布/启用前用 lint + stage preview + golden fixture + dry-run 证明能力包可靠”的独立主题。本提案填补的是 skill 生态从可扩展走向可维护的质量门禁。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，因此不更新 `docs/current-plan.md`，也不新增 handoff 或归档计划页。若后续实际落地本提案，应新增或复用 `docs/current-plans/skill-authoring-eval-lab.md`，并在新增 CLI/API/UI、CI-safe fixture 或长期 skill authoring 规则时同步更新 `docs/repo-map.md`、`docs/invariants.md` 和相关 runbook。
