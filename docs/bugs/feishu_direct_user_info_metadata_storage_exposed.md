# Bug: Feishu 直聊用户信息汇总外露渠道元数据与本地存储状态

- 发现时间：2026-06-13 15:03 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检时间窗：2026-06-13 11:03-15:03 CST。
  - 同窗有 12 个 user turn 与 12 个 assistant turn，最近 Feishu direct / scheduler 会话均以 assistant 收口；普通 scheduler 2 条为 `completed + sent + delivered=1`。
  - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`。
  - user `ordinal=369` / `2026-06-13T14:11:54+08:00`：用户要求 `列出当前你掌握的所有的用户信息`。
  - assistant `ordinal=370` / `2026-06-13T14:13:10+08:00`：final 正常落库并投递，但用户可见正文包含：
    - `当前会话 ID：Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`
    - `我能看到飞书 open_id、chat_id、手机号等元数据`
    - `当前工作区里存在公司画像目录：公司画像公司画像`
    - `当前工作区可见：data/sessions.sqlite3 / company_profiles 目录 / data/cron_jobs 目录 / data/portfolio 目录 / data/notif_prefs 目录 / uploads 目录`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 同窗 heartbeat 新增 70 条 `noop + skipped_noop + delivered=0`、33 条 `execution_failed + skipped_error + delivered=0` 与 1 条 `completed + sent + delivered=1`。
  - heartbeat 失败分布为：`heartbeat 输出不是结构化 JSON` 31 条、未知状态 1 条、OpenAI-compatible context window exceeded 1 条，仍落在既有 heartbeat 结构化 / context overflow 文档范围。
- 最近四小时无非文档代码提交。

## 端到端链路

1. Feishu direct 用户请求系统列出当前掌握的用户信息。
2. 直聊 runner 从会话上下文、任务列表、历史投资偏好和本地可见运行状态中组织回答。
3. final 成功生成并发送给用户。
4. 回复在“身份与会话”“本地可见长期画像目录”“当前本地文件状态”段落中，把渠道元数据、会话 ID、本地 SQLite / cron / portfolio / notif storage 目录状态当作用户信息输出。
5. 用户拿到的不是纯业务画像 / 可管理个人资料，而是混入内部渠道标识与本地运行时存储口径的结果。

## 期望效果

- 用户要求查看“你掌握的用户信息”时，回复应只列出用户可理解、可验证、可更正的业务资料，例如投资偏好、关注标的、持仓摘要、定时任务摘要和信息来源边界。
- 对 Feishu `open_id`、`chat_id`、手机号 metadata、内部 session id、本地 SQLite 文件、cron / portfolio / notif storage 目录等实现细节，应默认不进入用户可见 final。
- 如果需要做数据透明度说明，也应使用产品化措辞，例如“我会使用当前飞书会话的身份信息来区分你的任务和历史记录”，而不是列出内部字段名或本地文件状态。

## 当前实现效果

- 回复完成了用户请求的大部分业务意图：整理了投资偏好、定时任务、宏观指标、历史持仓和关注标的。
- 同一回复也把内部渠道字段名、当前会话 ID 和本地运行时存储目录状态作为“用户信息”输出。
- 回复还出现 `公司画像公司画像` 这类重复占位词，说明公司画像落点说明仍会退化，但该现象已由既有 `web_company_profile_relative_path_exposed.md` 跟踪；本单只登记更宽的用户信息汇总外露链路。

## 用户影响

- 这是用户信息 / 隐私边界上的功能性缺陷，不只是措辞偏好：用户请求个人信息汇总时，系统把本不应面向终端用户公开的渠道元数据和本地存储状态混入结果。
- 当前证据没有显示跨用户数据泄露、完整手机号外泄、token / 凭据外泄、错投或消息链路失败；用户请求也来自同一 Feishu direct 会话。
- 因此定级为 `P2`：影响信息边界与信任，但未达到 `P1` 的跨用户、大面积、凭据或核心投递链路事故。

## 根因判断

- 初步判断是 Feishu direct answer 阶段把 runtime / session metadata、工作区文件状态和用户业务画像混在同一“可用信息”集合里，没有区分“内部可见调试状态”与“用户可见个人资料”。
- 共享 `sanitize_user_visible_output(...)` 现有规则已覆盖部分本机绝对路径、cron 工具不可用、skill 文件状态和 `data_fetch` 外露形态，但没有覆盖“列出用户信息”场景中自然语言提及 `open_id`、`chat_id`、`手机号 metadata`、`当前会话 ID`、`data/sessions.sqlite3`、`data/portfolio`、`data/notif_prefs` 等字段 / 目录。
- 与 `feishu_direct_cron_management_tool_unavailable_internal_state_exposed.md` 不同：本轮定时任务管理主链路已能列出 / 修改任务；外露发生在用户信息汇总场景，而不是 cron 工具不可用后退化自查。
- 与 `feishu_company_profile_absolute_path_leak.md` 不同：本轮没有裸 `/Users/...` 绝对路径或本机 Markdown 链接，外露的是渠道元数据和本地存储状态。

## 下一步建议

- 在 Feishu direct / 共享 prompt policy 中补“用户信息汇总边界”：允许列出业务画像和用户可管理资料，禁止列出 raw channel identifiers、session id、手机号 metadata 字段名、本地文件 / 数据库 / 目录状态。
- 扩展用户可见输出净化，覆盖 `open_id`、`chat_id`、`手机号等元数据`、`当前会话 ID`、`data/sessions.sqlite3`、`data/portfolio`、`data/notif_prefs` 等短语在 final 中的泄漏。
- 增加回归样本：当用户要求“列出你掌握的所有用户信息”时，final 应给出业务资料摘要和隐私边界说明，但不得包含内部字段名、本地存储路径或会话 ID。
