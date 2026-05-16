# Proposal: Redacted Support Bundle and Diagnostics Pack

status: proposed
priority: P1
created_at: 2026-05-16 08:03:10 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_update-compatibility-center.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `bins/hone-cli/src/reports.rs`
- `bins/hone-cli/src/main.rs`
- `bins/hone-desktop/src/sidecar/runtime_env.rs`
- `bins/hone-desktop/src/sidecar/processes.rs`
- `crates/hone-web-api/src/routes/logs.rs`
- `crates/hone-web-api/src/routes/meta.rs`
- `crates/hone-web-api/src/routes/llm_audit.rs`
- `crates/hone-web-api/src/routes/task_runs.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `scripts/diagnose_fmp_tavily.sh`
- `scripts/diagnose_llm.sh`
- `docs/runbooks/hone-cli-install-and-start.md`
- `docs/runbooks/desktop-release-app-runtime.md`

## 背景与现状

Honeclaw 已经有多种真实运行形态：源码启动、CLI 安装包、Homebrew、Tauri desktop bundled backend、desktop remote backend、公开 Web、Hone Cloud API、多 IM channel 和 event-engine 主动推送。用户遇到问题时，维护者需要判断的问题不再只是“程序有没有启动”，而是：

- 当前配置源到底是 canonical `config.yaml` 还是生成的 `data/runtime/effective-config.yaml`。
- 运行方式是源码、安装包、Homebrew、desktop bundled 还是 remote backend。
- Web backend、channel sidecar、runner CLI、`hone-mcp`、FMP/Tavily、Hone Cloud、OpenCode/Codex ACP 是否可用。
- 最近失败来自配置、版本兼容、runner、模型路由、channel auth、delivery target、日志面板、session persistence，还是用户数据本身。

仓库已经有不少诊断原料：

- `bins/hone-cli/src/reports.rs` 的 `status` 能序列化当前 models、channels、API key 布尔状态、runtime binaries、canonical/effective config path；`doctor` 能检查 config 解析、目录、runner binary、runtime binary 和 channel auth。
- `crates/hone-web-api/src/routes/logs.rs` 会把内存 log buffer 和 `data/runtime/logs/*.log` tail 合并到 `/api/logs`，并已经处理非 UTF-8、ANSI code 和内存锁 poisoned 等异常路径。
- `crates/hone-web-api/src/routes/meta.rs` 返回 `/api/meta`、capabilities、deployment mode、channel status 和 sidecar heartbeat/process scan。
- `bins/hone-desktop/src/sidecar/runtime_env.rs` 已经定义 desktop diagnostic paths：config dir、data dir、logs dir、desktop log、sidecar log。
- `scripts/diagnose_fmp_tavily.sh` 已有 API key mask、FMP/Tavily probe 和 quota/invalid/network 状态判断。
- `crates/hone-channels/src/mcp_bridge.rs` 已实现一套 MCP 日志用的 JSON/text redaction，对 `api_key`、`token`、`access_token`、`password`、`secret`、`Bearer` 等字段脱敏。
- 已有提案计划 Runtime Readiness Matrix、Run Trace Workbench、Update Compatibility Center 和 User Data Trust Center，但它们分别关注运行可用性、一条 run 的 trace、版本兼容、用户数据权利，不等价于“可以安全发给维护者的一次性诊断包”。

也就是说，Hone 已经有足够信息源，但还缺一个产品化、可分享、默认脱敏、可复现的 **Support Bundle**。目前用户或 agent 排障时通常要手工运行 `hone-cli doctor`、打开 `/api/logs`、复制 settings 状态、查看 desktop logs、查 channel status、贴几段命令输出。这个流程容易漏关键信息，也容易误贴 API key、手机号、token、绝对路径、聊天内容或投资数据。

## 问题或机会

这是 P1 级提案，因为它显著降低安装版、桌面端、远端部署和多渠道支持成本，并且能提高用户对本地优先产品的信任。它不改变核心 agent 行为，但会让复杂故障从“来回问十轮”变成“先看一个脱敏包”。

当前缺口集中在五点：

1. **诊断信息分散，支持路径不可复现。**
   CLI `doctor/status`、Web logs、desktop sidecar log、channel heartbeat、task runs、LLM audit、runtime config path、release/install 状态都在不同入口。维护者很难确认用户提供的是同一时间窗口、同一 backend、同一 config 下的信息。

2. **脱敏策略不统一。**
   MCP bridge 有 redaction helper，诊断脚本有 `mask_key`，前端 settings 用 password input 隐藏字段，但没有一个统一的 support export redactor。用户手工复制 YAML、日志或 terminal 输出时，最容易泄露 `api_key`、bot token、Feishu secret、Bearer token、手机号、cookie、文件绝对路径和 prompt-audit 内容。

3. **桌面用户最需要支持包，却最难收集。**
   Desktop bundled mode 包含 Tauri shell、backend sidecar、多个 channel process、runtime locks、generated effective config、desktop config dir、logs dir 和 app sandbox data dir。现在代码已有 `DiagnosticPaths`，但没有“导出诊断包”按钮，也没有将这些路径下的证据按规则打包。

4. **远端和本地支持语义不同。**
   Remote desktop 不应运行本地 CLI probe 来解释远端问题；公开 Web 用户也不应看到服务器路径或完整 admin logs。当前缺一个 bundle profile 区分 `local_cli`、`desktop_bundled`、`desktop_remote`、`server_admin`、`public_user_report`。

5. **现有提案会产生更多诊断对象，需要统一容器。**
   Runtime readiness、update compatibility、run trace、output safety、delivery decision loop 如果陆续落地，会产生更多 verdict 和 reason code。没有 support bundle，这些证据仍会散落在多个页面。

机会是：先做一个本地生成、默认脱敏、用户可预览、管理员可导出的诊断包格式。它不需要外部 SaaS，也不上传任何数据。第一版只聚合已有信息源和静态文件 tail，就能显著提升支持效率。

## 方案概述

新增 **Redacted Support Bundle and Diagnostics Pack**，包含三个层次：

1. `SupportBundleManifest`
   记录 bundle schema、创建时间、deployment mode、收集范围、redaction version、included sections、omitted sections、生成命令、Hone version、OS/arch、time window 和 hash。

2. `SupportBundleCollector`
   按 profile 收集诊断信息：CLI reports、runtime paths、redacted config summary、backend meta、channel status、recent logs、task health summary、LLM audit aggregate、notification delivery summary、desktop diagnostic paths、optional trace references。

3. `SupportRedactor`
   统一脱敏器，递归处理 YAML/JSON/text/log，默认替换 secrets、tokens、cookies、phone/email、absolute paths、local file URIs、prompt-audit raw content、user chat transcript，并保留足够支持判断的结构化摘要。

第一版目标：

- `hone-cli support-bundle --output <zip>` 可在本地/安装版生成 zip。
- Admin Web/desktop 可通过按钮生成相同格式的 bundle。
- Desktop bundled 使用已有 `DiagnosticPaths` 自动纳入 desktop/sidecar/backend/channel log tail。
- Remote desktop 只请求远端 backend 生成 server-side redacted bundle，不混入本机 CLI 状态。
- Public user 只能生成非常窄的 `user_report`：最近失败时间、client/browser/backend coarse status、account id hash、trace/reference id，不包含 admin logs 或用户数据包。

## 用户体验变化

### 用户端

- 桌面设置页增加 `Export Diagnostics` 按钮，用户点击后看到：
  - 包含哪些类别：版本、运行模式、配置摘要、最近日志、channel 状态、任务摘要。
  - 不包含哪些敏感内容：API key、bot token、完整聊天记录、公司画像正文、portfolio 数值、上传文件。
  - 生成后显示 zip 路径和 bundle id。
- Public Web 出错时只显示短引用：`support_ref=...`，用户可以在反馈中附上该引用，不需要复制日志。
- CLI 用户可以运行：

```shell
hone-cli support-bundle --last 2h --output ./hone-support.zip
```

### 管理端

- Dashboard / Settings 增加 `Support Bundle` 区块：
  - 选择 profile：`server_admin`、`runtime_only`、`channel_delivery`、`model_routes`。
  - 选择时间窗口：30 分钟、2 小时、24 小时。
  - 预览 manifest 和 redaction warnings。
- 用户反馈某条消息没回时，管理员可从未来 Run Trace Workbench 或 logs 页面点击 `Export related support bundle`，只纳入相关 trace/task/channel 的证据。
- Bundle 可作为 issue/PR/交接材料引用，减少手工贴命令输出。

### 桌面端

- Bundled mode 自动收集：
  - Tauri shell version 和 deployment mode。
  - desktop config dir/data dir/logs dir 的相对标签，不暴露真实用户名路径。
  - `desktop.log`、`sidecar.log`、backend runtime logs、channel process heartbeat summary。
  - 当前 backend status、channel cleanup/duplicate process scan 摘要。
- Remote mode 明确显示“诊断包来自远端 backend”，本地桌面只附加 shell/backend URL 兼容摘要，不跑本机 channel binary 检查。

### 多渠道

- IM channel 出错的用户可收到短 support reference，例如 `diag:feishu:20260516:abc123`。
- 管理端按 channel/message id 生成窄范围 bundle：最近 channel heartbeat、outbound error、delivery log、runtime readiness verdict，不导出群聊完整内容。
- 群聊场景默认不包含原始群消息，只包含 hashed session identity、message count、error class 和 timestamps。

## 技术方案

### 1. Bundle profile 与 manifest

建议在 `hone-core` 或 `hone-web-api` 定义纯类型，CLI/Web/Desktop 复用：

```rust
pub enum SupportBundleProfile {
    LocalCli,
    DesktopBundled,
    DesktopRemote,
    ServerAdmin,
    PublicUserReport,
    TraceScoped,
}

pub struct SupportBundleManifest {
    pub schema_version: u32,
    pub bundle_id: String,
    pub created_at: String,
    pub profile: SupportBundleProfile,
    pub deployment_mode: String,
    pub time_window: SupportTimeWindow,
    pub redaction_version: String,
    pub sections: Vec<SupportBundleSection>,
    pub omitted: Vec<SupportBundleOmission>,
}
```

Bundle zip 建议结构：

```text
hone-support-bundle/
  manifest.json
  README.md
  environment/runtime-paths.json
  environment/status-report.json
  environment/doctor-report.json
  environment/meta.json
  environment/update-compatibility.json
  environment/runtime-readiness.json
  config/redacted-config-summary.json
  logs/backend.log
  logs/desktop.log
  logs/sidecar.log
  logs/channels/*.log
  channels/status.json
  tasks/task-health-summary.json
  notifications/delivery-summary.json
  llm/llm-audit-summary.json
  traces/trace-refs.json
  redaction/redaction-report.json
```

第一版可以允许部分文件缺失，manifest 必须写明 omission reason，例如 `backend_unreachable`、`desktop_remote_no_local_logs`、`not_admin`、`section_not_available_in_this_version`。

### 2. 统一 SupportRedactor

新增脱敏模块，优先复用并上移 `crates/hone-channels/src/mcp_bridge.rs` 里的 redaction 思路，而不是每个 collector 手写：

- JSON/YAML key redaction：`api_key`、`api_keys`、`token`、`access_token`、`refresh_token`、`app_secret`、`secret`、`password`、`cookie`、`authorization`、`bearer`、`session`。
- Text marker redaction：`Bearer ...`、`api_key=...`、`token: ...`、`Cookie: ...`。
- Contact redaction：手机号、邮箱、public invite code、API key prefix 只保留 hash 或 last4。
- Path redaction：把 `/Users/<name>/...`、`/home/<name>/...` 归一为 `<home>/...`；actor sandbox 内路径改成 sandbox-relative。
- Local file URI redaction：`file:///abs/path.png` 改成 `file://<redacted-local-artifact>` 或包内相对 label。
- Transcript redaction：默认不导出完整 user/assistant content；只导出 message counts、last status、error class、trace ids。Trace scoped bundle 也默认只导出摘要。

`redaction-report.json` 记录每类替换次数，帮助维护者判断包是否可能丢失太多上下文。

### 3. Collector 数据源

CLI collector：

- 直接调用现有 `build_status_report()` 和 `build_doctor_report()`。
- 读取 `config.yaml` 后只输出 redacted summary，不输出完整 YAML。
- 收集 runtime log tail，默认每文件最多 200 行，总大小上限。
- 如果 backend 可访问，拉取 `/api/meta`、`/api/channels`、`/api/logs`。

Web/server collector：

- 新增 admin API：
  - `POST /api/support-bundles`
  - `GET /api/support-bundles/:id/download`
  - `GET /api/support-bundles/:id/manifest`
- 生成文件落在 `data/runtime/support-bundles/`，短 TTL 自动清理。
- 收集 `/api/logs` 同源数据、task runs、notification summary、LLM audit aggregate、channel status、runtime paths label。

Desktop collector：

- 新增 Tauri command `export_support_bundle`。
- Bundled mode 从本机 app data/config/logs 读取，再可选请求 backend 追加 server sections。
- Remote mode 只请求远端 backend API，desktop 本地只附加 shell/backend selection summary。

### 4. Section 级权限

不同 profile 的默认包含范围不同：

- `PublicUserReport`：不含 logs、config、LLM audit、task details，只含 coarse backend status、support ref、browser/client info、account hash、timestamp。
- `ServerAdmin`：可含 redacted logs、doctor/status、task/notification/LLM audit aggregate，不含完整 prompt/user data。
- `TraceScoped`：只含指定 trace/task/channel 的时间窗口证据，默认不含完整 transcript。
- `DesktopBundled`：含 desktop/sidecar/backend logs 和 process status，但不含用户 portfolio/company profile docs。
- `LocalCli`：含 CLI reports、runtime paths、binary checks、local log tail。

所有导出都必须先通过 redactor；不提供 `--no-redact` 开关。若未来需要内部 full bundle，应放在本机手工诊断脚本中，不进入产品 UI。

### 5. API、CLI 与前端落点

CLI：

```shell
hone-cli support-bundle --profile local-cli --last 2h --output hone-support.zip
hone-cli support-bundle --profile desktop-bundled --output hone-desktop-support.zip
```

Web API：

- `POST /api/support-bundles` body: `{ profile, from, to, trace_id?, channel?, include_sections? }`
- `GET /api/support-bundles/:id/download`

Frontend：

- `packages/app/src/lib/support-bundle.ts`
- `packages/app/src/pages/settings.tsx` 增加 Support/Diagnostics 面板。
- `packages/app/src/pages/logs.tsx`、`task-health.tsx`、未来 `traces.tsx` 增加 scoped export action。

### 6. 大小、保留与安全边界

- 默认 zip 上限，例如 10 MB；超限时按 log tail、section priority 截断，并在 manifest 记录。
- 生成的 bundle TTL 默认 24 小时，download token 只对 admin/desktop session 有效。
- Public user report 不落长期文件或只保留极短 TTL。
- Bundle 不自动上传，不内置外部发送功能。
- `gitleaks` 或轻量 secret scanner 可作为生成后自检，命中则拒绝导出并写 redaction failure。

## 实施步骤

### Phase 1: CLI redacted local bundle

- 新增 `SupportRedactor`，覆盖 JSON/YAML/text/path/contact 基础脱敏。
- 新增 `hone-cli support-bundle`，复用 `build_status_report`、`build_doctor_report` 和 runtime log tail。
- 输出 zip、manifest、redaction report。
- 添加单元测试和 fixture，验证 API key、Bearer、bot token、手机号、邮箱、home path 被脱敏。

### Phase 2: Admin Web bundle

- 新增 `/api/support-bundles` admin route。
- 收集 backend meta、channels、logs、task-health summary、notifications summary、LLM audit aggregate。
- Settings/Dashboard 加导出入口和 manifest 预览。
- 生成文件放在 runtime support-bundles 目录，带 TTL 清理。

### Phase 3: Desktop bundled/remote integration

- 新增 Tauri `export_support_bundle` command，复用 `DiagnosticPaths`。
- Bundled mode 纳入 desktop/sidecar/backend/channel log tail。
- Remote mode 请求远端 backend 生成 server-side bundle，并只附加 desktop shell/remote URL 状态摘要。
- UI 明确区分本地与远端诊断范围。

### Phase 4: Scoped support and future verdict integration

- 接入 future `RuntimeReadinessMatrix`、`UpdateCompatibilityCenter`、`RunTraceWorkbench`、`InvestmentOutputSafetyGate` 的 verdict JSON。
- 支持 `trace_id`、`task_run_id`、`channel` scoped bundle。
- 从 error UI 一键生成相关窗口的 bundle。

## 验证方式

- 单元测试：
  - redactor 对 JSON/YAML/text 递归脱敏，不破坏合法 UTF-8。
  - `api_key`、`api_keys`、`token`、`Bearer`、`app_secret`、`password`、cookie、手机号、邮箱、home path、`file://` marker 都被处理。
  - manifest omissions 在 section 缺失时稳定输出。
- CLI 回归：
  - `hone-cli support-bundle --output <tmp>.zip` 在无 backend 运行时也能生成 local bundle。
  - zip 内必含 `manifest.json`、`status-report.json`、`doctor-report.json`、`redaction-report.json`。
  - 对含假 secret 的 fixture config/log 生成包后，`rg` 不应命中原始 secret。
- Web/API 测试：
  - 未授权不能生成 admin bundle。
  - public user report 不能包含 logs/config/LLM audit。
  - generated bundle TTL 清理不影响正常 runtime logs。
- Desktop 手工验收：
  - bundled mode 导出包包含 desktop log、sidecar log、backend meta、channel status。
  - remote mode 不误收集本机 CLI doctor 作为远端健康结论。
  - 生成包不包含真实 home 用户名、API key、bot token 或完整聊天内容。
- 支持流程指标：
  - 维护者定位安装/配置/channel 问题所需往返次数下降。
  - 用户手工贴原始 config/log 的次数下降。
  - support bundle redaction failure 数量可观测。

## 风险与取舍

- **风险：误把敏感数据打包。**
  取舍：默认强制 redaction，不提供 UI 层 no-redact；生成后跑轻量 secret scan，失败则拒绝导出。

- **风险：脱敏过度导致无法排障。**
  取舍：保留结构、布尔状态、hash、last4、error class、timestamps、counts、version、reason code。需要完整用户数据时走 User Data Trust Center 或本机人工诊断，不混进 support bundle。

- **风险：支持包变成新一套 telemetry。**
  取舍：只本地生成和下载，不自动上传；manifest 明确生成者、范围和文件列表。

- **风险：实现面太宽。**
  取舍：Phase 1 只做 CLI 本地包，Phase 2 再做 admin Web，Phase 3 再接 desktop。每个 section 可缺省并在 manifest 中声明。

- **风险：和 Runtime Readiness / Run Trace 重叠。**
  取舍：本提案不定义 readiness 或 trace 语义，只把它们作为可选 section 打包。Readiness 是 verdict，Trace 是单次运行对象，Support Bundle 是可分享诊断容器。

- **不做边界：**
  - 不导出公司画像正文、portfolio 明细、上传文件或完整聊天记录。
  - 不实现云端遥测或自动上传。
  - 不替代 `doctor/status`、Run Trace、User Data Trust Center 或 Update Compatibility。
  - 不提供自动修复，只提供诊断证据和下一步入口。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 全部自动提案和历史 `docs/proposals/`。

- 与 `auto_p1_runtime_readiness_matrix.md` 不重复：readiness 判断当前配置/能力是否可执行；本提案把 readiness 结果、CLI doctor、logs、channel status 等打成可分享的脱敏诊断包。
- 与 `auto_p1_run_trace_workbench.md` 不重复：run trace 聚焦一次 agent run 的时间线；本提案覆盖安装、配置、桌面、channel、日志和部署状态，并可把 trace refs 作为一个 section。
- 与 `auto_p1_update-compatibility-center.md` 不重复：update compatibility 判断版本/安装组合是否受支持；本提案只把版本 verdict 和安装摘要纳入支持包，不决定升级策略。
- 与 `auto_p1_user-data-trust-center.md` 不重复：Trust Center 给用户导出/删除个人数据；Support Bundle 默认排除个人投资数据和完整 transcript，只服务排障。
- 与 `auto_p0_operator-access-audit.md` 不重复：operator audit 记录谁访问/操作了管理面；本提案需要导出操作审计，但核心是诊断包格式和 redaction。
- 与 `auto_p1_delivery_decision_loop.md`、`auto_p1_temporal-operations-calendar.md` 不重复：它们解释通知为什么发/不发、何时运行；本提案只收集相关摘要帮助支持。
- 与 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 不重复：desktop startup UX 解决 bundled runtime ownership/takeover；本提案利用 desktop diagnostic paths 导出证据，不改变启动接管协议。
- 与 `docs/proposals/skill-runtime-multi-agent-alignment.md` 不重复：skill runtime 提案定义 skill 可见性和执行语义；本提案只在需要时记录当前 skill registry/runner capability 摘要。

差异结论：现有提案已经覆盖运行判断、单次 trace、版本治理、数据权利和通知解释，但还没有一个面向用户和维护者的“安全可分享诊断包”产品层。这个主题能把现有支持原料收束成稳定工件，是开源本地 AI agent 从能跑到可维护的重要基础。

## 文档同步说明

本轮只新增 proposal，不开始实施，不改变模块边界、入口、长期规则或运行流程，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。如果后续开始实施本提案，应按动态计划准入标准新增或复用 `docs/current-plans/redacted-support-bundle.md`，并在实际新增 CLI/API/Desktop 入口时同步更新 `docs/repo-map.md` 和相关 runbook。
