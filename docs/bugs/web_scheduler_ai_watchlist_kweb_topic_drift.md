# Bug: Web heartbeat AI watchlist drifts to KWEB ETF analysis

## 发现时间

2026-08-05 22:02 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-05 18:03-22:02 CST。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 20:30 CST 触发 prompt 明确要求每 30 分钟检查 `BE, TEM, STX, SATS, COHR, LITE, QCOM, DELL, AAOI, TSLA, PLTR, CRCL, HOOD, ORCL, INTC, FLY, META` 的财报、SEC 文件、AI 硬件、光通信、服务器、存储、电力、半导体、自动驾驶、稳定币、云基础设施、评级或已核验异常波动。
  - 20:30 CST 工具层出现 `FMP data_fetch cache hit ... quote/KWEB`；同轮 raw preview 围绕 `KWEB` 价格、PE、50 日 / 200 日均线、year-to-date 与 China internet policy 风险展开。
  - 20:30 CST deliver preview 开头写 `行情口径：KWEB ...`，主体解释 `KWEB 是什么`，并给出“可以关注，但还不是重仓时机”的 ETF 判断；KWEB 不在该任务的目标列表中。
  - 22:00 CST 同一 job 再次 raw preview 写 `Let me now do a proper analysis of KWEB`，deliver preview 仍在 BE 行情口径之后转为 `KWEB 当前处于"低估但趋势未确认"` 的主题分析。
- `data/sessions.sqlite3`
  - 本轮无法用 SQLite 交叉验证该 Web heartbeat final：`session_messages.max(timestamp)=2026-08-01T14:13:46.183054+08:00`、`session_messages.max(imported_at)=2026-08-02T20:59:58.506373+08:00`，18:03 CST 后 `sessions` / `session_messages` / `cron_job_runs` / `web_push_messages` 增量均为 0。
- 最近非文档代码提交
  - 18:06 CST `2c2cd1db fix: serialize structured skill payloads`，本轮日志样本发生在该提交之后，但问题表现为 heartbeat 任务主题漂移，未见该提交能证明本缺陷已修复。

## 端到端链路

1. Web scheduler 到点触发 `AI与科技持仓观察关键事件心跳提醒`。
2. 当前任务配置限定一组 AI / 科技持仓与观察标的，不包含 KWEB。
3. function-calling runner 调用 `data_fetch quote` 等工具时拿到或复用 KWEB quote。
4. heartbeat answer 阶段把 KWEB 当成主体，生成 ETF 解释、估值和重仓时机判断。
5. 出站层把该内容按 `PlainTextTriggered` 送入 deliver，用户看到的不是目标标的关键事件提醒。

## 期望效果

- Heartbeat 应严格以当前 job 的目标标的和触发条件为边界。
- 工具缓存、历史上下文或其它任务的 ETF 主题不得覆盖当前任务主体。
- 如果本轮只核验到少量目标标的，应说明其它目标未核验或输出 noop，而不是转向未请求 ETF。

## 当前实现效果

- 同一个 Web heartbeat job 至少两轮把 KWEB 作为主要分析对象。
- 20:30 CST deliver preview 从 `KWEB 是什么` 开始解释，22:00 CST deliver preview 继续围绕 KWEB 估值和趋势判断组织结论。
- 调度、runner 和出站链路都完成，未见空回复、错投对象、原始 provider 报错、凭据泄露或系统级不可用。

## 用户影响

- 用户订阅的是具体 AI / 科技标的关键事件提醒，却收到未请求的 KWEB ETF 投资分析。
- 用户需要自行识别这不是目标任务内容，降低 heartbeat 可信度和可操作性。
- 为何不影响功能链路，因此定级为 P3：该样本没有阻断 Web scheduler 触发、模型生成或出站投递，也没有数据破坏、错对象投递、敏感信息泄露或全渠道不可用；主要问题是 AI 返回内容焦点和任务结构不符合用户配置，因此按质量性 `P3` 登记。

## 根因判断

- 初步判断是 heartbeat answer 阶段没有强制当前 job target whitelist，工具结果或历史上下文中的 KWEB 被模型提升为主任务。
- `data_fetch` cache hit 显示 KWEB quote 在同轮工具链中出现，但当前触发 prompt 不包含 KWEB，说明工具调用规划或上下文隔离存在漂移。
- 该问题不同于 `feishu_scheduler_company_news_task_drifts_to_portfolio_trade_advice.md`：本样本来自 Web heartbeat，且是未请求 ETF 抢占当前目标标的，而不是普通 Feishu scheduler 公司资讯被持仓复盘模板覆盖。
- 该问题也不同于 heartbeat JSON / noop 解析缺陷：本样本已成功 deliver，核心问题是内容主题错误。

## 下一步建议

1. 在 heartbeat prompt / answer 阶段加入当前 job target whitelist，要求任何主体标的必须来自配置或明确说明关联关系。
2. 对工具调用 planner 增加校验：当 `data_fetch quote` 请求了不在 job target / user prompt 的 ticker，记录候选降级并避免将其作为主结论。
3. 增加 Web heartbeat 回归样本：目标列表不含 KWEB 时，输出不得以 KWEB ETF 解释、估值或重仓建议作为主体。
