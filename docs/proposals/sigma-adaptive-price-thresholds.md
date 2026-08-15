# σ-自适应价格警报阈值 — 设计与验收标准

- **日期**: 2026-08-15
- **状态**: 已实施（2026-08-15，A1–A11 全过；回归测试
  `sigma_adaptive_thresholds_regression`，实现 `crates/hone-event-engine/src/volatility.rs`）
- **背景**: 2026-08 推送体检（`docs/bugs/event_engine_quiet_flush_price_band_ladder_noise.md`）
  确认固定 `price_alert_low_pct=2.5 / high_pct=6.0` 与持仓组合严重不匹配：
  持仓全部为高波动 AI-infra 标的（60 日日波动率 σ 介于 2.4%–9.3%），
  SNDK（σ≈8.9）约每 1.2 个交易日触发一次警报，而这些波动对它自己是"寻常日"。
  用户已被迫手动把 actor 直推阈值调到 ±8%（`price_high_pct_up/down_override=8.0`），
  证明系统默认档位失真。

## 设计

按标的自身波动率缩放 poller 层的 low/high 阈值：

```
σ = 过去 60 个交易日 close-to-close 简单收益率的样本标准差（%，n-1）
low_eff  = clamp(1.75σ, 2.0, 8.0)    # 事件是否存在（入 digest 的门槛）
high_eff = clamp(3.50σ, 5.0, 12.0)   # poller 基础 High / 盘中 band 起点
```

- **数据源**: FMP `/v3/historical-price-full/{sym}?timeseries=…&serietype=line`，
  每标的每 ET 交易日拉一次并缓存（复用 extended_hours prev_close_cache 模式）。
- **回退**: σ 不可得（新股、样本 <20 天、API 失败）→ 沿用配置的固定阈值。
  失败不缓存，下个 tick 重试。
- **同日稳定**: σ 按 (symbol, ET 日期) 缓存，盘中阈值绝不漂移（band id 依赖此稳定性）。
- **适用范围**: `PricePoller`（盘中 + 收盘）与 `ExtendedHoursPoller`（盘前/盘后振幅）。
  `price_realert_step_pct`（band 步长 2%）**本期不动**——band 起点已随 high_eff 上移，
  digest 侧阶梯合流已在上一轮修复。
- **透明度**: 事件 payload 增加 `hone_price_sigma_pct` / `hone_price_low_threshold_pct`
  / `hone_price_high_threshold_pct`；summary 附注本次波动 ≈ 多少个 σ。
- **配置**: `thresholds.price_sigma.{enabled,lookback_days,min_samples,low_mult,high_mult,
  low_floor_pct,low_cap_pct,high_floor_pct,high_cap_pct}`，默认启用，
  默认值即上式常数。`enabled: false` 时行为与现状完全一致。

### 关键约束（为什么上限是 8.0 / 12.0）

- **low_cap = 8.0** 必须 ≤ 用户 actor 的 `price_high_pct_override`（当前 8.0）：
  保证 |涨跌| ≥ 8% 的事件永远被产出，router 层的用户覆盖不会被 poller 静默架空。
  这同时是绝对兜底：无论标的多疯，±8% 的日子一定至少进 digest。
- **high_cap = 12.0**: SNDK/NBIS/AAOI 这类 σ≈9 的标的 3.5σ≈31%，不设上限等于
  永不 High。12% 是"即使对最疯的标的也值得立即知道"的绝对档。
- **乘数锚定**: 对典型 σ≈1.4% 的市场股，low_eff≈2.45 / high_eff≈4.9 —— 与现行
  默认 2.5/6.0 基本重合，普通标的行为几乎不变，变化集中在极端波动标的上。

## 验收标准（实施前定稿）

回归数据集：`crates/hone-event-engine/testdata/daily_closes_2026-01-02_2026-08-14.json`
（18 个 watch-pool 标的 × 155 个交易日真实 FMP 日线，共 2790 行；评估窗口
2026-04-27 → 2026-08-14 即真实推送期 77 个交易日，之前的数据仅做 σ 热身）。
回归测试必须用**生产代码路径**（σ 计算函数 + 阈值缩放函数）重放该数据集，
逐日无前视（σ 只用 t 日之前的收盘价）。

| # | 断言 | 预期（模拟实测） |
|---|------|-----------------|
| A1 | 警报日总数（\|日收益\| ≥ low_eff）较固定阈值下降 ≥ 60% | 841 → 255（−70%） |
| A2 | High 日总数（\|日收益\| ≥ high_eff）下降 ≥ 70% | 380 → 90（−76%） |
| A3 | **零漏报**：所有 \|日收益\| ≥ 3σ 或 ≥ 10% 的极端日仍然触发 | 0 漏 |
| A4 | 被抑制警报的最大 z 值 < 2.0σ（只静音统计上寻常的波动） | max z = 1.74 |
| A5 | 各标的警报率（警报日/交易日）的跨标的标准差严格下降 | 0.141 → 0.099 |
| A6 | 最吵标的警报率 ≤ 0.45（固定阈值下 SNDK = 0.82） | max = 0.42 |
| A7 | SNDK 警报率 ≤ 0.35（用户原始投诉标的） | 0.31 |
| A8 | σ 样本 < min_samples 时回退固定阈值（单测） | — |
| A9 | 同一 (symbol, 交易日) 的阈值在多次调用间不变（单测） | — |
| A10 | `enabled: false` 时所有产出与现状 bit-for-bit 一致（单测） | — |
| A11 | 现有全量测试 + `replay_push_quality_audit` 离线回放全部通过 | — |

注：A1–A7 以设计期模拟数据为基准的**特征固定测试**（characterization test），
防未来回归；数值门槛已留余量（实测 −70% 断言 ≥60%）。

## 明确不做（本期）

- band 步长 σ 缩放（等观察 high_eff 上移后的实际 band 量再定）
- per-actor σ 乘数覆盖（先验证系统级效果）
- 成交量 σ（`volume_sigma` 是另一条路径，不合并）
