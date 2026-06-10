# Bug: Feishu 直聊定时任务管理工具未暴露且外露沙盒存储细节

- **发现时间**: 2026-06-11 03:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检时间窗：2026-06-10 23:02-2026-06-11 03:02 CST。
  - 本窗有 11 个 user turn 与 11 个 assistant final，Feishu direct / scheduler 会话均有 assistant 收口；普通 scheduler 4 条均 `completed + sent + delivered=1`，未见普通 scheduler 发送失败。
  - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1` 在 23:43-00:09 CST 连续 5 次请求管理定时任务：
    - 23:43 用户要求“取消每天早上和晚上的自动任务，并把剩下启用的定时任务列给我”，assistant 回复“没有拿到可操作的定时任务工具，无法真实执行取消操作”，且不能可靠列出剩余任务。
    - 23:46 用户要求“列出我所有的定时任务”，assistant 回复“真实 cron 列表工具这一轮仍未暴露”，并外露 `session_messages`、`session_metadata`、`data/sessions.sqlite3` 等存储表 / 文件口径。
    - 23:50 用户要求“列出我当前设定的任务”，assistant 再次回复“定时任务列表工具当前未暴露”，并外露“当前目录只看到 data/sessions.sqlite3”“sessions.sqlite3 当前没有可查询的任务表”。
    - 23:54 用户要求“取消每个交易日8:30和20:30的自动任务”，assistant 回复“工具列表里没有 cron_job / scheduled_task 的 list 或 remove 接口”，取消动作未完成。
    - 00:08 用户要求每天 08:30 和 20:00 创建宏观数据推送任务，assistant 回复 `data/cron_jobs` 是空目录、没有写入接口，不能真实创建任务。
  - 同窗 assistant final 污染扫描命中多条用户可见内部实现词：`data/cron_jobs`、`data/sessions.sqlite3`、`session_messages`、`session_metadata`、`cron_job / scheduled_task`、`当前沙盒`。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 同窗普通 scheduler 仍有 4 条 `completed + sent + delivered=1`，heartbeat 有 72 条 `noop + skipped_noop + delivered=0`、30 条 `execution_failed + skipped_error + delivered=0` 与 2 条 `completed + sent + delivered=1`。
  - 这说明全局 scheduler 执行台账仍在推进，本轮异常集中在 Feishu direct 的定时任务管理工具暴露 / 调用链路，而不是整个调度器完全停摆。

## 端到端链路

1. Feishu direct 用户请求列出、取消或创建个人定时任务。
2. 直聊 runner 应暴露 cron task list / create / remove 等任务管理能力，或调用后端安全接口完成操作。
3. 本轮直聊环境没有暴露可执行的定时任务管理工具。
4. assistant 只能从沙盒目录和 SQLite 表结构角度推断，最终向用户回复“不能真实执行”，并外露内部存储路径 / 表名 / 工具接口名。
5. 用户连续 5 次仍无法完成定时任务管理。

## 期望效果

- Feishu direct 应能对同 actor 的定时任务执行可靠的 list / create / update / remove。
- 若任务管理后端暂时不可用，应返回安全、用户态的失败说明，并保留可审计错误分类。
- 最终用户可见文本不应包含沙盒目录、SQLite 表名、内部工具接口名或“工具未暴露”这类实现层诊断。

## 当前实现效果

- 取消、列出、创建定时任务均未完成。
- 回复虽然没有谎称成功，但连续暴露内部运行状态：`data/cron_jobs` 空目录、`data/sessions.sqlite3`、`session_messages`、`session_metadata`、`cron_job / scheduled_task` 工具名与“当前沙盒”。
- 普通 scheduler 仍在执行，说明当前不是全局 cron loop 停摆，而是 direct 管理入口不可用或未随该会话注入。

## 用户影响

- 这是功能性 bug，不是单纯质量问题：用户明确要求管理自动任务，但系统无法列出、取消或创建任务。
- 影响范围目前证据覆盖单个 Feishu direct actor 的连续多轮真实会话；没有跨用户批量失败、错投、数据破坏或敏感凭据泄露证据，因此定级为 P2 而不是 P1。
- 伴随的内部存储 / 工具状态外露会降低可信度，并可能误导用户以为可以通过本地目录或数据库判断任务真实状态。

## 根因判断

- 直接证据显示 direct runner 的可用工具集中没有任务管理 list / create / remove 能力，或能力发现失败后没有转入稳定的后端管理接口。
- 与 `feishu_actor_scope_cron_portfolio_empty.md` 不同：那条缺陷是权威 Cron / portfolio 数据存在但 actor data root 读成空；本轮样本没有证明权威任务文件存在且被读空，而是管理工具本身未暴露，assistant 退化到沙盒 / SQLite 自查。
- 与 `feishu_direct_cron_list_enabled_flag_exposed.md` 不同：那条缺陷主链路成功列出任务，仅外露 `enabled=true` 字段；本轮主链路未完成，且外露的是存储和工具链状态。
- 与 `feishu_direct_empty_reply_false_success.md` 不同：本轮不是空回复伪成功，也没有已发生的 cron_job 副作用被 fallback 遮蔽；本轮明确没有完成任务管理操作。

## 下一步建议

- 复核 Feishu direct prompt / MCP bridge / tool registry 中 cron management 工具注入条件，确认普通用户是否应具备安全的 list / create / remove 能力。
- 如果工具因权限、额度或后端故障不可用，统一返回用户态“任务管理暂时不可用，请稍后重试”类文案，并在内部日志记录 `failure_kind`，不要把沙盒目录、SQLite 表名或工具接口名发给用户。
- 增加 Feishu direct 回归样本：用户要求列出 / 取消 / 创建定时任务时，最终回复不得包含 `data/cron_jobs`、`data/sessions.sqlite3`、`session_messages`、`session_metadata`、`cron_job / scheduled_task`、`工具未暴露`，且成功路径必须给出真实任务 ID 或状态。

