---
name: Missed Digest Items
description: 查回 digest 没推但用户可能想看的事件(数量上限截断 / 同 ticker 冷却 / 噪音过滤等)
when_to_use: 用户问「我没看到什么」「digest 漏了什么」「/missed」「最近被砍掉的事件」
user-invocable: true
context: inline
allowed-tools:
  - missed_events
---

## 职责

当用户敲 `/missed` 或自然语言问「digest 没推什么」时,调 `missed_events` 工具捞回
event-engine 的 `delivery_log`,把被主动筛掉的事件用人话报给用户。

## 工具调用模板

默认看过去 24 小时:

```
missed_events()
```

用户提到「最近一周」「7 天」之类时把 `since_hours` 调到 168:

```
missed_events({"since_hours": 168})
```

用户嫌列表太长,可以缩 `limit`(默认 30):

```
missed_events({"since_hours": 24, "limit": 10})
```

## 输出格式约定(回给用户)

按下面这个 schema 渲染,**不要把工具的原始 JSON 直接贴出来**:

```
🗂 过去 N 小时被筛掉的事件 · 共 X 条

【数量上限截断】(capped / price_capped)
• $AMD [新闻] · AMD's earnings preview 标题
  来源:fmp.stock_news:foo  · 时间:HH:MM

【同 ticker 冷却】(cooled_down / price_cooled_down)
• ...

【噪音过滤】(omitted)
• ...

【你的偏好过滤】(filtered)
• ...
```

按 status 分组,每组只在确实有条目时才列出标题。每条 1-2 行(标题 +
来源 + 时间)。Symbol 用 `$AAPL` 形式开头让用户一眼看到关注的标的。
URL 有就用 `<a>` 链接(渠道支持时);否则不强加。

## 注意

- 用户问「为什么 X 没推」: 工具结果里 `status` + `reason` 字段已经回答了,
  原文报给用户。如果工具没返回 X,大概率 X 根本没进 router(没匹配到持仓,
  或被全局 disabled_kinds 砍),这种情况告诉用户「没在最近 N 小时的 digest
  策略里被处理过,可能根本没进 event 引擎」。
- 工具只查**当前用户自己**的;构造期已硬绑定 actor,用户不能用此命令查
  别人。
- 工具默认 24 小时;不要无理由扩到 168(全 7 天数据量大,渲染长)。
