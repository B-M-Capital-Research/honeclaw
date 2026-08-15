# Bug: 同分钟到期任务齐射无背压，成批打挂上游建连

## 发现时间

- 2026-08-15 22:20 CST（GCE 开启 `runtime_role=all` 后排查失败尖峰）

## Bug Type

- Reliability / Resource Contention

## 严重等级

- P1

## 状态

- Fixed（2026-08-15，代码级；待生产波次复核）

## 现象

到期任务在各渠道消费端都是裸 `while recv() { tokio::spawn(...) }`，
**没有任何并发上限**；同一分钟到期的任务全部同时起跑。

2026-08-15 北京 22:09，8 个 web 渠道任务同分钟到期，16 次 LLM 调用**全部**
`provider_transport_error`（`所有 OpenAI-compatible API Key 的流式请求均失败：
error sending request for url (https://api.minimaxi.com/v1/chat/completions)`），
而同一时刻在同一台机器上手工单发流式 POST **2.2 秒即成功**
——是并发突刺打挂了**建连**阶段，不是连通性问题。

雪上加霜的是流式路径当时**完全没有重试**：
`openai_compatible.rs` 的 `chat_with_tools_stream` 失败即 `continue` 换下一把
key，而生产只配了一把（`api_keys: []`），于是「重试」退化成一次性调用。
非流式路径反而早有 `for attempt in 0..=1` + 固定 2s 重试。

## 影响面

生产任务时刻高度聚簇（`cloud_cron_jobs`，100 个 enabled）：

- **25 个 enabled heartbeat 每个半点整齐射**，每天 48 轮，约 1200 次执行
- 叠加 20:00 的 12 个 daily 等，峰值单分钟 **86 次执行**（08:31）
- 峰值分钟失败率显著高于基线：20:01 为 35%、08:31 为 28%，基线约 13%

另外 `llm.providers.<name>.max_retries`（默认 3）是 **dead config**：
定义在 `config/agent.rs`，但全仓没有任何读取点，`LlmResolver` 构造 provider 时
只取 `timeout` 与 key pool。

## 修复记录

三层，各司其职：

1. **heartbeat 派发抖动**（`memory/src/cron_job/storage.rs`）
   按 `job_id` 的 FNV-1a 哈希把同半点槽的 heartbeat 确定性摊到
   `[0, JITTER_SPREAD_MINUTES=4)` 分钟内。
   - 只对 heartbeat 生效：它压根不看 `schedule.hour/minute`（每半点自动触发），
     推迟几分钟对用户不可见。**用户显式设定时刻的定时任务（如 20:00 日报）
     不加抖动**——改触发时刻是产品语义，需单独决策；它们靠第 2 层削峰。
   - 偏移必须严格小于 `DUE_WINDOW_MINUTES=5`：被推迟的任务要靠后续 tick 重新
     进入同一到期窗口，偏移够到窗口边界会导致该轮任务被整个丢掉。已用
     `const _: () = assert!(...)` 在编译期钉死。
   - 用手写 FNV-1a 而非 `DefaultHasher`：后者不保证跨 Rust 版本稳定，
     升级工具链会让所有任务的触发分钟集体漂移。
   - 幂等不受影响：claim 键 `due_key` 锚定的是**计划时刻**而非实际执行时刻。

2. **job 层并发闸**（`crates/hone-scheduler/src/lib.rs::acquire_job_slot`）
   全局 `Semaphore`，默认 4，可用 `HONE_SCHEDULER_JOB_CONCURRENCY` 覆盖；
   web / feishu / telegram / discord 四个消费端统一接入。
   - 闸设在 job 层而**不是** LLM provider 内部，原因有二：
     (a) provider 实例并不共享——`main` profile 是 `HoneBotCore` 上的单例，
     但 event-engine 另建了约 10 个独立 Provider（各自独立 `reqwest::Client`
     与连接池），闸放实例内部限不住全局并发；
     (b) `chat_with_tools_stream` 返回 `BoxStream`，permit 必须随 stream 一起
     move 才能覆盖读流阶段，否则只闸住建连、形同虚设。
   - permit 在**已 spawn 的任务内部**获取，而不是 recv 之前：后者会让一个卡住
     的任务连带堵死整个消费循环。

3. **流式传输错误退避重试**（`crates/hone-llm/src/openai_compatible.rs`）
   复用同文件的 `is_retryable_transport_error`，加指数退避 + 确定性抖动。
   **只对同一把 key 重试**——`docs/invariants.md` 规定只有认证 / 配额 / 限流
   失败才允许轮换凭据，传输失败不得跨凭据 fan-out。
   顺带把 `max_retries` 从 dead config 接上（`with_transport_retries`）。

### 验收

- `stream_retries_same_key_after_transport_failure_at_connect`：假服务器第一次
  接受连接后立刻断开，断言客户端发起了 **2 次**连接且最终拿到健康流。
- `production_connect_failure_text_is_retryable`：把生产实际错误文案钉进测试，
  否则这条重试路径永远不会触发。
- `heartbeat_dispatch_jitter_is_deterministic_and_stays_inside_due_window`：
  偏移确定性、有界、且真的摊开到每个偏移分钟。
- `stream_retry_jitter_is_deterministic_and_bounded`。

## 证据来源

- GCE journal 2026-08-15 14:09 UTC 的 16 条 `provider_transport_error`
- 同机同时刻手工 `curl` 流式 POST 成功（2.2s）
- `cloud_cron_jobs` 时刻聚簇统计、`cloud_cron_job_runs` 按分钟的失败率分布
