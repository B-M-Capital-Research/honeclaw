# 2026-09-06 行业本体 V3：HOne 八行业前瞻估值逻辑与倍数锚

## 输入

用户给的《HOne 八行业前瞻估值逻辑与倍数锚 · HOne 执行版 V3.0》（Word）。它把每一行压成两类规则：
**底层估值逻辑**（未来 1–3 年收入、利润、现金流为什么变：AI 需求 → 资源强度 → 物理需求 → 合格供给 → 产能释放
→ FY+1~3 盈利 → 前瞻倍数，每行带量化关系式）和**倍数锚**（什么阶段用哪个前瞻财年、哪一族倍数、什么权重、
上沿由什么决定、什么被禁止）；外加八条通用执行规则（Forward denominator / Capacity Unlock Gate / Earnings Revision
Optionality / Multiple selection / Market-implied check / DCF 默认禁用 / Hard checks / 后视镜错误）与 14 个强制输出字段。

## 设计：知识怎么存、怎么被消费

- **存**：`industry-map.json` schema 3。根上挂 `methodology`（七段需求链、通用执行规则、强制输出字段、最终原则）；
  每行挂 `valuation = {logic, anchor, subtypes[]}`。**子类型**是新的最小消费单位：同一行里价值链位置不同的公司
  （平台型 / 连接·IP 型 / 制造封装型；DRAM·HBM / NAND·eSSD / HDD；器件激光器 / 制造组装 / 连接芯片 / 网络系统……）
  各自有主锚、次锚、适用阶段与提醒，`members` 指向公司；文档没点名的公司按最近子类型归类并标 `inferred_members`，
  页面上带「推断」标签，管理员一键改归属。
- **消费**：`prompt.rs::industry_baseline` 重排为「上游最近动作 → 估值执行卡（命中公司所属子类型的主锚/次锚/适用阶段/提醒
  + 行级上沿/上修期权/禁止清单）→ 底层估值逻辑（一句 + 两条公式 + 未来 1–3 年先看 + 典型 State）→ 带日期的传导链
  → 核心关注点」，头部带通用执行规则压缩版与 14 个输出字段名；单行命中守在 4,200 字以内。
  `valuation-audit` 加「树内公司的前瞻估值执行卡」十步（State / Demand vs Qualified Supply / Capacity Unlock /
  Valuation Horizon 含 Gate 五项 / Primary·Secondary 权重 / Earnings Revision Optionality / Bear·Base·Bull /
  Market Implied 不用 Reverse DCF / DCF Disabled / Integrity Check 自检），全部落在对账表、三问、三情景的现有位置。
- **编辑**：新 op `set_valuation_field` / `set_valuation_list` / `upsert_subtype` / `remove_subtype` / `set_member_subtype`，
  页面与对话工具 `industry_map_edit` 同一份追加日志；旧日志里的所有 op 原样重放。
- **交互**：页面顶部方法论条（七段需求链、核心原则、可展开的通用规则与输出字段）；行详情第一块是估值执行卡
  （子类型 chip → 卡片 → 成员 chip）；成员表多一列子类型和「问 HONE」按钮（跳到对话并自动发送按本体做前瞻估值的提问）；
  树上方加找公司（代码或中文名 → 定位到行与子类型）；上游最近动作超过 100 天显示「可能已过期」。

## 发布与复测

（待填）
