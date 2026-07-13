# Bug: 普通 scheduler 金融复盘把周末日期误判为美股交易日或收盘日

- **发现时间**: 2026-07-13 07:02 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New
- **GitHub Issue**: 无，当前不是 P1。

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - `2026-07-14T05:12:53.221513+08:00`
    - `session_id=Actor_feishu__direct__ou_5fea712445d905e8418bde07dbcf2cbfb2`
    - Feishu scheduler `美股收盘资金流向简报` 在北京时间 `2026-07-14 05:10`、美东 `2026-07-13 17:10` 的盘后窗口，assistant final 写出 `最近完成的美股交易日为 2026-07-11 周五`，并按该口径生成主要指数、板块 ETF、资金方向和市场状态判断。
    - 该窗口实际已经是 2026-07-13 美股正常交易日收盘后；回复把最近完成交易日回退到 2026-07-11，继续说明普通 scheduler 金融复盘的交易日历 / 时间口径仍会漂移。
    - 同条消息正常收口，没有投递失败、错投、内部路径或原始工具 JSON 外泄；问题仍主要影响金融复盘日期与数据新鲜度可信度，因此维持质量性 `P3 / New`，非 P1。
  - `2026-07-13T20:30:22.205527+08:00`
    - `session_id=Actor_feishu__direct__ou_5f79ee8185333e5db4a55e5eca0d8d2f7e`
    - Feishu scheduler assistant final 写出 `当前北京时间：2026年7月13日20:30。今天是周一，美股常规盘已收盘；本次以2026年7月11日美股收盘数据为准（7月12日、13日为周末休市）`。
    - 该北京时间对应美东周一 08:30 左右的盘前窗口，不是常规盘收盘后；同时 2026-07-13 是周一，不应写成周末休市。
  - 同窗对照：
    - 19:00-23:02 CST 共 49 个 user turn / 60 条 assistant 记录，均以 assistant 终态收口；该样本没有阻断生成 / 落库 / 投递主链路。
    - assistant final 污染扫描未命中 `<think>`、本机路径、provider 原始错误、panic、quota、原始工具 JSON 或结构化 JSON 外泄。
    - 因为问题仍主要影响金融复盘的日期、星期和交易时段可信度，不影响直聊 / 调度 / 投递主功能链路，严重等级继续维持质量性 `P3`。
  - `2026-07-13T09:01:08.984163+08:00`
    - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
    - Feishu scheduler `核心观察池早间简报` assistant final 写出 `北京时间 2026-07-13 09:00，美股周一正常交易`，但该北京时间对应美东周日夜间，尚未进入美股周一常规交易。
  - `2026-07-13T08:46:00.477845+08:00`
    - `session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7`
    - Feishu scheduler `A股盘前高景气产业链推演` assistant final 写出 `美股最新完整交易日为2026年7月11日`，继续把周末日期当成最新完整交易日。
  - `2026-07-13T08:31:04.794138+08:00`
    - `session_id=Actor_feishu__direct__ou_5f1fdfeceacb0f2ece1a2c88c5a7d17e34`
    - Feishu scheduler `闪迪(SNDK)每日行情与行业简报` assistant final 写出 `SNDK 最新可核常规盘为7月9日收于 1,915.92 美元`，同时本窗多条 scheduler 又将 `2026-07-11` 写作最新完整美股交易日，说明交易日口径仍漂移。
  - `2026-07-13T07:03:10.200694+08:00`
    - `session_id=Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3`
    - Feishu scheduler `美股持仓收盘后早报` assistant final 写出 `美股 7月11日（周五）常规收盘复盘`，并按该口径生成持仓涨跌贡献与操作框架。
  - `2026-07-13T05:01:24.301383+08:00`
    - `session_id=Actor_web__direct__web-user-afc1cabadbf8`
    - `ordinal=101`
    - Web scheduler `盘后美股复盘与SNDK/MU存储产业链日报` assistant final 写出 `2026-07-12 美股正常交易日盘后复盘，不是半日交易，也不是休市日`，并继续把本轮定义为 `2026-07-12 交易日` 的增量复盘。
  - `2026-07-13T06:00:30.721800+08:00`
    - `session_id=Actor_feishu__direct__ou_5f11da38ad70c47cf87c0b106b6408b190`
    - `ordinal=240`
    - Feishu scheduler `每日美股盘后收盘复盘` assistant final 写出 `美东 2026-07-12 周六`，并把最新完整数据写成 `美东 2026-07-11 周五收盘`。
  - `2026-07-13T06:32:34.247328+08:00`
    - `session_id=Actor_web__direct__web-user-14f4cadb069f`
    - `ordinal=94`
    - Web scheduler `1亿美元AI科技组合每日跟踪` assistant final 写出 `本次复盘口径为 2026-07-11 美股常规收盘`，并按 `2026-07-11 常规收盘口径` 计算组合变化。
- 同窗对照：
  - `2026-07-13 07:00-10:30 CST` 共 27 个 user turn / 27 条 assistant final，均成对收口。
  - assistant final 污染扫描未确认空回复、`<think>`、本机路径、provider 原始错误、原始工具 JSON 或结构化 JSON 外泄。
  - `2026-07-13 03:00-07:02 CST` 共 9 个 user turn / 9 条 assistant final，均成对收口。
  - assistant final 污染扫描未确认空回复、`<think>`、本机路径、provider 原始错误、原始工具 JSON 或 `company_profiles/` 外露。
  - `cron_job_runs.max(executed_at)` 仍停在 `2026-07-10T14:01:27.621121+08:00`，本轮用户可见证据以 `session_messages` 为准。

## 端到端链路

1. Feishu / Web 普通 scheduler 在北京时间 2026-07-13 清晨触发美股盘后、组合跟踪、资金流向或持仓早报任务。
2. 权威触发配置要求模型先校准北京时间、美东时间和美股交易日历，再决定复盘口径。
3. assistant final 成功生成并落库，但多条回复把 2026-07-12 周日或 2026-07-11 周六误写为美股正常交易日、周五收盘或常规收盘口径。
4. 回复继续据此给出指数、个股涨跌、组合贡献、风险和次日操作框架。

## 期望效果

- 普通 scheduler 应以权威触发时间和真实交易日历为准。
- 北京时间 2026-07-13 清晨对应美东 2026-07-12 周日；若美股周末休市，应明确使用最近一个实际交易日的收盘数据，并写对日期与星期。
- 不能把周末日期描述为正常交易日，也不能把不存在的周六 / 周日常规收盘当作组合变化和操作建议的计算基础。

## 当前实现效果

- 调度、生成和落库链路正常，用户收到了完整可读的复盘。
- 但用户可见 final 的日期 / 星期 / 交易日口径不一致：
  - Web scheduler 明确说 `2026-07-12` 是美股正常交易日。
  - Feishu / Web scheduler 多次把 `2026-07-11` 写成周五或常规收盘日。
- 这些错误随后被用于解释市场、计算组合涨跌和给出观察框架，降低金融复盘可信度。

## 用户影响

- 用户会误以为周末存在新的美股常规交易和收盘数据，可能错误理解行情新鲜度、组合盈亏、板块强弱和下一个交易日风险。
- 这是质量类缺陷：消息仍正常生成、落库和投递，未发现错投、无回复、系统崩溃、数据破坏或内部错误外泄。
- 因不影响直聊 / 调度 / 投递主功能链路，主要影响 AI 返回的时间口径与金融复盘质量，所以定级为 `P3`，非 P1，不创建 GitHub Issue。

## 根因判断

- 初步判断是普通 scheduler 的时间 / 交易日上下文没有被足够强地约束到权威触发时间和交易日历，模型在周末窗口自行推断日期和星期。
- 该问题与 `scheduler_heartbeat_trigger_time_mismatch.md` 的 heartbeat 时间口径漂移相似，但本轮证据进入的是普通 scheduler assistant final，而不是 heartbeat raw / deliver preview，因此登记为独立受影响链路。
- 该问题也不同于旧价格 fallback：本轮核心不是 `data_fetch` 失败后沿用旧价格，而是用户可见复盘口径把周末日期写成交易日 / 收盘日。

## 下一步建议

- 为普通 scheduler 的金融 / 复盘类任务增加统一交易日历约束：final 中引用 `当前复盘口径`、`最近实际交易日`、`下一交易日` 时必须来自权威时间计算，而不是模型自由推断。
- 增加出站前 sanity check：若 final 同时出现 `周六/周日` 与 `正常交易日/常规收盘/盘后复盘`，或日期与星期不匹配，则改写为安全说明或降级为失败。
- 增加回归样本覆盖北京时间周一清晨、美东周日傍晚、节假日和半日市。
