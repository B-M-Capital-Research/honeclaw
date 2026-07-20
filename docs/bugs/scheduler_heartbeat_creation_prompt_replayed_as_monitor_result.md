# Bug: Heartbeat 已创建监控任务仍反复输出“无法创建定时任务”

## 发现时间

- 2026-07-11 19:01 CST

## Bug Type

- Business Error

## 严重等级

- P2

## 状态

- New

## 修复进展

- `2026-07-20 11:01-15:05 CST` 运行态复核确认同根继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-20`
    - 11:30 CST `NVDA 关键事件心跳提醒` 已作为现有 heartbeat job 周期触发，deliver preview 却把上游 heartbeat 配置 / 系统级文本当作“配置文本与近期监控说明”，反问用户“你现在想做什么”，没有执行 NVDA 关键事件判断。
    - 12:00 CST `存储板块关键事件心跳提醒` 已作为现有 heartbeat job 周期触发，deliver preview 写成“这是市场事件监控的设置/确认问题”，并外露 `cron_job` 工具、事件推送优先级等设置说明，而不是执行存储板块事件监控。
    - 12:00 CST `heartbeat_绿田机械基本面跟踪` 已作为现有 heartbeat job 周期触发，deliver preview 继续写出 `cron_job` 工具不在可用函数列表、`notification_prefs` 替代方案等任务管理漂移文本。
  - 会话质量对照：
    - 同窗 `data/runtime/logs/web.log.2026-07-20` 仍有 686 条 `[HeartbeatDiag]` 与 83 条 `deliver job_id`，说明 heartbeat live 仍在运行；`cron_job_runs.max(executed_at)` 仍停在 `2026-07-19T13:31:15.040172+08:00`，本地 cron mirror 继续失真。
  - 判断：
    - 最新样本仍是已创建 heartbeat job 的执行意图被“创建 / 配置 / 能力介绍 / prompt 识别”语义污染，而不是具体市场监控判断。
    - 这是功能性监控链路缺陷，定级仍为 P2；当前影响 heartbeat 任务输出和信噪比，未见全渠道停摆、跨用户错投、数据破坏或敏感信息泄露，因此不升级 P1，不创建 GitHub Issue。

- `2026-07-20 03:02-07:02 CST` 运行态复核确认同根继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-19`
    - 03:30 CST `SIVE POET/Nokia/1.6T DFB 心跳检测` 已作为现有 heartbeat job 周期触发，deliver preview 却把任务当作“系统提示词或配置说明”，反问用户是否要修改 / 新建 / 查看心跳监控，而不是执行 SIVE / POET / Nokia / 1.6T DFB 事件判断。
    - 03:30 CST `持仓重大事件心跳提醒` 已作为现有 heartbeat job 周期触发，却输出 Hone 能力介绍，讲“美股事件引擎”“个性化研究档案”等产品能力，而不是检查持仓重大事件。
    - 07:00 CST `光模块板块关键事件心跳提醒` raw preview 继续把 heartbeat JSON 合同和触发规则识别成“system prompt injection test”，落成 `JsonNoop`，没有执行光模块板块监控判断。
  - 会话质量对照：
    - 同窗 `data/runtime/logs/web.log.2026-07-19` 仍有 703 条 `[HeartbeatDiag]` 与 96 条 `deliver job_id`，说明 heartbeat live 仍在运行；`cron_job_runs.max(executed_at)` 仍停在 `2026-07-19T13:31:15.040172+08:00`，本地 cron mirror 继续失真。
  - 判断：
    - 最新样本仍是已创建 heartbeat job 的执行意图被“创建 / 配置 / 能力介绍 / prompt 识别”语义污染，而不是具体市场监控判断。
    - 这是功能性监控链路缺陷，定级仍为 P2；当前影响 heartbeat 任务输出和信噪比，未见全渠道停摆、跨用户错投、数据破坏或敏感信息泄露，因此不升级 P1，不创建 GitHub Issue。

- `2026-07-20 03:02 CST` 运行态复核确认代码级 `Fixed` 后同根复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-07-19`
    - 03:00 CST `heartbeat_绿田机械基本面跟踪` 已作为现有 heartbeat job 周期触发，runner raw preview 却写出 `cron_job` 工具当前不在可用函数列表中、`hone_admin` 技能仅含重启与配置查看能力，不含定时任务创建`。
    - 同一轮落成 `parse_kind=PlainTextTriggered`，并生成 350 字 deliver preview，向用户解释工具不可用、建议手动设置 / 调整 `notification_prefs`，而不是执行 605259.SH 基本面 heartbeat 判断。
  - 会话质量对照：
    - 2026-07-19 23:02-2026-07-20 03:02 CST `data/sessions.sqlite3` 只有 3 条 scheduler user turn / 3 条 assistant final，均来自 `AAOI/TEM/RKLB 每日动态监控`，同一 session 以 assistant 收口；未见直聊 user-only 残留、空回复、错投或 assistant final 原始错误外泄。
    - 同窗 `cron_job_runs.max(executed_at)` 仍停在 `2026-07-19T13:31:15.040172+08:00`，但 runtime 日志有 716 条 `[HeartbeatDiag]`、95 条 `deliver job_id` 和 49 条 `duplicate_suppressed`，说明 heartbeat live 仍在运行，本地 cron mirror 继续失真。
  - 判断：
    - 该样本仍是已创建 heartbeat job 的执行意图被“创建 / 管理定时任务”语义污染，只是话术从“无法创建定时任务”变成了“`cron_job` / `hone_admin` 工具不可用”。
    - 这是功能性监控链路缺陷，定级仍为 P2；当前影响单个 heartbeat 任务输出和信噪比，未见全渠道停摆、跨用户错投、数据破坏或敏感信息泄露，因此不升级 P1，不创建 GitHub Issue。

- `2026-07-15 03:04 CST` 代码级修复补强，状态更新为 `Fixed`：
  - `crates/hone-channels/src/scheduler.rs`
    - `heartbeat_management_drift_message(...)` 扩展识别“无法建立”“自动循环”“自动推送”“推送流水线”“循环监控”等残留任务治理话术，覆盖 2026-07-12 / 07-13 复发样本里的“无法建立每30分钟自动循环执行的监控任务”与“自动推送流水线”文案。
    - duplicate suppression 同步跳过这类“无法建立自动监控”旧坏基线，避免真实监控结论再次被管理漂移文本误压成重复。
  - 新增 / 复跑回归：
    - `cargo test -p hone-channels heartbeat_management_drift_message_with_unable_to_establish_copy_is_suppressed --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_duplicate_preview_match_ignores_unable_to_establish_management_baseline --lib -- --nocapture`
  - 本轮未重启 live runtime；先按代码级 `Fixed` 回写，待后续巡检继续复核是否还有其它未覆盖的话术变体。

- `2026-07-13 11:04-15:01 CST` 运行态复核确认同一链路继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-13`
    - 12:30 CST `中际旭创关键事件心跳提醒` 已作为 heartbeat job 周期触发，但 deliver preview 写出“我无法建立每30分钟自动循环执行的监控任务”，并把替代方案写成“你现在发一句查一下中际旭创”。
    - 14:30 CST `SIVE POET/Nokia/1.6T DFB 心跳检测` 同样已经周期触发，但 deliver preview 写出“我无法创建定时心跳任务、循环监控或自动推送流水线”。
  - 会话质量对照：
    - 同窗 `data/sessions.sqlite3` 只有 3 组 user / assistant direct 或 scheduler final，均正常收口；本地 `cron_job_runs` 仍停滞，因此继续以 runtime web log 判断 heartbeat 运行态。
  - 判断：
    - 该复发仍是同一根因链路：已创建 heartbeat job 的执行意图被“创建/设置自动监控”请求语义污染。
    - 这是功能性监控链路缺陷，定级仍为 P2；当前证据覆盖 heartbeat 子链路，未见全渠道停摆、错对象投递、数据安全泄露或 P1 级全局任务丢失，因此不升级为 P1，不创建 GitHub Issue。

- `2026-07-12 15:01 CST` 运行态复核确认同一链路继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-07-12`
    - 12:00 CST `美股黄金坑信号心跳检测` 已作为 heartbeat job 被 scheduler 周期触发，但 raw preview 仍把任务理解为“用户要求每 30 分钟监控，我不能创建自动化任务”，而不是执行已存在的监控判断。
    - 同一轮 duplicate suppression 再次匹配旧坏基线：`我已经多次说明：无法创建30分钟自动化心跳监控任务`，最终压成未发送。
  - 会话质量对照：
    - 11:00-15:01 CST `data/sessions.sqlite3` 按真实 `timestamp` 没有新增 user / assistant 消息；`session_messages.imported_at` 在 12:33 CST 推进的是 2026-03/05 旧会话重导入。本地 `cron_job_runs` 仍停在 2026-07-10 14:01 CST，因此当前 heartbeat 运行态仍以 runtime web log 为主。
    - 最近四小时非文档提交 `6e688921`、`afda13ba`、`60ef12c8`、`cea93f67`、`6d5075a4` 集中在 public mobile navigation、Apple release checksum、v0.14.0 release、CLI probe stream reset 与 public tool-assisted replies，未改变本缺陷判断。
  - 判断：
    - 该复发仍是同一根因链路：已创建 heartbeat job 的执行意图被“创建/设置自动监控”请求语义污染，且旧“无法创建”坏基线继续参与 duplicate suppression。
    - 这是功能性监控链路缺陷，定级仍为 P2；当前证据覆盖 heartbeat 子链路，未见全渠道停摆、错对象投递、数据安全泄露或 P1 级全局任务丢失，因此不升级为 P1，不创建 GitHub Issue。

- `2026-07-12 11:01 CST` 运行态复核确认代码级修复后仍复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-07-12`
    - 08:00 CST `美股黄金坑信号心跳检测` 已作为 heartbeat job 被 scheduler 触发，但 raw preview 仍把任务理解为“用户想让我每 30 分钟创建市场监控”，deliver preview 写出“当前无法创建30分钟自动化心跳监控任务”；随后 duplicate suppression 匹配旧“无法创建30分钟自动化心跳监控任务”基线，最终未发送。
    - 11:00 CST `中际旭创关键事件心跳提醒` 同样已经作为 heartbeat job 周期触发，但 deliver preview 写出“当前系统无法建立‘每30分钟自动循环’的自动监控”；matched preview 又命中“当前系统工具链中不存在 `cron_job` 类型的任务创建工具，无法以‘每 30 分钟检查一次’为周期建立自动循环监控”。
  - 会话质量对照：
    - 07:01-11:01 CST `data/sessions.sqlite3` 按真实 `timestamp` 新增 2 个 user turn / 2 条 assistant final，均为 Feishu scheduler 文章跟踪任务正常收口；本地 `cron_job_runs` 仍停在 2026-07-10 14:01 CST，因此当前 heartbeat 运行态仍以 runtime web log 为主。
    - 最近四小时非文档提交 `6339c511`、`7cdbb12b` 集中在移动端手势 / 分享卡片与持久化日历图片服务，未改变本缺陷判断。
  - 判断：
    - 该复发仍是同一根因链路：已创建 heartbeat job 的执行意图被“创建/设置自动监控”请求语义污染，且旧“无法创建”坏基线继续参与 duplicate suppression。
    - 这是功能性监控链路缺陷，定级仍为 P2；当前证据覆盖 heartbeat 子链路，未见全渠道停摆、错对象投递、数据安全泄露或 P1 级全局任务丢失，因此不升级为 P1，不创建 GitHub Issue。

- `2026-07-12 03:04 CST` 代码级修复完成，状态更新为 `Fixed`：
  - `crates/hone-channels/src/scheduler.rs`
    - heartbeat prompt 新增执行期约束：即使 `task_prompt` 保留“帮我创建/设置/每30分钟监控”措辞，也必须解释为“已有 heartbeat 任务的执行说明”，不得把本轮运行当成新的创建请求。
    - heartbeat 出站新增 `heartbeat_management_drift_message(...)` 检测；若模型返回“无法创建定时任务 / 不能设置监控 / 第三次提出创建”这类任务治理残留话术，即使表面是 `triggered` 消息，也会在投递前压回 `noop`，不再污染用户可见提醒。
    - duplicate suppression 会跳过这类“创建失败/任务治理”旧基线，避免真实市场判断再次被“无法创建”历史文本误压成未发送。
  - 新增 / 复跑回归：
    - `cargo test -p hone-channels heartbeat_management_drift_message_is_suppressed --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_duplicate_preview_match_ignores_management_drift_baseline --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_prompt_treats_creation_wording_as_existing_monitor --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_prompt_ --lib -- --nocapture`
    - `cargo test -p hone-channels heartbeat_duplicate_preview_match_ --lib -- --nocapture`
    - `cargo check -p hone-channels --tests`
    - `git diff --check`
  - 本轮没有重启 live runtime；当前先按代码级 `Fixed` 回写，待后续 `bug` 巡检结合真实 heartbeat 窗口继续复核是否仍有旧 prompt 残留或其它独立根因。

## GitHub Issue

- 无，当前不是 P1。

## 证据来源

- `data/runtime/logs/web.log.2026-07-11`
  - 15:00-19:00 CST `美股黄金坑信号心跳检测` 每 30 分钟继续被 scheduler 触发，说明系统侧已经存在并运行该 heartbeat job。
  - 15:30 CST raw preview 仍把任务理解成“用户想让我每 30 分钟创建市场监控”，随后按 `JsonNoop` 跳过。
  - 16:30 CST 同 job 输出自然语言市场判断后落成 `JsonMalformed + execution_failed`，本轮不发送。
  - 18:00 CST 同 job deliver preview 给出市场判断，但 duplicate suppression 匹配到旧的“无法创建自动化心跳监控”文本，最终未发送。
  - 19:00 CST 同 job deliver preview 直接写出“这是你第三次提出建立每30分钟自动化心跳监控的请求，当前无法创建此类定时任务”，而不是执行已创建监控的市场条件判断。

## 端到端链路

1. 用户曾要求创建“美股黄金坑信号”类 30 分钟心跳监控。
2. 系统已经产生并周期触发 `美股黄金坑信号心跳检测` heartbeat job。
3. heartbeat runner 把 job prompt 送入 function-calling LLM。
4. LLM 多次把 prompt 当成“创建定时任务请求”而不是“执行已存在监控任务”。
5. 出站层在自由文本、malformed JSON、duplicate suppression 和 skipped noop 之间漂移，用户无法稳定收到该 job 的有效监控结果。

## 期望效果

- 已创建的 heartbeat job 每次触发时只执行监控判断。
- 如果当前条件未触发，应返回稳定结构化 `noop`，并且不要给用户发送“无法创建定时任务”。
- 如果条件触发，应发送与监控条件相关的提醒正文。
- job prompt 应保存为可执行监控说明，而不是保留用户最初的“帮我创建/设置”请求语义。

## 当前实现效果

- 同一个已存在的 heartbeat job 在真实运行中仍反复解释为“创建自动化监控请求”。
- 部分窗口输出“无法创建自动化心跳监控 / 当前无法创建此类定时任务”，与 job 已被周期触发这一事实矛盾。
- 该输出还会进入 duplicate suppression 基线，导致后续真实市场判断文本被旧“无法创建”基线压成未发送。

## 用户影响

- 用户以为已经创建的 30 分钟监控不会稳定提供监控结果。
- 该问题影响单个 heartbeat 任务的核心用途：周期检查市场回撤/买点条件。
- 这是功能性监控链路缺陷，定级 P2；当前证据集中在一个 job，未见全渠道停摆、错对象投递、数据安全泄露或 P1 级全局任务丢失，因此不升级为 P1。

## 根因判断

- 初步判断是 job 创建 / prompt 持久化边界没有把“创建请求”规范化为“执行请求”，导致 runner 后续周期执行时仍收到用户原始意图。
- duplicate suppression 只基于近似文本匹配，可能把“无法创建”这类错误基线当成同 job 的历史结果，进一步压制后续有效检查文本。
- 该根因不同于通用 heartbeat JSON 结构化退化：即使解析层完全稳定，job prompt 仍可能执行错误任务。

## 下一步建议

- 后续 `bug` 巡检优先复核 `美股黄金坑信号心跳检测` 是否仍有旧 prompt 残留；若 runtime 仍把任务当创建请求，再把问题下沉到 heartbeat job 创建/持久化时的 prompt 规范化或迁移工具。
- 若其它 heartbeat job 也复发“无法创建 / 不能设置 / 已配置监控”类话术，应复用本次 `management_drift` 路径继续扩展样本，而不是新建重复缺陷。
