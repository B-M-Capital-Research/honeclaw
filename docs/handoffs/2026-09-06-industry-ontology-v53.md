# 2026-09-06 · 行业本体 V5.3：把《HOne Industry Map 最终完整替换稿》装进底稿、注入、skill 与研究台

上一版（V3，见 `2026-09-06-industry-ontology-v3-hone-forward-valuation.md`）把「估值执行卡」做成了子类型级的知识单元。
这一版按用户给的 V5.3 替换稿（`HOne_Industry_Map_最终完整替换稿_V5.3_2026-09-06.docx`）把行业本体升到十行，
并把 V5.3 新增的三层知识做成一等数据：**可观测变量**（定义 · 取数 · 节奏 · 财务传导）、**变量传导与证伪**、
**十二个估值回答执行字段**（连同全局技术口径与 20 条验收算例）。

提交：`17a25de6`（main）。生产：后端切到 17a25de6，harness 装 industry-map / valuation-audit 两个 skill，Pages 自动上线。

## 1. 数据层：规则与事实分开合并

底稿 `skills/industry-map/references/industry-map.json` 升到 **schema 4**。合并脚本（会话 scratchpad `merge_v53.py`，
转写稿 `industry_v53.json` 由子代理从 docx 文本程序化转写，每个片段断言是原文子串）按一条原则处理：

- **V5.3 是规则层与定义层，整表替换**：方法论（21 节 → `execution_rules`，12 执行字段 → `output_fields`，
  `technical_conventions` 11 条、`acceptance_cases` 20 条、`references` 6 条）、每行的 `valuation.logic`（公式、一段话、原文）、
  `valuation.anchor`、`subtypes`（32 个，每家成员恰好归入一个）、`upstream_summary`、`transmission`、`observables`（70 条）、
  `subtype_intro`、`sources_note`；`one_liner`、`state_note` 也取 V5.3。
- **带日期的事实层原样保留**：`driver_chain`、`key_variables`（带日期的量）、`core_watch` 里的数字与截至日、
  `upstream_signals`（含 `latest`）、`sources`。V5.3 的「核心关注点」是没有数字的关注句，追加到 `core_watch` 末尾（按开头去重）。
- **V5.3 没写的字段不用空值或套话覆盖**：`upper_range_drivers` 在光通信 / 新云 / 服务器 / 云厂 / 太空为空 → 旧行沿用 V3；
  `forward_focus` 只有 AI 芯片、服务器、云厂有 → 其余沿用 V3；`revision_optionality` 若只剩那句通用的「盈利上修与倍数扩张分别举证…」
  → 存储 / 光通信 / 电力沿用 V3 的行内句。`hindsight_error`、`demand_chain` 也保留 V3（页面还在用）。
- **行名**沿用站内「中英之间留空格」写法：`光通信 → 光通信与 AI 互连`、`电力 → 电力与供电`；新行 `space`「商业太空」
  （RKLB；SpaceX 未上市只写在子类型 `scope_note`）与 `ai-apps`「AI 应用与数据服务」（PLTR / SNOW / TEM / HIMS）。
  新行没有 V2 的长短版锚，用 V5.3 的倍数锚原文与禁止清单填 `multiple_anchor` / `anti_pattern`（短版裁到 110 字）。

模型层（`hone-core::industry_map`）新增：`Methodology.{technical_conventions, acceptance_cases, references}`、
`Observable{name, definition, source, cadence, transmission}`、`IndustryValuation.{upstream_summary, transmission, observables,
subtype_intro, sources_note}`、`Subtype.scope_note`；四个散文字段进 `VALUATION_TEXT_FIELDS`（页面与 `industry_map_edit` 都能改），
`upsert_subtype` 接受 `scope_note`。可观测变量表与全局口径只在底稿里改（没有编辑入口，故意的）。

## 2. 消费层：注入与 skill

`prompt.rs::industry_baseline`（每轮命中树内公司时注入）：

- 头部改成 V5.3 的规则：典型经营状态定阶段 → 远期收入按项目 / 批次 / 客户群评 **A / B / C 级** → FY+2 / FY+3 进基准的七项核可 →
  主锚次锚照子类型卡、EV/Sales 旁列隐含 EV/EBIT → **当前共识倍数 vs 当前模型倍数** → 盈利上修与倍数扩张分别举证并写交互项
  （总价差 = e + r + e×r）→ 倍数不变结果 → 资本桥不闭合只给条件 EV → DCF 定价权重为零但时间换算照做。
- 压缩版全局规则改挑 V5.3 的六节（远期收入的证据等级 / 共识与市场起点 / 利润与普通股资本桥 / 估值贡献拆解 /
  双重上修与不重估检验 / 结论与红绿灯边界）；字段清单换成十二个执行字段名。
- 行内多三行：`上游信号（需求怎么传到本行）`、`变量传导与证伪`、`可观测变量（…未公开的标缺失，不拿行业代理量冒充本公司实测）：名字×5`。
- 预算：单行命中 < 5,600 字（原 4,600），测试守着。

`skills/valuation-audit/SKILL.md`：执行卡整段换成十二个执行字段（每个字段写明落在对账表 / 三问 / 三情景 / 反向估值 / 结论的哪一处），
收尾由三行改五行（两种倍数 / 倍数不变结果 / 价差贡献 / 估值状态 / 红绿灯），加双重上修依赖与红绿灯边界。
`skills/industry-map/SKILL.md` 补 V5.3 字段表。`industry_map_edit` 的描述列出新字段。

## 3. 研究台（管理员页）

页面在 bf 拆模块之后（`public-industry-map/*.tsx`）加的都是新区块，不动权限与装配：

- 方法论条：`全局技术口径`、`估值引擎验收算例`、`方法与核验来源` 三个折叠表。
- 估值执行卡：子类型芯片上方一段 `subtype_intro`；选中子类型的卡里多一行「范围」（`scope_note`）；子类型表单多「范围」一栏。
- 底层估值逻辑：`变量传导与证伪` 段 + 四列 `可观测变量` 表（变量 / 定义与口径 / 去哪取 · 多久更新 / 怎么传到财务）。
  旧的「传导链与旧版锚（带日期的量）」表还在——那是事实层，两张表各司其职。
- 最近变化：读模式开头一段 `upstream_summary`；编辑态多一个编辑框。研报与数据来源：`sources_note` 一段 + 编辑框。
- 数据中心场景（`data-center-model.ts`）：光 / 电两个区的行名跟着改；AI 应用并入「AI 软件与云平台」区；新增「太空与轨道连接」区
  （锚点放在屋顶远角上方，故意不画几何），这样契约测试「每一行都能从场景到达」继续成立。

新字段在前端类型里全是可选：后端与前端分开上线，旧后端不带这些字段时页面按空值渲染。
**踩过的坑**：页面各模块拿到的是 `valuationOf(industry)` / `methodologyOf(snapshot)` 归一化后的对象（`lib/industry-valuation.ts`），
它们是白名单式重建——第一版忘了把新字段带过去，API 明明返回了 70 条可观测变量，页面一个都没渲染（只有随 `subtypes` 整体透传的
`scope_note` 出来了）。以后给 valuation / methodology 加字段，先改这两个函数与它们的测试。
CSS 只能用已定义的 `--hone-*` 令牌（契约测试 `public-design-token-contract` 会拦），正文色取 `--hone-ink-800`、备注 `--hone-ink-400`。

## 4. 测量

打分器 `score_v53.py`（scratchpad）看回答里有没有 V5.3 的痕迹：证据等级、两种倍数、价差贡献、倍数不变、资本桥、估值状态、
上游动作、子类型/锚、红绿灯、无 LaTeX。

V3 时期基线（ont4 批次，8 题）：证据等级 1 / 两种倍数 0 / 价差贡献 0 / 倍数不变 0 / 资本桥 8 / 估值状态 0 / 上游动作 6 / 子类型锚 7 / 红绿灯 0 / 无 LaTeX 5。

V5.3 上线后第一轮（v53 批次，8 题：NVDA、SNDK、RKLB、PLTR、MU、VST、光通信行业、TEM，全部 success）：
证据等级 **0** / 两种倍数 **7** / 价差贡献 **6** / 倍数不变 **6** / 资本桥 8 / 估值状态 **5** / 上游动作 **8** / 子类型锚 6 / 红绿灯 **1** / 无 LaTeX **8**。
RKLB 的回答落在「商业太空」行（Electron / Neutron / SpaceX 只作范围）、PLTR 落在「AI 应用与数据服务 · 企业工作流软件」，
两条新行第一次就被消费。没落地的两项——**证据等级**与**红绿灯**——在头部规则里有、在回答模板里没有位置，
所以第二轮给它们各加一个落点：对账表多一行「远期收入证据等级（A / B / C 各级规模）」，收尾四行改五行（第五行是四盏灯）。
第二轮（v53b 批次，4 题：NVDA、SNDK、RKLB、PLTR，harness 884b1a96）：证据等级 **4/4** / 两种倍数 4 / 价差贡献 4 / 倍数不变 4 /
资本桥 4 / 估值状态 4 / 上游动作 3（PLTR 所在的 AI 应用行还没有上游信号，合理）/ 子类型锚 4 / 红绿灯 **4/4** / 无 LaTeX 3
（SNDK 一处 `\times`，前端 `normalizeMathToPlainText` 会转成纯文本，读者看不到）。四份回答都在对账表里写出了
「远期收入证据等级 | A 级 $… / B 级 $… / C 级 $…」，收尾都有「红绿灯」小节——同一条规则，头部里说了没人用，模板里给一行就全用。

## 5. 部署与核验

- 后端：Runtime Image 34028421410 → `crane digest` → stage → **先装 harness 再切 symlink** → restart → `127.0.0.1:8077/api/meta` 的
  `build.git_sha` = 17a25de6…；`bin/hone-cli` 里 grep「十二个执行字段」。
- harness：`git archive 17a25de6 skills soul.md`，66 个文件，CHANGED 只有 industry-map 与 valuation-audit，soul.md 不变。
- 前端：Pages 在 push 后 ~2 分钟出新分块，`public-industry-map-C3lcXNrq.js` 里能 grep 到 `industry-observables`，
  `public-data-center-ClI8Q0iG.js` 里能 grep 到「太空与轨道连接」。

## 6. 已知边界

- hone-channels 全量并行套件仍有 ~38 个环境漂移失败（`PostgreSQL must be configured for the runtime`，mcp_bridge / scheduler 等），
  与本次无关；本次相关的 `industry_map` / `industry` / `prompt::industry_baseline` 全绿。
- 转写稿里 references 最后一条（NVIDIA KV 缓存文档）在 docx 里没有说明行，`requirement` 为空。
- `space` 的「综合太空与连接平台」子类型没有成员（SpaceX 无代码），页面芯片显示 0，是有意的。
