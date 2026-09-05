# Bug: 心跳输出违反 JSON 契约时从不重试，客户该轮监控静默落空

## 发现时间

- 2026-08-15 22:40 CST（排查 GCE 开启推送后的 CPU 打满时，翻生产库
  `cloud_cron_job_runs` 发现）

## Bug Type

- System Error

## 严重等级

- P1

## 状态

- Fixed（2026-08-15，代码级；待生产窗口复核）

## 现象

心跳任务的模型输出必须是约定 JSON。`inspect_heartbeat_result`
（`crates/hone-channels/src/scheduler.rs`）已经会先调
`strip_internal_reasoning_blocks` 剥掉 `<think>` 块，但模型仍有约 12% 的概率
不产出合规 JSON，落进 `PlainTextSuppressed` / `JsonMalformed` /
`JsonUnknownStatus`，经 `heartbeat_parse_error_message` 判为
`execution_failed`，**客户这一轮监控直接落空且无任何补救**。

根因是重试循环的判定口径：`run_heartbeat_task` 只在
`result.response.success == false` 时才走 `heartbeat_recovery_reason` →
`BudgetRecovery`。而契约违规属于 **`success == true` 但内容不合规**，
永远进不了重试分支。

## 影响面

生产库 `cloud_cron_job_runs`（2026-08-08 → 08-15，7 天）：

- heartbeat：noop 2654 / 成功 637 / **失败 475**（失败率 12.6%）
- 全部失败 593 条中，**380 条（64%）是心跳输出不合契约**
  （`heartbeat 输出不是结构化 JSON` 354 + `不是合法 JSON` 16 + `未知状态` 10）
- 散布在 **14 / 29 个心跳任务**上，单任务失败率 1%–23%
  （TEM 大事件心跳 23%、ASTS/RKLB/AAOI 均 15–16%）——是模型契约的系统性不稳定，
  不是某个任务配坏了
- 失败样本的 `raw_preview` 显示模型在 `<think>` 块里大段推理后并未给出 JSON

该问题至少持续了两周而无人发现，因为用户 cron 完全不在
`/api/admin/task-runs` 的健康视图里（见同批修复）。

历史上已有 e30c9d3e / f72aeefc / 72f5c39f 等一长串「再补一个 noop 模式匹配」
的提交，属于在解析侧打补丁；本次改为给模型一次真正的重试机会。

## 修复记录

- `HeartbeatRecoveryReason` 新增 `ContractViolation`，与既有
  `ContextOverflow` / `TransportError` / `MaxIterationsExceeded` 同构。
- 新增 `heartbeat_contract_recovery_profile(profile, content)`：仅在
  `profile == Primary` 且 `inspect_heartbeat_result` 判定为契约失败时，
  返回 `BudgetRecovery { ContractViolation }`；在 `run_heartbeat_task` 的
  `success == true` 分支于 `return Ok(..)` 之前接入。
- `build_heartbeat_recovery_prompt` 增加对应话术：「上一轮已经完成检查，但最终
  输出违反心跳 JSON 契约；本轮不得复述分析，只能重新给出严格 JSON」。
- 打一行 `[HeartbeatDiag] retry_with_contract_recovery`，沿用既有诊断惯例。
- **一次性**：`profile != Primary` 提前返回，重试轮再次违约不会二次进入
  recovery，避免 `run_heartbeat_task` 的 loop 无限重跑。

### 验收

- 单测 `heartbeat_plain_text_contract_failure_retries_once_then_accepts_json_noop`：
  覆盖「Primary + 违约 → 进入 recovery」「recovery 轮再违约 → 不再重试」
  「Primary + 合规 JSON → 不做无谓重试」三条。
  注意其中「一次性」那条必须用**违约内容**断言，用合规 JSON 断言是恒真的
  （`profile != Primary` 已提前返回），守不住任何回归。
- 真实 LLM 契约冒烟：`cargo run --example heartbeat_prompt_llm_smoke -p hone-channels`。

## 证据来源

- `data/logs/hone-console-page-source.log`
  - 2026-08-26 22:01 CST 运行态待部署复核，状态维持代码级 `Fixed（待生产窗口复核）`。
  - 2026-08-26 18:00-22:01 CST 同窗继续有 3 条 `execution_failed` / `跳过发送`，失败覆盖 `heartbeat 输出不是结构化 JSON` 与 `heartbeat 输出包含未知状态`；代表样本包括 18:00 CST `光模块板块关键事件心跳提醒`、22:00 CST `持仓重大事件心跳提醒`。
  - 同窗 parse 分布为 `PlainTextTriggered=58`、`JsonNoop=19`、`JsonUnknownStatus=4`、`PlainTextNoop=2`、`JsonEmptyStatus=2`、`JsonTriggered=1`、`PlainTextSuppressed=1`，仍可见模型非 JSON / `<think>` 先行输出导致的契约退化。
  - 同窗 heartbeat 仍有 `run_start=58`、`run_finish=58`、`deliver=29`，说明不是 scheduler 全局停摆；未见 live runtime 加载 2026-08-15 contract recovery 修复的确认信号，因此暂不回退代码级 `Fixed`，但继续保留待部署复核。

- `data/logs/hone-console-page-source.log`
  - 2026-08-26 18:01 CST 运行态待部署复核，状态维持代码级 `Fixed（待生产窗口复核）`。
  - 2026-08-26 14:02-18:01 CST 同窗继续有 4 条 `execution_failed` / `跳过发送`，失败覆盖 `heartbeat 输出不是结构化 JSON`、`heartbeat 输出不是合法 JSON` 与 `heartbeat 输出包含未知状态`；代表样本包括 15:30 CST `持仓重大事件心跳提醒`、17:00 CST `TEM AAOI KRMN RKLB MRVL 关键事件心跳提醒`、18:00 CST `光模块板块关键事件心跳提醒`、18:01 CST `AI与科技持仓观察关键事件心跳提醒`。
  - 同窗 parse 分布为 `PlainTextTriggered=48`、`JsonNoop=24`、`JsonTriggered=5`、`JsonUnknownStatus=4`、`JsonMalformed=2`、`PlainTextSuppressed=1`、`PlainTextNoop=1`，仍可见模型非 JSON / `<think>` 先行输出导致的契约退化。
  - 同窗未见 runtime 重启、revision 切换或确认加载 2026-08-15 heartbeat contract recovery 修复的日志证据，因此本条仍按“代码级 Fixed / 待部署复核”记录，不回退为活跃 `P1`。
  - 影响仍集中在部分 heartbeat 轮次跳过发送；同窗还有 `run_start=56`、`run_finish=56`、`deliver=26`，未见全渠道不可用、错对象投递或数据破坏证据。本轮不创建 GitHub Issue。
  - 2026-08-24 22:02 CST 运行态待部署复核，状态维持代码级 `Fixed（待生产窗口复核）`。
  - 2026-08-24 18:00-22:01 CST 同窗继续有 3 条 `heartbeat 输出不是结构化 JSON` / `execution_failed` / `跳过发送`，代表样本包括 18:01 CST `web-user-be13e1f84d14`、20:01 CST `web-user-d415e2c11ced`、21:00 CST `web-user-d415e2c11ced` 的 heartbeat 任务。
  - 同窗 parse 分布为 `PlainTextTriggered=64`、`PlainTextSuppressed=3`、`PlainTextNoop=7`、`JsonNoop=15`、`JsonTriggered=5`、`JsonEmptyStatus=1`，仍可见模型非 JSON / `<think>` 先行输出导致的契约退化。
  - 同窗未见 runtime 重启、revision 切换或确认加载 2026-08-15 heartbeat contract recovery 修复的日志证据，因此本条仍按“代码级 Fixed / 待部署复核”记录，不回退为活跃 `P1`。
  - 影响仍集中在部分 heartbeat 轮次跳过发送；同窗还有 `run_start=56`、`run_finish=62`、`deliver=33`，未见全渠道不可用、错对象投递或数据破坏证据。本轮不创建 GitHub Issue。
- `data/logs/hone-console-page-source.log`
  - 2026-08-24 10:02 CST 运行态待部署复核，状态维持代码级 `Fixed（待生产窗口复核）`。
  - 2026-08-24 06:00-10:02 CST 同窗继续有 3 条 `heartbeat 输出不是结构化 JSON` / `execution_failed` / `跳过发送`，代表样本包括 06:00 CST `web-user-879a3b18fce2`、06:30 CST `web-user-499a1c6331c4`、10:01 CST `web-user-be13e1f84d14` 的 heartbeat 任务。
  - 同窗 parse 分布为 `PlainTextTriggered=72`、`JsonNoop=17`、`PlainTextSuppressed=3`、`PlainTextNoop=1`，仍可见模型非 JSON / `<think>` 先行输出导致的契约退化。
  - 同窗未见 runtime 重启、revision 切换或确认加载 2026-08-15 heartbeat contract recovery 修复的日志证据，因此本条仍按“代码级 Fixed / 待部署复核”记录，不回退为活跃 `P1`。
  - 影响仍集中在部分 heartbeat 轮次跳过发送；同窗还有 `run_start=56`、`run_finish=58`、`deliver=36`，未见全渠道不可用、错对象投递或数据破坏证据。本轮不创建 GitHub Issue。
- 生产库 `cloud_cron_job_runs`（`db_bamang_research`）7 天窗口聚合
- `detail->>'parse_kind'` 分布与 `raw_preview` 样本
- `crates/hone-channels/src/scheduler.rs` 的 `run_heartbeat_task` 重试循环
