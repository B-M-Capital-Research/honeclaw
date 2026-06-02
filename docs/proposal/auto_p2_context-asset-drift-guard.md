# Proposal: Context Asset Drift Guard for Agent-Maintained Architecture Docs

status: proposed
priority: P2
created_at: 2026-06-02 14:06:09 +0800
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
- `docs/deliverables.md`
- `docs/templates/plan.md`
- `docs/templates/handoff.md`
- `docs/templates/decision.md`
- `docs/archive/index.md`
- `tests/regression/run_ci.sh`
- `tests/regression/ci/test_skill_runtime_stage_consistency.sh`
- `tests/regression/ci/test_ops_script_argument_quality.sh`
- `scripts/ci/check_fmt_changed.sh`
- `Cargo.toml`
- `package.json`
- `.github/workflows/ci.yml`
- `docs/proposal/`
- `docs/proposals/`

## 背景与现状

Honeclaw 已经把仓库内上下文文档提升为 agent 协作的核心基础设施，而不只是普通说明文档：

- `AGENTS.md` 明确要求 agent 开始实现前形成 todo，并在影响模块边界、长期约束、运行流程或交付规则时同步更新对应文档。
- `docs/decisions.md` 记录了“把 LLM 协作上下文留在仓库内”“动态计划只做活跃索引”“完成定义包含上下文资产同步”等长期决策。
- `docs/repo-map.md` 是新会话理解代码边界、入口、主数据流和常见联动改动的低成本入口。
- `docs/invariants.md` 承载 ActorIdentity / SessionIdentity、云/本地存储权威、runner、skill、company portrait、public auth、配置等不可轻易破坏的运行约束。
- `docs/current-plan.md` 与 `docs/current-plans/` 负责活跃任务接力，`docs/archive/index.md` 和 `docs/handoffs/` 负责历史检索与交接。
- 默认 CI 已经覆盖 Rust、前端和 CI-safe 回归脚本；`tests/regression/ci/` 中也有技能运行时、安装脚本和运维脚本契约测试。

这套文档体系的价值很高：它让不同 agent、自动化和人类维护者可以在同一仓库内恢复架构语境，减少“只看局部代码就改错边界”的概率。问题在于，目前这些上下文资产主要靠人工纪律维护。仓库没有一个 CI-safe 的“上下文资产漂移守护”来回答：

- repo-map 里声明的关键入口是否还存在？
- current-plan 索引的活跃计划是否都有对应计划文件？
- 已完成计划是否仍滞留在活跃索引？
- proposal / handoff / plan 是否包含最小字段？
- 代码模块、workspace manifest、Web route、CLI subcommand、runbook 变动后，相关文档是否可能漏更新？
- automation 是否反复产出同类 proposal，而没有一个机器可读的主题索引帮助查重？

对一个高度依赖 agent 接力的产品来说，这不是文档洁癖。上下文资产一旦漂移，后续 agent 会基于过期边界做修改；用户看到的是修复变慢、重复提案增加、架构决策反复、发布风险上升。

## 问题或机会

### 问题

1. **上下文文档缺少机器可验证的健康状态。**  
   `docs/repo-map.md`、`docs/invariants.md` 和 `docs/current-plan.md` 都是长期真相源，但当前没有脚本检查关键链接、路径、计划索引和最小字段。只有当人或 agent 读到矛盾时才会发现漂移。

2. **动态计划与归档契约容易靠记忆执行。**  
   规则要求活跃任务退出后移除索引、归档计划页、更新 archive index；但仓库没有检查“current-plan 中的计划文件是否存在”“archive index 是否能反查已归档计划”“计划页是否同时出现在 active 和 archive”。

3. **repo-map 可能落后于真实代码入口。**  
   当前 repo-map 记录了 Web admin/public、desktop sidecar、channel bootstrap、runner、skill runtime、cloud runtime 等入口。随着 `Cargo.toml` workspace、`package.json` scripts、route 文件、CLI subcommand 和 channel bins 变化，人工同步成本会持续上升。

4. **自动化 proposal 查重仍主要依赖人工阅读。**  
   `docs/proposal/` 已有大量自动化提案，且主题跨度很大。每轮自动化都要重新扫目录并人工判断差异；随着数量增加，重复风险和读上下文成本都会升高。

5. **文档同步责任没有进入开发者体验。**  
   CI 已经能告诉开发者 Rust/前端/回归失败，但不能告诉他“你改了 `crates/hone-channels/src/ingress.rs`，很可能需要检查 repo-map 的 Main Flow 和 Fragile Areas”。这会让文档同步永远排在最后，容易被赶进度时遗漏。

### 机会

新增 Context Asset Drift Guard 可以把 Hone 的 agent 协作优势产品化：

- 对维护者：减少接力成本，让架构文档从“善意约定”变成“可观测资产”。
- 对 agent：每次开工前可快速读取一份 context health summary，而不是盲目相信所有文档都新鲜。
- 对发布/运维：在 release 或大改前提前发现文档、计划和交付物索引漂移。
- 对自动化：proposal 生成、bug burn-down、CI 修复和 release automation 都可以用同一套索引避免重复劳动。

这是 P2。它不直接改变终端用户功能，也不应打断当前 ACP/runtime/skill/cloud 主线；但它能显著提升长期维护效率和 agent 协作可靠性，尤其适合在仓库复杂度继续上升前补上。

## 方案概述

新增一个 CI-safe 的上下文资产健康层，第一版不引入服务端产品功能，只围绕仓库文档和 manifest 产出检查、报告和 agent 可读摘要。

核心对象：

1. `ContextAssetManifest`
   声明需要守护的上下文资产：stable docs、dynamic docs、templates、runbooks、proposal dirs、archive dirs、workspace manifests、关键入口路径。

2. `ContextAssetCheck`
   每条检查包含 id、severity、输入来源、检测逻辑、失败提示和建议修复文档。

3. `ContextHealthReport`
   机器可读 JSON 与人类可读 Markdown 双输出，记录通过项、警告项、阻塞项、可能漂移的文档和建议下一步。

4. `ProposalTopicIndex`
   从 `docs/proposal/` 与 `docs/proposals/` 抽取标题、priority、created_at、关键词和差异段落，生成轻量索引供后续自动化查重。

5. `DocTouchHint`
   根据 git diff 中的路径模式提示需要检查哪些上下文文档，不强制失败，但在 PR / automation 输出中给出具体指引。

第一版原则：

- 先做只读检查，不自动改写文档。
- CI 门禁只阻塞硬结构错误；语义漂移先作为 warning。
- 不要求每个小 patch 都更新文档，只根据 AGENTS 的准入标准给出提示。
- 不把大模型判断放进默认 CI；需要语义审查时作为手工或 automation 辅助。

## 用户体验变化

### 维护者 / 开发者

- 本地运行 `bash tests/regression/ci/test_context_assets.sh` 后，可以看到：
  - 活跃计划索引是否完整。
  - repo-map 中列出的关键路径是否存在。
  - proposal / handoff / plan 是否缺少最小字段。
  - 当前 diff 是否触发文档同步提示。
- CI 失败信息不只是“文档有问题”，而是指向具体文件和修复动作，例如：
  - `docs/current-plan.md references missing docs/current-plans/foo.md`
  - `docs/archive/plans/bar.md is still listed as active`
  - `Cargo.toml added workspace member crates/new-runtime; check docs/repo-map.md`

### Agent / 自动化

- 每次自动化开工前可以读取 `data/runtime/context-health/latest.md` 或脚本输出，优先处理明确漂移，而不是在长文档中手动搜索。
- 自动 proposal 任务可读取 `ProposalTopicIndex`，快速排除已覆盖主题，并把查重证据写入新 proposal。
- CI 修复类 agent 可先看 `DocTouchHint`，避免只修代码不补上下文说明。

### 管理端 / 桌面端

第一版不需要在产品 UI 中展示。后续如果与 admin diagnostics 合并，可以在管理端只读显示“仓库上下文健康”：

- 当前 release 的 repo-map 更新时间。
- active plan 数量与缺失文件数。
- proposal 查重索引更新时间。
- 最近一次 context guard 结果。

这应保持为维护者诊断面，不面向普通投资用户。

### 多渠道

多渠道用户体验不直接改变。间接收益是后续 Feishu / Telegram / Discord / iMessage 的行为改动更容易被 repo-map 和 invariants 捕获，减少通道语义漂移。

## 技术方案

### 1. 新增上下文检查脚本

建议新增 `scripts/ci/check_context_assets.py`，保持无外部依赖，读取仓库文件并输出 JSON / Markdown：

```text
scripts/ci/check_context_assets.py
  --format text|json|markdown
  --changed-only
  --base-ref origin/main
```

第一版检查：

- `docs/current-plan.md`
  - 活跃任务链接的 `docs/current-plans/*.md` 必须存在。
  - 活跃计划文件不能同时位于 `docs/archive/plans/`。
  - 状态字段必须是约定值之一：`planned`、`in_progress`、`blocked`、`done`、`archived`。
- `docs/current-plans/*.md`
  - 至少包含 title/status/created_at/updated_at/owner/related_files/verification/risks 或同等字段。
- `docs/handoffs/*.md`
  - 新模板 handoff 至少包含状态、相关文件、验证、风险或后续项。
- `docs/proposal/*.md` 和 `docs/proposals/*.md`
  - 标题、status、priority、created_at、owner、related_files、验证方式、风险、差异说明必须存在。
  - 文件名必须匹配 `auto_p[0-4]_*.md`，历史非 auto 文件只作为 grandfathered warning。
- `docs/repo-map.md`
  - 其中出现的高价值路径清单必须存在；第一版可通过一个 allowlist manifest 避免误报普通文本。
- `docs/archive/index.md`
  - archive plans 中的新文件应至少能在 index 或 handoff 中找到入口；第一版 warning，不阻塞。

### 2. 增加显式 manifest

建议新增 `docs/context-assets.yaml`，用结构化方式声明高价值路径和检查策略：

```yaml
stable_docs:
  - AGENTS.md
  - docs/repo-map.md
  - docs/invariants.md
  - docs/decisions.md
  - docs/deliverables.md

dynamic_docs:
  active_index: docs/current-plan.md
  active_plan_dir: docs/current-plans
  archive_index: docs/archive/index.md
  archive_plan_dir: docs/archive/plans
  handoff_dir: docs/handoffs

entrypoints:
  rust_workspace: Cargo.toml
  web_workspace: package.json
  web_app: packages/app/src/app.tsx
  web_api_routes: crates/hone-web-api/src/routes
  channel_runtime: crates/hone-channels/src
  desktop_host: bins/hone-desktop/src
```

这样脚本不需要靠正则猜完整 repo-map，后续改入口时也能明确更新 manifest。

### 3. 加入 CI-safe 回归

新增 `tests/regression/ci/test_context_assets.sh`：

```bash
#!/usr/bin/env bash
set -euo pipefail
python3 scripts/ci/check_context_assets.py --format text
```

它应纳入现有 `tests/regression/run_ci.sh`，因为该 runner 自动执行 `tests/regression/ci/test_*.sh`。

门禁策略：

- P0/P1 硬错误：缺失活跃计划文件、proposal 必备字段缺失、context manifest 指向不存在的核心路径。
- Warning：repo-map 更新时间较旧、archive index 可能漏入口、diff 触发文档提示但未更新文档。
- 第一版不要因为 warning 失败，避免把文档治理变成噪音。

### 4. 生成 proposal 主题索引

扩展同一脚本或新增 `scripts/ci/build_proposal_topic_index.py`，输出：

```text
data/runtime/context-health/proposal-topic-index.json
docs/proposal/topic-index.generated.md
```

建议把 JSON 放 runtime 目录，不提交；Markdown 是否提交可后续决定。索引字段：

- path
- title
- priority
- created_at
- normalized_topic_tokens
- related_files
- existing_difference_section_excerpt

自动化 proposal 任务可以读取 JSON，先按 token 重合度筛候选，再人工/agent 判断是否重复。

### 5. Git diff 文档提示

`--changed-only` 模式读取 `git diff --name-only <base>`，按路径给出提示：

- 改 `crates/hone-channels/src/ingress.rs`：检查 `docs/repo-map.md` Main Flow / Fragile Areas，必要时检查 `docs/invariants.md` 的 ActorIdentity/SessionIdentity。
- 改 `memory/src/company_profile/*`：检查 repo-map company profile 段、invariants company portrait 约束。
- 改 `bins/hone-desktop/src/*` 或 `scripts/prepare_tauri_sidecar.mjs`：检查 desktop runbook、repo-map Desktop Structure。
- 改 `tests/regression/ci/*`：检查 AGENTS / invariants 测试组织策略是否需要更新。
- 改 `config.example.yaml` 或 config modules：检查 config source-of-truth 决策和 runbook。

这部分只提示，不阻塞，除非后续团队决定对 release 分支提高门槛。

## 实施步骤

### Phase 1: 结构健康检查

- 新增 `docs/context-assets.yaml`，只声明当前稳定上下文资产和关键入口。
- 新增 `scripts/ci/check_context_assets.py`，实现 current-plan、proposal、handoff、manifest path 的结构检查。
- 新增 `tests/regression/ci/test_context_assets.sh`，纳入现有 CI-safe runner。
- 为脚本增加少量 fixture 或 unit-style self-check，确保缺字段、缺文件、非法 priority 能被识别。

### Phase 2: Diff 提示与上下文报告

- 增加 `--changed-only --base-ref` 模式，按改动路径输出文档同步提示。
- 生成 `data/runtime/context-health/latest.json` 与 `latest.md`，供自动化读取。
- 在 release / proposal automation 的 runbook 中建议先读取 context health report。
- 对 warning 文案做降噪，确保小型纯执行任务不会被误导为必须更新动态计划。

### Phase 3: Proposal 主题索引

- 从 `docs/proposal/` 和 `docs/proposals/` 抽取主题索引。
- 自动化 proposal 任务在开头读取索引，输出查重范围和相似主题。
- 可选：在 proposal 新增验证中检查 filename、priority、status、related_files、verification、risks、差异说明。

### Phase 4: 维护者诊断入口

- 如果后续 redacted support bundle 或 admin diagnostics 落地，可把 context health summary 纳入维护者支持包。
- 可选：在 desktop/admin diagnostics 只读显示最近一次 context guard 结果。
- 不向普通 public 用户展示，避免把仓库维护状态误解为产品服务状态。

## 验证方式

### 自动化验证

- `python3 scripts/ci/check_context_assets.py --format text`
  - 应能在当前仓库通过，只输出 warning 或 clean result。
- `bash tests/regression/ci/test_context_assets.sh`
  - 应无外部账号、无网络、可在 CI 重复执行。
- `bash tests/regression/run_ci.sh`
  - 新脚本被现有 runner 自动包含。
- 负例 fixture：
  - 构造缺 `priority` 的 proposal，脚本应失败。
  - 构造 current-plan 指向不存在的 plan，脚本应失败。
  - 构造 manifest 指向不存在的核心路径，脚本应失败。

### 手工验收

- 修改一个 channel/runtime 文件后运行 `--changed-only`，能看到具体文档提示。
- 新增一篇 proposal 后，脚本能识别文件名、priority/status/related_files/verification/risks 和差异说明。
- 把已归档计划误挂回 current-plan，脚本能指出冲突。
- 自动化 proposal 任务能读取主题索引并减少重复扫描成本。

### 成功指标

- active plan 缺文件、proposal 缺字段、archive/current 混挂等结构问题在 CI 中可被发现。
- 后续三到五次自动化 proposal 能引用主题索引查重，而不是完全重新人工扫目录。
- 大范围架构改动的 PR 中，文档同步遗漏能更早以 warning 形式出现。

## 风险与取舍

- **风险：CI 噪音增加。**  
  取舍：第一版只阻塞明确结构错误；语义漂移和 diff 文档提示只 warning。

- **风险：脚本误判 Markdown 自由格式。**  
  取舍：只检查最小字段和显式 manifest，不强制所有历史文档改模板；历史例外作为 grandfathered warning。

- **风险：维护 manifest 也会漂移。**  
  取舍：manifest 只放高价值入口，不列全仓文件；它的目标是降低 repo-map 检查的歧义，而不是替代 repo-map。

- **风险：proposal topic index 可能把相近但不同主题误判为重复。**  
  取舍：索引只做候选提示，不自动禁止创建 proposal；最终仍由 agent 在“与已有提案的差异”中说明。

- **风险：过度强化文档流程会拖慢小修复。**  
  取舍：严格遵守 AGENTS 的动态计划准入标准。小型纯执行任务只需要当前会话 todo，不应被要求写 current-plan 或 handoff。

- **不做的边界：**
  - 不用 LLM 在默认 CI 中判断文档语义是否准确。
  - 不自动改写 repo-map、invariants 或 decisions。
  - 不把普通用户产品健康和仓库上下文健康混在一个 UI。
  - 不要求历史所有 handoff/proposal 立即重排格式。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 下 95 篇自动化提案，以及历史目录 `docs/proposals/` 下 2 篇提案。相关但不重复的主题：

- 不重复 `auto_p1_run_trace_workbench.md`：Run Trace 关注一次 agent run 的运行后证据、日志和审计；本提案关注仓库上下文文档、计划、handoff、proposal 和 repo-map 是否漂移。
- 不重复 `auto_p1_user-journey-replay-lab.md`：User Journey Replay 面向产品用户路径回归；本提案面向维护者和 agent 接力上下文。
- 不重复 `auto_p1_redacted-support-bundle.md`：Support Bundle 收集排障证据并脱敏；本提案在 CI / automation 阶段提前发现上下文资产结构问题。
- 不重复 `auto_p1_release-provenance-verification.md`：Release Provenance 保护发布产物和安装链路；本提案保护 repo 内架构协作资产。
- 不重复 `auto_p2_surface-design-contract.md` / `auto_p2_locale-content-contract.md`：它们治理前端视觉和内容一致性；本提案治理工程上下文资产的一致性。
- 不重复历史 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该提案聚焦 skill runtime 与多 agent handoff 语义；本提案只提供仓库级文档和计划漂移守护。

差异结论：现有提案已经覆盖产品体验、运行追踪、发布可信、支持包、前端一致性和技能语义，但尚未覆盖“仓库上下文资产本身如何被机器检查和供 agent 复用”。该主题贴合 Honeclaw 当前高度依赖 agent 协作、动态计划和长期架构文档的维护模式。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。因此无需更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`，也无需归档计划页。

若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/context-asset-drift-guard.md`，并在新增 `docs/context-assets.yaml`、CI 回归脚本、上下文健康报告或自动化查重索引后，同步更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/runbooks/task-delivery.md`，必要时补充一条 decision 说明 context guard 的门禁范围。
