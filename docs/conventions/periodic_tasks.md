# 周期性任务表达约定

honeclaw 同时跑着事件源拉取、digest scheduler、daily report、retention cleanup、
subscription 热刷新、thesis distill cron、heartbeat 等多套"长跑后台 task"。
本文沉淀 5 条 idiom 约定,目的是让"读代码的人无论看哪段周期任务,
都能用同一个心智模型快速理解",并作为后续 PR 的对齐模板。

> 范围:`crates/hone-event-engine/`、`crates/hone-core/heartbeat.rs`、
> `crates/hone-web-api/src/lib.rs` 启动时 spawn 的所有后台 task。
>
> **不在范围**:`crates/hone-scheduler/`(它是终端用户层的"cron job 调度器",
> 语义独立,不跟内部周期任务混在同一抽象里)、channel binary 各自的
> `scheduler.rs`(那是 SchedulerEvent 消费者,不是周期任务)。

---

## 1. interval-only loop 模板

固定周期、不跟时刻对齐的 loop(retention cleanup、subscription 热刷新、
news/price poller 等)统一这样写:

```rust
let mut ticker = tokio::time::interval(Duration::from_secs(N));
ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
loop {
    ticker.tick().await;
    if let Err(e) = work().await {
        warn!(task = "<name>", "tick failed: {e:#}");
    }
}
```

要点:

- **必须**用 `tokio::time::interval`,**禁止**用 `tokio::time::sleep` 拼循环
  (sleep 会在 work 耗时上漂移,interval 不会)
- **必须**显式设置 `MissedTickBehavior::Delay`(默认 Burst 会一次性把欠的 tick 全部
  补回来,在长任务恢复后突发风暴)
- 若需要"冷启动立即跑一次而不是等到第一个 tick",在 loop 前先 `work().await`,
  并在进 loop 前 `ticker.tick().await` 消耗掉立即返回的第一次 tick
  (参考 `pipeline::cron_aligned_loop` 的处理)

参考实现:
- [crates/hone-event-engine/src/engine.rs](../../crates/hone-event-engine/src/engine.rs)
  的 retention cleanup / subscription hot-refresh 两段
- [crates/hone-event-engine/src/spawner.rs](../../crates/hone-event-engine/src/spawner.rs)
  的 `spawn_event_source` FixedInterval 分支

---

## 2. cron-aligned loop 唯一入口

需要"在本地某 HH:MM 时刻命中后跑一次"的任务(digest_scheduler、daily_report、
FMP earnings/macro/corp_action 等日频 poller),
**必须**走 [`pipeline::cron_aligned_loop`](../../crates/hone-event-engine/src/pipeline.rs)
或拟新增的 [`pipeline::cron_minute_tick`](../../crates/hone-event-engine/src/pipeline.rs)
(Stage 2 引入,用于带 fired HashSet 状态的本地任务)。

**禁止**手撸下列模板:
```rust
// 反例:DON'T copy this anymore
let mut ticker = tokio::time::interval(Duration::from_secs(60));
let mut fired = HashSet::new();
let mut last_date = String::new();
loop {
    ticker.tick().await;
    let now = Utc::now();
    let today = digest::local_date_key(now, tz_offset);
    if today != last_date { fired.clear(); last_date = today; }
    /* ...自定义命中逻辑... */
}
```

它已经在 `pipeline::cron_aligned_loop` 里实现一次,
不要在 `engine.rs::start()` 内、或新模块里再抄。

---

## 3. tracing 字段标准化

所有周期任务的 `info!` / `warn!` **必须**带 `task` 字段
(过去 poller 用 `poller`、internal task 啥字段都没带,这次统一):

| 字段 | 取值范围 | 说明 |
|---|---|---|
| `task` | 形如 `poller.fmp.earnings` / `internal.daily_report` / `thesis_cron` / `heartbeat.feishu` | 稳定标识,跟 `task_runs.jsonl` 的 task 字段一致 |
| `outcome` | `ok` / `skipped` / `failed` | tick 结果。skipped 表示按业务策略主动跳过(如 thesis cron 的 staleness 判断),不是失败 |
| `items` | 整数,可省 | 本次处理条数(事件、推送、蒸馏对象等) |
| `degraded` | `true`,可省 | 仅指**上游网络降级**(API 限流、超时、5xx),不等同 `outcome=failed`。当一次 tick 部分上游失败但总体仍走完,用 `outcome=ok` + `degraded=true` |

`poller=` 字段保留向下兼容(已经在 `process_events` / `log_poller_error`
里用了),但新代码统一用 `task=`。

---

## 4. checkpoint 命名

任何"上次跑了什么时候 / 跑过什么 key"的持久化字段:

- **RFC3339 时间戳**:统一命名 `last_<verb>_at`
  (已有:`last_thesis_distilled_at`、`cron_jobs.last_run_at`)。
  `<verb>` 用过去式动词,不要用名词
  (`last_distill_at` ✗ → `last_distilled_at` ✓)
- **内存 fired key**(防止同窗口重复触发):
  `format!("{date}@{label}@{hhmm}")`,例如 `"2026-04-26@pre@08:30"`。
  事实标准已经在 `pipeline::cron_aligned_loop` / `spawner.rs` 内一致,
  新代码沿用
- **跨日清理**:fired HashSet 在 `today != last_date` 时 `clear()`,
  不要做"删除超过 N 小时的 key"那种增量回收(无意义复杂度)

---

## 5. 失败处理梯度

绝大多数周期任务用 Tier-A,升级到 Tier-B 或 Tier-C 必须在 PR 描述里写明动机:

### Tier-A:下次 tick 自动恢复(默认)

`work()` 返回 `Result<_, _>`,失败只 `warn!(task=..., "tick failed: {e:#}")`,
**不**重试,**不**记 backoff,**不**触发告警。理由:大多数周期任务的 tick 间隔
本身就够短(60s ~ 24h),下一 tick 自动恢复比手撸 backoff 简单且效果相当。

适用:所有 event-engine pollers / digest scheduler / daily report / cleanup /
hot-refresh / thesis_cron。

### Tier-B:consecutive_failures 计数 + 文件 sidecar

任务自身带 `consecutive_failures: AtomicU32`,失败时累加,成功时清零。
超过阈值 N 后写一个 `.error` sidecar 文件供外部(web-api / 监控)被动查询。

适用:**仅 heartbeat**
([crates/hone-core/src/heartbeat.rs](../../crates/hone-core/src/heartbeat.rs))。
理由:heartbeat 是"系统活着"的最后一道证明,其失败本身就是要被 web-api UI
显式看到的状态,跟普通"任务跑挂了下次再说"语义不同。

### Tier-C:exponential backoff

**目前没有任务用,不要预设抽象**。如果将来某个上游 API 真的需要(如外部
LLM provider 限流),按需引入 [`backon`](https://crates.io/crates/backon) 之类
专门的 crate,而不是在通用周期任务框架里塞 backoff 字段。

---

## PR checklist(给写新周期任务的人)

新增一个周期任务时,逐条对照:

- [ ] interval / cron 哪个语义?如果是 cron-aligned,有没有走 `pipeline::cron_aligned_loop`?
- [ ] `set_missed_tick_behavior(Delay)` 设了吗?
- [ ] 是不是真的需要冷启动跑一次?如果是,有没有处理"第一次 ticker.tick() 立即返回"?
- [ ] 日志带 `task=<name>` 字段了吗?`outcome` 字段呢?
- [ ] 有没有持久化 checkpoint?字段命名是 `last_<verb>_at` 吗?
- [ ] 有没有写一行 `task_runs.jsonl`(Stage 3 落地后)?调用点选在哪?
- [ ] 失败处理选了哪个 Tier?如果不是 A,在 PR 描述里说理由了吗?

---

## 不约束的事(防止过度治理)

以下事情**没有**统一约定,因为本来就不需要(过度统一反而拖累迭代):

- **任务的"业务逻辑函数"怎么组织**:每个任务自己定义 `tick_once` /
  `distill_tick` / `purge_*` 等业务函数,签名各异。约定只覆盖"循环外壳",
  不碰内部
- **是否要 trait 化所有周期任务**:
  没有 `PeriodicTask` / `Heartbeat` / `JobRunner` 这种全局 trait。
  每个任务直接 `tokio::spawn` 一个独立 future,
  保持"一个文件读完就懂这个任务在做什么"的可读性
- **是否要全局 supervisor / task registry**:
  honeclaw 单 binary 单进程,5-10 个长跑 task 直接 `tokio::spawn` 管够。
  Supervisor 真正的价值在跨进程 / 资源边界 / 重启策略,跟当前架构不匹配
- **HoneScheduler(用户 cron job)与本约定的关系**:
  HoneScheduler 是用户层产品功能(让 agent 周期跑某 skill),
  跟 event-engine 内部任务不是一个层面,不强行合并
