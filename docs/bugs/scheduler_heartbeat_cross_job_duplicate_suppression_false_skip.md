# Bug: Heartbeat 跨 job 预览去重把不同标的误判为重复，导致真实触发被压成 noop 漏发

- 发现时间：2026-05-04 23:10 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：New

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=15576`
  - `job_id=j_9ee85d42`
  - `job_name=Cerebras IPO与业务进展心跳监控`
  - `executed_at=2026-05-04T22:31:43.364323+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 为 `【Cerebras IPO重大进展 | 检查时间: 2026-05-04 22:30 北京时间】...`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=15588`
  - `job_id=j_db12f27f`
  - `job_name=持仓重大事件心跳检测`
  - `executed_at=2026-05-04T23:01:27.260262+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - `delivered=0`
  - `detail_json.parse_kind=JsonTriggered`
  - `detail_json.duplicate_suppressed=true`
  - `detail_json.matched_preview` 却指向上一窗 `Cerebras IPO重大进展`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=15591`
  - `job_id=j_39a96b7a`
  - `job_name=ORCL 大事件监控`
  - `executed_at=2026-05-04T23:01:37.212021+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - `delivered=0`
  - `detail_json.parse_kind=JsonTriggered`
  - `detail_json.duplicate_suppressed=true`
  - `detail_json.matched_preview` 同样错误指向上一窗 `Cerebras IPO重大进展`
- 同一 actor 最近窗口对照：
  - `2026-05-04 22:00-22:02`：`ORCL` 为 `noop`、`持仓重大事件` 为 `execution_failed`
  - `2026-05-04 22:30-22:31`：`Cerebras IPO` 成功送达，`ORCL` 为 `execution_failed`、`持仓重大事件` 为 `noop`
  - `2026-05-04 23:00-23:01`：`Cerebras IPO` 自己回落为 `noop`，但 `ORCL` 与 `持仓重大事件` 都在本窗首次生成 `JsonTriggered` 后被跨 job 去重层压成 `noop`
- 相关已知缺陷对照：
  - [`scheduler_heartbeat_retrigger_duplicate_alerts.md`](./scheduler_heartbeat_retrigger_duplicate_alerts.md) 关注的是“旧事件重复推送”
  - 本单相反：去重层把不同标的、不同主题的真实新触发误判成重复，导致漏发，因此不是同一坏态

## 端到端链路

1. 同一 Feishu 目标 `+8613867793336` 在 `2026-05-04 22:30` 窗口先收到 `Cerebras IPO与业务进展心跳监控` 的正式送达。
2. 约 30 分钟后，`2026-05-04 23:00` 窗口里：
   - `持仓重大事件心跳检测` 生成了 `JsonTriggered`
   - `ORCL 大事件监控` 也生成了 `JsonTriggered`
3. 这两条本应独立判定的 heartbeat 没有按各自内容走送达，而是在落库前被“最近已送达 preview 去重”拦下。
4. 去重命中的 `matched_preview` 不是同一 job、同一 ticker、同一主题，而是上一窗完全不同主题的 `Cerebras IPO重大进展`。
5. 最终数据库把两条真实触发都记成 `noop + skipped_noop`，用户没有收到原本该送达的 ORCL / 持仓提醒。

## 期望效果

- 跨 job 去重只能拦截“同一事实被改写重发”的样本，不能把不同 ticker、不同事件类型的 heartbeat 互相吞掉。
- 若本轮 `parse_kind=JsonTriggered` 的正文主题与最近已送达 preview 明显不同，应继续送达，而不是压成 `duplicate_suppressed`。
- 去重命中时，至少应保留可审计的“本轮原始触发摘要”，避免台账只剩一个空的 `noop` 终态。

## 当前实现效果

- `2026-05-04 23:00` 最新真实窗口里，`ORCL 大事件监控` 与 `持仓重大事件心跳检测` 都先通过模型侧触发，`detail_json.parse_kind` 明确为 `JsonTriggered`。
- 但最终台账没有记录原始触发正文，只剩 `duplicate_suppressed=true` 和一个错误的 `matched_preview=Cerebras IPO重大进展`。
- 这说明当前去重逻辑在 actor 级跨 job 复用时过于宽松，已经从“抑制旧事件重复提醒”退化成“误吞不同主题的新提醒”。

## 用户影响

- 用户会漏收原本应该送达的真实 heartbeat，影响提醒链路的完整性，不只是噪音问题。
- 这不是 `P3` 质量波动：损害点是实际漏发，用户无法感知本窗已触发的 ORCL / 持仓提醒。
- 之所以定级为 `P2` 而不是 `P1`，是因为当前证据集中在单个 actor 的两条 heartbeat 被误抑制，尚未证明跨大面积用户扩散或造成数据安全事故。

## 根因判断

- 当前 actor 级 heartbeat 去重很可能只做了宽松的 preview token overlap，没有建立足够强的 ticker / 事件主体一致性校验。
- 因此当上一窗 `Cerebras IPO` preview 足够长、包含大量通用财经词和事件模板词时，下一窗 `ORCL` / `持仓事件` 的触发正文可能被错误判成“高度相似”。
- 现有台账在 `duplicate_suppressed` 路径下也没有保留本轮原始触发摘要，导致巡检时只能从 `parse_kind=JsonTriggered` 与 `matched_preview` 的矛盾间接反推漏发。

## 下一步建议

- 先收紧 heartbeat 跨 job 去重键：至少要求 ticker、主题实体或事件主语一致，再允许进入 preview overlap 去重。
- 在 `duplicate_suppressed` 终态里补记本轮原始 `raw_preview` / `deliver_preview` 摘要，避免漏发后完全丢失审计线索。
- 增加回归样本：上一窗 `Cerebras IPO` 已送达时，下一窗 `ORCL +5%` 与 `持仓 TEM/ORCL` 触发不得被压成 `duplicate_suppressed`。
