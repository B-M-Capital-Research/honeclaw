# hone-web-api 无视会话影子库的关闭开关，导致「无本地存储」是假的

- 状态：已修复
- 发现：2026-08-16，排查 GCE 生产主机时
- 严重度：P1（客户会话内容在运营方声明「无本地存储」的前提下持续落本地盘）

## 现象

GCE 生产机 `/etc/hone/runtime.env` 是：

```
HONE_CLOUD_MODE=cloud
HONE_CLOUD_STRICT_NO_LOCAL_STORAGE=true
HONE_CLOUD_KEEP_SESSION_SQLITE_SHADOW=false
```

但 `/srv/honeclaw/data/sessions.sqlite3` 有 **75 MB**，最后写入时间是当天 13:56，
库里有 **294 个会话 / 6559 条消息 / 967 条元数据**，且仍在增长。

同时 `strict_no_local_storage=true` 的启动检查照常通过，`/api/meta` 报告零本地依赖。

## 根因

同一条策略在两个进程里各算了一遍，而且算法不同：

| 位置 | 判断 |
|---|---|
| `crates/hone-channels/src/core/bot_core.rs:89` | 配置字段 **且** `HONE_CLOUD_KEEP_SESSION_SQLITE_SHADOW` |
| `crates/hone-web-api/src/lib.rs:448` | **只看配置字段**，无条件传 `Some(path)` |

于是环境变量只关掉了渠道进程的影子库，web 进程照写。

危害不止磁盘占用：`local_durable_dependencies()`（`cloud_runtime.rs`）用的正是
「两个条件都满足」的判断，所以它认为影子库是关的、报告零本地依赖，
`strict_no_local_storage` 这道闸也就形同虚设——这正是该函数注释里
写过的「否则 strict_no_local_storage 会给出假的『无本地依赖』」，只是漏在了另一个进程里。

## 修复

把策略抽成唯一权威函数 `hone_core::cloud_runtime::session_sqlite_shadow_enabled(&config)`，
`bot_core`、`hone-web-api`、`local_durable_dependencies` 三处全部改调它，
从结构上消除再次分叉的可能。

## 回归测试

`crates/hone-core/src/cloud_runtime.rs` 的
`cloud_session_sqlite_shadow_is_opt_in_and_reported_as_a_local_dependency` 扩充为双向断言：

- 配置字段为 true 但环境变量未设 → 必须关闭
- 环境变量已设但配置字段为 false → 必须关闭
- 两者同时成立 → 打开，并出现在 `local_durable_dependencies` 里

## 遗留

GCE 上那个 75 MB 的 `sessions.sqlite3` 是既有数据，本次只堵住继续写入。
它已被冷备份到 `/srv/honeclaw/backups/pre-pg-migration/`
（sha256 `57e2cf32…`，本地副本校验一致）。
会话权威存储是 `cloud_sessions`，所以这份影子数据是否需要保留，
留给 SQLite→PostgreSQL 迁移的阶段 3 一并决定。
