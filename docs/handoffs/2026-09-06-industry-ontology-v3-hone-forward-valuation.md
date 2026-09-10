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

- 改前基线 `v3base`（生产 05fdf675，8 道估值题：SNDK / LITE / BE / CRWV / DELL / KLAC / NVDA / ORCL）：
  State 0/8、Capacity Unlock Gate 五项 0/8、主次锚带权重 2/8、Market Implied 1/8、DCF 禁用 0/8、Integrity Check 0/8、
  Earnings Revision Optionality 0/8；上游最近动作 8/8（V2 的成果）。回答平均约 5,000 字。
- 发布：`9f67f32b`（镜像 `…@sha256:f01fb3af10006a0dc07055792660e9d26b7473075c194cef8a0b7c490af83937`），2026-09-06 05:18 UTC
  切换：current → 9f67f32b，previous → 05fdf675，e867eae3 已清，磁盘 6.1G；NRestarts=0、无 error；harness 换了
  fundamentals / industry-map / valuation-audit，落盘底稿 schema 3、22 个子类型；Pages 已带估值执行卡 / 找公司 / 问 HONE。
  本地用真实 `hone-console-page` 验过：方法论条、子类型 chip 与卡片、找公司「SNDK · 存储 · 纯NAND/eSSD公司」定位、
  「问 HONE」跳到对话并自动发送提问（本地无 LLM key 所以回答失败，跳转与预填正确）。
- 第一轮复测 `v3new`（同 8 题）：State 6/8、Horizon 8/8、Gate 五项 1/8、主次锚权重 3/8、Market Implied 2/8、DCF 0/8、
  Integrity 0/8、ERO 0/8——执行卡的前半段（State / 财年 / 子类型）落了，收尾字段没落：十步清单太长，末三步被吃掉。
- 修正 `92a27c34`（只改 `valuation-audit`，harness 05:23 UTC 装入）：三情景表之后紧接三行 `Earnings Revision Optionality` /
  `DCF：Disabled / 0%` / `Integrity Check：PASS / WARN / FAIL + Hard checks`，权重与 ERO 并入三问的产出。
- 第二轮复测 `v3new2`：

| 字段 | 改前 | V3 第一版 | V3 第二版 |
|---|---|---|---|
| State 四选一 | 0/8 | 6/8 | 6/8 |
| Capacity Unlock Gate 五项 | 0/8 | 1/8 | 3/8 |
| 主次锚带权重 | 2/8 | 3/8 | 7/8 |
| Market Implied（现价隐含 FY+1~3 倍数） | 1/8 | 2/8 | 8/8 |
| DCF Disabled | 0/8 | 0/8 | 6/8 |
| Integrity Check | 0/8 | 0/8 | 7/8 |
| Earnings Revision Optionality | 0/8 | 0/8 | 7/8 |
| 上游最近动作（V2） | 8/8 | 8/8 | 8/8 |

  SNDK / LITE / NVDA 三题十项全中；KLAC 最弱（只落了财年、Market Implied、子类型），设备行的高服务占比子类型卡
  还需要再看一轮真实提问。回答平均约 6,000 字，比改前多约 1,000 字。

## 下一步

- 设备 / 云厂两行的子类型卡写得比其它行薄（文档本身对这两行只有一条通用规则），管理员可在页面上补 `note` 与权重。
- 「推断」成员 29 家（AMD / AVGO / 制造封装型 4 家 / 新云 3 家 / 设备 10 家…）等人工确认：页面成员表的子类型下拉一键改。
- 上游最近动作的季度更新仍是手动（英伟达财报后改 `latest`）；可做成财报后自动起草、管理员确认。
