# Bug: delivered-push context claim 竞态导致 agent_session 测试间歇失败

- 发现时间：2026-08-03 CST
- Bug Type：System Error（测试可靠性 / 潜在业务影响待确认）
- 严重等级：P2
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

- 连续运行 `cargo test -p hone-channels --lib` 的失败率按提交明显分层（同一台机器，相同并发）：
  - `d52cc768`（`39470783` 之前）：2/16 失败。
  - `215004fb`（含 `39470783 fix(agent): require current evidence for listing denials`，不含本轮改动）：7/10 失败。
  - `1a55c2da`（`215004fb` + 本轮改动）：8/10 失败。
  即失败率的跃升出现在 `39470783`，与本轮交互式研究预算改动无关；后者只在同一水平上下浮动。
- 失败集中在 `agent_session::tests` 的两个家族，每次只挂其中一个，且单独运行必过：
  - `native_interactive_turn_consumes_delivered_pushes_once_without_mutating_user_text`
    —— `crates/hone-channels/src/agent_session/tests.rs:1100` `.expect("P1 in U1 turn")` 失败，即已投递推送没有被投影进 U1 的 runtime input。
  - `run_zero_daily_conversation_limit_bypasses_quota` / `run_rejects_over_daily_limit_with_user_turn_and_friendly_error`
- 该家族由当日提交 `f90fcfe0 feat(agent): 将已送达推送接入下一轮上下文` 引入；`39470783` 之后失败率从偶发变成多数轮次失败。
- `39470783` 大幅改写了 `agents/function_calling` 的批次执行与终稿发布路径（逐个隔离格式错误只读调用、上市发布边界与一次同-Agent 纠正轮），会改变一次 Agent turn 的实际轮数与耗时，因而直接影响这条依赖墙钟时序的断言。

## 初步分析（未定论）

- `claim_delivered_push_context_with_native_observation`（`crates/hone-event-engine/src/store.rs:1207`）在取记录之前，会先按 `native_session_id` + `now_ms` 执行一次 “native 已观察则推进日志、不再重复注入” 的 UPDATE。
- 候选查询本身用 `delivered_at_ms <= ?` 是闭区间，同毫秒不会被排除；因此更可能的方向是上面那次 native-observation 推进在测试的 Scheduled 轮与 U1 轮之间按时序发生分叉，使 U1 拿不到本应注入的记录。
- 尚未证实是否只影响测试。若真实链路上同一 native session 的 scheduled 轮与紧随其后的用户轮存在同样时序，用户可能会丢失一次“已送达推送进入下一轮上下文”的投影。

## 影响与定级

- 当前可确认的影响是 CI 间歇性红，属于测试可靠性问题。
- 潜在业务影响（用户轮丢失已送达推送上下文）尚未在真实会话中取证，因此定 P2 而非 P1。

## 下一步

- 从 `store.rs` 的 native-observation UPDATE 与候选 SELECT 之间的时序入手，补一个确定性时钟或显式序号，让 “scheduled 轮已观察” 与 “U1 轮应注入” 不再依赖墙钟毫秒。
- 补一条在同一 native session 下 scheduled → user 连续两轮的确定性回归，替代当前依赖真实时间的断言。
- 复核真实 Feishu / Web 会话是否出现过用户轮缺失已送达推送上下文。

## 备注

- 本条不是 2026-08-03 交互式研究预算修复引入的：该修复只触及 `investment_response_guard`、`response_finalizer`、`prompt`、`bot_core` 工具注册与 `function_calling` 循环，未触及 delivered-push 或 quota 链路；按上面的分层数据，去掉该修复后失败率不变。
- 当前 `main` 上 `cargo test -p hone-channels --lib` 多数轮次会红，等同于该 crate 的测试门实际失效。建议优先处理：在修好时序之前，CI 上的绿灯不再能证明 hone-channels 无回归。
