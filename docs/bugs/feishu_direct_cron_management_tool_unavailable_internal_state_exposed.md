# Bug: Feishu 直聊定时任务管理工具未暴露且外露沙盒存储细节

- **发现时间**: 2026-06-11 03:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
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
- `data/sessions.sqlite3` -> `session_messages`
  - 2026-06-11 23:03 CST 巡检窗口：2026-06-11 19:02-23:02 CST。
  - 同一 session `Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1` 在 20:55 CST 再次出现定时任务创建请求失败样本，assistant `ordinal=354` 于 `2026-06-11T20:55:37.563050+08:00` 正常落库 final。
  - final 写出：`本轮未暴露可执行的定时任务创建接口，因此这两个推送任务没有成功创建`，随后只整理了用户希望创建的 08:30 / 20:00 推送规格。
  - 本次没有继续外露 `data/cron_jobs` / `data/sessions.sqlite3` 等存储路径，但主链路仍未能真实创建用户要求的定时任务。
- 同窗摘要：
  - 2026-06-11 19:02-23:02 CST `data/sessions.sqlite3` 有 39 个 user turn 与 39 个 assistant final，最近 Feishu direct / scheduler 会话均以 assistant final 收口。
  - 普通 scheduler 33 条均为 `completed + sent + delivered=1`；异常仍集中在 Feishu direct 任务管理工具未暴露 / 未注入，不是全局 scheduler 停摆。
- `data/sessions.sqlite3` -> `session_messages`
  - 2026-06-15 23:04 CST 巡检窗口：2026-06-15 19:03-23:04 CST。
  - `session_id=Actor_feishu__direct__ou_5fba037d8699a7194dfe01a1fda5ced052` 在 21:31 CST 收到用户请求：`请每两周检查一次 PKE，只有接近合理买入区才提醒我，谢谢。`
  - assistant 于 21:33 CST 回复：已把 PKE 条件写入长期画像，但“本轮尚未正式创建自动推送任务”，并要求用户补充每两周的具体检查时间。
  - 用户于 21:42 CST 补齐：`隔周一 09:00 可以。另外，请只提示买入机会，不用提示风险复查。`
  - assistant 于 21:43 CST 正常落库 final，但回复写出：`自动定时任务注册工具没有暴露出来，所以我不能确认任务已经正式创建成功`。
  - 回复只更新画像 / 条件，没有返回真实任务 ID，也没有确认创建每两周 PKE 自动检查任务。
  - 同窗 `data/sessions.sqlite3` 有 45 个 user turn 与 45 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 34 条均为 `completed + sent + delivered=1`，说明异常仍集中在 Feishu direct 任务创建入口。

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
- 2026-06-11 20:55 CST 复发样本已经没有继续暴露本地路径 / SQLite 表名，但仍明确写出“定时任务创建接口未暴露”，并且没有真实创建用户要求的两个定时任务。
- 2026-06-15 21:43 CST 复发样本继续写出“自动定时任务注册工具没有暴露出来”，并在用户已经补齐检查时间后仍未创建 PKE 双周提醒任务；该样本没有继续外露本地路径或 SQLite 表名，但 direct 定时任务创建主链路仍不可用。
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

## 复发记录

- 2026-06-11 23:03 CST 补充同根复发证据：
  - 20:55 CST 同一 Feishu direct session `Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1` 再次处理用户创建 08:30 / 20:00 推送任务请求。
  - assistant 回复 `本轮未暴露可执行的定时任务创建接口，因此这两个推送任务没有成功创建`，只整理任务规格，没有真实创建任务。
  - 本次没有继续外露本地路径或 SQLite 表名，说明用户态文案有所收敛；但 Feishu direct 定时任务创建主链路仍不可用，因此状态保持 `P2 / New`。非 P1，不创建 GitHub Issue。

## 修复记录

- 2026-06-21 03:06 CST：本轮继续修复 Feishu direct 定时任务管理回复的确定性收口，而不再只依赖 prompt 文案约束。
  - `crates/hone-channels/src/response_finalizer.rs` 现在会在模型最终回复退化成过渡句或被共享净化层统一改写成“定时任务管理暂时不可用，请稍后再试”时，优先从真实 `cron_job` 工具结果恢复用户态答复。
  - 新增覆盖面包括：
    - `cron_job(action="list")`：直接把任务列表恢复成用户可读摘要，避免调用成功后仍只回“我先查一下”或“工具没暴露”。
    - `cron_job(action="remove")` 且 `needs_confirmation=true`：恢复成明确的删除确认提示或候选任务列表，而不是暴露内部工具状态。
    - `cron_job(action="add"/"update")`：即使原始模型文案被净化层压成通用失败提示，只要真实工具已成功写入，最终仍回写真实任务名、时间与任务 ID。
  - 新增回归：
    - `finalize_agent_response_recovers_cron_job_list_from_tool_result`
    - `finalize_agent_response_recovers_cron_job_remove_confirmation_from_tool_result`
    - `finalize_agent_response_recovers_cron_job_result_after_sanitization_strips_internal_copy`
  - 本轮验证：`cargo test -p hone-channels finalize_agent_response_recovers_cron_job_ --lib -- --nocapture`、`cargo test -p hone-channels finalize_agent_response_recovers_portfolio_confirmation --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。
  - 当前仍未重启 Feishu live 服务，也没有新的运行态会话样本，因此先按代码级 `Fixed` 记录；若部署当前代码后 Feishu direct 仍把已成功的 `cron_job` 调用收口成“工具没暴露/暂时不可用”，再按新样本重新打开。

- 2026-06-12 08:06 CST 代码级修复：
  - `crates/hone-channels/src/prompt.rs` 的默认定时任务策略明确要求列出、检查、创建、更新、取消或删除任务时必须调用真实 `cron_job` 工具完成，不能用沙盒目录、SQLite、会话历史或文件列表自查替代。
  - 同一策略补充故障边界：若真实 `cron_job` 工具不可用或调用失败，只能返回用户态“定时任务管理暂时不可用，请稍后再试”，禁止向用户输出 `工具未暴露`、`接口未暴露`、`cron_job / scheduled_task`、`data/cron_jobs`、`sessions.sqlite3`、`session_messages`、`session_metadata` 或“当前沙盒”等实现细节。
  - `crates/hone-channels/src/runtime.rs` 的共享 `sanitize_user_visible_output(...)` 补齐定时任务工具不可用文案改写和 cron / SQLite / session 存储自查句过滤，避免同类实现层诊断进入 Feishu direct 最终回复。
  - 本轮复核代码确认 Feishu direct 私聊仍通过 `ChatMode::Direct -> with_cron_allowed(true)` 进入 cron-enabled runner，`CronJobTool` 已支持 `list/add/update/remove`；本次修复收敛 prompt 执行约束与用户可见安全边界，不依赖当前机器 live 进程或生产日志。
  - 验证通过：`cargo test -p hone-channels sanitize_user_visible_output_rewrites_cron_tool_unavailable_copy --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_strips_cron_storage_self_inspection_copy --lib -- --nocapture`、`cargo test -p hone-channels resolve_prompt_input_maps_cron_enabled_flags_to_user_language --lib -- --nocapture`、`cargo test -p hone-channels sanitize_user_visible_output_ --lib -- --nocapture`。
  - 本轮未重启 Feishu 服务，也不使用当前机器运行态作为恢复证据；状态更新为代码级 `Fixed`，后续若部署当前代码后仍出现真实 `cron_job` 工具不可用，应基于新样本重新打开。

## 复发确认（2026-06-12 19:02 CST）

- 巡检窗口：2026-06-12 15:02-19:02 CST。
- `data/sessions.sqlite3` -> `session_messages` 显示同窗有 16 个 user turn 与 15 个 assistant turn；多数 Feishu direct 会话正常收口。
- `session_id=Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c`：
  - `2026-06-12T18:37:04.068615+08:00` 用户请求：每天早上 8 点发送昨日收盘总结并复盘持仓股。
  - `2026-06-12T18:37:55.822231+08:00` assistant 正常落库 final，但回复写出：`本轮没有可用的定时任务注册入口，因此不能直接完成自动创建`。
  - 回复只整理了任务规格建议，没有返回真实任务 ID，也没有完成用户要求的任务创建。
- 该样本晚于 2026-06-12 08:06 CST 代码级修复记录；虽然本轮没有继续外露本地路径、SQLite 表名或 `cron_job / scheduled_task` 裸工具名，但 Feishu direct 定时任务创建主链路仍不可用。
- 状态从 `Fixed` 调回 `New`。仍定级 `P2`：本轮证据覆盖单个 Feishu direct actor 的任务创建失败，普通 scheduler 同窗仍有 `completed + sent + delivered=1`，未见跨用户批量失败、错投、数据破坏或敏感信息泄露。非 P1，不创建 GitHub Issue。

## 复发补证（2026-06-12 23:02 CST）

- 巡检窗口：2026-06-12 19:02-23:02 CST。
- `data/sessions.sqlite3` -> `session_messages` 显示同窗有 42 个 user turn 与 42 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口，无新的 user-only 残留；普通 scheduler 34 条为 `completed + sent + delivered=1`。
- `session_id=Actor_feishu__direct__ou_5f9f2cd3505aab8fed0a6ffd582df285b1`：
  - `2026-06-12T20:00:39.195441+08:00` 用户重发持仓列表，并要求每天北京时间 20:00 整理持仓盘前美股要闻、宏观数据与评级变化后发送。
  - `2026-06-12T20:01:54.690960+08:00` assistant 正常落库 final，但回复写出：`当前环境没有可用的定时任务写入工具，所以我不能确认“每天20:00自动推送”已经创建成功`。
  - 回复称“关注列表已写入成功，共 12 个标的”，但没有返回真实任务 ID，也没有完成用户明确要求的每天 20:00 自动推送创建。
- 本次仍未继续外露本地路径、SQLite 表名或裸 `cron_job / scheduled_task` 工具名；用户态文案比 2026-06-11 初始样本有所收敛。
- 但该样本晚于 2026-06-12 08:06 CST 代码级修复记录，且晚于 18:37 CST 复发样本；说明 Feishu direct 定时任务创建主链路仍不可用。状态保持 `P2 / New`。非 P1，不创建 GitHub Issue。

## 复发补证（2026-06-15 23:04 CST）

- 巡检窗口：2026-06-15 19:03-23:04 CST。
- `data/sessions.sqlite3` -> `session_messages` 显示同窗有 45 个 user turn 与 45 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 34 条为 `completed + sent + delivered=1`。
- `session_id=Actor_feishu__direct__ou_5fba037d8699a7194dfe01a1fda5ced052`：
  - `2026-06-15T21:31:16.483718+08:00` 用户请求：每两周检查一次 PKE，只有接近合理买入区才提醒。
  - `2026-06-15T21:33:09.951835+08:00` assistant 要求补充检查时间，建议隔周一 09:00。
  - `2026-06-15T21:42:08.994357+08:00` 用户确认：隔周一 09:00，并只提示买入机会。
  - `2026-06-15T21:43:31.329261+08:00` assistant 正常落库 final，但回复写出：`自动定时任务注册工具没有暴露出来，所以我不能确认任务已经正式创建成功`。
- 本次未继续外露 `data/cron_jobs`、`data/sessions.sqlite3` 或裸 `cron_job / scheduled_task` 工具名；用户态文案较 2026-06-11 初始样本有所收敛。
- 但该样本发生在用户补齐任务创建条件后，仍没有真实任务 ID 或创建确认，说明 Feishu direct 定时任务创建主链路仍不可用。状态保持 `P2 / New`。非 P1，不创建 GitHub Issue。
