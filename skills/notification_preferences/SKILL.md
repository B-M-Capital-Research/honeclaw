---
name: Notification Preferences
description: 管理当前用户的市场事件推送偏好,把中文自然语言映射到 notification_prefs 工具
when_to_use: 当用户想调整"要收什么 / 不要收什么"类的推送开关时(静音、只看重要、不要新闻、只要财报等)
user-invocable: true
context: inline
allowed-tools:
  - notification_prefs
---

## 职责

管理当前 actor(也只能是当前 actor——工具在构造时已绑定身份)对市场事件推送的个人偏好。

## 合法 kind tag

`allow_kinds` / `block_kinds` 的值必须从以下 16 个里选,其它值会被工具直接拒绝:

```
earnings_upcoming, earnings_released,
news_critical, press_release,
price_alert, weekly52_high, weekly52_low, volume_spike,
dividend, split, buyback,
sec_filing, analyst_grade,
macro_event,
portfolio_pre_market, portfolio_post_market
```

## 常见意图 → 工具调用

| 用户说法 | 调用 |
|---------|------|
| "先别推了" / "静音" | `notification_prefs(action="disable")` |
| "恢复推送" / "开回来" | `notification_prefs(action="enable")` |
| "只看重要的" | `notification_prefs(action="set_min_severity", value="high")` |
| "一般的也推吧" | `notification_prefs(action="set_min_severity", value="medium")` |
| "只推我持仓相关的" | `notification_prefs(action="set_portfolio_only", value=true)` |
| "什么都推" | `notification_prefs(action="set_portfolio_only", value=false)` |
| "不要新闻 / 不要分析师评级" | `notification_prefs(action="block_kinds", value=["news_critical","press_release","analyst_grade"])` |
| "只要财报和 SEC" | `notification_prefs(action="allow_kinds", value=["earnings_released","earnings_upcoming","sec_filing"])` |
| "别再限制 kind 了" | `notification_prefs(action="clear_allow")` 或 `clear_block` |
| "看看现在是什么设置" | `notification_prefs(action="get")` |
| "全部恢复默认" | `notification_prefs(action="reset")` |

## 工作流

1. 对非 `get` / `reset` 的改动,先调一次 `get` 看当前状态,避免误覆盖。
2. 应用对应 action。一次对话里可以连续调多个 action(例如先 `set_min_severity` 再 `block_kinds`)。
3. 用返回的 `prefs` 字段给用户复述生效后的设置,确认是否还有需要调整的。

## 注意

- 用户一次说"只要财报",`allow_kinds` 的数组里需要包含 `earnings_upcoming` **和** `earnings_released`——两者是不同的 kind,只填一个会漏。
- `block_kinds` 优先级高于 `allow_kinds`。如果用户之前设过白名单,之后 block 一个重叠的 kind 也会生效。
- 全局(部署方)可能在 `config.yaml` 里把某个 kind 关了——那类事件无论如何都不会推给任何人,即使用户把它放进 `allow_kinds`。这条解释给用户,不要误以为是工具问题。
- 用户没说"所有人"、"全局"时,不要尝试改别人的配置——本工具构造时已硬绑定到当前用户,也没有这个能力。
