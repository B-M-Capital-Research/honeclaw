# Bug: /api/meta 在负载下无限挂起，拖死部署验收与就绪探测

## 发现时间

- 2026-08-15 22:20 CST（GCE 排查期间，`/api/meta` 连续多次 120 秒零字节）

## Bug Type

- Availability / Blocking IO on async path

## 严重等级

- P2

## 状态

- Fixed（2026-08-15，代码级；待生产复核）

## 现象

GCE 生产机上 `/api/meta` 反复出现 **120 秒零字节不返回**，`curl` 连
`%{http_code}` 都打不出来（连接一直挂着）。同期 PG 完全健康
（6 连接 / 上限 100，`SELECT 1` 正常），排除数据库故障。

handler 里有三处会在 async 路径上阻塞，且都不在任何超时保护内：

1. **`hone_core::current_build_info()` 在请求路径上对整个二进制做同步
   SHA-256**。生产二进制 `hone-console-page` 为 **278 MB**，实测纯 CPU
   3.8 秒（页缓存冷时 5.2 秒），CPU 打满时成倍放大。它是 `LazyLock`，
   并发请求会全部阻塞在同一个 `Once` 上，**挂住 runtime worker 线程**
   （不是 yield）。渠道进程早已通过 `hone_channels::bootstrap` 预热，
   **web 进程入口 `bins/hone-console-page/src/main.rs` 一直漏着**。
2. **PG 探活的 5 秒超时只包住了建连**：`CloudPgRuntime::health()` 里
   `client.query_one("SELECT 1", &[])` 在 `tokio::time::timeout` 窗口
   **之外**、完全无界。连接建好但 backend 排在锁/CPU 队列后面、链路半开或
   代理静默 stall 时，这里可以挂任意长时间。
3. **handler 无总预算**：`OssObjectStore::from_config` 每次请求都同步新建一个
   `reqwest::Client`（TLS / proxy 初始化），这段在各自的 timeout 之外。

连带影响：`scripts/deploy_source_runtime.sh` 的就绪循环用的正是
`/api/meta --max-time 3`，handler 一挂就恒返回空串，循环只能空转到超时
——这正是「零字节」被观察到的形态。而
`docs/runbooks/backend-deployment.md` 本来就警告过不要拿 `/api/meta` 当就绪
探针（它会做实时 PG / 对象存储检查），就绪应走
`/api/runtime/active-chat-runs`（纯内存计数、零 IO）。

## 影响面

- 部署验收手册依赖的端点在高负载时不可用；就绪循环空转到超时。
- 挂起会被误读成 `pg.ok=false / oss.ok=false`，看起来像云存储失效。
  **实际上 `cloud_storage_authoritative` 是纯上报字段**：全部消费
  `is_cloud_authoritative()` 的调用点读的都是配置 `effective_mode()`，
  没有任何一处消费 `/api/meta` 里那个布尔值。探活超时**不会**把系统切回
  本地存储。（本次排查中我一度据此误判过一次。）

## 修复记录

- `bins/hone-console-page/src/main.rs` 启动时 `spawn_blocking` 预热
  `current_build_info()`，把 278 MB 哈希移出请求路径。
- `CloudPgRuntime::health()` 把「建连 + 查询」整体纳入 5 秒超时窗口；
  超时 detail 改为 `postgres health probe_timeout (5s, connect+query)`，
  并在注释里写明「探活超时 ≠ 云存储已失效」。
- `routes/meta.rs` 新增 `probe_with_budget`：每个云探活再加一道 8 秒总预算，
  超时降级成 `ok=false` + `probe_timeout`，绝不 5xx、更不挂住连接
  ——照 `routes/users.rs` 的既有惯例。
- `scripts/deploy_source_runtime.sh` 的 `wait_web_ready` 改为先用
  `/api/runtime/active-chat-runs` 判就绪，进程确实在监听之后再用宽预算
  （20 秒）读一次 `/api/meta` 做 revision 验收。

### 验收

- `bash tests/regression/ci/test_source_runtime_deploy_contract.sh` 通过
  （`state=ready` 走通）。

## 证据来源

- GCE 上 `curl http://127.0.0.1:8077/api/meta` 连续 120s / 60s 零字节
- `ls -la /opt/hone/current/bin/`（278 MB）与 `time sha256sum`（3.8s / 5.2s）
- 同期 `pg_stat_activity` 6 连接、`SHOW max_connections` = 100
- `docs/runbooks/backend-deployment.md` 关于就绪探针的既有警告
