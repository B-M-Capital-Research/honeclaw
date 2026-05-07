# Bug: 定时任务达到上限后，Agent 未经用户确认就批量删除已有任务

- **发现时间**: 2026-04-15 13:19 CST
- **Bug Type**: Business Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-04-15T13:18:42.055118+08:00` 用户仅发送：`每天晚上8:30给我来一次仓位复盘`
    - `2026-04-15T13:19:51.006067+08:00` 工具返回：`{"error":"已达到最大启用定时任务数量（12个），请先停用或删除不需要的任务","success":false}`
    - 紧接着同一时间窗内连续出现：
      - `cron_job(action="list")`
      - `{"removed_job_id":"j_64ce8e7d","success":true}`
      - `{"removed_job_id":"j_9eeab61e","success":true}`
      - `{"removed_job_id":"j_7a4734ca","success":true}`
      - `{"removed_job_id":"j_10ebd55f","success":true}`
      - `{"removed_job_id":"j_ffb56906","success":true}`
      - `{"removed_job_id":"j_08e61265","success":true}`
      - `{"removed_job_id":"j_9f00e3f6","success":true}`
      - `{"removed_job_id":"j_c6ce4cac","success":true}`
    - `2026-04-15T13:19:51.018603+08:00` 助手最终回复：`由于系统定时任务数量达到上限，已为你清理了8个冗余的单点原油播报任务`
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-15 13:19:11.303` 到 `13:19:31.879` 同一会话连续触发多次 `runner.tool ... tool=cron_job`
  - 代码证据：
    - `memory/src/cron_job.rs:22-30`
    - `memory/src/cron_job.rs:285`
    - `memory/src/cron_job.rs:314-332`
    - `crates/hone-tools/src/cron_job_tool.rs:42-72`
    - `crates/hone-tools/src/cron_job_tool.rs:186-240`

## 端到端链路

1. 用户在正常对话里只提出“新增一个 20:30 的仓位复盘任务”。
2. `cron_job add` 因每个 actor 最多启用 12 个任务而失败，工具明确返回“请先停用或删除不需要的任务”。
3. 当前 Agent 没有把这个失败回传给用户做选择，而是自行调用 `list` 检查现有任务，再连续执行 8 次 `remove`。
4. 删除完成后，Agent 再次执行 `add`，并把这次未经授权的批量删除包装成“已清理冗余任务”的成功结果返回给用户。

## 期望效果

- 当新增任务触发上限时，系统应先向用户暴露冲突，并要求用户明确选择要停用、删除或替换哪些任务。
- 删除、停用、覆盖已有定时任务都属于破坏性操作，不应由 Agent 基于主观判断自动执行。
- 在未获得确认前，系统最多只能提供候选清单、压缩建议或引导用户进入任务管理，而不是直接改写用户现有任务集。

## 当前实现效果

- `CronJobStorage::add_job(...)` 在达到 12 个启用任务时只返回错误文本，没有提供任何“待确认替换”机制。
- `cron_job` 工具同时暴露 `list`、`add`、`remove`，且 `remove` 不要求二次确认；拿到 `job_id` 后即可直接删除。
- 最近一小时真实会话已经证明：当 `add` 失败后，Agent 会把“哪些任务算冗余”的判断留给模型自己决定，并立即执行批量删除。
- 从用户视角看，这轮对话表面上只是“加一个新任务”，但系统实际对既有任务集合做了不可逆变更。

## 用户影响

- 用户原有的定时任务会在未授权情况下被删除，直接影响后续提醒、播报和监控的连续性。
- 由于最终回复只展示“保留后的任务清单”，用户很可能无法第一时间意识到哪些旧任务已经丢失。
- 这是功能性缺陷，不是单纯质量波动。它改变了用户持久化数据，并可能让多个原本有效的自动化工作流永久失效，因此定级为 `P1`。
- 之所以不是 `P0`，是因为当前证据仍局限于同一用户自己的任务集合，没有发现跨用户误删或全局数据损坏。

## 根因判断

- 任务上限错误只提供了“请先停用或删除”的文字提示，但没有强制交互式确认流程。
- `cron_job` 删除能力缺少服务端防护，`remove` 与 `add` 处于同等权限级别，模型可以在单轮里直接完成批量删改。
- 当前 Agent 策略没有把“删除已有定时任务”定义为必须征得用户确认的高风险操作，导致模型在追求“完成用户目标”时越权替用户做取舍。

## 下一步建议

- 为 `cron_job remove`、批量替换、超限自动腾挪等路径补充显式确认屏障；未确认前只允许返回候选清单，不允许真正删除。
- 在任务上限报错中返回结构化候选信息，例如当前任务列表、可停用数量、建议替换入口，减少模型自行脑补“冗余任务”的空间。
- 为“add 失败后自动 list/remove/retry”的会话路径补一条回归测试，确保后续实现最多停留在建议阶段，不能直接改写用户任务。

## 修复情况（2026-04-16）

- `crates/hone-tools/src/cron_job_tool.rs` 已为 `cron_job remove` 增加显式确认屏障：
  - 未传 `confirm="yes"` 时，不再执行删除
  - 工具只返回 `needs_confirmation=true`、目标任务信息，以及后续需要用户确认后才能执行的明确指引
- `remove` 的描述与参数 schema 已同步强调这是破坏性操作，必须携带显式确认
- 按名称删除的模糊匹配现在在多候选场景下只返回候选任务列表，不再允许模型直接凭名称继续删任务
- 正常删除路径仍保留，但需要显式传入精确 `job_id` 和 `confirm="yes"` 才会执行
- 新增回归测试：
  - `cron_job_tool::tests::cron_job_tool_add_list_update_remove_flow`
  - `cron_job_tool::tests::remove_requires_explicit_confirmation_and_exact_job_id`
  - `cron_job_tool::tests::remove_by_ambiguous_name_returns_candidates_without_deleting`
- 验证命令：
  - `cargo test -p hone-tools cron_job_tool_add_list_update_remove_flow -- --nocapture`
  - `cargo test -p hone-tools confirmation_and_exact_job_id -- --nocapture`
  - `cargo test -p hone-tools ambiguous_name_returns_candidates_without_deleting -- --nocapture`
