# Proposal: Storage Schema Migration Registry for Durable Local and Cloud Data

status: proposed
priority: P1
created_at: 2026-05-28 14:04:46 +0800
owner: automation

related_files:
- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/current-plans/cloud-pg-oss-runtime-migration.md`
- `memory/src/session_sqlite.rs`
- `memory/src/llm_audit.rs`
- `memory/src/cron_job/history.rs`
- `memory/src/web_auth.rs`
- `crates/hone-event-engine/src/store.rs`
- `crates/hone-core/src/config/materialize.rs`

## 背景与现状

Honeclaw 已经从本地单机助手扩展到 public Web、Hone Cloud API、多渠道 IM、桌面 bundled/remote runtime、event-engine 主动推送、公司画像和长期 session 记忆。数据层也随之从单一 JSON 文件变成多种本地和准云端状态：

- `memory/src/session.rs` 仍支持 JSON session 真相源，并通过 `memory/src/session_sqlite.rs` 建立 SQLite mirror / runtime backend。
- `memory/src/llm_audit.rs` 用 SQLite 记录 LLM 调用审计，并已有 `migrate_token_columns` 这类字段补齐逻辑。
- `memory/src/cron_job/history.rs` 把 cron 执行历史写入 SQLite，schema 通过 `CREATE TABLE IF NOT EXISTS` 初始化。
- `memory/src/web_auth.rs` 管理 invite users、hashed API key、public login sessions，并通过 `ensure_column` 兼容旧表。
- `crates/hone-event-engine/src/store.rs` 自带 events、engine_meta、delivery_log 表。
- 当前活跃计划 `Cloud PG / OSS Runtime Migration` 正在把 runtime 迁向 PG / OSS 所有权，但 session、quota、audit、portfolio、cron、notification prefs 等仍有本地 JSON / SQLite / directory fallback。

这些实现说明 Hone 已经进入“多 store、长生命周期、需要升级兼容”的阶段。但 schema 演进目前分散在各模块：有的只有 `CREATE TABLE IF NOT EXISTS`，有的临时 `ALTER TABLE ADD COLUMN`，session SQLite 有局部 `migration_runs`，event-engine 有 `engine_meta` 但没有统一 migration ledger。后续一旦引入 PG-backed stores、更多 public/desktop 版本共存、桌面自动升级和 rollback，这种分散迁移会成为核心可用性风险。

## 问题或机会

1. **无法回答当前数据版本是否健康。**  
   管理员可以看到 channels、logs、task health、LLM audit，但不能从一个地方确认 `sessions.sqlite3`、web auth DB、cron history DB、event store、未来 PG schema 是否处在预期版本，哪些迁移刚执行过，哪些失败后需要人工处理。

2. **升级失败难以安全恢复。**  
   当前迁移逻辑多为幂等建表或补列，适合小改动；但未来涉及数据重写、索引重建、JSON -> SQLite -> PG cutover、旧字段废弃时，需要 dry-run、事务边界、前置检查、失败记录和 rollback 指南。

3. **桌面、本地 CLI、hosted public 部署的升级窗口不同。**  
   桌面用户可能跨多个版本直接升级，本地开源用户可能保留旧 `data/runtime/`，hosted 部署则可能多实例访问同一 PG。没有统一 schema registry，代码只能在每个 store 里自行猜测旧状态。

4. **现有云迁移缺少可复用的 store cutover 证据。**  
   PG / OSS 迁移需要证明哪些本地 stores 已迁、哪些仍 fallback、哪些需要 dual-write。现在这些结论写在计划和 handoff 里，缺少 runtime 可查询对象。

5. **研发成本会线性上升。**  
   每新增一个 SQLite/PG-backed product object，都会重新实现建表、补列、版本探测、诊断输出和测试模式。后续 proposal 中的 response feedback、usage entitlement、artifact library、calibration ledger 等都会放大这个问题。

## 方案概述

新增 **Storage Schema Migration Registry**：一个面向本地 SQLite 和未来 PG store 的轻量迁移与诊断框架。它不替代每个业务 store 的读写 API，也不强制一次性迁移所有数据；第一阶段只把“schema 版本、迁移运行、健康状态、失败证据、回滚提示”统一起来。

核心对象：

- `StoreId`：稳定标识一个 store，例如 `session_runtime`, `llm_audit`, `cron_history`, `web_auth`, `event_engine`, `public_upload_index`, `pg_session_runtime`。
- `SchemaVersion`：单调递增整数或 `major.minor`，由代码声明当前目标版本。
- `MigrationStep`：从版本 A 到 B 的可执行步骤，包含前置检查、事务执行、后置校验、可重入策略和风险级别。
- `MigrationRun`：记录某次运行的 store、from/to version、started/completed time、status、error、code version、data path / database URL hash。
- `SchemaHealth`：面向 admin/doctor 的只读摘要，说明当前版本、目标版本、是否需要迁移、最近错误、建议动作。

第一版建议只支持 SQLite，并抽象出未来 PG adapter 所需接口；等 Cloud PG stores 落地时复用同一 registry。

## 用户体验变化

### 用户端

- 普通 public `/chat` 用户不直接看到 schema 细节。
- 如果后端因为安全迁移锁或 store 版本不兼容暂时不可用，应返回稳定、简短、非技术化错误，例如“系统正在升级数据结构，请稍后重试”，并带 request id。
- Public `/me` 或 `/portfolio` 不应泄漏本地路径、表名或迁移错误堆栈。

### 管理端

- 在 `Settings` 或 `Task Health` 增加 `Storage` / `Data Health` 区块：
  - 每个 store 的 current version、target version、last migration status。
  - 是否处于 fallback、dual-write、read-from-json、read-from-sqlite、read-from-pg。
  - 最近一次失败原因和建议命令，例如 `hone-cli doctor storage`、`hone-cli storage migrate --store session_runtime --dry-run`。
- 当 session SQLite mirror 落后、web auth 旧列未补齐、cron history DB 无法打开、event store 版本不匹配时，管理端应能显示“影响范围”，而不是只在日志里出现 SQLite error。

### 桌面端

- 桌面启动时将 schema health 纳入 preflight：可自动执行低风险幂等迁移；高风险迁移先显示 blocking 状态和升级说明。
- bundled mode 下 sidecar 启动失败时，桌面 shell 可区分“端口/进程冲突”和“数据 schema 迁移失败”。
- remote mode 只展示远端 `/api/meta` 暴露的 schema health，不尝试操作远端数据库。

### 多渠道与自动化

- Feishu / Telegram / Discord / iMessage listener 启动前读取同一 schema readiness 结果；如果核心 store 不可用，channel 应 fail fast 或降级到只读提示，而不是在收到消息时才失败。
- Cron / heartbeat 运行前检查 `cron_history` 和 `event_engine` schema 是否 ready；失败写入可诊断状态，避免 silent skip。
- 自动化和 agent worker 可以查询 store health，决定是否继续执行会写状态的任务。

## 技术方案

### 1. 新增轻量迁移框架

建议在 `memory` 或新的 shared crate 内新增模块，例如 `memory/src/schema_registry.rs`：

```rust
pub trait SchemaStore {
    fn store_id(&self) -> &'static str;
    fn current_version(&self) -> HoneResult<Option<i64>>;
    fn target_version(&self) -> i64;
    fn pending_steps(&self) -> HoneResult<Vec<MigrationStepDescriptor>>;
    fn migrate(&self, mode: MigrationMode) -> HoneResult<MigrationReport>;
    fn health(&self) -> HoneResult<SchemaHealth>;
}
```

第一阶段只提供 SQLite helper：

- `ensure_schema_meta(conn)`：创建统一 `schema_meta` 和 `schema_migration_runs`。
- `get_schema_version(conn, store_id)`。
- `run_sqlite_migrations(conn, store_id, steps, mode)`。
- `record_migration_run(...)`。
- `check_expected_columns/indexes(...)`。

对已有 store 不要求共享同一个 SQLite 文件；每个 DB 内保留自己的 `schema_meta` / `schema_migration_runs`，并由 API 聚合。

### 2. 从高风险 store 开始接入

接入顺序建议：

1. `memory/src/web_auth.rs`  
   Public login、invite list、API key 都依赖它。先把 `ensure_column` 变成声明式 migration step，并记录旧 plaintext session token compatibility 是否仍存在。

2. `memory/src/session_sqlite.rs`  
   现有 `migration_runs` 更像 JSON backfill run，不是 schema migration run。保留它作为 backfill ledger，另加 schema registry，避免两个概念混淆。

3. `memory/src/llm_audit.rs`  
   将 token columns migration 改成版本化 step，保留只读打开路径的 schema flags。

4. `memory/src/cron_job/history.rs` 与 `crates/hone-event-engine/src/store.rs`  
   为主动通知链路补 health，支撑 heartbeat、digest、direct push 的运维判断。

5. 未来 PG-backed stores  
   新 PG table 必须从一开始声明 `StoreId` 和 `SchemaVersion`，避免复制当前 SQLite 的分散模式。

### 3. API 与 CLI

新增只读诊断：

- `GET /api/storage/health`
- `GET /api/storage/migrations?store=...`
- `hone-cli doctor storage`

可选维护命令：

- `hone-cli storage migrate --store all --dry-run`
- `hone-cli storage migrate --store session_runtime`
- `hone-cli storage verify --store web_auth`

默认启动行为：

- 低风险、幂等、补列/建索引类迁移可自动执行。
- 数据重写、删除旧列、跨 backend cutover 必须要求显式命令或 release note 前置说明。
- hosted 多实例部署需要 DB-level advisory lock 或 single-run guard；SQLite 使用 process lock + transaction。

### 4. 配置与状态边界

- 不新增第二套业务配置。`config.yaml` 仍是 backend 选择、路径和 cloud env reference 的来源。
- `schema_meta` 只记录 store 内部 schema version，不记录 secrets、不记录完整 remote DB URL。
- `data/runtime/effective-config.yaml` 不作为迁移状态源；删除 `data/runtime/` 仍应是安全 runtime reset。
- 对本地 JSON truth source 的迁移，只记录 projection / mirror 状态，不改变 `storage.session_runtime_backend` 的读权威语义。

### 5. 测试策略

- 每个接入 store 至少有一个“旧 schema fixture -> 新 schema”的单元测试。
- 增加 CI-safe 回归脚本 `tests/regression/ci/test_storage_schema_migrations.sh`，构造临时数据目录并运行 `hone-cli doctor storage` 或直接运行 store-level tests。
- 对破坏性迁移必须有 dry-run 输出 golden assertion：会改哪些表、扫描多少行、失败时保留什么证据。
- 对 web auth/session store 增加 downgrade-aware fixture：旧数据仍能读，迁移后不泄漏 raw token/path。

## 实施步骤

### Phase 1: Registry Skeleton

- 新增 `SchemaStore` / `MigrationStep` / `SchemaHealth` 类型。
- 提供 SQLite `schema_meta` / `schema_migration_runs` helper。
- 给 registry helper 写纯单元测试：初始化、版本读取、重复运行、失败记录、dry-run。

### Phase 2: Web Auth and LLM Audit

- 将 `web_auth` 的 `ensure_column` 调整为版本化 steps。
- 将 `llm_audit` 的 token column migration 调整为版本化 steps。
- 保留现有行为和兼容测试，不改变 public login / audit 写入 API。

### Phase 3: Session and Cron/Event Stores

- 为 `session_sqlite` 新增 schema registry，区分 schema migration 与 JSON backfill `migration_runs`。
- 为 cron execution history 和 event-engine store 增加 version / health。
- 把 store health 暴露给 Web API，并让 `task-health` 或 settings 页面只读展示。

### Phase 4: CLI, Desktop Preflight, and Cloud Readiness

- 增加 `hone-cli doctor storage` 和 dry-run 命令。
- 桌面 bundled preflight 读取 storage health；失败时显示具体 store 和建议动作。
- Cloud PG-backed stores 开始落地时必须接入同一 registry，并在 release note 中列出 required migrations。

## 验证方式

- 单元测试：
  - SQLite registry helper 的版本递增、幂等、失败记录和 transaction rollback。
  - `web_auth` 从缺列旧表迁移后仍能登录、record TOS、hash session token。
  - `llm_audit` 从旧表补 token columns 后能写入 token usage。
  - `session_sqlite` schema registry 不破坏现有 JSON backfill ledger。
- 回归测试：
  - `tests/regression/ci/test_storage_schema_migrations.sh` 在临时目录创建旧 DB fixture，运行 health / migrate / verify。
  - `bash tests/regression/run_ci.sh` 纳入该脚本后保持无外部账号依赖。
- 手工验收：
  - 桌面 bundled mode 使用旧数据目录启动，自动迁移低风险 schema，UI 可显示 storage health。
  - hosted / remote backend 返回 `/api/storage/health`，public route 不泄漏内部错误。
- 指标：
  - 升级后 store migration failure rate。
  - doctor storage 中 `blocked` store 数量。
  - schema mismatch 导致的启动失败数量。
  - 从旧版本直接升级到最新版本的成功率。

## 风险与取舍

- **风险：框架化过早，增加维护成本。**  
  取舍：只抽象 schema 版本、运行记录和 health，不改业务 repository API；先接入已有高风险 stores。

- **风险：自动迁移在用户本地数据上造成不可逆改变。**  
  取舍：第一版自动执行只限建表、补列、建索引等低风险步骤；数据重写和 cutover 必须 dry-run + 显式命令。

- **风险：和现有 session JSON backfill `migration_runs` 混淆。**  
  取舍：明确命名为 `schema_migration_runs`；session 的 JSON import/backfill ledger 保留原语义。

- **风险：多实例 hosted PG 迁移并发。**  
  取舍：SQLite 阶段使用 process/transaction guard；PG 阶段必须引入 advisory lock 或部署侧 single migration job。

- **风险：管理端暴露过多内部细节。**  
  取舍：UI 展示 store id、状态和建议动作；错误详情保留在日志或 redacted support bundle，不暴露路径、token、完整 DB URL。

- **边界：不在本提案中实现 PG store。**  
  本提案只为 PG-backed stores 预留 adapter 契约，实际 session/audit/quota/portfolio 的 PG 实现仍属于 Cloud PG / OSS runtime migration 主线。

## 与已有提案的差异

查重范围：已检查 `docs/proposal/`、历史 `docs/proposals/`，并重点对比 storage、runtime、release、cloud、artifact、audit 相关 proposal。

- 不重复 `auto_p1_storage-budget-artifact-lifecycle.md`：该提案关注用户产物、上传、报告、支持包等 artifacts 的 retention / quota / cleanup；本提案关注数据库 schema 版本、迁移运行和 store health。
- 不重复 `auto_p1_update-compatibility-center.md`：该提案关注安装版本、Web bundle、sidecar、API 兼容窗口；本提案关注数据 store 内部 schema 是否可读、可迁、可诊断。
- 不重复 `auto_p1_product-rollout-kill-switch.md`：该提案决定功能是否对某些用户/入口开放；本提案决定底层 store schema 是否已满足该功能运行条件。
- 不重复 `auto_p1_local-backup-restore-vault.md`：该提案关注用户数据备份与恢复点；本提案关注升级过程中 schema 如何演进和记录。
- 不重复 `auto_p0_operator-access-audit.md` 或 `auto_p1_external-egress-ledger.md`：它们关注谁访问/外发了什么；本提案只记录 schema migration 的技术运行证据。
- 不重复当前活跃的 `Cloud PG / OSS Runtime Migration`：云迁移是把 store 所有权迁到 PG/OSS；本提案提供在本地 SQLite 和未来 PG 之间都能复用的 schema 迁移与健康诊断契约。

差异结论：当前仓库已有多个 SQLite store 和云迁移压力，但没有统一的 schema migration registry。这个 P1 提案填补的是“升级数据结构是否安全、可见、可恢复”的基础能力，直接支撑 public Web、桌面自动升级、多渠道自动化和未来 PG cutover。
