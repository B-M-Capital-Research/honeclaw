# Proposal: Skill Trust Marketplace and Compatibility Gate

status: proposed
priority: P1
created_at: 2026-05-12 17:06:07 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `crates/hone-tools/src/skill_runtime.rs`
- `crates/hone-tools/src/skill_registry.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-web-api/src/routes/skills.rs`
- `crates/hone-web-api/src/types.rs`
- `crates/hone-channels/src/turn_builder.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `packages/app/src/context/skills.tsx`
- `packages/app/src/pages/skills.tsx`
- `packages/app/src/components/skill-detail.tsx`
- `skills/skill_manager/SKILL.md`
- `skills/deep_stock_research/SKILL.md`
- `skills/hone_admin/SKILL.md`
- `skills/chart_visualization/SKILL.md`

## 背景与现状

Hone 已经把 skill 做成一等运行时能力，而不是普通 prompt 片段：

- `SkillRuntime` 从系统目录、自定义目录和动态 `.hone/skills` 发现 `SKILL.md`，解析 `allowed-tools`、`aliases`、`user-invocable`、`model`、`effort`、`context`、`paths`、`hooks`、`arguments`、`script`、`shell` 等 frontmatter。
- skill 披露已经是两阶段模型：默认只暴露 compact listing，`skill_tool(...)` 或 slash skill 被调用后才注入完整 prompt。
- `SkillStageConstraints` 已能按 MCP stage 的 `allow_cron` 和工具 allowlist 隐藏不可用 skill，避免“看得到但不能用”的体验。
- `data/runtime/skill_registry.json` 是全局启停覆盖层；禁用后 skill 会从 discover/search/list 和 slash/skill_tool 调用面消失。
- `SkillTool` 支持显式 `execute_script=true` 执行 skill 声明的脚本，并验证本地图像 artifact 的路径与扩展名。
- Web API `/api/skills*` 和管理端 `/skills` 已能查看注册 skill、详情、来源、是否有 script/path gate、allowed tools，并支持启停和 reset registry。
- 内置技能已经覆盖投资研究、公司画像、定时任务、通知偏好、图表、PDF、图片理解、管理操作等高价值能力，其中 `hone_admin`、`deep_stock_research` 等显然具备高权限或高成本特征。

这说明 Hone 已经接近一个可扩展 agent 平台。下一步如果希望开源生态、团队部署、专业投研 workflow 和商业化增长成立，skill 不能只靠用户把目录复制到 `data/custom_skills/` 或仓库 `.hone/skills/`。它需要一个可信分发和安装控制面：用户知道某个 skill 来自哪里、需要什么权限、是否兼容当前 runner/channel/stage、会不会执行脚本、能访问哪些路径、启用后会影响哪些用户。

现有提案 `skill-runtime-multi-agent-alignment` 重点是把 skill activation、runner stage、multi-agent 和 Claude Code 语义对齐；`investment_playbook_launcher` 重点是把内置 skill 组合成可启动的投资工作流。本提案关注另一个层面：**第三方或团队 skill 如何被发现、审查、安装、启用、升级和回滚**。

## 问题或机会

### 问题

1. **安装与信任边界缺失。**
   当前 runtime 支持多个目录来源，但没有 manifest 级别的 publisher、版本、签名、checksum、license、兼容版本、权限说明或安装记录。管理员看到的是 `loaded_from=system/custom/dynamic`，无法判断一个 custom skill 是谁安装的、从哪个仓库来、是否被篡改、是否过期。

2. **权限声明可见，但没有产品化审批。**
   `allowed-tools`、`script`、`shell`、`paths` 已被解析和展示，但启用开关只有 enabled/disabled。高副作用 skill 与只读研究 skill 在安装体验上没有差别，无法要求管理员确认“允许执行脚本”“允许 cron_job”“允许读取特定 path gate”“允许用户 slash 调用”等权限。

3. **兼容性只能运行后暴露。**
   Stage-aware 可见性已经在 runtime 层存在，但管理端没有安装前检查：某个 skill 是否需要当前 runner 不具备的工具、是否依赖 `chart_visualization` 的 Python/matplotlib、是否需要 `cron_job` 但 MCP stage 禁止 cron、是否只能 admin 使用、是否要求桌面/本机文件能力。

4. **升级和回滚没有状态模型。**
   custom skill 一旦以文件形式存在，registry 只记录禁用覆盖，不记录安装版本、上次升级时间、旧版本 checksum、谁批准升级、升级后是否需要重新验证。对团队部署或商业交付而言，这会让 skill 变成隐形变更源。

5. **增长与生态缺少可发现入口。**
   Hone 的开源价值不只是内置技能，而是让投资研究、数据源、报表、行业模板和团队 SOP 能扩展。没有 marketplace / catalog 层，外部贡献者只能提交代码或让用户手工复制目录，难以形成“可安装、可试用、可评价、可关闭”的插件式增长。

### 机会

AI agent 产品正在从“单个模型 + 工具表”走向“可组合能力包”。对 Hone 来说，skill 是最自然的能力包边界：它可以承载投研方法、行业模板、数据处理脚本、图表产物和团队工作流。若补上信任与兼容性控制面，Hone 可以同时获得：

- 更低的开源共建门槛：贡献者发布 skill 包，不必改核心代码。
- 更清晰的商业化包装：专业 skill pack、团队 SOP、行业研究包、数据源 connector 可以作为增值能力。
- 更稳的安全姿态：高权限 skill 在安装和启用前被显式审查。
- 更好的运维可解释性：某次回答或脚本执行可追溯到 skill 包版本与批准记录。

## 方案概述

新增 **Skill Trust Marketplace and Compatibility Gate**，不是替换现有 skill runtime，而是在其上增加分发、安装、权限审批、兼容性检测和回滚记录。

核心对象：

1. `SkillPackageManifest`
   描述一个可安装 skill 包：id、display name、publisher、version、source URL、license、checksum、min/max Hone version、required capabilities、declared permissions、included skills、scripts、assets、release notes。

2. `SkillInstallRecord`
   记录本机或团队实例安装状态：package id、version、installed_at、installed_by、source、checksum、approval state、enabled skills、previous version、rollback target。

3. `SkillPermissionReview`
   把 runtime 已有 frontmatter 转成审批项：tool permissions、script/shell execution、path gates、cron access、admin-only intent、file artifact access、external network expectation、user slash exposure。

4. `SkillCompatibilityReport`
   安装前和升级前生成报告：当前 Hone 版本、runner、MCP tool allowlist、platform、Python/script prerequisites、required skills/capabilities 是否满足，以及禁用原因。

5. `SkillCatalog`
   轻量 marketplace 目录。第一版可从本地 JSON 或 GitHub raw catalog 读取，不需要中心化商店；后续再支持官方/社区/私有 catalog 源。

第一版目标不是开放任意远程代码自动执行，而是做“可审查的安装包”：下载/导入后默认 disabled，必须通过 compatibility + permission review 才能启用。

## 用户体验变化

### 管理端

- `/skills` 从“注册列表 + 详情”升级为三个区域：
  - Installed：当前实例已安装 skill 和包版本。
  - Catalog：可安装的官方/社区/私有 skill pack。
  - Review Queue：待审批安装、升级、权限变化和失败的兼容性检查。
- skill 详情页除现有 Markdown 外，增加 Trust panel：
  - publisher、version、source、checksum、last installed/updated。
  - permissions：tools、script、shell、paths、cron、slash exposure。
  - compatibility：当前 stage/runner/channel 下是否可用。
  - risk level：read-only、writes user state、runs script、admin/system。
- 安装流程：
  1. 预览 manifest 和 release notes。
  2. 显示 diff：新增/删除/变更的 `SKILL.md`、script、assets。
  3. 显示 permission review。
  4. 默认安装为 disabled。
  5. 管理员确认后启用指定 skills。
- 升级流程必须展示权限 diff；如果新版本新增 script、shell、cron 或更宽 path gate，必须重新审批。
- 支持 rollback 到上一安装版本，并保留禁用覆盖层。

### 用户端

- 普通 public 用户不直接看到 marketplace，避免把安全决策推给终端用户。
- 当用户尝试调用未安装或被禁用的 skill 时，agent 可以稳定回答“当前实例未启用该能力”，并给管理员可操作的 skill id。
- 面向付费或团队用户，可以在后续版本展示“可用能力包”列表，但只允许请求安装，不直接启用。

### 桌面端

- Desktop bundled 模式可提供本机 skill pack 安装入口，但默认只允许本地文件导入或官方 catalog。
- 若 skill 需要脚本、Python、系统命令或本机路径，桌面端在安装前显示本机兼容性结果。
- remote backend 模式不允许桌面壳直接写远端 skill 文件，只能通过后端 admin API 发起 install request。

### 多渠道

- Feishu/Telegram/Discord/iMessage 不提供安装入口，只复用 runtime 启用状态。
- 当 slash skill 不可用时，多渠道回复应短而稳定：skill disabled、stage unavailable、permission not approved、not installed。
- 群聊不暴露 catalog 或安装状态细节，只提示联系管理员。

## 技术方案

### 1. Manifest 与安装记录

新增 package manifest 格式，建议放在每个 skill pack 根目录：

```yaml
package_id: hone.skillpacks.charting
version: 1.2.0
publisher: Hone Official
source_url: https://github.com/.../skillpacks/charting
license: MIT
hone_version:
  min: 0.8.0
skills:
  - id: chart_visualization
    path: chart_visualization/SKILL.md
permissions:
  tools:
    - skill_tool
  scripts: true
  shell:
    - python3
  writes_artifacts: true
  paths: []
compatibility:
  platforms:
    - macos
    - linux
  requires:
    - python3
    - matplotlib
```

安装记录可放在 `data/runtime/skill_packages.json` 或新的 SQLite 表，第一版 JSON 足够：

- `package_id`
- `version`
- `source`
- `checksum`
- `installed_at`
- `installed_by`
- `approval_state`
- `enabled_skill_ids`
- `previous_versions`

`skill_registry.json` 继续只表达 runtime enabled/disabled 覆盖；不要把 package 安装状态和启停状态混在一个文件里。

### 2. Package 读取与安全边界

新增 `hone-tools` 层能力：

- `read_skill_package_manifest(path_or_url)`
- `validate_skill_package(manifest, extracted_dir)`
- `compute_skill_package_checksum(extracted_dir)`
- `install_skill_package(package, target_custom_dir)`
- `rollback_skill_package(package_id, version)`

安全原则：

- 远程 catalog 只能下载到 staging 目录，验证 checksum 后再复制到 `data/custom_skills/<package>/<skill_id>/`。
- 安装包不得覆盖 system skill；同 id 冲突必须走显式 override review。
- package 内 path 必须归一化，拒绝 `..`、绝对路径和 symlink escape。
- script 默认不执行；安装、查看、启用都不能触发 script。
- 如果 manifest 和 `SKILL.md` frontmatter 权限不一致，取更严格结果并标记 mismatch。

### 3. Compatibility Gate

在 `SkillRuntime` 现有 `SkillStageConstraints` 之上新增静态检查：

- Hone version 是否满足。
- 当前 platform 是否满足。
- required tools 是否在当前 registry/tool allowlist 中。
- `script` 是否存在且在 package 内。
- `shell` 是否允许；第一版仅允许显式 allowlist，如 `bash`、`python3`。
- `paths` 是否过宽；例如 `/**` 或 repo root write intent 应标记 high risk。
- `user_invocable=true` 且高权限时要求额外确认。
- admin-only skill 必须声明 admin gate；未声明但使用管理工具时标记风险。

报告通过 API 返回，前端显示为 blocking / warning / info。

### 4. Web API

扩展 `crates/hone-web-api/src/routes/skills.rs` 或新增 `skill_packages.rs`：

- `GET /api/skill-packages/installed`
- `GET /api/skill-packages/catalog`
- `POST /api/skill-packages/preview`
- `POST /api/skill-packages/install`
- `POST /api/skill-packages/{package_id}/approve`
- `POST /api/skill-packages/{package_id}/rollback`
- `GET /api/skills/{id}/trust`

所有写操作必须走 admin auth；public API 不暴露 catalog 写入能力。

### 5. 前端

复用 `packages/app/src/context/skills.tsx` 的状态模型，新增 package 维度：

- `installedPackages`
- `catalogEntries`
- `reviewQueue`
- `compatibilityByPackage`

`SkillDetail` 保持现有 skill Markdown 详情，增加 Trust panel 和 Package diff drawer。Catalog 页面不要做营销式市场页，保持管理工具风格：来源、权限、兼容性、版本、操作。

### 6. 审计与追踪

安装、审批、启用、禁用、升级、回滚应写入轻量审计日志：

- 操作者
- package id/version
- skill ids
- permission diff
- checksum
- action result

后续 Run Trace Workbench 落地后，可把一次 run 使用的 skill id + package version 写入 trace metadata。

## 实施步骤

### Phase 1: 本地 package manifest 与只读 trust view

- 定义 `SkillPackageManifest`、`SkillInstallRecord`、`SkillPermissionReview`、`SkillCompatibilityReport` 类型。
- 为现有 system/custom/dynamic skill 生成 best-effort trust view：无 package 的标记为 `unpackaged_local`。
- 在 `/api/skills/{id}/trust` 返回当前 skill 的权限、来源、兼容性和风险摘要。
- 管理端 `SkillDetail` 增加 Trust panel，不提供安装功能。

### Phase 2: 本地导入与审批

- 支持从本地 zip/目录 preview skill package。
- 做 path/symlink/checksum/frontmatter/manifest 校验。
- 安装到 staging，默认 disabled。
- 管理端提供 approve + enable 指定 skill。
- 写 `skill_packages.json` 和审计记录。

### Phase 3: Catalog 与升级回滚

- 支持配置一个或多个 catalog JSON URL 或本地文件。
- Catalog entry 只提供 metadata、manifest URL、checksum 和 release notes。
- 支持 upgrade preview、permission diff、approve upgrade、rollback。
- Desktop bundled 只允许官方 catalog + 本地导入；remote 模式只能调后端 API。

### Phase 4: 运行链路追踪

- 在 skill invocation metadata、session restore metadata、prompt audit 或未来 trace 中写入 `package_id/version/checksum`。
- 当 skill 被禁用、回滚或权限撤销时，历史 session 不重新注入已撤销 skill 的 live prompt，只保留 transcript。
- 为高权限 skill 增加“本轮调用需要管理员确认”的后续扩展点。

## 验证方式

- 单元测试：
  - manifest 解析、版本范围、checksum、path traversal、symlink escape、frontmatter/manifest 权限 diff。
  - `SkillCompatibilityReport` 覆盖 script、cron、allowed-tools、platform、admin-only、path gate。
  - `skill_registry.json` 和 `skill_packages.json` 分离，启停不污染安装记录。
- API 测试：
  - preview 不写盘。
  - install 默认 disabled。
  - approve 后 skill 出现在 registered list，禁用后从 active list 消失。
  - rollback 恢复上一版本 checksum 和文件内容。
- 前端测试：
  - Trust panel 展示 script/path/allowed-tools/source/version。
  - 权限 diff 出现 blocking 项时 approve 按钮不可用。
  - remote desktop 模式不出现本地文件写入入口。
- 手工验收：
  - 导入一个只读研究 skill。
  - 导入一个带 `script: scripts/run.sh` 的图表 skill，确认必须审批脚本权限。
  - 尝试恶意 zip：`../escape/SKILL.md`、绝对路径、symlink escape，必须拒绝。
  - 升级 skill 新增 `cron_job` 权限，必须重新审批。
- 指标：
  - 安装失败原因分布。
  - 被禁用/回滚 skill 数量。
  - marketplace skill 调用成功率。
  - 高权限 skill 调用次数和失败原因。

## 风险与取舍

- **复杂度增加。** skill runtime 已经处于活跃重构中，过早引入 package 层可能增加维护面。缓解方式是 Phase 1 只读 trust view，不改变现有加载路径。
- **安全承诺不能夸大。** manifest 和审批不能完全防止恶意 prompt 或脚本；产品文案必须强调“权限可见与默认禁用”，不是“绝对安全”。
- **catalog 供应链风险。** 第一版不应自动更新或后台安装；所有远程来源必须 checksum 校验并人工审批。
- **与 plugin/playbook 概念边界。** skill package 是能力分发单元，playbook 是工作流启动单元。不要把 playbook manifest 和 skill package manifest 合并。
- **团队权限模型暂缺。** 当前 admin auth 能力有限，细粒度 RBAC 可等待 operator access audit 提案落地后再扩展。
- **不做的边界。** 第一版不做付费结算、评分评论、远程脚本执行沙箱、中心化云商店，也不允许普通用户直接安装。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 下全部 `auto_p*.md` 和历史 `docs/proposals/`：

- 不重复于 `docs/proposals/skill-runtime-multi-agent-alignment.md`：该提案解决 skill 激活状态、runner stage、allowed-tools 语义和 multi-agent 传递；本提案解决 skill 包的来源、安装、权限审批、兼容性、升级和回滚。
- 不重复于 `docs/proposal/auto_p1_investment_playbook_launcher.md`：playbook 是面向用户的投资工作流启动器；本提案是面向管理员和生态的 skill 分发与信任控制面。
- 不重复于 `docs/proposal/auto_p1_user-data-trust-center.md`：该提案关注用户数据导出/删除/隐私；本提案关注能力包供应链与执行权限。
- 不重复于 `docs/proposal/auto_p0_operator-access-audit.md`：operator audit 关注管理端访问控制和操作审计；本提案只在 skill 安装/审批场景内记录能力包审计，并明确依赖未来更完整的 RBAC。
- 不重复于 `docs/proposal/auto_p1_runtime_readiness_matrix.md`：readiness matrix 关注模型路由和 runtime 能力是否可用；本提案把 readiness 应用于单个 skill package 的安装前兼容性检查。

本主题的独立价值在于：Hone 已经有强 skill runtime，但还没有把第三方/团队 skill 变成可审查、可升级、可回滚的产品能力。这个缺口直接影响开源生态、团队部署、安全信任和未来商业化能力包。
