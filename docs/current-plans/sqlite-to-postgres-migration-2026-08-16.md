# SQLite 全量迁移到 PostgreSQL

状态：`planned`
创建：2026-08-16
执行方：Codex CLI（本文件即交接书，Codex 按阶段推进，每阶段独立可合并）
协调方：Claude（本会话产出计划、验收标准与阶段评审）

---

## 0. 结论先行

**这不是一次从零的存储迁移。** 仓库里 PG 侧已经有 25 张表、一套 `CloudPgRuntime`
（`crates/hone-core/src/cloud_runtime.rs`，6516 行）、以及一条已经能跑的
`hone-cli cloud migrate --from-data-dir` 导入通道。真正缺的只有两块：

| 缺口 | 体量 | 说明 |
|---|---|---|
| **event-engine 的 5 张表完全没有 PG 实现** | `crates/hone-event-engine/src/store.rs` 3547 行 | 本轮 80% 的工作量 |
| `web_user_external_state` 没有 PG 对应表 | `memory/src/web_auth.rs` 约 6 处引用 | 半天 |

其余 memory/ 模块（session / web_auth / billing / llm_audit / cron_job）**已经是
Sqlite / Cloud 双后端**，迁移动作是"删掉 Sqlite 分支"，不是"新写 PG 实现"。

**用户 2026-08-16 决策：不留任何兼容层，SQLite 逻辑全部删除**，包括
`CloudMode::{Local, Auto}` 与 22 个 `is_cloud_authoritative()` 分支（见 5.1）。
**本地 Mac 的存量 SQLite 数据也要一并迁进 PG**（见 5.5）。

依据是实测事实：GCE 生产 `HONE_CLOUD_MODE=cloud`、
`HONE_CLOUD_STRICT_NO_LOCAL_STORAGE=true`，且 `/srv/honeclaw/data/` 下
**零个 sqlite3 文件** —— 客户侧早就是纯 PG，双后端只服务于本地开发，
本地换 Docker PG 之后它就失去了全部存在理由。

---

## 1. 范围

### 1.1 在范围内

honeclaw 自己拥有的全部 SQLite 数据：

| 模块 | SQLite 表 | PG 对应 | 状态 |
|---|---|---|---|
| `memory/src/session_sqlite.rs` (832) | `sessions` / `session_messages` / `session_metadata` / `migration_runs` | `cloud_sessions` | ✅ 已有（整会话 JSONB 粒度） |
| `memory/src/web_auth.rs` (3461) | `web_invite_users` | `cloud_web_invite_users` | ✅ 已有 |
| | `web_auth_sessions` | `cloud_web_auth_sessions` | ✅ 已有 |
| | `web_admin_actions` | `cloud_web_admin_actions` | ✅ 已有 |
| | `web_user_external_state` | — | ❌ **缺** |
| `memory/src/billing.rs` (1627) | `billing_entitlements` / `billing_webhook_events` | 同名表 | ✅ 已有 |
| `memory/src/llm_audit.rs` (924) | `llm_audit_records` | `cloud_llm_audit_records` | ✅ 已有 |
| `memory/src/cron_job/history.rs` (1360) | `cron_job_runs` / `web_push_messages` | `cloud_cron_job_runs` / `cloud_web_push_messages` | ✅ 已有 |
| **`crates/hone-event-engine/src/store.rs` (3547)** | `events` / `engine_meta` / `delivery_log` / `delivered_push_context` / `earnings_continuity_jobs` | — | ❌ **全缺** |

### 1.2 明确不在范围内

- **`bins/hone-imessage/src/main.rs`** —— 它以 `SQLITE_OPEN_READ_ONLY` 打开的是
  **macOS 自己的 `~/Library/Messages/chat.db`**，不是 honeclaw 的数据。这个
  `rusqlite` 依赖必须保留，一行都不要动。任何"全仓移除 rusqlite"的动作都要
  显式排除 `bins/hone-imessage`。
- JSON 明盘目录（`portfolio_dir` / `notif_prefs_dir` / `conversation_quota_dir` /
  `sessions_dir` / `skill_registry.json`）。它们不是 SQLite，且 PG 侧表已存在、
  `cloud migrate` 已覆盖，属于下一轮"关掉本地 JSON 回退"的题目。
- `data/events.jsonl`（278 MB 的人肉兜底镜像）。它是 SQLite 故障时的兜底，
  PG 化之后是否保留另议，本轮**保持原样**。

---

## 2. 阶段 0：本地 Docker PostgreSQL

这是后面每一个阶段的前提，必须第一个做完并单独提交。

**交付物**

1. `docker-compose.dev.yml` —— `postgres:16-alpine`，命名卷持久化，端口 `5433`
   （避开可能已占用的 5432），健康检查 `pg_isready`。
2. `scripts/dev_pg.sh` —— `up` / `down` / `reset` / `psql` 四个子命令。
   `reset` 必须二次确认，不允许无提示 drop。
3. `.env.example` 增加本地连接串示例；`docs/runbooks/` 增加一页本地 PG 启停说明。
4. 打通 `cargo run -p hone-cli -- cloud doctor --ensure-schema --json`，
   在空库上一次建出全部 schema。

**验收**：`docker compose -f docker-compose.dev.yml up -d` 后，
`hone-cli cloud doctor --ensure-schema` 返回 `pg.ok=true`，且 `\dt` 能看到 25 张表。

**注意**：`PostgresConfig`（`crates/hone-core/src/config/server.rs:679`）已经支持
`database_url` 或 `host/port/user/password/database` 两种形式，且每个字段都有
对应的 `*_env` 覆盖。**不要新增配置字段**，用现成的环境变量走通即可。

---

## 3. 阶段 1：补齐 `web_user_external_state`

小、独立、可先合，用来把阶段 0 的本地 PG 链路真正跑热。

- 在 `ensure_schema()` 增 `cloud_web_user_external_state`，字段对齐
  `memory/src/web_auth.rs:235` 的 SQLite 定义，`email_address` 建索引
  （SQLite 侧 `:245` 有）。
- 在 `CloudPgRuntime` 补对应的读写方法，参考 `:413` / `:442` / `:480` / `:1126`
  四个 SQLite 调用点，以及 `:2088` 的 `LEFT JOIN`。
- `WebAuthStorage::Cloud` 分支接上。

**验收**：`memory` crate 现有 web_auth 测试在 Cloud 后端下全绿（见第 7 节的双后端测试要求）。

---

## 4. 阶段 2：event-engine store → PG（本轮主体）

### 4.1 现状

`EventStore::open`（`store.rs:105`）打开一个 `Connection`，包在 `Mutex` 里，
`busy_timeout = 5s`（因为 event-engine 与 channel runtime 会各自打开同一个文件）。
全部方法是**同步**的，被 `engine.rs:276` / `spawner.rs` / router 大量同步调用。

本地 `data/events.sqlite3` 已 **154 MB**。

### 4.2 五张表的迁移要点

```
events                    id TEXT PK, kind_json, severity, symbols_json,
                          occurred_at_ts INTEGER, title, summary, url,
                          source, payload_json, created_at_ts INTEGER
engine_meta               key TEXT PK, value TEXT
delivery_log              id INTEGER PK AUTOINCREMENT, event_id, actor,
                          channel, severity, sent_at_ts, status, body
delivered_push_context    delivery_log_id INTEGER PK, actor, source_id,
                          delivered_at_ms, body, observed_native_session_id,
                          claimed_turn_id, claim_expires_at_ms,
                          consumed_turn_id, consumed_at_ms,
                          UNIQUE(actor, source_id)
earnings_continuity_jobs  job_key TEXT PK, actor_json, event_json, status,
                          attempts, next_attempt_ts, lease_until_ts,
                          last_error, created_at_ts, updated_at_ts
```

必须逐条处理的方言差异：

1. **`INTEGER PRIMARY KEY AUTOINCREMENT` → `BIGSERIAL PRIMARY KEY`**。
   `delivered_push_context.delivery_log_id` 引用 `delivery_log.id`，迁移历史数据时
   必须保住这个对应关系 —— 先导 `delivery_log` 并保留原 id（用
   `OVERRIDING SYSTEM VALUE` 或显式 id + 迁移后 `setval`），再导
   `delivered_push_context`。**不要让 PG 重新分配 id**。
2. **`INSERT OR IGNORE` → `INSERT ... ON CONFLICT DO NOTHING`**。
   `insert_event` 的去重语义（`source.rs:44` 明确依赖它）不能变。
3. **时间戳保持 `INTEGER` epoch → `BIGINT`**，不要顺手改成 `TIMESTAMPTZ`。
   现有代码到处在做 epoch 算术（`store.rs:690` 的 `IntegralValueOutOfRange` 转换等），
   改类型会引入一次没人要的时区语义变更。
4. **`GREATEST(0, NULL)` 陷阱**：PG 的 `GREATEST` **忽略 NULL 返回 0**，
   SQLite 的标量 `MAX(0, NULL)` 返回 **NULL**，两者语义相反。
   2026-08-15 已经在 `cloud_cron_job_runs.duration_ms` 上踩过一次，静默写出
   "0 毫秒"。凡涉及可空列的聚合/算术，一律显式写 `CASE WHEN x IS NULL THEN NULL ELSE ... END`。
5. `busy_timeout` 的语义在 PG 下由连接池 + 事务隔离替代，**不要**尝试翻译它。
6. `earnings_continuity_jobs` 的 `lease_until_ts` 是租约语义，PG 下应改用
   `SELECT ... FOR UPDATE SKIP LOCKED` 领取，比现在的"读-判-写"更安全。
   这是本轮唯一允许的行为增强，且必须单独一个 commit。

### 4.3 同步 API 桥接（必读，这里有刚踩过的坑）

`EventStore` 的所有方法是同步的，`CloudPgRuntime` 全是 `async`。
**不要每次调用新建 `tokio::runtime::Runtime`。**

2026-08-15 的 `62d0c889` 刚修完这个问题：旧代码在 tokio 上下文里
`std::thread::spawn(...).join()`，内部再 Builder 一个完整 Runtime，
生产实测进程 26 分钟烧掉 47 CPU 分钟。

正确做法**已经写好了**，直接复用 `memory/src/cron_job/mod.rs` 的 `run_cloud_cron`：

- 一个 `LazyLock` 的长驻多线程 runtime（2 worker，命名线程）；
- 不在 tokio 上下文里 → 直接 `runtime.block_on`；
- 在 tokio 上下文里 → `runtime.spawn` + `sync_channel` + `recv_timeout`
  （`Handle::current().block_on` 在 runtime 线程上会 panic，不能用）；
- schema 用 `AtomicBool` 保证进程内只 `ensure_schema` 一次
  —— `ensure_schema` 是 430 行 DDL，绝不能放在热路径。

把这段抽成 `hone-core` 的共享工具再给两边用，比复制粘贴好，但**不要为此重构
`CronJobStorage` 的现有行为**。

### 4.4 连接复用

`connect_cached_client`（`cloud_runtime.rs`）已经带存活检查与死连接驱逐。
event-engine 侧一律用它，不要用 `connect_client`。
注意：`connect_new_client` 会把连接驱动 spawn 到**当前** runtime 上，
临时 runtime 一销毁驱动就死 —— 这正是长驻 runtime 的另一个理由。

### 4.5 迁移历史数据

- 给 `hone-cli cloud migrate` 增 `--event-store-only`，从
  `<data_dir>/events.sqlite3` 分批（建议 5000 行/批）导入。
- 默认 dry-run，`--apply` 才写，与现有 migrate 的行为一致。
- 报告里输出每张表的 `changed` / `skipped` 行数，供对账。

---

## 5. 阶段 3：删除 memory/ 的 SQLite 分支

涉及的后端枚举：

- `memory/src/session.rs:99` `SessionRuntimeBackend { Json, Sqlite, CloudPg }`
- `memory/src/web_auth.rs:145` `WebAuthBackend { Sqlite, Cloud }`
- `memory/src/billing.rs:114` `BillingBackend { Sqlite, Cloud }`
- `memory/src/llm_audit.rs` / `memory/src/cron_job/` 的 `Option<CloudPgRuntime>` 分叉

`memory/src/session_sqlite.rs`（832 行）整文件删除。

### 5.1 `CloudMode::Local` 一并删除（用户 2026-08-16 决策：不留兼容）

**PG 成为唯一存储，不保留任何 SQLite 回退。**

事实依据（已实测）：GCE 生产 `/etc/hone/runtime.env` 是
`HONE_CLOUD_MODE=cloud` + `HONE_CLOUD_STRICT_NO_LOCAL_STORAGE=true` +
`HONE_CLOUD_KEEP_SESSION_SQLITE_SHADOW=false`，且 `/srv/honeclaw/data/` 下
**一个 sqlite3 文件都没有**。生产早就是纯 PG，双后端只服务于本地开发。
本地开发改用 Docker PG 之后，双后端就没有任何存在理由。

要删的：

- `CloudMode::{Local, Auto}` 与 `is_cloud_authoritative()` 的 **22 个调用点**
  （web-api 路由 7 处、channels/bot_core 6 处、response_finalizer 2 处、
  attachments 2 处、agent_session、company_ratings、web_users、users、
  portfolio、company_profiles、meta），全部改成无分支直连 PG。
- `CloudConfig::effective_mode` / `effective_enabled` 及其配置校验分支。
- `keep_cloud_session_sqlite_shadow()` 与 `session_sqlite_shadow_write_enabled`
  整条影子库通路。
- `local_durable_dependencies()`：PG 化后只剩对象存储那一项，按实际情况收缩，
  不要留下会给出假"无本地依赖"的死代码。
- `effective_strict_no_local_storage()`：若删完之后恒真，就连同配置项一起删掉，
  不留骗人的开关。

顺序要求：**这一步必须在阶段 2（event-engine 迁移）完成并验收之后**。
先删后端会让 event-engine 无处可退。

---

## 5.5 阶段 3.5：本地 Mac 存量数据迁移（用户 2026-08-16 要求）

本地 Mac 是用户自己的实例（Discord + event engine），数据必须一起搬进 Docker PG，
不能只搬 GCE。当前存量：

| 文件 | 大小 | 去向 |
|---|---|---|
| `data/events.sqlite3` | **154 MB** | 阶段 2 新建的 5 张 PG 表 |
| `data/sessions.sqlite3`（+ 7.1 MB WAL） | 3.9 MB | `cloud_sessions` |
| `data/llm_audit.sqlite3`（+ 427 KB WAL） | 340 KB | `cloud_llm_audit_records` |

执行要求：

1. **迁移前先 `sqlite3 <db> "PRAGMA wal_checkpoint(TRUNCATE);"`**。
   三个库都有未合并的 WAL（sessions 的 WAL 比主库还大），直接读主文件会漏数据。
2. 迁移前把三个文件连同 WAL 一起冷备份到 `data/backups/pre-pg-migration/`，
   迁完并对账通过之前不许删。
3. 全部走 `hone-cli cloud migrate --from-data-dir ./data`，先 dry-run 出报告，
   人工过一遍再 `--apply`。
4. 逐表 count 对账，写进交接文档。`events` 表还要额外抽样比对 `payload_json`
   的内容哈希，确认没有编码/转义损坏。
5. `data/events.jsonl`（278 MB）本轮不动，正好留作这次迁移的独立比对基准。

---

## 6. 阶段 4：清理

- workspace `Cargo.toml:73` 的 `rusqlite`：从 `crates/hone-event-engine`、`memory`、
  `bins/hone-cli` 移除依赖；**`bins/hone-imessage` 保留**（见 1.2）。
  `bins/hone-cli` 需要确认 `cloud migrate` 的 SQLite 读取端是否仍需要它
  —— 如果迁移工具要长期支持从旧 SQLite 导入，那 hone-cli 也保留。
- 更新 `local_durable_dependencies`。
- 更新 `docs/conventions/`、`docs/runbooks/backend-deployment.md`。
- `docs/session-sqlite-migration-plan.md`（2026-03-25 的旧提案）移入
  `docs/archive/plans/`，并在本文件注明它被取代。

---

## 7. 验收

### 7.1 每阶段共同门禁

```bash
cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
bash tests/regression/run_ci.sh
```

`run_ci.sh` 在非交互 shell 下会因为 `rg` 不存在而失败（本机 `rg` 只是 shell 函数）。
先在 PATH 放一个 shim：`exec -a rg "$CLAUDE_CODE_EXECPATH"`。

### 7.2 双后端测试要求

memory/ 现有测试大量直接建 SQLite 临时库。删掉 Sqlite 后端后，这些测试必须
改成打本地 Docker PG，**不允许简单删除测试来让编译通过**。
按 `AGENTS.md` 第 7 条，每个 bugfix 至少一个回归测试，纯重构不得削弱已有覆盖面。

若 CI runner 上没有 PG，用 `#[ignore]` + 本地/手动跑，并在
`tests/regression/manual/` 留一条真实连通性验证 —— 这是仓库既有惯例。

### 7.3 event-engine 迁移专项验收

1. 把本地 154 MB `data/events.sqlite3` 完整导入本地 Docker PG，逐表对账行数。
2. 跑既有的两周事件重放（`replay_push_quality_audit` ignored 测试），
   PG 后端下的推送结果与 SQLite 后端**逐条一致**。这是唯一能证明去重、
   delivery_log 关联和 push context 认领语义没被改坏的证据。
3. `delivered_push_context.delivery_log_id` 与 `delivery_log.id` 的对应关系
   在迁移前后完全一致（抽样 + 全量 count 双查）。

### 7.4 上线

按仓库既有顺序：实现 → 自定义验收 → 本地源码部署（`scripts/deploy_source_runtime.sh`）
→ 验证 runtime → `git push` → CI 出 runtime image → GCE。

**GCE 侧特别注意**：客户库是活的，`cloud migrate` 必须先 dry-run 出报告、
人工过一遍再 `--apply`。event-engine 在 GCE 当前是 `role=web`，**没有运行**，
所以那边没有 events.sqlite3 要迁 —— 这降低了生产迁移的风险，但也意味着
event-engine 的 PG 路径在 GCE 上没有历史数据可对账，只能靠本地验收。

---

## 8. 风险

| 风险 | 缓解 |
|---|---|
| `delivery_log` 自增 id 在迁移中被重排，`delivered_push_context` 关联错乱 | 显式保留原 id + 迁移后 `setval`；全量 count 对账（7.3.3） |
| 同步桥接再次引入 per-call Runtime，CPU 打满 | 计划 4.3 已写死复用 `run_cloud_cron` 模式；评审时逐行确认 |
| 可空列上的 `GREATEST` / 聚合语义翻转，静默写错数 | 计划 4.2.4；review 时全文搜 `GREATEST` / `COALESCE` |
| 删 SQLite 后端时顺手删测试 | 7.2 明令禁止；review 对比测试数量 |
| 误删 `hone-imessage` 的 rusqlite | 1.2 + 6 两处标注 |
| 本地 PG 与生产 PG 版本不一致导致方言问题 | Docker 镜像固定 `postgres:16`，与生产大版本对齐（部署前确认生产版本） |

---

## 9. 给 Codex 的工作约定

- 每个阶段一个 commit，`fix:` / `feat:` / `refactor:` 前缀，结尾带
  `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`。
- 提交前工作树必须干净；`docs/handoffs/2026-08-09-stripe-webhook-triage-and-log-hygiene.md`
  是用户自己的未提交文件，**不要提交、不要修改**。
- 每阶段完成后停下来等评审，不要连着往下做。
- 遇到与本计划冲突的事实（比如某张表其实已有 PG 实现、或某个调用点比计划里
  写的多），**以代码为准，回来改计划**，不要为了贴合计划而写多余代码。
- 阶段 2 是主体，不要先做阶段 3 —— 先删 SQLite 后端会让 event-engine 无处可退。
