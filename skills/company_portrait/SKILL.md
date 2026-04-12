---
name: Company Portrait
description: 建立或更新公司画像，在长期研究过程中把核心结论沉淀到 profile.md 与事件时间线
when_to_use: 当用户正在系统研究某家公司，想沉淀长期画像、补充 thesis、追加财报/公告事件，或询问是否已有画像时使用
allowed-tools:
  - company_profile
user-invocable: true
context: inline
aliases:
  - company portrait
  - 公司画像
  - 长期画像
---

## Company Portrait

这个 skill 用于把单次研究升级为可持续维护的“公司画像”。

### 何时触发

- 用户正在系统研究一家公司，而不是只问一句短问题
- 用户明确说希望长期跟踪、沉淀画像、建立档案、记录 thesis
- 用户在查看财报、公告、管理层变化后，希望把新信息追加到已有画像

### 首次建档流程

1. 先调用：

```text
company_profile(action="exists", company_name="...", stock_code="...")
```

2. 如果还没有画像：
   - 必须先询问用户是否要建立长期画像
   - 不要自动创建

3. 用户确认后，再调用：

```text
company_profile(
  action="create",
  company_name="...",
  stock_code="...",
  sector="...",
  industry_template="general|saas|semiconductor_hardware|consumer|industrial_defense|financials",
  sections={
    "投资主张": "...",
    "商业模式": "...",
    "行业与竞争格局": "...",
    "护城河": "...",
    "管理层与治理": "...",
    "财务质量": "...",
    "资本配置": "...",
    "估值框架": "...",
    "核心风险": "...",
    "关键跟踪清单": "...",
    "未决问题": "..."
  }
)
```

4. 创建后告诉用户：
   - 已建立画像
   - 后续研究会默认追加到这份画像
   - 只有出现实质新增事实时才会写入时间线

### 已有画像时的更新流程

1. 先读取：

```text
company_profile(action="get_profile", profile_id="...")
```

2. 用已有画像作为本轮研究上下文。

3. 若本轮只是在重复已有结论，不要写任何更新。

4. 若出现实质新增事实，例如：
   - 财报 / 指引变化
   - 积压订单 / ARR / NRR / 净息差等关键指标变化
   - 护城河强化或弱化
   - 管理层与资本配置判断变化
   - 风险兑现或 thesis 被削弱

   则调用：

```text
company_profile(
  action="append_event",
  profile_id="...",
  title="...",
  event_type="earnings|filing|news|management_change|review|metric_delta|capital_allocation",
  occurred_at="...",
  thesis_impact="positive|neutral|negative|mixed|unknown",
  changed_sections=["..."],
  refs=["..."],
  what_happened="...",
  thesis_effect="...",
  follow_up="..."
)
```

5. 若长期判断已经变化明显，再调用：

```text
company_profile(
  action="rewrite_sections",
  profile_id="...",
  sections={
    "投资主张": "...",
    "护城河": "...",
    "财务质量": "..."
  }
)
```

### 行业模板选择

- `saas`：SaaS / 软件订阅
- `semiconductor_hardware`：半导体 / 硬件 / 设备
- `consumer`：消费 / 零售 / 品牌
- `industrial_defense`：工业 / 制造 / 国防
- `financials`：银行 / 保险 / 券商等金融
- 无法明确归类时使用 `general`

### 严格规则

- 未经用户确认，不得首次创建画像
- 不要把日内价格波动写进长期画像
- 不要把一句空泛评价写成事件
- 只有“净新增事实”才值得追加事件
- 画像是长期研究资产，不是交易建议清单；不要输出直接买卖指令
