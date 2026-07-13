# Bug: Feishu 直聊强时效金融问答输出未核验来源和精确买入区间

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

## 修复记录

- 2026-07-13 15:01 CST 补充同根复发证据，状态维持 `New`：
  - 11:04-15:01 CST `data/sessions.sqlite3` 按真实 `timestamp` 新增 3 个 user turn 与 3 条 assistant final，Feishu direct、Feishu scheduler 与 Web direct 均以 assistant 收口。
  - 12:17 CST Web direct session `Actor_web__direct__web-user-e05f5e5f74a3` 中，用户询问“今天海力士怎么了，怎么07709跌了这么多，Sk Hynix本身什么情况，会带动这个存储一起走下坡路么”。
  - 该 assistant row 的 `metadata_json` 为空，没有可审计 `assistant.tool_calls`；未留下本轮网页、行情、公告或媒体来源核验工具结果。
  - final 先说 `07709` 需要确认具体标的，但随后直接按 SK Hynix 展开，输出 `7 月 10 日在韩国 IPO`、发行价、上市首日高点、HBM 竞争、PE `8-10` 倍和“分批买比一次性梭哈更合理”等强时效金融判断。
  - 回复正常收口且未见错投、投递失败或内部实现外露；问题在于强时效金融来源 / 行情核验不可审计，并且在标的仍未确认时继续给出交易动作建议。因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-10 11:02 CST 补充同根复发证据，状态维持 `New`：
  - 07:01-11:02 CST `data/sessions.sqlite3` 按真实 `timestamp` 新增 18 个 user turn 与 19 条 assistant final，Feishu / Discord direct 与 scheduler 会话均已 assistant 收口；普通 scheduler 18 条均为 `completed + sent + delivered=1`。
  - 08:31 CST Feishu session `Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1` 的 assistant `metadata_json` 为空，没有可审计 `assistant.tool_calls`，但 final 输出 Brent、XAU/USD、VIX、10Y / 30Y 美债、Fed Rate Monitor 概率、CAPE、巴菲特指标等强时效市场数字和动作建议。
  - 08:32 CST Feishu session `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 的 assistant `metadata_json` 为空，但 final 写出 QQQ、WTI、10Y 美债、NVDA、MU、SNDK、CIEN、AVGO、COHR、TEM、VST、GEV、BE、RKLB、GOOGL 等最新价格、来源链接和操作参考。
  - 09:01 CST Feishu session `Actor_feishu__direct__ou_5f95ab3697246ded86446fcc260e27e1e2` 的 assistant `metadata_json` 为空，但 final 声称“最新可核验行情口径”，输出 TSLA / RKLB 收盘、盘后、目标价、Iridium 交易、Robotaxi / SpaceX 合并叙事和 80 / 90 / 100 美元操作观察位。
  - 09:02 CST Feishu scheduler session `Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a` 的 assistant `metadata_json` 为空，但 final 列出 Barron’s、MarketWatch、IBD、AP、Business Insider 等来源链接，输出 MU / SNDK / LITE / RKLB / BE / A 股标的强时效结论、目标价和操作建议；同条还出现 `<absolute-path>/` 占位符和标题拼接破损，本轮作为格式观察记录，不拆新缺陷。
  - 09:31 CST Discord scheduler session `Session_discord__group__g_3a1469549745654468692_3ac_3a1469549746518622371` 的 assistant `metadata_json` 为空，但 final 继续输出 FOMC、FedWatch、CPI、非农、指数点位和来源链接等强时效宏观市场判断。
  - 回复均正常收口且未见错投、投递失败或内部实现外露；问题在于强时效金融来源 / 行情核验不可审计，且多条 final 使用“已核验 / 最新可核验”表述或来源链接。因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-10 07:05 CST 状态从代码级 `Fixed` 回退为当前运行态 `New`：
  - 03:01-07:02 CST `data/sessions.sqlite3` 按真实 `timestamp` 新增 7 个 user turn 与 7 条 assistant final，Feishu / Web direct 与 scheduler 会话均已 assistant 收口；普通 scheduler 6 条均为 `completed + sent + delivered=1`。
  - 04:32 CST Feishu session `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` final 的 `metadata_json` 为空，没有可审计 `assistant.tool_calls`，但正文声称 `QQQ 最新可核验约 722.44`、`WTI 最新可核验约 71.75`，并继续输出 `SNDK 1,896.53`、`MU 999.56`、BofA 目标价 `1,550`、AVGO / CIEN 等强时效行情和市场判断。
  - 05:31 CST Feishu session `Actor_feishu__direct__ou_5f636d6d7c80d333e41b86ae79d07adca8` 与 06:01 CST Feishu session `Actor_feishu__direct__ou_5f11da38ad70c47cf87c0b106b6408b190` 的 assistant `metadata_json` 同样没有可审计工具证据，但继续输出美股指数、MU / AMD / AVGO / NVDA 等强时效行情、新闻归因和 A/H 次日预判。
  - 这些样本晚于 `96182d43 fix: tighten finance evidence guardrails` 代码提交，但当前 live 服务未确认重启加载该提交；从用户当前可见运行态看，缺陷仍会影响回答可信度，因此回退为 `New`。若后续确认 live 已重启且新代码运行态不再复发，可再转回 `Fixed` 或 `Closed`。
  - 回复正常收口且未见错投、投递失败或内部实现外露；问题在于强时效金融来源 / 行情核验不可审计，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-10 04:10 CST 代码级修复，状态更新为 `Fixed`：
  - 金融系统 prompt 新增“可审计核验约束”：若本轮没有可审计的网页、行情、公告、财报或新闻工具结果支撑，禁止使用“已核验”“可核验口径”“据公开报道已确认”等表述，也不得输出精确 IPO 发行价/区间、募资额、市值、成交额、首日可买条件或分档买入区间。
  - multi-agent search-stage / answer-stage guidance 同步新增同样护栏：缺少本轮 verified transcript 时，只能给情景框架、风险边界和待核验项，不能把私营公司 IPO、ADR 上市或媒体报道包装成已核验结论。
  - 验证 `cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture`、`cargo test -p hone-channels search_input_guidance_allows_direct_replies_for_greetings --lib -- --nocapture`、`cargo check -p hone-channels --tests`、`git diff --check` 通过。
  - 本轮未重启当前 live 服务，也未做线上运行态复核；先按代码级 `Fixed` 记录，后续如新运行态仍在无本轮工具证据时声称“已核验”并给出精确 IPO / ADR 操作锚点，再基于新证据回退。

- 2026-07-10 03:02 CST 补充同根复发证据，状态维持 `New`：
  - 23:02-03:02 CST `data/sessions.sqlite3` 按真实 `timestamp` 新增 16 个 user turn 与 16 条 assistant final，Feishu / Web direct 与 scheduler 会话均已 assistant 收口。
  - 00:01 CST Feishu direct session `Actor_feishu__direct__ou_5fea712445d905e8418bde07dbcf2cbfb2` 回答“海力士在美股上市后对港股南方2倍做多海力士有什么影响”。
  - 01:44 CST Web direct session `Actor_web__direct__web-user-400794904801` 回答“美光投资2500亿美元扩大生产规模，海力士ADR定价149美金...明天ADR上市有涨幅吗”。
  - 两条 assistant final 均围绕 `SKHY` ADR、`149 美元` 定价、首日涨幅、韩国正股次日走势和杠杆产品影响给出强时效金融判断；当前巡检未见可审计 `assistant.tool_calls` 证据证明这些时效锚点经过本轮来源 / 行情核验。
  - 回复正常收口且未见错投、投递失败或内部实现外露；问题在于强时效金融来源 / 行情核验不可审计，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-09 23:02 CST 补充同根复发证据，状态维持 `New`：
  - 19:02-23:02 CST `data/sessions.sqlite3` 按真实 `timestamp` 新增 38 个 user turn 与 38 条 assistant final，Feishu / Web direct 与 scheduler 会话均已 assistant 收口。
  - 22:17 CST Feishu direct session `Actor_feishu__direct__ou_5fe8ba64a3098d9fa009889f8e2ebfdce2` 回答“海力士adr上市对美光及三星以及闪迪影响怎么样”。
  - 该 assistant final 的 `metadata_json` 没有可审计 `assistant.tool_calls`；未留下本轮网页、行情、公告或媒体来源核验工具结果。
  - final 仍输出强时效金融锚点与判断，包括 `SKHY` Nasdaq ADR、发行价约 `144.5-149 美元/ADR`、发行规模约 `265 亿至 280 亿美元`、预计美国时间 `2026-07-10` 开始交易，并进一步给出对 MU、Samsung、SanDisk 的情绪、估值和交易影响判断。
  - 回复正常收口且未见错投、投递失败或内部实现外露；问题在于强时效金融来源 / 行情核验不可审计，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

- 2026-07-09 19:02 CST 状态从代码级 `Fixed` 回退为运行态 `New`：
  - 15:01-19:02 CST `data/sessions.sqlite3` 新增 5 个 user turn 与 5 条 assistant final，Feishu direct 与普通 scheduler 均以 assistant 收口。
  - 16:17 / 16:29 CST Feishu direct session `Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc` 连续回答长鑫存储上市影响、首日涨幅 / 市值 / 成交量和可买条件问题。
  - 两条 assistant final 的 `metadata_json` 只有渠道元数据，没有 `assistant.tool_calls`；未留下本轮网页、行情、公告或媒体来源核验工具结果。
  - final 仍声称“已核验”或“可核验口径”，引用 FT / 市场报道口径，并输出募资约 295 亿元、估值上限约 3 万亿元、首日涨幅区间、收盘市值区间、可买条件等强时效金融锚点。
  - 该样本晚于 2026-06-22 金融系统 prompt 代码级修复；主链路正常收口、无错投或投递失败，因此按质量性 `P3 / New` 回退，非 P1，不创建 GitHub Issue。

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

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检时间窗：2026-07-09 15:01-19:02 CST。
  - 本窗有 5 个 user turn 与 5 条 assistant final，Feishu direct 与普通 scheduler 均以 assistant 收口；普通 scheduler 1 条 `A股港股收盘后跨市场复盘` 为 `completed + sent + delivered=1`。
  - `session_id=Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc`。
  - 2026-07-09 16:15 CST 用户输入摘要：预测长鑫存储上市对全球存储行业格局、A 股短期流动性和半导体细分板块的影响。
  - 2026-07-09 16:17 CST assistant final 声称“可核验口径”包括长鑫科技集团冲刺科创板、计划募资约 295 亿元、全球 DRAM 三巨头份额和长鑫全球份额约 4%，但该 assistant row 的 `metadata_json` 没有 `assistant.tool_calls`。
  - 2026-07-09 16:27 CST 用户继续要求预测上市第一天涨幅、市值、成交量，以及什么情况下第一天可以买。
  - 2026-07-09 16:29 CST assistant final 开头写“已核验到的关键约束”，引用市场报道估值上限约 3 万亿元、募资约 295 亿元、FT 关于 Apple 测试和产出影响的口径，并给出发行估值、首日涨幅、收盘市值、换手 / 成交和可买条件分层；该 assistant row 的 `metadata_json` 同样没有 `assistant.tool_calls`。
  - 两条回复没有空回复、错投、投递失败、原始工具 JSON、token、本机路径或思维痕迹进入 final；本轮问题集中在强时效金融来源核验边界和精确操作锚点可信度。

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
- 2026-07-09 16:17 / 16:29 CST 长鑫存储样本说明，即使不是拼写近似 ticker，普通未上市公司 IPO / 科创板上市推演也会在没有本轮工具核验证据时输出“已核验”口径、媒体来源名和精确可买条件；2026-06-22 prompt 修复没有完整覆盖“无工具证据但声称已核验并给出强时效 IPO 操作锚点”的路径。

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
- 2026-07-09 长鑫存储样本说明根因也包括 answer 阶段缺少“final 声称已核验 / 可核验口径时必须存在本轮工具证据”的一致性校验；该路径不依赖 ticker 歧义也会复发。
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
- 2026-07-09 19:02 CST 复核只维护缺陷台账，未修改业务代码、测试代码或配置代码，未运行代码测试；已验证范围：15:01-19:02 CST SQLite 会话收口、assistant final 污染扫描、长鑫存储两条 assistant `metadata_json`、`cron_job_runs` 状态分布、最近四小时非文档提交检查。
