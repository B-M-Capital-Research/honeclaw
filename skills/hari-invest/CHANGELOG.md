# Hari Invest 变更记录

版本口径见 `references/provenance.md` 的「版本号」一节：产品版本、蒸馏框架版本、已确认逻辑版本
各标一件事，不互相换算。本文件记的是**产品版本**这一轴。

## [0.3.0] - 2026-08-29

把 `hari-invest` 从「投研轮的终点」改成「投研轮的路由点」。此前 `prompt.rs` 每轮强制加载它，
而它的资源路由只列自己的 references，模型加载完就动笔，结论普遍停在「持有区（中置信度）」
这类分档标签上。

### 新增

- 「先按问题类型接力，再谈本 skill 的 references」：一张「本轮问题 → 除本 skill 外还要加载谁」
  的表，指向 `stock_research` / `valuation-audit` / `market_analysis` / `sector-to-stock` /
  `etf-analysis` / `analyst-coverage` / `moat` / `scarcity-differentiation` / `first-principles` /
  `fundamentals` / `position_advice` / `portfolio_management`。**只给指针，取数口径与产出格式
  一律以被指向的 skill 为准。**
- `references/provenance.md` 的 A–E 来源分层表：已确认逻辑 / 产品方法 / 历史基线 / 本轮证据 /
  尚未确认，并写明 D 可以推翻 A、B、C 的历史结论。
- `references/logic-index.md` 的「产品方法与候选隔离」：产品方法不编号、不算第七条逻辑；
  拥挤度与基本面冲突时孰先仍是开放问题，不编固定减仓次序。

### 修正

- 估值一节改为「没有第三方一致预期时用一手数据自建分母」，倍数仍走本轮三问推导。
- 版本号自相矛盾（正文 `0.1.0` 与 provenance `0.2.0`）已统一到三轴口径。

### 从 oldwang 分支的 0.3.0 提案中**未采纳**的部分

`origin/oldwang` 在同一天独立提出了一个 0.3.0。两者共享版本号与来源分层思想，但内容不同，
以下几项经核对后没有并入 main，理由记在此处以免下次重复讨论：

- **7 步传导链写进本 skill**：逐环节已被 `first-principles`（需求函数与单位用量）、
  `moat`、`scarcity-differentiation`（供给/替代与价值捕获）、`fundamentals`（财务兑现）、
  `valuation-audit`（三情景、跨族交叉、反向估值）、`stock_research`（收口）覆盖，
  且那边的写法更可判定。搬进来等于同一规则两处口径。
- **「没有一致预期就给 12x/15x/18x 机械反向门槛」**：与 `valuation-audit`
  「倍数必须本轮推导、类型表只是选区间的先验」冲突。
- **`kernel-manifest.md` 的「代码强制点 / 默认调用顺序 / 验收资产」三节**：其中点名的
  `investment_decision_context.rs`、evals runner 与 491 轮台账在 main 上不存在或未接线，
  照搬即写入不实文档。只取了 A–E 来源分层表，已并入 `references/provenance.md`。
- **`symbol == "SNDK"` 的专项提示词与关键词校验器**：ticker 硬编码 + 关键词命中式内容门禁，
  与仓库 `AGENTS.md` 的门禁禁令冲突。语义已泛化进 skill 层，实现未采用。

## [0.2.0] - 2026-08-11

形成 HONE 对话决策层、四态研究区间与「结论优先」的输出契约。

## [0.1.0] - 2026-08-02

六条已确认逻辑 `LOG-V0001`—`LOG-V0006` 进入本地 HONE 内部草案，证据截止 2026-08-02。
