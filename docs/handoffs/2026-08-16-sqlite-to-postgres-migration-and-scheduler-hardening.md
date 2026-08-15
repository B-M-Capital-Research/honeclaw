# SQLite → PostgreSQL 全量迁移 + 调度可靠性整治

- 状态：`done`
- 日期：2026-08-16（通宵完成）
- 最终 revision：`95932629`（本地源码部署与 GCE 生产均已上线）
- 相关：`docs/current-plans/sqlite-to-postgres-migration-2026-08-16.md`（交接书）、
  `docs/current-plans/sqlite-to-postgres-implementation-spec.md`（10-agent 实施规格）

## 结果

### 存储
SQLite 已从运行时**完全移除**。收口 grep 后源码只剩两个有意保留的例外：

| 例外 | 原因 |
|---|---|
| `bins/hone-imessage` | 读的是 macOS 自己的 `~/Library/Messages/chat.db`，不是我们的数据 |
| `bins/hone-cli` | `cloud migrate` 的历史数据导入通道，删了就没法迁旧库 |

`rusqlite` 消费者也只剩这两个。

### 数据迁移与对账

| 环境 | 迁移内容 | 对账 |
|---|---|---|
| 本地 Mac | events 76,147 / delivery_log 83,928 / push_context 154 / 22 会话 / 143 公司画像 / 4 通知偏好 / 2 组合 | 逐表 1:1，0 孤儿，序列正确 |
| GCE 生产 | delivery_log 1,397 / delivered_push_context 1,397 | 1:1，0 孤儿，序列正确 |

`delivery_log.id` 与 `delivered_push_context.delivery_log_id` 的对应关系用 SQLite/PG
两侧内容哈希逐位比对确认一致（`1fffce97…` / `c4060f37…`），这是 id 未被 PG 重新分配的直接证据。

冷备份：
- 本地 `data/backups/pre-pg-migration/`（sqlite3 `.backup` 在线 API，含 WAL）
- 生产 `/srv/honeclaw/backups/pre-pg-migration/`，另有本地副本 `data/backups/gce-pre-pg-migration/`，sha256 校验一致

**对账全部通过前不要删除这些备份。**

### 生产指标（修复上线 00:47 → 05:05）

| 指标 | 7 天基线 | 现在 |
|---|---|---|
| 心跳成功 / 失败 | 失败率 12.6% | **96 / 2** |
| 失败构成 | 契约违规占 62% | **契约违规 0**，两条都是上游 HTTP 529 |
| `duration_ms` | 列不存在 | 真实数据，均值 32.3 秒 |
| 后端负载 | role=all 下约 1.8 / 2 核 | 0.89 |
| `/api/meta` | 挂起 >120 秒 | 3.4 毫秒 |

失败模式从「我们自己的 bug」变成「上游容量」，这是本轮最有意义的变化。

## 顺带修掉的缺陷

1. **PG 参数类型推断回归**（`340e51b9`，我自己引入又自己发现）：四条 cron 终态 UPDATE 里
   `$1` 既赋给 text 列又带 `::timestamptz`，PG 在 parse 阶段就失败，**整条语句没执行**。
   客户投递不受影响，但 cron 记录全部写不进去。修为 `$1::text::timestamptz`。
2. **会话影子库关闭开关失效**（P1，`docs/bugs/session_sqlite_shadow_ignored_the_kill_switch_in_web_api.md`）：
   `hone-web-api` 只看配置字段、不看环境变量，于是生产在声明「无本地存储」的前提下
   持续把 75 MB 会话内容写本地盘，而 `strict_no_local_storage` 报告零依赖。
3. **per-call `Runtime::new()` 反模式还活在 8 个模块里**（`62d0c889` 只修了 1 个）。
   生产实测这个写法让进程 26 分钟烧掉 47 CPU 分钟。全部改接共享长驻 runtime。
4. **CI 从 `67d292b1` 起一直是红的**：两个回归脚本无条件调用 `rg`，而 runner 上没有。
   push 门禁实际已失效一段时间。
5. **CI 覆盖面缺口**：迁移把 91 个测试改成打真实 PG 的 `#[ignore]`，而 CI 不传 `--ignored`。
   已补一步（只针对 `hone-memory`；其它 crate 的 ignored 是外部依赖用例）。
6. **两处测试竞态/隔离缺陷**：lease teardown 未持锁；一个测试直连 `public` 而其余走 `pg_temp`。

## 后续接手者必须知道的

- **PostgreSQL 现在是配置校验的必需项**，OSS 已解耦为可选。
- **本地 `config.yaml` 必须有 `cloud.postgres` 块**，光靠 `.env` 不行：
  `load_dotenv_if_present()` 按相对 CWD 读 `./.env`，而部署后的渠道二进制从
  `data/releases/source/<sha>/` 启动。第一次部署就栽在这里（web 起来了、Discord 起不来并自动回滚）。
- **`cargo test` 需要活的 PG，且不读 `.env`，必须 export**。先 `bash scripts/dev_pg.sh up`。
- **测试隔离**：`pg_temp` 是会话局部的；`ensure_cloud_schema_once` 是进程级 AtomicBool。
  这类 bug 的特征是「单独跑过、和别人一起跑挂」，且**在有历史数据的开发库上会侥幸通过**——
  验证必须用全新空库。
- **4 处「PG 失败退回本地存储」的静默降级已改成硬失败**（hone-cli cron 初始化、
  bot_core cron storage、web-api 的飞书 cron/session 联系人读取）。
  代价：PG 短暂不可用时进程起不来。收益：不再悄悄把客户数据写到临时盘。

## 未决项

1. **生产 PG 是 17.11，本地 compose 与 CI 仍钉 16。** 对齐需要重建本地数据卷，未做。
2. **`cron_job_runs` 有 17 行历史没有导入通道**（本地冷备份里）。需要决定是补通道还是接受丢失。
3. **`cloud_documents` 的 225 条 `local://` URI 指向 `/tmp/hone-pg35-91227a9f.44BhUL`**，
   删除该目录会让它们失效。需要决定是配 OSS 重跑 `--upload-oss` 还是在稳定路径重建索引。
4. **`actor_channel='web'` 的 43 行僵尸 `running` 记录**（最早 2026-06-01）。
   `recover_stale_started_rows` 对 web/imessage 的调用在 `handle_scheduler_events` 里，
   被 `runtime_role.runs_worker_tasks()` 门控，而 GCE 跑 `role=web` ⇒ 这条回收路径永不执行。
   根因是已知的 worker 角色缺口（`9b502efa`），不是回收逻辑本身有错。
5. **流式重试对 `upstream HTTP 529` 的分类值得复查**：529 是「集群过载」，
   立即重试未必有帮助，可能需要更长退避，或干脆记为供应商侧事件而不算任务失败。
6. **`data/events.jsonl`（278 MB）保持原样**，本轮作为独立比对基准，去留待定。

## 验收记录

```
cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
  2579 passed / 0 failed   （在全新空库上跑，等价 CI 条件）

cargo test -p hone-memory --all-targets -- --ignored
  92 passed / 0 failed     （真实 PostgreSQL 集成测试）

bash tests/regression/run_ci.sh
  exit 0，22 个脚本         （PATH 上无 rg，等价 CI 条件）

bun run test:web
  486 passed / 0 failed
```
