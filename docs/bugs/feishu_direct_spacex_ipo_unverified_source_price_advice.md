# Bug: Feishu 直聊 SpaceX IPO 估值问答输出未核验来源和精确买入区间

## 发现时间

2026-06-11 15:02 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检时间窗：2026-06-11 11:02-15:02 CST。
  - 本窗只有 3 个 user turn 与 3 个 assistant final，均为 Feishu direct / scheduler 正常成对收口；普通 Feishu scheduler 1 条为 `completed + sent + delivered=1`。
  - `session_id=Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a`
  - 2026-06-11 14:05 CST 用户输入摘要：`预估sapcex上市多少钱，多少钱能买`。
  - 2026-06-11 14:07 CST assistant final 直接按 `sapcex` 理解为 SpaceX，并输出“公开报道的 IPO 锚点约 135 美元 / 估值约 1.75 万亿美元”，随后给出 `150 美元以上不追`、`115-125 美元观察仓`、`95-105 美元开始有安全边际`、`80 美元以下赔率改善` 等具体买入区间。
  - 同一 final 以 `MarketWatch`、`WSJ`、`Investopedia`、`Business Insider` 链接作为来源，但该 assistant row 的 `metadata_json` 只包含渠道元数据，没有 `assistant.tool_calls`，也没有本轮搜索、打开网页或读取来源的工具调用证据。
- `data/runtime/logs/acp-events.log`
  - 14:05 CST 同 session 的 `session/prompt` 可见本轮用户输入是 SpaceX IPO 估值 / 买入价问题。
  - 该 prompt 上下文包含历史会话和旧工具轨迹，但没有看到本轮针对 SpaceX IPO 来源链接、IPO 价格、估值或买入区间的可审计工具结果进入当前 assistant final。
- 本轮巡检汇总：
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、`company_profiles/...`、`data_fetch`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`Param Incorrect`、panic 或 `index out of bounds`。
  - `cron_job_runs` 同窗 heartbeat 仍有 37 条 `execution_failed + skipped_error + delivered=0` 与 68 条 `noop + skipped_noop + delivered=0`，失败形态主要为既有结构化收口问题，未进入用户可见 final。
  - 最近四小时只有文档提交 `8cd36b4f Update bug patrol ledger`，没有非文档代码提交可证明该链路已修复。

## 端到端链路

1. Feishu 用户用拼写近似的 `sapcex` 询问 SpaceX 未来 IPO 价格和可买区间。
2. assistant 高概率纠正实体为 SpaceX，并进入强时效、强操作语义的金融回答。
3. assistant 没有留下本轮可审计来源核验工具调用，却输出精确 IPO 发行价、估值、募资额、市销率、多个媒体来源链接和分档买入区间。
4. 会话最终正常 `end_turn` 收口并投递给用户。

## 期望效果

- 对未上市公司 IPO、估值、募资额、上市价和买入区间这类强时效金融问题，应先核验当前可靠来源，并在 final 中只使用已核验来源支持的事实。
- 如果本轮无法稳定核验，应明确说“未完成稳定校验”，只给估值框架、风险边界和待核验清单，不应输出精确价格锚点或具体可照抄买入区间。
- 拼写近似实体可以合理纠正，但应说明是按 SpaceX 理解；若后续涉及操作建议，仍需遵守最新来源和交易建议约束。

## 当前实现效果

- 回复结构完整、语言正常，并且没有空回复、错投、投递失败或内部工具协议外泄。
- 但 final 把未见本轮工具核验证据的“公开报道”当作事实锚点，并给出具体价格区间和操作分档。
- 这会让用户误以为系统已经完成了最新 IPO 来源核验，并可能把精确区间当作可执行交易纪律。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- Feishu direct 主链路已完成：用户提问被识别、assistant 有最终回复、会话正常落库和投递。
- 当前证据没有显示错投、未回复、重复投递、数据写坏、链路中断或内部报错外泄。
- 影响集中在强时效金融答案的可信度：用户可能基于未核验的 IPO 价格和估值区间做投资判断。
- 因此它不影响主功能链路，按规则定级为 `P3`，不是 `P1/P2`。

## 根因判断

- 初步判断是 Feishu direct 的金融 prompt / answer 阶段只要求“涉及实时信息必须调用真实数据工具”，但实际 final 没有强制校验“本轮是否存在支持当前来源和精确数字的工具结果”。
- `feishu_direct_storage_price_unverified_before_tool_complete.md` 覆盖的是已发起行情工具但未充分等待 / 消费结果时输出精确行情；本轮是未上市公司 IPO 估值与媒体来源链接没有本轮可审计工具证据，链路相邻但触发条件不同。
- 该问题也不同于路径或内部工具名外露缺陷：本轮用户可见文本没有泄露内部实现，问题是强时效金融来源和可操作价格区间的核验边界不足。

## 下一步建议

- 在金融系统 prompt 或 answer guard 中补一条硬约束：涉及 IPO、未上市公司估值、募资额、媒体报道来源和具体买入价位时，若本轮没有可审计工具结果，不得输出精确数字、来源链接或分档买入区间。
- 对“来源”段增加一致性检查：final 中列出的外部链接必须来自本轮工具结果或明确标注为用户提供 / 历史上下文，不能凭模型记忆生成。
- 增加回归样本：用户问 `预估 sapcex 上市多少钱，多少钱能买`，无工具结果时 final 只能给核验缺失说明和估值框架，不得给 `135 美元`、`1.75 万亿美元` 或具体买入区间。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`data/sessions.sqlite3` 最近四小时会话收口、assistant final 污染扫描、`cron_job_runs` 状态分布、`acp-events.log` 当前 prompt 上下文、最近四小时提交检查。
