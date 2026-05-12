# Proposal: Update Compatibility Center for Installed and Desktop Runtime

status: proposed
priority: P1
created_at: 2026-05-12 23:04:40 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `Cargo.toml`
- `package.json`
- `bins/hone-desktop/tauri.conf.json`
- `bins/hone-desktop/tauri.generated.conf.json`
- `.github/workflows/release.yml`
- `scripts/install_hone_cli.sh`
- `scripts/update_homebrew_formula.sh`
- `scripts/prepare_release_notes.sh`
- `crates/hone-web-api/src/routes/meta.rs`
- `crates/hone-web-api/src/types.rs`
- `bins/hone-cli/src/reports.rs`
- `bins/hone-cli/src/start.rs`
- `bins/hone-desktop/src/commands.rs`
- `bins/hone-desktop/src/sidecar.rs`
- `packages/app/src/context/backend.tsx`
- `packages/app/src/lib/backend.ts`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Honeclaw 已经从源码仓库里的 Rust agent 发展为多种安装和运行形态并存的产品：

- `README.md` 同时推荐 `curl | bash`、Homebrew、源码 `cargo run -p hone-cli -- start --build`、Web admin/user UI 和 Tauri desktop。
- `.github/workflows/release.yml` 在 `v*` tag 上构建 Linux/macOS CLI bundle，打包 `hone-cli`、`hone-console-page`、多个 channel bin、`hone-mcp`、Web 产物、skills 和 `config.example.yaml`，再发布 GitHub release 与 Homebrew formula。
- `scripts/install_hone_cli.sh` 通过 GitHub release asset 下载平台包，维护 `~/.honeclaw/releases/<version>`、`~/.honeclaw/current`、wrapper、config、data 和后续 `hone-cli doctor/onboard/start`。
- `docs/repo-map.md` 记录了 release notes、install script、Homebrew tap、desktop sidecar、bundled/remote backend、effective config 和 process lock 等发布/运行约定。
- `/api/meta` 当前返回 `version`、`api_version`、capabilities、deployment mode 和 language，前端 `BackendContext` 用 `api_version` 做兼容判断。
- `hone-cli status/doctor` 已经能输出模型、channel、API key、runtime binaries 和文件系统状态，但它关注“当前能不能跑”，不是“当前安装是否过期、是否与远端/桌面/sidecar 兼容、是否需要升级或迁移”。
- `bins/hone-desktop` 暴露 backend config、start/stop bundled backend、channel cleanup、agent settings、CLI probe 等命令；desktop packaged 模式还依赖 sidecar 打包脚本生成 Tauri 配置。

这些基础说明 Hone 已经有发行链路和局部版本信息，但还没有一个面向用户、管理员和运维的 **Update / Compatibility** 产品层。用户现在需要从 release notes、Homebrew、GitHub asset、desktop 设置、CLI doctor、backend meta、日志和失败提示里拼接判断：我现在跑的版本是否落后？桌面壳、backend sidecar、Web bundle、skills、config schema、runner CLI 和远端 public API 是否彼此兼容？如果要升级，下一步是什么？如果升级失败，怎么回滚？

这在开源本地产品里是 P1 问题。它不会像 safety gate 那样直接决定投资输出是否可信，但会显著影响安装版留存、桌面端信任、远端运维支持效率和 release 质量感知。AI agent 产品的关键体验不是“安装一次能跑”，而是长期使用过程中持续升级、模型/runner/skill 演进和配置迁移仍然可解释、可恢复。

## 问题或机会

当前缺口集中在六类链路：

1. **版本来源分散，用户不知道自己处于哪个兼容窗口。**
   `Cargo.toml`、Tauri config、release tag、GitHub asset、Homebrew formula、installed `current` symlink、backend `/api/meta.version`、Web bundle和 desktop shell 都可能表达版本，但没有统一 `InstallManifest` 或 `CompatibilityManifest` 把它们串起来。

2. **API 兼容只检查 `api_version`，不检查产品能力迁移。**
   `packages/app/src/context/backend.tsx` 能拒绝不支持的 backend API version，但无法说明“当前 Web bundle 需要 company profile transfer v2 / public SMS auth / local file proxy / channel settings schema”，也无法提醒“远端 backend 太旧，当前 desktop 应切换到 bundled 或升级服务端”。

3. **安装版缺少可见的升级与回滚路径。**
   `install_hone_cli.sh` 已经把每个 release 放进 `releases/` 并用 `current` symlink 指向当前版本，这天然支持回滚；但 `hone-cli` 没有一等命令或 UI 告诉用户当前 release、最新 release、资产校验状态、上一次升级结果和可回滚版本。

4. **桌面 bundled 模式最容易遇到“壳、sidecar、Web 产物、config schema”漂移。**
   Desktop 是用户最不愿理解内部结构的入口，但它实际上托管 backend、channel listener、Web bundle、本地 config、actor sandbox 和 runtime locks。升级时如果某个 sidecar 版本、Web build 或 config migration 不匹配，用户只会看到 backend 连接失败或功能异常。

5. **发布质量缺少安装后自证。**
   Release workflow 已经生成 assets、checksum 和 Homebrew formula，但没有把“这个安装包包含哪些 binary/build hash/schema/min API”的 manifest 一起放进包内，也没有提供 `hone-cli update check --json` 这类安装后证明，导致支持问题难以快速定位到包内容、用户配置或运行环境。

6. **增长和商业化入口缺少低摩擦升级提示。**
   Public/Hone Cloud、桌面、本地开源用户面对的升级策略不同。远端 public 用户不该看到本地 CLI 升级文案；桌面用户需要应用内升级提醒；Homebrew 用户需要 `brew upgrade`；curl 安装用户需要 rerun installer 或 `hone-cli update apply`。这些差异现在没有产品化表达。

机会是新增 **Update Compatibility Center**：一层只读为主、可渐进开启自动更新的版本/兼容治理面。它不替代 Runtime Readiness Matrix。Readiness 回答“当前配置能不能完成某个产品动作”；Update Compatibility 回答“当前安装和组件组合是否处在受支持版本窗口，如何升级/回滚/迁移”。

## 方案概述

新增一组可序列化对象和用户界面：

1. `BuildManifest`
   每个 release asset 和 desktop bundle 内置的构建清单，记录 Hone version、git commit、target triple、packaged binaries、Web asset build id、skills snapshot id、config schema version、min/max supported backend API version、release notes path 和 checksum source。

2. `InstallState`
   安装现场状态，记录 install method、install root、current release dir、available local releases、wrapper path、canonical config path、data dir、detected Homebrew formula version、desktop app version、backend meta version 和 last update check。

3. `CompatibilityVerdict`
   对当前组合给出 `supported`、`update_recommended`、`update_required`、`rollback_recommended`、`unknown` 等状态，并列出 reason codes，例如 `web_backend_api_too_old`、`desktop_sidecar_mismatch`、`config_schema_migration_pending`、`release_notes_missing`、`asset_checksum_unverified`、`runner_cli_below_minimum`。

4. `UpdateAction`
   面向安装方式的下一步动作：`brew upgrade B-M-Capital-Research/honeclaw/honeclaw`、`curl ... | bash`、`hone-cli update apply vX.Y.Z`、`desktop_download_latest`、`switch_desktop_remote_backend`、`rollback_to_local_release`。

第一版目标是“可见、可诊断、可回滚”，不是立刻自动静默升级：

- CLI bundle 和 desktop bundle 都内置 manifest。
- `hone-cli status --json` 和 `/api/meta` 扩展返回 build/install/compatibility 摘要。
- 管理端/桌面设置页增加 Update Compatibility 面板。
- `hone-cli update check` 只检查最新 release 和本地安装状态；`hone-cli update rollback` 只切换本地 `current` symlink 到已存在 release；自动下载安装作为第二阶段。

## 用户体验变化

### 用户端

- Public Web 用户不看到本地升级细节，只在服务端版本不支持当前 public UI 时看到稳定文案：“服务正在升级，请稍后重试”或“当前客户端版本过旧，请刷新页面”。
- API-key 用户在 OpenAI-compatible endpoint 遇到兼容问题时，错误响应带稳定 reason code，而不是底层 runner 或 route 错误。
- 试用用户如果使用桌面/本地客户端访问远端 backend，能看到“远端服务版本较旧/较新，本地客户端建议升级”的简洁提示。

### 管理端

- Dashboard 或 Settings 新增 `Update` 面板：
  - 当前 backend version、API version、git commit、deployment mode。
  - 当前 Web bundle build id 与 backend 支持窗口。
  - 最新可用 release、release notes 链接、是否需要升级。
  - 配置 schema 是否需要迁移、是否已完成。
  - Homebrew / curl / source checkout / desktop remote 的推荐动作。
- `/logs` 和 `/runtime readiness` 可引用 compatibility verdict，区分“配置缺失”与“版本不兼容”。
- 管理员在远端部署时可以导出一份 `support bundle summary`，包含版本矩阵但不包含密钥或用户数据。

### 桌面端

- Desktop Settings 顶部显示三段状态：
  - `App shell`: Tauri app version 与 Web bundle build id。
  - `Bundled backend`: sidecar manifest、backend `/api/meta.version`、channel bin 版本。
  - `Remote backend`: 远端 version、API compatibility、capability window。
- bundled 模式若发现 sidecar 与 shell manifest 不匹配，优先提示“重启完成更新”或“重新安装桌面包”，而不是把它表现成普通 backend 连接失败。
- remote 模式下只检查远端 meta，不运行本地 sidecar 升级动作，避免误导。
- 如果升级后无法启动 backend，桌面可提示回滚到上一个本地 release 或切换 remote backend。

### 多渠道

- Channel 进程 heartbeat 可以附带 binary version / build id；`/api/channels` 聚合时显示“进程在线但版本落后/未知”。
- 当用户在 Feishu/Telegram/Discord 中遇到“功能不可用”，系统可以在短回复里说明是服务维护或版本升级，而不是暴露内部 stack trace。
- 主动推送任务在 `update_required` 状态下可进入保守模式：不新增高风险任务，只允许已有低风险任务继续或提示管理员确认。

## 技术方案

### 1. Release build manifest

在 release workflow 的 package step 中生成：

```text
share/honeclaw/build-manifest.json
```

建议字段：

```json
{
  "schema_version": 1,
  "product": "honeclaw",
  "version": "0.11.2",
  "git_commit": "...",
  "target": "aarch64-apple-darwin",
  "built_at": "2026-05-12T15:04:40Z",
  "api_version": "desktop-v1",
  "supported_api_versions": ["desktop-v1"],
  "config_schema_version": 1,
  "web_build_id": "...",
  "public_web_build_id": "...",
  "skills_snapshot_id": "...",
  "binaries": [
    {"name": "hone-cli", "version": "0.11.2"},
    {"name": "hone-console-page", "version": "0.11.2"},
    {"name": "hone-mcp", "version": "0.11.2"}
  ],
  "release_notes": "docs/releases/v0.11.2.md"
}
```

Release workflow 已经知道 tag、target、asset name、checksum 和包内容；manifest 应在 tarball 生成前写入，desktop sidecar prepare/build 流程也应生成同类 manifest。

### 2. Installed state reader

在 `hone-cli` 新增只读检测层：

- 读取 `HONE_INSTALL_ROOT`、`HONE_HOME`、`~/.honeclaw/current`、`~/.honeclaw/releases/*/share/honeclaw/build-manifest.json`。
- 识别 install method：
  - Homebrew：可从 wrapper 路径、Cellar path 或 `brew list --versions honeclaw` 推断，失败时标为 `unknown`。
  - Curl bundle：`~/.honeclaw/current` symlink。
  - Source checkout：存在 `.git` 和 `cargo run` 路径，标为 `source`.
  - Desktop packaged：Tauri resource/bundle manifest。
- 不读取用户密钥，不上传任何信息。

`hone-cli status --json` 增加 `install` 与 `compatibility` 字段；文本版只显示摘要和下一步。

### 3. Update check source

第一版支持两个来源：

- GitHub release API：查询最新 `v*` release、assets 和 release notes URL。
- Homebrew tap：对 brew 安装用户优先提示 `brew upgrade`，不绕过 brew 自己的更新机制。

由于网络可能不可用，`update check` 结果必须可缓存，且离线时只给出 `unknown`，不能阻塞 `hone-cli start`。

### 4. Backend meta 扩展

`MetaInfo` 建议扩展：

- `build`: backend build manifest 摘要。
- `compatibility`: server 侧对当前 Web/API 的兼容窗口。
- `install`: local deployment 时可选返回安装摘要；public/remote 部署默认只返回非敏感字段。

前端兼容策略：

- 保留现有 `api_version` 快速拒绝。
- 新增 soft warnings：版本落后、远端过旧、Web bundle 与 backend build 不匹配。
- 不把完整本地路径暴露给 public user；admin/desktop 才看路径。

### 5. Desktop integration

Desktop command 层新增只读命令：

- `get_update_compatibility`
- `check_for_updates`
- `rollback_bundled_runtime`（只允许切换到本地已存在、manifest 验证通过的 release）

bundled backend 启动前读取 sidecar manifest，启动后用 `/api/meta` 对照版本。若不一致：

- warn：Web bundle build id 不一致但 API 兼容。
- block：backend API 不被当前 shell 支持。
- recover：允许用户切换 remote backend 或回滚。

### 6. Config schema 与 migration compatibility

当前 canonical config 已经是长期用户配置源，不能在升级中被 install asset 覆盖。Update center 应只做兼容判断：

- `config_schema_version` 记录在 manifest 和 effective config snapshot 中。
- 启动时如果检测到旧 schema，运行只读 explain 或已有迁移；需要破坏性迁移时必须要求确认。
- `doctor` 输出“schema compatible / migration available / manual action required”。

### 7. 安全边界

- 不自动上传本地版本、路径、config 或 actor 数据。
- Public meta 不暴露 install root、release dirs、local usernames 或 raw filesystem paths。
- Rollback 只允许指向 manifest 验证通过的本地 release dir，不能接受任意路径。
- 自动下载升级必须验证 GitHub release checksum；第一版可以只提供 check 与手动 action。

## 实施步骤

### Phase 1: Manifest and read-only compatibility

- 在 release workflow 生成 CLI bundle `build-manifest.json`。
- 在 desktop sidecar prepare/build 流程生成同类 manifest。
- 新增 Rust 类型：`BuildManifest`、`InstallState`、`CompatibilityVerdict`、`UpdateAction`。
- `hone-cli status --json` 暴露 install/manifest 信息。
- `/api/meta` 返回 backend build summary 和 supported API window。

### Phase 2: CLI update check and rollback

- 新增 `hone-cli update check [--json]`，支持 GitHub latest release 查询和离线 cache。
- 对 curl bundle 安装新增 `hone-cli update rollback --to <version>`，只切换到已存在本地 release。
- 对 Homebrew 安装只提示 `brew upgrade`，不直接修改 Cellar。
- `hone-cli doctor` 增加 compatibility summary。

### Phase 3: Web/admin/desktop surfaces

- Admin Settings/Dashboard 新增 Update Compatibility 面板。
- Desktop Settings 增加 app shell / bundled backend / remote backend 三段版本状态。
- Backend connection error 增加版本不兼容文案和可执行下一步。
- Channel status 聚合 binary version/build id。

### Phase 4: Release quality gate

- Release workflow 在打包后验证 manifest 中列出的 binaries、Web assets、skills 和 release notes 都存在。
- 上传 checksum 时把 manifest 纳入校验。
- 增加 CI-safe 脚本验证 `build-manifest.json` schema 和 `MetaInfo` JSON backward compatibility。

### Phase 5: Controlled auto-update later

- 在前四阶段稳定后再考虑 `hone-cli update apply`。
- Desktop 自动下载更新只作为显式确认动作，不做静默升级。
- 自动升级失败时必须保留上一版本并提供 rollback。

## 验证方式

- 静态验证：
  - `build-manifest.json` schema test：缺少 version、target、api_version、binaries、web_build_id 时失败。
  - Release package test：manifest 中列出的 binary 和 Web dirs 在 tarball 内真实存在。
  - Meta JSON compatibility test：旧前端仍能读取 `api_version`，新字段缺失时默认 unknown。
- CLI 验证：
  - `hone-cli status --json` 在 source checkout、模拟 curl bundle、模拟 missing manifest 下返回稳定结构。
  - `hone-cli update check --json` 在网络失败时返回 `unknown`，不退出非 0，除非参数错误。
  - `hone-cli update rollback --to <version>` 只接受已有且 manifest valid 的 release dir。
- Desktop 验证：
  - bundled 模式 app shell 与 backend manifest 一致时显示 supported。
  - remote mode 不运行本地 sidecar check，只使用远端 `/api/meta`。
  - backend API 不兼容时显示版本问题，不误报为普通连接失败。
- 手工验收：
  - 从旧 release 安装后升级到新 release，确认 config 不被覆盖、current symlink 指向新版本、旧 release 可回滚。
  - Homebrew 安装场景只显示 brew 指令。
  - Public Web 不泄露本地 install path。
- 指标：
  - 首次安装后因版本/包内容问题导致的支持请求下降。
  - Desktop backend connection failure 中 “unknown error” 占比下降。
  - Release 后 24h 内 update check 成功率和 rollback 成功率可观测。

## 风险与取舍

- 风险：把 update center 做成过重的自动更新系统，增加安全和维护成本。取舍：第一版只做 manifest、check、diagnose、manual action 和 local rollback，不做静默自动升级。
- 风险：GitHub release API 不可用导致误判。取舍：网络失败只返回 `unknown`，不阻塞本地运行。
- 风险：暴露本地路径和安装细节。取舍：public meta 默认隐藏本地路径；只有 admin/desktop 本地上下文显示完整诊断。
- 风险：兼容矩阵和 Runtime Readiness 重叠。取舍：Compatibility 只判断版本/安装/升级/回滚；Readiness 继续判断配置、凭证、进程和能力是否可用。
- 风险：source checkout 用户没有 release manifest。取舍：source mode 使用 git commit、Cargo version、dirty flag 生成 best-effort install state，明确标为非发布包。
- 风险：rollback 切回旧版本后 config schema 已被新版本迁移。取舍：破坏性迁移必须显式确认并留备份；第一版 rollback 对 schema downgrade 只给出警告，不自动改写 config。

不做的边界：

- 不新增业务代码执行路径，不改变 agent runner、skill runtime、scheduler 或 channel delivery 语义。
- 不接入外部商业更新服务。
- 不做强制升级或静默升级。
- 不把 release notes 当作产品公告系统；只提供版本与兼容入口。
- 不把用户 actor 数据、session、portfolio 或 company profiles 纳入 update check。

## 与已有提案的差异

查重范围：

- 已检查 `docs/proposal/` 下所有 `auto_p*` 提案。
- 已检查历史目录 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 与 `docs/proposals/skill-runtime-multi-agent-alignment.md`。
- 重点对照 `auto_p1_runtime_readiness_matrix.md`、`desktop-bundled-runtime-startup-ux.md`、`auto_p1_run_trace_workbench.md`、`auto_p1_user-data-trust-center.md`、`auto_p1_hone-cloud-api-contract.md`、`auto_p1_skill-trust-marketplace.md`。

差异结论：

- 不重复 `auto_p1_runtime_readiness_matrix.md`：readiness 解决“当前配置、凭证、进程、runner 和 capability 能不能完成动作”；本提案解决“安装包、桌面壳、sidecar、Web bundle、API version、release 和 config schema 是否处于兼容版本窗口，以及如何升级/回滚”。
- 不重复 `desktop-bundled-runtime-startup-ux.md`：该提案处理锁冲突、旧进程接管和组件级恢复；本提案处理版本 manifest、sidecar/build mismatch、升级提示与本地 rollback。
- 不重复 `auto_p1_run_trace_workbench.md`：run trace 解释一次 agent 运行为什么失败；本提案解释当前安装/部署组合是否受支持。
- 不重复 `auto_p1_user-data-trust-center.md`：trust center 处理用户数据导出/删除/隐私；本提案不读取 actor 数据，只处理安装和版本元数据。
- 不重复 `auto_p1_hone-cloud-api-contract.md`：Hone Cloud API contract 面向开发者 API 与 public service；本提案只要求 meta/compatibility 在本地、desktop 和远端部署中可诊断。
- 不重复 `auto_p1_skill-trust-marketplace.md`：skill marketplace 处理 skill 信任、兼容和启用；本提案只把 release 包内 skills snapshot 作为版本 manifest 的一个组成部分，不改变 skill 分发治理。

查重结论：现有提案覆盖运行可用性、桌面启动恢复、技能兼容、数据隐私、API contract 和运行追踪，但没有覆盖“已安装产品如何知道自己是否过期、组件版本是否匹配、如何安全升级或回滚”的产品/架构层。因此本主题是新的、可落地的 P1 提案。
