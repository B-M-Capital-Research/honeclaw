# Bug: Web heartbeat target list drifts to unrelated ticker analysis

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
  - 最新巡检窗口：2026-08-09 18:03-22:03 CST（UTC 2026-08-09 10:03-14:03）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 19:00 CST deliver preview 主体转为“你的推送日程 / 即时推阈值 / 最低严重度”等配置说明；21:30 CST deliver preview 转为通用“选股框架”；22:00 CST deliver preview 转为 DeepSeek 与 Nvidia 叙事分析。该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是配置查询、选股教程或单一叙事问答。
  - `job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒` 与 `job_id=j_19dd9a1e` / `job=闪迪关键事件心跳提醒`。
  - 18:30 CST `中际旭创` deliver preview 转向 NBIS 投研；21:30 CST `闪迪` deliver preview 写“实体口径重要更新：SNDK 已退市，本轮触发实际监控对象切换”，主体转向 WDC / Western Digital。两者都偏离原始 heartbeat 标的约束。
  - 调度和投递主链路仍可收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-09 10:00-14:01 CST（UTC 2026-08-09 02:00-06:01）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 10:30 / 11:00 CST deliver preview 继续以 `NVDA x Lancium`、SpaceX 优先供货和能源基础设施为主体，未稳定覆盖 AAPL / NVDA / BE 全目标关键事件筛查；13:30 又只围绕 NVDA 新闻组织检查结果。
  - `job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 11:00 CST deliver preview 改用 NBIS 行情口径并讨论 NBIS 财报节点；13:30 CST 又以 NVDA / 北京亦庄新城关系核验为主体。该 job 名义目标是中际旭创关键事件监控。
  - `job_id=j_19dd9a1e` / `job=闪迪关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 12:00 / 13:00 CST deliver preview 转向 Western Digital 与 SK hynix 竞争关系和 NAND Flash 市场分析，未稳定执行 SNDK 关键事件监控。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 11:30 / 12:00 CST deliver preview 在 PLTR 单标的估值、追入价位和持仓策略上展开，未覆盖原目标列表的关键事件筛查。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-09 06:00-10:02 CST（UTC 2026-08-08 22:00-2026-08-09 02:02）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 07:00 CST deliver preview 转向 `NVDA x Lancium $30亿投资` 的单项分析；09:30 CST 同 job 继续以 NVDA / Lancium 投资为主体，而不是 AAPL / NVDA / BE 全目标关键事件筛查。
  - `job_id=j_aa99140a` / `job=光迅科技关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 07:00 CST deliver preview 改为 TSMC 建仓 / 估值窗口；07:30 / 08:00 CST 又转向光迅科技持仓成本、加仓窗口与 PE 149x 判断。该 job 名义目标是光迅科技关键事件监控，不应持续输出建仓/持仓策略长文。
  - `job_id=j_eab1a3b2` / `job=NBIS关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 07:30 CST deliver preview 以光迅科技持仓分析为主体；08:00 CST 转向 NAND Flash 短缺与 WDC / SNDK 受益链条；均偏离 NBIS 关键事件监控。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 10:00 CST deliver preview 在工具持续限速后转为 Palantir 业务模式 / Gotham / Foundry 介绍，未覆盖原目标列表的关键事件筛查。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-09 02:01-06:01 CST（UTC 2026-08-08 18:01-22:01）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 02:31 CST deliver preview 转成 DeepSeek 与 Nvidia 的关系分析；03:30 CST 同 job 写成“待机，无新增触发事件”并复述监控覆盖范围；06:00 CST 又输出“过去 24 小时无遗漏事件”和监控范围说明。该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不应被关系问答或配置说明反复抢占。
  - `job_id=j_5b3cb604` / `job=光模块板块关键事件心跳提醒` / `target=web-user-499a1c6331c4`。
  - 02:30 CST duplicate preview 主体仍是 `GTC 与光模块 CapEx 的关系` 产业链知识问答；06:00 CST deliver preview 明写“本轮查询为关系问答，无即时价格监控”。
  - `job_id=j_eab1a3b2` / `job=NBIS关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 06:00 CST deliver preview 主体转向 NAND Flash 短缺全景与 WDC/SNDK 利好分析，说明 job 内容仍易被单一历史主题或工具结果抢占。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-08 22:02-2026-08-09 02:02 CST（UTC 2026-08-08 14:02-18:02）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 02:00 CST deliver preview 转成“现在美国处于什么时期”的宏观周期问答，并说明“这不是证券/股票/ETF/加密货币/金融市场问题”；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是美国宏观时期问答。
  - `job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 01:30 CST deliver preview 主体转为博通（AVGO）公司分析与估值判断；该 heartbeat job 名义目标是中际旭创关键事件，不是 AVGO 单标的投研。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 01:31 CST deliver preview 以 fenced JSON / `status=triggered` 协议载荷组织 COHR 事件；该样本同时归入 JSON 载荷质量缺陷，但也说明该 job 的用户可见内容仍被单一工具结果 / 协议形态抢占。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-08 18:01-22:01 CST（UTC 2026-08-08 10:01-14:01）。
  - `job_id=j_c83f66ac` / `job=NVDA 关键事件心跳提醒` / `target=web-user-6eedc778b4c5`。
  - 21:00 CST deliver preview 退化成“你已经连续多轮只发 NVDA”“你需要我做什么”的 direct follow-up 澄清，并要求用户选择暂停、设置触发条件或做深度分析；该 heartbeat job 名义上应执行 NVDA 关键事件检查，不应在到点运行时反问用户。
  - `job_id=j_b95a8df6` / `job=持仓重大事件心跳提醒` / `target=web-user-d415e2c11ced` 在 21:01 CST 转成 `Starlink Direct-to-Cell 是什么` 的科普问答；22:00 CST 同 job 又以 SPY / QQQ 大盘复盘为主体，未稳定围绕配置的持仓重大事件核验。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d` 在 22:00 CST 转成 `DeepSeek vs. Nvidia` 市场叙事分析，开头还写“这个不是证券买卖问题”；该 job 名义目标是 AAPL / NVDA / BE 关键事件，不是 DeepSeek 关系问答。
  - `job_id=j_19dd9a1e` / `job=闪迪关键事件心跳提醒` / `target=web-user-c2776780c59d` 在 22:01 CST 明说 DataFetch 因工具预算限制未返回 WDC / SK hynix 报价与新闻，随后主体转成 WDC 与 SK hynix 是否直接竞争的关系分析；该 job 名义目标是闪迪关键事件心跳。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-08 14:02-18:01 CST（UTC 2026-08-08 06:02-10:01）。
  - `job_id=j_75af226e` / `job=存储板块关键事件心跳提醒` / `target=web-user-499a1c6331c4`。
  - 15:00 CST deliver preview 从“每 30 分钟的存储板块心跳监控已在运行中”退化成“你想了解什么”的 direct follow-up 询问；16:00 / 16:30 / 17:30 / 18:00 CST 多轮主体转成“光纤光缆与光电 / 光子产业链全景”，并写明“本轮为产业链知识问答 / 无即时价格监控需求”。该 heartbeat job 名义目标是存储板块关键事件监控，不是光纤光缆产业链知识问答。
  - `job_id=j_5b3cb604` / `job=光模块板块关键事件心跳提醒` / `target=web-user-499a1c6331c4` 在 17:00 CST 直接写“这是一个纯产业链知识问答，不涉及具体持仓的即时价格监控触发或新事件核验”，主体同样转成光纤光缆与光电产业链。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d` 在 17:00 CST deliver preview 改以 `TQQQ` 当前报价和昨收表格为主体；TQQQ 不在该 job 名义目标列表中。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-08 02:01-06:01 CST（UTC 2026-08-07 18:01-22:01）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 03:30 CST deliver preview 主体转为 DeepSeek 与 Nvidia 的关系解释，04:00 CST 转为“当前配置状态确认”，05:00 CST 又转为 NVDA 单项估值和财报催化分析；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是配置问答或单一主题深度报告。
  - `job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒` / `target=web-user-c2776780c59d` 在 03:30 CST 退化成“无法直接创建每 30 分钟自动检查任务”的配置说明。
  - `job_id=j_c83f66ac` / `job=NVDA 关键事件心跳提醒` / `target=web-user-6eedc778b4c5` 在 06:00 CST 退化成连续直聊澄清和触发条件选择。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-07 22:01-2026-08-08 02:01 CST（UTC 2026-08-07 14:01-18:01）。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 23:00 CST deliver preview 主体转为 `NBIX` 工具结果、Neurocrine Biosciences 公司身份和财报事件核验；该 heartbeat job 名义目标是 AI / 科技持仓观察关键事件，不是 NBIX 单标的问答。
  - 02:00 CST 同一 job deliver preview 主体转为 `META` 新墨西哥州儿童安全案追加判罚；该任务目标虽含 META，但正文同时写明其余标的因工具上限未完成核验，说明目标列表执行仍被单一工具结果抢占。
  - `job_id=j_aa99140a` / `job=光迅科技关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 01:00 CST deliver preview 退化成“无法创建每 30 分钟自动检查任务”的工具 / 配置说明，而非光迅科技关键事件检查；02:00 CST raw preview 仍围绕任务创建请求而不是当前 heartbeat 结果组织。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-07 18:01-22:01 CST（UTC 2026-08-07 10:01-14:01）。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 21:30 CST deliver preview 主体转为 AI 基础设施电力瓶颈与核能赛道定价逻辑，输出能源赛道投研框架；该 heartbeat job 名义目标是 AI / 科技持仓观察关键事件，不是核能主题深度报告。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 22:00 CST deliver preview 主体转为 DeepSeek 与 Nvidia 关系解释，且写明 NVDA 行情接口未稳定返回；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是 DeepSeek 关系问答。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-07 14:02-18:01 CST（UTC 2026-08-07 06:02-10:02）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 16:00 CST deliver preview 主体转为解释当前通知偏好和立即推送规则，列出 `earnings_released`、`earnings_upcoming` 等配置项；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是通知配置说明。
  - `job_id=j_eab1a3b2` / `job=NBIS关键事件心跳提醒` 与 `job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒`。
  - 16:30 CST `NBIS` deliver preview 主体转为 NAND Flash 短缺与 NBIS 成本压力的关系分析；15:30 CST `中际旭创` deliver preview 以 NBIS 行情口径和 NBIS 分析组织正文；17:30 CST `中际旭创` 又转成光迅科技上游材料体系全图。
  - 18:00 CST `持仓财报与重大新闻心跳提醒` 一度退化为“我目前具备以下几个核心能力模块”的产品能力介绍，而非持仓财报与重大新闻监控结果。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-07 10:02-14:02 CST（UTC 2026-08-07 02:02-06:02）。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 10:30 CST deliver preview 主体继续转向“小米与英伟达的关系”；11:30 / 12:30 CST deliver preview 又退化成“我是一个以金融分析为核心能力的投研助理平台”产品能力说明。该 heartbeat job 名义目标是 AI / 科技持仓关键事件，不是小米与英伟达关系问答或产品介绍。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 11:30 CST deliver preview 主体转为“当前推送配置确认”，13:30 CST 又转成 NVDA 单独涨跌幅即时价格提醒配置说明；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是通知配置问答。
  - `job_id=j_eab1a3b2` / `job=NBIS关键事件心跳提醒`、`job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒` 在 12:30 CST 前后分别转向 CoWoS 产能与 NAND / NBIS 关系分析，继续体现工具/历史上下文主题抢占当前 heartbeat 目标。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-07 06:02-10:02 CST（UTC 2026-08-06 22:02-2026-08-07 02:02）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 06:30 / 07:00 / 10:00 CST deliver preview 主体继续转为 `grade` / `analyst_grade` 配置解释，10:00 CST 还写“你写的是 **\"grade\"**，结合上下文，这应该是在问 `analyst_grade`（机构评级变动）这条通知的配置状态”，并列出 `immediate_kinds` 等配置字段；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是通知配置问答。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 07:00 / 10:00 CST deliver preview 主体继续转向“小米与英伟达的关系”，包含车载 AI 芯片客户 / 供应商关系说明；该 heartbeat job 名义目标是 AI / 科技持仓关键事件，不是小米与英伟达关系问答。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-07 02:02-06:02 CST（UTC 2026-08-06 18:02-22:02）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 03:00 / 05:30 CST raw 与 deliver preview 继续把 heartbeat 主体转成 `grade` / `analyst_grade` 配置解释，05:30 deliver preview 还列出 `block_kinds`、`allow_kinds`、`immediate_kinds` 等配置字段；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是通知配置问答。
  - `job_id=j_5b3cb604` / `job=光模块板块关键事件心跳提醒` / `target=web-user-499a1c6331c4` 在 05:30 CST deliver preview 变成“你问的是刚才”澄清式正文，并要求用户明确具体想问哪件事；该 heartbeat job 名义目标是光模块板块关键事件监控，不应退化成 direct follow-up 澄清。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-06 22:01-2026-08-07 02:01 CST（UTC 2026-08-06 14:01-18:01）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 23:00 / 23:30 CST deliver preview 主体转为解释 `analyst_grade` / `grade 是什么`，18:00 UTC / 02:00 CST 同一 job 又输出“你的当前推送配置如下”与定时推送配置表；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是通知配置问答。
  - `job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒` 在 23:30 CST 转向 `CoreWeave（CRWV）与 Navitas（NVTS）关系`，02:00 CST 又以 `NBIS $196.87` 行情口径收口；`job_id=j_aa99140a` / `job=光迅科技关键事件心跳提醒` 在 01:30 / 02:00 CST 也以 NBIS 行情和估值分析为主体。
  - 调度和投递主链路正常收口，这些样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-06 18:01-22:01 CST（UTC 2026-08-06 10:01-14:01）。
  - `job_id=j_bb4bbb99` / `job=AI与科技持仓观察关键事件心跳提醒` / `target=web-user-be13e1f84d14`。
  - 22:00 CST raw preview 写工具预算受限后改用已有 web search 信息，并转向“小米与英伟达”的客户 / 供应关系分析。
  - 22:00 CST deliver preview 开头写 `行情口径：待核验`，随后主体为 `小米与英伟达的关系`、车载 AI 计算平台、客户 / 供应关系；该 heartbeat job 名义目标是 AI / 科技持仓观察，不是小米与英伟达关系问答。
  - 调度和投递主链路正常收口，该样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-06 06:01-10:01 CST（UTC 2026-08-05 22:01-2026-08-06 02:01）。
  - `job_id=j_348d0f87` / `job=中际旭创关键事件心跳提醒` / `target=web-user-c2776780c59d`。
  - 10:00 CST raw preview 写 `the user's question about NBIS vs NAND Flash relationship`，随后围绕 `NBIS is AI cloud computing` 与 `NAND Flash` 关系组织分析。
  - 10:00 CST deliver preview 开头写 `行情口径：NBIS $218.99`，随后主体进入 `NBIS vs NAND Flash` 关系、估值和产业链解释；该 heartbeat job 名义目标是 `中际旭创关键事件`，不是 NBIS / NAND 通用关系问答。
  - 调度和投递主链路正常收口，该样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
- `data/logs/hone-console-page-source.log`
  - 最新巡检窗口：2026-08-06 02:00-06:01 CST（UTC 2026-08-05 18:00-22:01）。
  - `job_id=j_35a69a63` / `job=AAPL + NVDA + BE 关键事件提醒` / `target=web-user-9b62484ff43d`。
  - 06:00 CST 触发 prompt 明确是检测 `AAPL + NVDA + BE` 关键事件并即时推送；同窗 job 名和 run_start 均只指向这三个标的。
  - 06:01 CST raw preview 写 `I cannot fetch new data in this round`，随后从历史 reminders 中提取 `COHR: $335.02` 等信息继续组织回答。
  - 06:01 CST deliver preview 开头写 `行情口径：本轮最新可得、非逐笔（COHR 报价取自本轮已确认 NYSE 实体，前序会话最新核验价）`，主体转为 `COHR 当前处于深度超跌状态`、估值和是否重仓介入判断；COHR 不在该 job 目标列表中。
  - 调度和投递主链路正常收口，该样本只影响 heartbeat 内容焦点与目标约束，因此仍按质量性 `P3`；为何不影响功能链路：未见触发、runner、出站投递整体失败，也未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用。
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
- 该问题不同于 `feishu_scheduler_company_news_task_drifts_to_portfolio_trade_advice.md`：本样本来自 Web heartbeat，且是未请求 ticker / ETF 抢占当前目标标的，而不是普通 Feishu scheduler 公司资讯被持仓复盘模板覆盖。
- 该问题也不同于 heartbeat JSON / noop 解析缺陷：本样本已成功 deliver，核心问题是内容主题错误。

## 下一步建议

1. 在 heartbeat prompt / answer 阶段加入当前 job target whitelist，要求任何主体标的必须来自配置或明确说明关联关系。
2. 对工具调用 planner 增加校验：当 `data_fetch quote` 请求了不在 job target / user prompt 的 ticker，记录候选降级并避免将其作为主结论。
3. 增加 Web heartbeat 回归样本：目标列表不含 KWEB 时，输出不得以 KWEB ETF 解释、估值或重仓建议作为主体。

## 最新运行态复核（2026-08-09 18:03 CST）

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-09 14:02-18:03 CST。
  - 15:00 CST `AAPL + NVDA + BE 关键事件提醒` 的 `deliver_preview` 变成“你当前的推送配置如下”，列出定时推送、即时推、阈值与静音配置；该 heartbeat job 名义目标是 AAPL / NVDA / BE 关键事件，不是配置查询。
  - 14:30 / 17:00 / 18:00 CST `闪迪关键事件心跳提醒` 多次转向 NBIS 或 WDC / SK hynix 关系分析；15:00 CST `存储板块关键事件心跳提醒` 转向 SNDK 下跌归因和财报展望；这些输出均不是严格的关键事件监控收口。
- 本轮判断
  - 最新样本仍是 Web heartbeat 执行期主题漂移和目标约束失效；没有新的独立根因。
  - 为何不影响功能链路，因此定级为 P3：调度、runner、出站和去重链路仍在运行，未见错对象投递、数据破坏、敏感信息泄露或全渠道不可用；主要问题是 AI 返回内容焦点不符合用户配置，维持质量性 `P3 / New`。
