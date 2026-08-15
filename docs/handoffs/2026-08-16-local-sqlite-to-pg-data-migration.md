# 本地 Mac SQLite / JSON 存量数据迁入 PostgreSQL

- title: 本地 Mac SQLite / JSON 存量数据迁入 PostgreSQL
- status: `done`
- created_at: `2026-08-16`
- updated_at: `2026-08-16`
- owner: Codex CLI
- related_files:
  - `bins/hone-cli/src/cloud.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `memory/src/web_auth.rs`
  - `memory/src/llm_audit.rs`
- related_docs:
  - `docs/current-plans/sqlite-to-postgres-migration-2026-08-16.md` §0 / §5.5（本 detached worktree 的 `91227a9f` 尚无该文件，本次从主仓库只读）
- related_prs: 无

## Summary

已用 detached HEAD `91227a9f601be5c3606acacf8c77698af56a5109` 的 `hone-cli cloud migrate`，把当前命令已经支持的本地数据域迁入 `HONE_POSTGRES_DATABASE=honeclaw_p3`。

- dry-run 成功，源布局与报告计数人工核对一致。
- 首次 `--apply` 成功，所有受支持的结构化数据源与 PostgreSQL 逐表计数一致。
- `llm_audit` 明确从 684,032 bytes 的冷备份读取，冷备份 7 行、PG 7 行，ID 集合差集为 0；没有读取 348,160 bytes 的活库主文件作为迁移源。
- event-engine 的 5 张表按任务边界未迁移；冷备份中的实际行数是 `76147 / 1 / 83928 / 154 / 0`。
- 另外发现 `cron_job_runs` 有 17 行，而当前 `cloud migrate` 没有导入通道；`session_messages` / `session_metadata` 是已被完整 Session JSON 覆盖的关系型镜像，其余专用业务表的无通道源行数为 0，详见下文。
- 未配置 OSS。225 条 `cloud_documents` 已写入，但其 `oss_uri` 是指向临时快照的 `local:///tmp/...` URI；临时快照暂未删除。
- 冷备份目录始终只读使用。迁移后三个冷备份 SHA-256 不变，目录中没有 `-wal` / `-shm` / `-journal` sidecar。

## What Changed

本次没有修改任何 `.rs` 源码，也没有修改主仓库的活 `data/` 或冷备份。唯一仓库变更是本报告；目标数据库发生了预期的 schema 初始化和数据写入。

### 1. `cloud migrate` 的真实能力

代码依据：

- flag 定义：`bins/hone-cli/src/cloud.rs:70-111`
- 报告结构：`bins/hone-cli/src/cloud.rs:215-249`
- 主流程：`bins/hone-cli/src/cloud.rs:2743-3118`
- 结构化源解析：`bins/hone-cli/src/cloud.rs:3120-3423`
- 文件扫描与分类：`bins/hone-cli/src/cloud.rs:3568-3680`
- 对象 / 文档索引：`bins/hone-cli/src/cloud.rs:3450-3556`
- Web auth SQLite 导出：`memory/src/web_auth.rs:2071-2172`
- LLM audit SQLite 分页导出：`memory/src/llm_audit.rs:496-534`

| 数据域 | 实际源 | PostgreSQL 去向 | 关键限制 |
|---|---|---|---|
| 普通 sessions | `<from-data-dir>/sessions/**/*.json` | `cloud_sessions` | **不读** `sessions.sqlite3.sessions`；每个可解析 JSON 一行 |
| Conversation quota | `<from-data-dir>/conversation_quota/**/*.json` | `conversation_quota` | actor key 来自目录名 |
| Web invite users | 配置中的 `storage.session_sqlite_db_path` 的 `web_invite_users`，并 LEFT JOIN `web_user_external_state` | `cloud_web_invite_users` | 源路径**不受** `--from-data-dir` 控制；external state 嵌入 `record` JSONB |
| Web auth sessions | 配置中的 `storage.session_sqlite_db_path` 的 `web_auth_sessions` | `cloud_web_auth_sessions` | 同上 |
| Cron jobs | `<from-data-dir>/cron_jobs/*.json` 的每个 `jobs[]` | `cloud_cron_jobs` | 不导入 `cron_job_runs` 历史 |
| Skill registry | `<from-data-dir>/runtime/skill_registry.json` | `cloud_skill_registry` 的 `global` 行 | 文件不存在时为 0 行 |
| Notification prefs | `<from-data-dir>/notif_prefs/*.json` | `cloud_notification_prefs` | 每个可解析 JSON 一行 |
| Portfolio | `<from-data-dir>/portfolio/*.json` | `cloud_portfolios` | 每个可解析 JSON 一行 |
| Actor-scoped company profiles | `.../<channel>/<scoped-user>/company_profiles/<profile-id>/**/*.md` | `cloud_company_profile_files` | 只有符合 actor 目录结构的 Markdown 会结构化导入；根级 `company_profiles/*.json` 不会进入此表 |
| LLM audit | `<from-data-dir>/llm_audit.sqlite3` 的 `llm_audit_records` | `cloud_llm_audit_records` | `new_readonly_local` 打开，500 行分页；本次源为冷备份副本 |
| 文件 / 对象索引 | sessions、uploads、gen_images、company-profile JSON/Markdown、cron、quota 候选 | `cloud_documents`，可选 OSS | `--upload-oss` 未启用时仅写 `local://` URI、hash、size、metadata，不把 bytes 放进 PG |

所有 `.sqlite` / `.sqlite3` / `.db` 都会被候选扫描器计入 `sqlite_files`，但对象阶段固定给出 `sqlite structured import pending, skipped blob upload`。这不等于所有 SQLite 都没导入：`llm_audit.sqlite3` 已在对象阶段之前走专用结构化导入；`sessions.sqlite3` 只由 Web auth 专用路径读取；`events.sqlite3` 没有通道。

#### 全部 CLI flags 与默认行为

真实 help：

```text
$ /tmp/hone-pg35-target-91227a9f/debug/hone-cli cloud migrate --help
从本机 data/ dry-run 或幂等导入 PG/OSS。

Usage: hone-cli cloud migrate [OPTIONS] --from-data-dir <DIR>

Options:
      --config <CONFIG>
      --from-data-dir <DIR>
      --upload-oss
      --reuse-existing             Reuse existing OSS objects after a HEAD check instead of blindly overwriting
      --concurrency <CONCURRENCY>  Number of concurrent object uploads. Applies only with --upload-oss --apply [default: 6]
      --quota-only                 Only import conversation quota JSON into PG; skip object uploads and document indexing
      --session-only               Only import session JSON into PG; skip object uploads and document indexing
      --web-auth-only              Only import Web invite users and auth sessions from the configured SQLite DB into PG
      --cron-only                  Only import cron job JSON into PG
      --skill-registry-only        Only import runtime skill registry JSON into PG
      --notification-prefs-only    Only import notification preferences JSON into PG
      --portfolio-only             Only import portfolio JSON into PG
      --llm-audit-only             Only import LLM audit SQLite rows into PG
      --company-profiles-only      Only import actor-scoped company profile markdown files into PG
      --apply
      --json
  -h, --help                       Print help
```

- `--apply` 是普通 bool flag，未传即 `false`；所以默认是 dry-run。
- 9 个 `--*-only` 最多只能启用一个。
- `--upload-oss` 默认关闭；`--reuse-existing` 只在对象上传时做 HEAD 存在性检查。
- `--concurrency` 默认 6，运行时最小钳制为 1。
- dry-run 只扫描并分类文件，不连接 PG、不读取 SQLite 行、不计算 would-change。因此 dry-run 报告里的所有 `changed_*` / `skipped_*` 为 0 是实现语义，不表示源表为空。

#### 报告字段

顶层字段：

```text
mode, from_data_dir, upload_oss, reuse_existing, concurrency,
postgres_configured, oss_configured, counted,
uploaded_objects, reused_objects, indexed_documents,
changed_quota_rows, skipped_quota_rows,
changed_session_rows, skipped_session_rows,
changed_web_auth_users, skipped_web_auth_users,
changed_web_auth_sessions, skipped_web_auth_sessions,
changed_cron_rows, skipped_cron_rows,
changed_skill_registry_rows, skipped_skill_registry_rows,
changed_notification_prefs_rows, skipped_notification_prefs_rows,
changed_portfolio_rows, skipped_portfolio_rows,
changed_company_profile_files, skipped_company_profile_files,
changed_llm_audit_rows, skipped_llm_audit_rows,
skipped_objects, conflicts
```

`counted` 子字段：

```text
sessions, uploads_and_attachments, generated_images, company_profiles,
portfolio_json, cron_json, notification_prefs, quota_json,
skill_registry_json, sqlite_files, other_files
```

### 2. 幂等性结论

结论：**结构化 PG 行不会重复，可以安全重跑；但全命令不是严格“零写入 no-op”。**

代码证据：

- `cloud_sessions` 以 `session_id` 冲突，只有 actor/content 不同时 UPDATE；见 `crates/hone-core/src/cloud_runtime.rs:2684-2710`。
- Web user/auth 分别以 `user_id` / `session_hash` 冲突，并用 `IS DISTINCT FROM` 限制 UPDATE；见 `:3549-3613`。
- Cron、skill registry、notification prefs、portfolio、company profiles、LLM audit 都是唯一键 `ON CONFLICT ... DO UPDATE ... WHERE ... IS DISTINCT FROM`；见 `:3740-3764`、`:4419-4438`、`:4528-4550`、`:4652-4676`、`:4877-4918`、`:5064-5090`。
- `cloud_documents` 同样以 `(actor_storage_key, kind, document_id)` upsert，不会增加重复行；但其冲突分支没有 `WHERE`，每次都会刷新 `updated_at`，见 `:5142-5151`。
- 若启用 OSS，未加 `--reuse-existing` 时会再次 PUT 覆盖同 key；加了 `--reuse-existing` 则只做 HEAD 存在性判断，并不校验远端 hash。

实际同参数第二次 `--apply` 的结构化计数：

```text
changed_quota_rows=0 skipped_quota_rows=1
changed_session_rows=0 skipped_session_rows=22
changed_web_auth_users=0 skipped_web_auth_users=1
changed_web_auth_sessions=0 skipped_web_auth_sessions=1
changed_cron_rows=0 skipped_cron_rows=1
changed_skill_registry_rows=0 skipped_skill_registry_rows=0
changed_notification_prefs_rows=0 skipped_notification_prefs_rows=4
changed_portfolio_rows=0 skipped_portfolio_rows=2
changed_company_profile_files=0 skipped_company_profile_files=143
changed_llm_audit_rows=0 skipped_llm_audit_rows=7
indexed_documents=225
```

第二次报告的 `other_files` 从 1 变为 3，是因为第一次 LLM audit 读取临时 `llm_audit.sqlite3` 副本后在 `/tmp` 生成了 `llm_audit.sqlite3-wal` / `llm_audit.sqlite3-shm` sidecar；冷备份目录仍然没有 sidecar，冷备份 hash 不变。

### 3. 冷备份布局不满足 `--from-data-dir` 的完整预期

原始冷备份只有三个 SQLite：

```text
$ find /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration -maxdepth 1 -type f -print -exec stat -f '%z bytes %N' {} \;
/Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/events.sqlite3
161390592 bytes /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/events.sqlite3
/Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/sessions.sqlite3
4087808 bytes /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/sessions.sqlite3
/Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/llm_audit.sqlite3
684032 bytes /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/llm_audit.sqlite3
```

但命令还需要 JSON/Markdown/上传文件的完整相对布局，而且 Web auth 不看 `--from-data-dir`。为避免触碰活 SQLite，本次建立一次性快照：

```text
STAGE_ROOT=/tmp/hone-pg35-91227a9f.44BhUL
STAGED_FILES=235
STAGED_BYTES=172515363
sessions=22
conversation_quota=1
cron_jobs=1
notif_prefs=5
portfolio=2
company_profiles=161
gen_images=0
uploads=40
skill_registry=0
```

做法：

1. 用 `rsync -a --prune-empty-dirs` 只读复制 `sessions/`、`conversation_quota/`、`cron_jobs/`、`notif_prefs/`、`portfolio/`、`company_profiles/`、`gen_images/`、所有 `uploads/`、actor sandbox 下的 `company_profiles/` 与 `runtime/skill_registry.json`。
2. 用 `cp -p` 把冷备份的三个 SQLite 放到快照根。没有从活库复制 SQLite。
3. 因 `WalkDir::follow_links(false)`，没有用目录符号链接。
4. 运行时设置 `HONE_DATA_DIR=/tmp/hone-pg35-91227a9f.44BhUL/data`，使配置解析后的 `storage.session_sqlite_db_path` 也指向冷备份副本，从而避免 Web auth 路径回到活库。
5. 配置文件与 `soul.md` 也只复制到 `/tmp`；没有修改真实配置。

冷备份与初始快照副本 hash 一致：

```text
f594588e3345bd6225590c84e266253de1f0c2ed0be6ada4cc5e79eefebfa89c  /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/events.sqlite3
f594588e3345bd6225590c84e266253de1f0c2ed0be6ada4cc5e79eefebfa89c  /tmp/hone-pg35-91227a9f.44BhUL/data/events.sqlite3
0ee2afa5ecfc84dfb97de5a023105944d633b2be84b36b1a7894bf46b4904933  /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/sessions.sqlite3
0ee2afa5ecfc84dfb97de5a023105944d633b2be84b36b1a7894bf46b4904933  /tmp/hone-pg35-91227a9f.44BhUL/data/sessions.sqlite3
a8d0c04e77b4957f81f43f5db34fea11eac3a16d3cc114b572dfedeebfe7ee29  /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/llm_audit.sqlite3
a8d0c04e77b4957f81f43f5db34fea11eac3a16d3cc114b572dfedeebfe7ee29  /tmp/hone-pg35-91227a9f.44BhUL/data/llm_audit.sqlite3
```

### 4. 目标库确认

宿主环境端口是 5433，Docker 映射和数据库身份：

```text
$ echo "HONE_POSTGRES_PORT=$HONE_POSTGRES_PORT"
HONE_POSTGRES_PORT=5433

$ docker ps --format '{{.Image}}|{{.Ports}}|{{.Names}}' | rg '5433|postgres'
postgres:16-alpine|127.0.0.1:5433->5432/tcp|honeclaw-pg-postgres-1

$ psql ... -c 'SELECT current_database() AS target_database, current_user AS target_user, inet_server_port() AS container_port;'
 target_database | target_user | container_port
-----------------+-------------+----------------
 honeclaw_p3     | honeclaw    |           5432
```

迁移前：

```text
$ psql ... -c "SELECT tablename FROM pg_catalog.pg_tables WHERE schemaname='public' ORDER BY tablename;"
 tablename
-----------
(0 rows)
```

所有 `psql` 命令均显式使用 `PGDATABASE="$HONE_POSTGRES_DATABASE"`，没有连接或写入其它数据库。

## Verification

### 1. 构建与源库完整性

```text
$ CARGO_TARGET_DIR=/tmp/hone-pg35-target-91227a9f cargo build -p hone-cli
Finished `dev` profile [unoptimized + debuginfo] target(s) in 43.71s

$ for db in sessions llm_audit events; do
    sqlite3 "file:/tmp/hone-pg35-91227a9f.44BhUL/data/$db.sqlite3?immutable=1" \
      'PRAGMA query_only=ON; PRAGMA integrity_check;'
  done
ok
ok
ok
```

本次仅执行现有 CLI 和 SQL 对账，没有源码行为变更，因此没有运行全仓测试。

### 2. dry-run 完整报告

命令：

```bash
env -u HONE_HOME -u HONE_USER_CONFIG_PATH \
  HONE_DATA_DIR=/tmp/hone-pg35-91227a9f.44BhUL/data \
  HONE_CLOUD_MODE=local \
  HONE_POSTGRES_NO_PROXY=true \
  /tmp/hone-pg35-target-91227a9f/debug/hone-cli \
  --config /tmp/hone-pg35-91227a9f.44BhUL/config.yaml \
  cloud migrate \
  --from-data-dir /tmp/hone-pg35-91227a9f.44BhUL/data \
  --json
```

真实输出：

```json
{
  "mode": "dry-run",
  "from_data_dir": "/tmp/hone-pg35-91227a9f.44BhUL/data",
  "upload_oss": false,
  "reuse_existing": false,
  "concurrency": 6,
  "postgres_configured": true,
  "oss_configured": false,
  "counted": {
    "sessions": 22,
    "uploads_and_attachments": 40,
    "generated_images": 0,
    "company_profiles": 161,
    "portfolio_json": 2,
    "cron_json": 1,
    "notification_prefs": 4,
    "quota_json": 1,
    "skill_registry_json": 0,
    "sqlite_files": 3,
    "other_files": 1
  },
  "uploaded_objects": 0,
  "reused_objects": 0,
  "indexed_documents": 0,
  "changed_quota_rows": 0,
  "skipped_quota_rows": 0,
  "changed_session_rows": 0,
  "skipped_session_rows": 0,
  "changed_web_auth_users": 0,
  "skipped_web_auth_users": 0,
  "changed_web_auth_sessions": 0,
  "skipped_web_auth_sessions": 0,
  "changed_cron_rows": 0,
  "skipped_cron_rows": 0,
  "changed_skill_registry_rows": 0,
  "skipped_skill_registry_rows": 0,
  "changed_notification_prefs_rows": 0,
  "skipped_notification_prefs_rows": 0,
  "changed_portfolio_rows": 0,
  "skipped_portfolio_rows": 0,
  "changed_company_profile_files": 0,
  "skipped_company_profile_files": 0,
  "changed_llm_audit_rows": 0,
  "skipped_llm_audit_rows": 0,
  "skipped_objects": 0,
  "conflicts": []
}
```

人工核对结果：

- 161 个 `company_profiles` 候选 = 143 个可结构化 actor-scoped Markdown + 18 个根级 JSON。
- 4 个 notification prefs JSON；目录里另有 1 个 `.bak-*`，因此属于 `other_files=1`。
- 三个 SQLite 的结构和行数均独立查询过；dry-run 本身不读行。
- 迁移前目标 public schema 为空，所以首次 apply 的结构化行预期全部是 changed。

### 3. 首次 apply 完整报告

命令与 dry-run 相同，仅增加 `--apply`：

```bash
env -u HONE_HOME -u HONE_USER_CONFIG_PATH \
  HONE_DATA_DIR=/tmp/hone-pg35-91227a9f.44BhUL/data \
  HONE_CLOUD_MODE=local \
  HONE_POSTGRES_NO_PROXY=true \
  /tmp/hone-pg35-target-91227a9f/debug/hone-cli \
  --config /tmp/hone-pg35-91227a9f.44BhUL/config.yaml \
  cloud migrate \
  --from-data-dir /tmp/hone-pg35-91227a9f.44BhUL/data \
  --apply --json
```

真实输出：

```text
[cloud migrate] processed 100/234
[cloud migrate] processed 200/234
[cloud migrate] processed 234/234
```

```json
{
  "mode": "apply",
  "from_data_dir": "/tmp/hone-pg35-91227a9f.44BhUL/data",
  "upload_oss": false,
  "reuse_existing": false,
  "concurrency": 6,
  "postgres_configured": true,
  "oss_configured": false,
  "counted": {
    "sessions": 22,
    "uploads_and_attachments": 40,
    "generated_images": 0,
    "company_profiles": 161,
    "portfolio_json": 2,
    "cron_json": 1,
    "notification_prefs": 4,
    "quota_json": 1,
    "skill_registry_json": 0,
    "sqlite_files": 3,
    "other_files": 1
  },
  "uploaded_objects": 0,
  "reused_objects": 0,
  "indexed_documents": 225,
  "changed_quota_rows": 1,
  "skipped_quota_rows": 0,
  "changed_session_rows": 22,
  "skipped_session_rows": 0,
  "changed_web_auth_users": 1,
  "skipped_web_auth_users": 0,
  "changed_web_auth_sessions": 1,
  "skipped_web_auth_sessions": 0,
  "changed_cron_rows": 1,
  "skipped_cron_rows": 0,
  "changed_skill_registry_rows": 0,
  "skipped_skill_registry_rows": 0,
  "changed_notification_prefs_rows": 4,
  "skipped_notification_prefs_rows": 0,
  "changed_portfolio_rows": 2,
  "skipped_portfolio_rows": 0,
  "changed_company_profile_files": 143,
  "skipped_company_profile_files": 0,
  "changed_llm_audit_rows": 7,
  "skipped_llm_audit_rows": 0,
  "skipped_objects": 234,
  "conflicts": [
    "sqlite structured import pending, skipped blob upload: /tmp/hone-pg35-91227a9f.44BhUL/data/events.sqlite3",
    "sqlite structured import pending, skipped blob upload: /tmp/hone-pg35-91227a9f.44BhUL/data/sessions.sqlite3",
    "sqlite structured import pending, skipped blob upload: /tmp/hone-pg35-91227a9f.44BhUL/data/llm_audit.sqlite3"
  ]
}
```

`skipped_objects=234` 的含义：本次没有 OSS，所以所有 225 个已索引文件都记一次 skipped；另外 3 个 SQLite、4 个 prefs、2 个 portfolio 在对象阶段固定 skipped。它不表示结构化导入失败。

### 4. 源端独立计数

核心命令：

```bash
STAGE=/tmp/hone-pg35-91227a9f.44BhUL
find "$STAGE/data/sessions" -type f -name '*.json' | wc -l
find "$STAGE/data/conversation_quota" -type f -name '*.json' | wc -l
find "$STAGE/data/cron_jobs" -type f -name '*.json' -exec jq -r '.jobs | length' {} \; | awk '{s+=$1} END {print s+0}'
find "$STAGE/data/notif_prefs" -type f -name '*.json' | wc -l
find "$STAGE/data/portfolio" -type f -name '*.json' | wc -l
find "$STAGE/data/agent-sandboxes" -type f -path '*/company_profiles/*.md' | wc -l
find "$STAGE/data" -type f -path '*/uploads/*' | wc -l
sqlite3 "file:$STAGE/data/sessions.sqlite3?immutable=1" 'PRAGMA query_only=ON; SELECT COUNT(*) FROM web_invite_users; SELECT COUNT(*) FROM web_auth_sessions;'
sqlite3 "file:$STAGE/data/llm_audit.sqlite3?immutable=1" 'PRAGMA query_only=ON; SELECT COUNT(*) FROM llm_audit_records;'
```

合并后的真实输出：

```text
session_json_rows|22
session_sqlite_rows_reference|22
quota_rows|1
web_invite_user_rows|1
web_auth_session_rows|1
web_external_state_rows_embedded|0
cron_json_files|1
cron_job_rows|1
skill_registry_rows|0
notification_prefs_rows|4
portfolio_rows|2
company_profile_md_rows|143
company_profile_json_unstructured|18
llm_audit_rows|7
upload_files|40
generated_image_files|0
document_index_candidates|225
unclassified_files:
notif_prefs/discord__direct__483641214445551626.json.bak-20260509-232216
```

普通 session 虽然实际从 JSON 导入，但又用冷备份 SQLite 做了参考核对：两边都是 22 个 session，ID 集合双向差集都是 0；`session_messages` 的 312 行覆盖同样的 22 个 session，且无 orphan。

### 5. PostgreSQL 逐表计数

命令：

```sql
SELECT * FROM (
  SELECT 1 AS ord, 'cloud_sessions' AS pg_table, COUNT(*)::bigint AS pg_rows FROM cloud_sessions
  UNION ALL SELECT 2, 'conversation_quota', COUNT(*) FROM conversation_quota
  UNION ALL SELECT 3, 'cloud_web_invite_users', COUNT(*) FROM cloud_web_invite_users
  UNION ALL SELECT 4, 'cloud_web_auth_sessions', COUNT(*) FROM cloud_web_auth_sessions
  UNION ALL SELECT 5, 'cloud_cron_jobs', COUNT(*) FROM cloud_cron_jobs
  UNION ALL SELECT 6, 'cloud_skill_registry', COUNT(*) FROM cloud_skill_registry
  UNION ALL SELECT 7, 'cloud_notification_prefs', COUNT(*) FROM cloud_notification_prefs
  UNION ALL SELECT 8, 'cloud_portfolios', COUNT(*) FROM cloud_portfolios
  UNION ALL SELECT 9, 'cloud_company_profile_files', COUNT(*) FROM cloud_company_profile_files
  UNION ALL SELECT 10, 'cloud_llm_audit_records', COUNT(*) FROM cloud_llm_audit_records
  UNION ALL SELECT 11, 'cloud_documents', COUNT(*) FROM cloud_documents
) counts ORDER BY ord;

SELECT kind, COUNT(*) FROM cloud_documents GROUP BY kind ORDER BY kind;
```

真实输出：

```text
 ord |          pg_table           | pg_rows
-----+-----------------------------+--------
   1 | cloud_sessions              |      22
   2 | conversation_quota          |       1
   3 | cloud_web_invite_users      |       1
   4 | cloud_web_auth_sessions     |       1
   5 | cloud_cron_jobs             |       1
   6 | cloud_skill_registry        |       0
   7 | cloud_notification_prefs    |       4
   8 | cloud_portfolios            |       2
   9 | cloud_company_profile_files |     143
  10 | cloud_llm_audit_records     |       7
  11 | cloud_documents             |     225

      kind       | rows
-----------------+-----
 company_profile |  161
 cron            |    1
 quota           |    1
 session         |   22
 upload          |   40
```

逐域对账：

| 迁移域 | 源行数 | PG 表 / kind | PG 行数 | 结论 |
|---|---:|---|---:|---|
| Session JSON | 22 | `cloud_sessions` | 22 | 相等，ID 差集 0 |
| Conversation quota | 1 | `conversation_quota` | 1 | 相等 |
| Web invite users | 1 | `cloud_web_invite_users` | 1 | 相等 |
| Web auth sessions | 1 | `cloud_web_auth_sessions` | 1 | 相等 |
| Web external state | 0 | 嵌入 `cloud_web_invite_users.record` | 0 | 源为 0；无独立 PG 表 |
| Cron jobs | 1 | `cloud_cron_jobs` | 1 | 相等 |
| Skill registry | 0 | `cloud_skill_registry` | 0 | 相等，源文件不存在 |
| Notification prefs | 4 | `cloud_notification_prefs` | 4 | 相等 |
| Portfolios | 2 | `cloud_portfolios` | 2 | 相等 |
| Actor-scoped company profile Markdown | 143 | `cloud_company_profile_files` | 143 | 相等 |
| LLM audit | 7 | `cloud_llm_audit_records` | 7 | 相等，ID 差集 0 |
| 全部可索引 documents | 225 | `cloud_documents` | 225 | 相等；bytes 未进 PG/OSS |
| Document kind: company_profile | 143 Markdown + 18 根级 JSON = 161 | `cloud_documents.kind=company_profile` | 161 | 相等 |
| Document kind: session | 22 | `cloud_documents.kind=session` | 22 | 相等 |
| Document kind: cron | 1 | `cloud_documents.kind=cron` | 1 | 相等 |
| Document kind: quota | 1 | `cloud_documents.kind=quota` | 1 | 相等 |
| Document kind: upload | 40 | `cloud_documents.kind=upload` | 40 | 相等 |
| Generated images | 0 | `cloud_documents.kind=generated_image` | 0 | 相等 |

### 6. `llm_audit` WAL 完整性专项

命令与输出：

```text
$ stat -f '%N|%z bytes' \
    /Users/zhangxuanren/Workspace/honeclaw/data/llm_audit.sqlite3 \
    /Users/zhangxuanren/Workspace/honeclaw/data/llm_audit.sqlite3-wal \
    /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/llm_audit.sqlite3
/Users/zhangxuanren/Workspace/honeclaw/data/llm_audit.sqlite3|348160 bytes
/Users/zhangxuanren/Workspace/honeclaw/data/llm_audit.sqlite3-wal|436752 bytes
/Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/llm_audit.sqlite3|684032 bytes

$ sqlite3 "file:/tmp/hone-pg35-91227a9f.44BhUL/data/llm_audit.sqlite3?immutable=1" \
    "PRAGMA query_only=ON; SELECT 'page_size|' || page_size FROM pragma_page_size; SELECT 'page_count|' || page_count FROM pragma_page_count; SELECT 'backup_llm_audit_rows|' || COUNT(*) FROM llm_audit_records;"
page_size|4096
page_count|167
backup_llm_audit_rows|7

$ psql ... -At -c "SELECT 'pg_llm_audit_rows|' || COUNT(*) FROM cloud_llm_audit_records"
pg_llm_audit_rows|7

$ # SQLite backup IDs 与 PG IDs 排序后做 comm -3，只输出差集行数
llm_audit_id_symmetric_diff=0
```

`167 * 4096 = 684032`，与冷备份文件大小完全一致。迁入 PG 的 7 行就是含 WAL 的冷备份完整行集，而不是只看 348,160 bytes 活库主文件得到的旧快照。

### 7. 冷备份未被污染

迁移完成并安全重跑后：

```text
$ shasum -a 256 \
    /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/events.sqlite3 \
    /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/sessions.sqlite3 \
    /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/llm_audit.sqlite3
f594588e3345bd6225590c84e266253de1f0c2ed0be6ada4cc5e79eefebfa89c  /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/events.sqlite3
0ee2afa5ecfc84dfb97de5a023105944d633b2be84b36b1a7894bf46b4904933  /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/sessions.sqlite3
a8d0c04e77b4957f81f43f5db34fea11eac3a16d3cc114b572dfedeebfe7ee29  /Users/zhangxuanren/Workspace/honeclaw/data/backups/pre-pg-migration/llm_audit.sqlite3

$ find data/backups/pre-pg-migration -maxdepth 1 -type f \
    \( -name '*-wal' -o -name '*-shm' -o -name '*-journal' \) -print | awk 'END{print "backup_sidecar_count=" NR}'
backup_sidecar_count=0
```

## Risks / Follow-ups

### 仍然没有迁移通道的数据域

冷备份 SQLite 的真实计数：

```text
sessions.sqlite3:
billing_entitlements|0
billing_webhook_events|0
cron_job_runs|17
migration_runs|0
session_messages|312
session_metadata|67
sessions|22
web_admin_actions|0
web_auth_sessions|1
web_invite_users|1
web_push_messages|0
web_user_external_state|0

events.sqlite3:
events|76147
engine_meta|1
delivery_log|83928
delivered_push_context|154
earnings_continuity_jobs|0
```

当前目标库没有 event-engine 五张表：

```text
           name           | pg_relation
--------------------------+------------
 delivered_push_context   |
 delivery_log             |
 earnings_continuity_jobs |
 engine_meta              |
 events                   |
```

未迁移项：

1. **event-engine 五张表**：`events`、`engine_meta`、`delivery_log`、`delivered_push_context`、`earnings_continuity_jobs`。这是本次明确排除的范围；另一个 worktree 正在补 PG 实现与通道。
2. **Cron 执行历史**：`sessions.sqlite3.cron_job_runs` 有 17 行；PG 已有 `cloud_cron_job_runs`，但当前 `cloud migrate` 只导入 cron job 定义 JSON，不导入历史，所以目标表仍是 0 行。这是本次范围内发现的实际非零缺口，未擅自改代码。
3. **Billing / Web admin / Web push**：源表与 PG 表都存在，但 CLI 无导入 flag；本机源行数均为 0，所以本次没有实际数据缺失。
4. **`web_user_external_state`**：没有独立 PG 表；当前 exporter 会把它嵌入 `cloud_web_invite_users.record.external_state`。本机源行数为 0。若后续阶段要求独立表，需由对应 worktree 处理。
5. **Session SQLite 的关系型镜像表**：`session_messages` 312 行、`session_metadata` 67 行没有独立 PG 表/导入器；当前通道迁的是完整 Session JSON 到 `cloud_sessions.content`。22 个 JSON session 与 SQLite session ID 完全一致，所以不是 22 个 logical session 的行数缺口，但不是逐关系表迁移。
6. **`migration_runs`**：本地 SQLite backfill bookkeeping，无 CLI/PG 对应迁移，本机为 0 行。
7. **`data/events.jsonl`**：扩展名不被候选分类器识别，本次未复制、未迁移，按计划继续保留为独立对照基准。

### 文档 / 对象风险

- `oss_configured=false`，所以 225 条 `cloud_documents` 只是 metadata + hash + `local:///tmp/hone-pg35-91227a9f.44BhUL/data/...`。对象 bytes 没进 PostgreSQL，也没有上传 OSS。
- 当前临时快照必须保留，直到决定是配置 OSS 后重跑 `--upload-oss`，还是用长期稳定路径重新生成文档索引。直接删除 `/tmp/hone-pg35-91227a9f.44BhUL` 会让这些 `local://` URI 失效。
- 根级 `company_profiles/*.json` 只进入 `cloud_documents`，不会进入 `cloud_company_profile_files`；报告里的 161 与结构化 changed=143 的差 18 不是丢行，而是两种能力边界。
- apply 报告的 3 条 SQLite `conflicts` 是对象阶段的固定提示，不应据此否定已经成功的 LLM audit / Web auth 结构化导入。
- dry-run 不给出 would-change 行数；以后不能只看 changed/skipped=0 判断空源，必须另做源计数。

## Next Entry Point

1. event-engine worktree 完成 PG schema / importer 后，继续从同一份 `data/backups/pre-pg-migration/events.sqlite3` 冷备份迁移并逐表 count；`events` 还需按原计划抽样核对 `payload_json` hash。
2. 单独决定 `cron_job_runs` 17 行是否需要补导入通道；当前已如实保留在冷备份中。
3. 决定 `cloud_documents` 的长期路径：优先配置 OSS 后带 `--upload-oss --reuse-existing` 重跑，或在稳定路径上重建索引；在此之前不要删除 `/tmp/hone-pg35-91227a9f.44BhUL`。
4. 所有后续迁移与对账通过前，不要删除或 checkpoint 冷备份。
