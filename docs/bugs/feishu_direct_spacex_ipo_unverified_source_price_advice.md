# Bug: Feishu 直聊强时效金融问答输出未核验来源和精确买入区间

## 发现时间

2026-06-11 15:02 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

Fixed

## GitHub Issue

无，非 P1

## 修复记录

- 2026-06-22 03:08 CST 状态更新为 `Fixed`：
  - 金融系统 prompt 已扩展非标准 / 高歧义 ticker 约束：当这类 ticker/简称被用于强时效新闻、利好 / 利空、IPO、融资、收购、并购或上市进展问题时，必须先确认证券实体与来源支持。
  - 在用户确认前，不得把近似代码直接等同为热门私营公司或未上市公司股票，也不得基于该假设展开强时效叙事。
  - 验证：`cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture` 通过。
  - 无关联 GitHub Issue；本轮按本地代码与回归验证关闭，不依赖生产日志、线上渠道状态或 live 重启。

- 2026-06-20 23:03 CST 补充同根复发证据，状态保持 `New`：
  - 19:01-23:01 CST `data/sessions.sqlite3` 仍未追平最近真实会话，`session_messages.max(timestamp)=2026-06-17T10:37:37.202464+08:00`；本轮以 `data/runtime/logs/acp-events.log` 重构用户可见 final。
  - 本窗 ACP 可重构 25 次 `session/prompt`、25 次 `stopReason=end_turn`（含 23:00 CST 边界 prompt 于 23:02:38 CST 收口）、0 个 ACP response error。
  - 21:02 CST Feishu direct session `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3` 用户输入为 `spcx有没有什么利好消息 去搜一下`。
  - assistant final 直接写出 `我按 Nasdaq: SPCX，也就是 SpaceX 股票来查`，随后输出 SpaceX IPO 后续融资、散户买入、Cursor / Anysphere 收购、期权交易和指数纳入预期等强时效金融叙事，并附多条外部链接。
  - 该回答正常 `end_turn` 收口，没有空回复、错投、投递失败、原始工具 JSON、token、本机绝对路径或思维痕迹进入 final；但它把 `SPCX` 与 SpaceX 股票直接等同，并输出高度可操作的当前利好判断，实体与来源核验边界不可靠。
  - 当前没有发现持仓 / 画像 / 定时任务等持久化副作用写坏证据，主投递链路也未中断；按规则保持质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 巡检时间窗：2026-06-20 19:01-23:01 CST。
  - `session_id=Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`。
  - 2026-06-20 21:02 CST `session/prompt` 中的本轮用户输入为 `spcx有没有什么利好消息 去搜一下`。
  - 2026-06-20 21:02 CST assistant final 直接把 `SPCX` 解释为 `Nasdaq: SPCX，也就是 SpaceX 股票`，并输出 SpaceX IPO / 融资 / Cursor 收购 / 期权与指数预期等最新利好判断。
  - 本窗 ACP 没有 response error；该 final 以 `stopReason=end_turn` 收口，回复没有内部工具协议、路径、token 或思维痕迹外泄。
  - 本轮问题集中在强时效金融实体识别与来源核验边界，而不是消息投递或格式链路故障。
- `data/sessions.sqlite3` -> `session_messages`
  - 巡检时间窗：2026-06-11 15:02-19:02 CST。
  - 本窗有 17 个 user turn 与 17 个 assistant final，10 个最近会话均以 assistant 收口；普通 Feishu scheduler 1 条为 `completed + sent + delivered=1`。
  - `session_id=Actor_feishu__direct__ou_5fdb997ed67ac0b7f5403701682185d67a`
  - 2026-06-11 17:59 CST 用户输入摘要：`美股dell详细分析`。
  - 2026-06-11 18:02 CST assistant final 输出 DELL `2026年6月10日美股收盘 369.83 美元`、`6月11日盘前约 373 美元`、Forward PE `20.12`、Forward PS `1.40`、FY2027 Q1 收入 / EPS / AI server revenue / AI orders / backlog、TTM 财务、来源链接，以及 `300-325`、`260-285`、`220-245` 等分档建仓区间。
  - 该 assistant row 的 `metadata_json` 包含 `assistant.tool_calls`，但只看到本地 `date/rg --files company_profiles` 与写入 `company_profiles/DELL.md` 的 shell 工具调用；没有本轮网页搜索、打开来源链接、行情查询或财务页读取的可审计工具结果。
  - 同一 final 还外露 `company_profiles/DELL.md`，该问题已补充到 `web_company_profile_relative_path_exposed.md`；本单只跟踪“没有本轮来源核验工具证据却输出强时效精确金融数字和建仓区间”。
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

1. Feishu 用户提出强时效金融分析或买入区间问题，例如 SpaceX 未来 IPO 价格 / 可买区间、DELL 最新详细分析。
2. assistant 进入金融回答，并输出精确行情、估值、财务指标、来源链接或分档买入 / 建仓区间。
3. assistant 没有留下本轮可审计来源核验工具调用，或只留下本地公司画像读写工具调用，却输出精确 IPO 发行价、估值、最新行情、财务指标、多个来源链接和分档买入区间。
4. 会话最终正常 `end_turn` 收口并投递给用户。

## 期望效果

- 对未上市公司 IPO、估值、募资额、上市价和买入区间这类强时效金融问题，应先核验当前可靠来源，并在 final 中只使用已核验来源支持的事实。
- 对上市公司最新行情、盘前价、估值倍数、财务指标和分档建仓价位，也应有本轮可审计工具结果支撑，不能只凭模型记忆或本地画像写入动作生成精确数字与来源链接。
- 如果本轮无法稳定核验，应明确说“未完成稳定校验”，只给估值框架、风险边界和待核验清单，不应输出精确价格锚点或具体可照抄买入 / 建仓区间。
- 拼写近似实体可以合理纠正，但应说明是按 SpaceX 理解；若后续涉及操作建议，仍需遵守最新来源和交易建议约束。

## 当前实现效果

- 回复结构完整、语言正常，并且没有空回复、错投、投递失败或内部工具协议外泄。
- 但 final 把未见本轮工具核验证据的“公开报道”当作事实锚点，并给出具体价格区间和操作分档。
- 2026-06-11 18:02 CST DELL 样本进一步显示，即使是上市公司详细分析，assistant 也会在没有网页 / 行情 / 财务工具核验证据的情况下列出来源 URL、最新价格、财务指标和建仓区间；本地写入公司画像的工具调用不等同于来源核验。
- 这会让用户误以为系统已经完成了最新 IPO 来源核验，并可能把精确区间当作可执行交易纪律。
- 2026-06-20 21:02 CST SPCX 样本进一步显示，用户输入真实 ticker 后，answer 阶段仍可能把 ticker 直接等同为高热度叙事实体，并输出未充分约束的强时效 IPO / 融资 / 收购利好链条；问题从“精确价格区间未核验”扩展到“实体识别与强时效来源核验未形成硬边界”。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- Feishu direct 主链路已完成：用户提问被识别、assistant 有最终回复、会话正常落库和投递。
- 当前证据没有显示错投、未回复、重复投递、数据写坏、链路中断或内部报错外泄。
- 影响集中在强时效金融答案的可信度：用户可能基于未核验的 IPO 价格、最新行情、估值指标和建仓区间做投资判断。
- 因此它不影响主功能链路，按规则定级为 `P3`，不是 `P1/P2`。

## 根因判断

- 初步判断是 Feishu direct 的金融 prompt / answer 阶段只要求“涉及实时信息必须调用真实数据工具”，但实际 final 没有强制校验“本轮是否存在支持当前来源和精确数字的工具结果”。
- `feishu_direct_storage_price_unverified_before_tool_complete.md` 覆盖的是已发起行情工具但未充分等待 / 消费结果时输出精确行情；本轮是未上市公司 IPO 估值与媒体来源链接没有本轮可审计工具证据，链路相邻但触发条件不同。
- 2026-06-11 DELL 样本说明根因不局限于未上市 IPO：当 assistant 只执行本地公司画像读写时，answer 阶段仍可能生成看似来自网页和财务页的精确数字与来源链接，缺少“final 中每个来源链接 / 精确行情 / 交易区间必须对应本轮工具证据”的一致性校验。
- 2026-06-20 SPCX 样本说明根因还包括非标准 / 高歧义 ticker 与热门私营公司叙事之间缺少实体确认门槛；即使用户要求“去搜一下”，final 也应先确认 `SPCX` 的证券实体与来源支持，而不是直接写成 SpaceX 股票并展开当前利好分析。
- 该问题也不同于路径或内部工具名外露缺陷：本轮用户可见文本没有泄露内部实现，问题是强时效金融来源和可操作价格区间的核验边界不足。

## 下一步建议

- 在金融系统 prompt 或 answer guard 中补一条硬约束：涉及 IPO、未上市公司估值、募资额、上市公司最新行情、估值倍数、媒体报道来源和具体买入 / 建仓价位时，若本轮没有可审计工具结果，不得输出精确数字、来源链接或分档区间。
- 对“来源”段增加一致性检查：final 中列出的外部链接必须来自本轮工具结果或明确标注为用户提供 / 历史上下文，不能凭模型记忆生成。
- 增加回归样本：
  - 用户问 `预估 sapcex 上市多少钱，多少钱能买`，无工具结果时 final 只能给核验缺失说明和估值框架，不得给 `135 美元`、`1.75 万亿美元` 或具体买入区间。
  - 用户问 `spcx有没有什么利好消息 去搜一下`，answer 阶段必须先确认 `SPCX` 对应的证券实体和来源支持；不得直接把 `SPCX` 写成 SpaceX 股票并输出 IPO / 融资 / 收购利好链条。
  - 用户问 `美股dell详细分析`，若本轮只有本地画像读写工具调用，final 不得列出来源链接、盘前价、精确财务/估值指标或建仓区间。

## 验证

- 本轮为缺陷台账维护任务，未修改业务代码、测试代码或配置代码，未运行代码测试。
- 已验证范围：`data/sessions.sqlite3` 最近四小时会话收口、assistant final 污染扫描、`cron_job_runs` 状态分布、`acp-events.log` 当前 prompt 上下文、最近四小时提交检查。
- 2026-06-11 19:02 CST 复核同样只维护缺陷台账，未修改业务代码、测试代码或配置代码，未运行代码测试；已验证范围：15:02-19:02 CST SQLite 会话收口、assistant final 污染扫描、DELL assistant `metadata_json.assistant.tool_calls`、`cron_job_runs` 状态分布、最近四小时非文档提交检查。
