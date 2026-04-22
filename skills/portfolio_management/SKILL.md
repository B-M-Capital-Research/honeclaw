---
name: Portfolio Management
description: Manage user holdings AND watchlist — supports add / update / remove / watch / unwatch with ticker validation; watchlist entries receive the same active push events as real holdings
tools:
  - portfolio
  - data_fetch
---

## Portfolio Management Skill

每个用户有独立的持仓+关注记录。写操作前**必须**先验证 ticker，否则关注/持仓可能永远命中不到推送。

### 心智模型:持仓 vs 关注

| | 持仓 (Holding) | 关注 (Watchlist) |
|---|---|---|
| 何时使用 | 用户真实买入了标的 | 想追踪但未买入 |
| 必填字段 | ticker + shares + cost_basis | 仅 ticker |
| 影响资金统计 | ✅ 计入 total_shares / P&L | ❌ 不计入任何资金口径 |
| 主动推送 | ✅ 收新闻/价格异动 | ✅ 与持仓**同级**,同样收推送 |
| 底层字段 | `tracking_only: false/缺省` | `tracking_only: true` |

关键原则:**如果用户没说明 shares,一律用 `watch` 而不是 `add shares=0`**。前者走关注路径,后者会在资金统计中留下 0 股的持仓垃圾。

### Ticker 验证流程

用户常给缩写/中文名/口误,验证是硬性要求:

1. 不确定时先调 `data_fetch(data_type="search", symbol="...")`
2. 从搜索结果拿 `symbol` 和 `name`
3. 用确认后的 ticker 执行 `add` / `watch` / `remove` / `unwatch`

示例:
- 用户说"tem" → 搜 `TEM` → 得到 Tempus AI → 再写入
- 用户说"英伟达" → 搜 `nvidia` → 得到 NVDA
- 用户说"特斯拉" → TSLA 众所周知,可直接用

### Tool 调用速查

| 自然语言 | Tool 调用 |
|---|---|
| 查看我的持仓/关注 | `portfolio(action="view")` 返回 `{holdings: [...], watchlist: [...]}` |
| 帮我关注 NVDA | `portfolio(action="watch", ticker="NVDA")` (幂等) |
| 我以 175 买了 100 股苹果 | `portfolio(action="add", ticker="AAPL", quantity=100, cost_basis=175)` |
| 我买了之前关注的 NVDA,200 股 180 成本 | `portfolio(action="add", ticker="NVDA", quantity=200, cost_basis=180)` — 若 NVDA 在关注列表会**自动转持仓**,返回 `promoted_from_watchlist: true`,告诉用户"已从关注升级为持仓" |
| 取消关注 NVDA | `portfolio(action="unwatch", ticker="NVDA")` (仅删关注,若是真实持仓会拒绝,提示用 remove) |
| 我不持有苹果了 | `portfolio(action="remove", ticker="AAPL")` (持仓/关注通用) |
| 更新苹果成本价 | `portfolio(action="update", ticker="AAPL", cost_basis=...)` |

### 核心工作流

1. 先 `view` 查当前状态,判断用户意图是"加关注"还是"加持仓"
2. 没提 shares/cost → **关注路径**(`watch`)
3. 提了 shares/cost → **持仓路径**(`add`);若已在关注列表会自动 promote,务必向用户汇报这一点
4. 删除意图要区分:
   - 想删除"只是不想再看到推送"的关注 → `unwatch`
   - 想删除真实持仓 → `remove`
5. 期权关注同样支持:`watch` 时 ticker 可由 `underlying/expiration_date/option_type/strike_price` 自动生成

### 与通知偏好联动

若用户说"我只想收持仓和关注标的的推送,别的不要",去 `notification_preferences` skill 调 `set_portfolio_only=true`。关注标的**与持仓同级**触发 registry,自动进入白名单。
