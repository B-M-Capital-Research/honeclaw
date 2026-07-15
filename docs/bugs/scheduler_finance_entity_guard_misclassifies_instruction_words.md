# Bug: Scheduler finance entity guard misclassifies instruction words as securities

- **发现时间**: 2026-07-15 19:02 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 证据来源

- `data/sessions.sqlite3`，本轮真实消息窗口 `2026-07-15 19:01-23:01 CST`。
- `session_messages` 按真实 `timestamp` 新增 48 条 user / 55 条 assistant，近期 28 个 session 均以 assistant 收口，`last_message_role=user` 为 0；未见全渠道未回复、错投或敏感信息泄露。
- `2026-07-15T20:30:00.576659+08:00`，`session_id=Actor_feishu__direct__ou_5fbceebf26fcbb242fd6585745222c8063`，Feishu scheduler 触发 `老王说事与巴芒投资美股财报季个股判断`。任务正文是每天 20:30 的财报季个股判断，权威触发配置含 `repeat=trading_day`；assistant 在 `20:30:04.772759+08:00` 返回“当前无法稳定核验证券实体 `REPEAT`，已停止生成可能指向错误公司的分析。”
- `2026-07-15T21:00:01` 附近，Feishu scheduler `session_id=Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768` 与 `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 也分别返回同一 `REPEAT` 实体核验失败文案。
- 同窗仍有 20:30 / 21:35 / 23:00 成功 scheduler 样本，因此维持功能性 `P2 / New`，不升级为 P1。

- `data/sessions.sqlite3`，本轮真实消息窗口 `2026-07-15 15:02-19:02 CST`。
- 近期非文档代码提交：`c29de55c fix(agent): enforce verified stock response contracts`，提交时间 `2026-07-15 16:44:30 +0800`，早于本轮两条误杀样本。
- 同窗 `cron_job_runs.max(executed_at)` 仍停在 `2026-07-10T14:01:27.621121+08:00`，因此本轮以 `session_messages` 中的真实 scheduler/direct transcript 为准，不依赖本地 `cron_job_runs` 判断送达状态。

关键样本：

- `2026-07-15T17:30:00.856071+08:00`，`session_id=Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8`，`ordinal=527`，Feishu scheduler 触发 `A股港股收盘后跨市场复盘`。任务正文要求 A/H 收盘复盘，内容里包含估值指标 `EV/EBITDA` 一类普通金融术语。`ordinal=528` assistant 在 `17:30:04.210692+08:00` 返回：`当前无法稳定核验证券实体 `EBITDA`，已停止生成可能指向错误公司的分析。`
- `2026-07-15T18:00:00.987468+08:00`，`session_id=Actor_web__direct__web-user-ba50cb9401c0`，`ordinal=13`，Web scheduler 触发 `18:00 美股盘前 X 英文帖`。权威触发配置中含 `repeat=daily`，任务是生成英文 X 帖草稿，不是分析 `REPEAT` 证券。`ordinal=14/15` assistant 在 `18:00:04` 返回实体核验失败，并落成用户可见调度失败：`当前无法稳定核验证券实体 `REPEAT`，已停止生成可能指向错误公司的分析。`
- 本轮同窗 `session_messages` 按真实 `timestamp` 有 8 条 user / 9 条 assistant；近期会话均有 assistant 收口，没有发现全局未回复或错投，因此该问题按功能性 P2，而不是 P1。

## 端到端链路

1. 用户或 scheduler 触发金融相关定时任务。
2. Runner / answer contract 执行新增的证券实体核验 guard。
3. Guard 从完整任务文本中抽取大写词或指令字段，误把 `EBITDA`、`REPEAT` 这类指标 / 配置词当作证券实体。
4. 实体核验失败后直接中止生成，并向用户写入失败提示。
5. 原本应生成的市场复盘或英文 X 帖草稿没有产出。

## 期望效果

- 金融实体核验 guard 应只校验用户真正要求分析、定价、建仓或推送的证券实体。
- `EBITDA`、`EV/EBITDA`、`repeat=daily`、任务配置字段、格式指令和普通指标词不应被当作 ticker / 股票实体。
- 当实体识别存在歧义时，应优先忽略配置词，或把歧义限制在当前用户请求的真实标的候选上，而不是阻断整条 scheduler 链路。

## 当前实现效果

- Guard 把 `EBITDA` 和 `REPEAT` 当作需要稳定核验的证券实体。
- 两条不同渠道的 scheduler 样本均在数秒内失败，未生成业务正文。
- Web scheduler 还额外写入一条用户可见失败消息，明确暴露错误实体名 `REPEAT`。

## 用户影响

- 定时任务按时触发但产出被错误拦截，用户收不到应有的复盘或内容草稿。
- 失败信息让用户误以为任务中包含了不可核验的证券实体，实际只是系统把指标 / 配置词误识别。
- 当前证据覆盖 Feishu scheduler 与 Web scheduler 两条链路，但同窗仍有其他直聊正常收口，未见全渠道不可用或数据泄露，因此定为 P2。

## 根因判断

- 与 `c29de55c fix(agent): enforce verified stock response contracts` 时间相邻，且失败文案来自新增的投研完整性 / 证券实体核验 guard，疑似实体抽取范围过宽。
- 抽取逻辑没有区分任务配置、金融指标、正文格式指令和真正的证券 ticker。
- Scheduler 任务会把权威触发配置、任务说明和用户原始正文一起送入模型 / guard，增大了误抽取 `REPEAT` 这类配置词的概率。

## 下一步建议

- 在金融实体 guard 中增加 denylist 或 token role 过滤，至少覆盖 `EBITDA`、`EV`、`REPEAT`、`DAILY`、`WEEKLY`、`MONTHLY`、`X` 等指标 / 配置 / 格式词。
- 更稳妥的做法是让 guard 只校验模型准备输出交易结论的实体，或只校验工具已解析出的候选 ticker，而不是扫描完整 prompt 文本。
- 增加回归样本：
  - A/H 复盘任务正文包含 `EV/EBITDA` 时不应被拦截。
  - Web scheduler 权威配置包含 `repeat=daily` 时不应把 `REPEAT` 当成证券实体。
  - 真正的模糊 ticker / 非标准 ticker 仍应被 guard 拦截或要求澄清。
