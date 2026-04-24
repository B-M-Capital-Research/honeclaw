# Bug: event-engine 收盘大幅波动永远不会即时推送

状态：`Fixed`

修复进展：2026-04-24 已修复。`price_close` 现在仍保留独立 id / close window，但当绝对涨跌幅达到全局 `price_alert_high_pct` 时会生成 `Severity::High`；router 的 per-actor `price_high_pct_override` 也不再排除 `window="close"`，因此超过系统直推地板或满足大仓位敏感阈值的收盘波动可以进入即时 sink。普通收盘波动仍维持 Low / digest，避免把所有 close 事件都升级成凌晨即时提醒。

## Summary

持仓命中的 `fmp.quote` 收盘异动即使绝对涨跌幅已经超过全局 `price_alert_high_pct=6.0`，当前实现仍会把 `price_close:*` 固定记为 `severity=medium` 并仅进入 digest，不走即时 sink；这会让用户对重要收盘波动的提醒至少延后到下一个 digest 窗口。

## Observed Symptoms

- 全局阈值配置明确把高优先级价格异动门槛设为 `6.0`，而默认盘后汇总窗口故意延后到北京时间 `09:00`：

```text
config.yaml:162-187
event_engine:
  enabled: true
  dryrun: false
  ...
  digest:
    timezone: Asia/Shanghai
    pre_market: "19:00"
    post_market: "09:00"
```

- 命中的 direct actor `telegram__direct__8039067465` 还把自己的 digest 窗口改成了 `02:30/09:00/12:00/16:00/19:00/21:00`，并配置了更激进的 `price_high_pct_override=4.0`，说明用户对价格异动提醒的敏感度是被显式调高过的：

```text
data/notif_prefs/telegram__direct__8039067465.json:1-28
{
  "timezone": "Asia/Shanghai",
  "digest_windows": ["02:30","09:00","12:00","16:00","19:00","21:00"],
  "price_high_pct_override": 4.0,
  ...
}
```

- 但在上次巡检之后的新事件里，3 个持仓命中的收盘价异动都只被写成了 `medium + queued`，没有任何 `sink sent` 记录：

```text
data/events.sqlite3 (read-only query)
price_close:AAOI:2026-04-23|medium|2026-04-23 20:02:46|AAOI -7.82%|queued:telegram::::8039067465
price_close:RKLB:2026-04-23|medium|2026-04-23 20:02:46|RKLB -6.04%|queued:telegram::::8039067465
price_close:TEM:2026-04-23|medium|2026-04-23 20:02:47|TEM -7.31%|queued:telegram::::8039067465
```

- 这 3 条事件仍停留在当前 digest buffer 中，表明它们没有在事件生成后被立即发送：

```text
data/digest_buffer/telegram__direct__8039067465.jsonl
5: {"event":{"id":"price_close:AAOI:2026-04-23","kind":{"type":"price_alert","pct_change_bps":-782,"window":"close"},"severity":"medium","occurred_at":"2026-04-23T20:00:00Z","title":"AAOI -7.82%","source":"fmp.quote",...}}
7: {"event":{"id":"price_close:RKLB:2026-04-23","kind":{"type":"price_alert","pct_change_bps":-604,"window":"close"},"severity":"medium","occurred_at":"2026-04-23T20:00:00Z","title":"RKLB -6.04%","source":"fmp.quote",...}}
9: {"event":{"id":"price_close:TEM:2026-04-23","kind":{"type":"price_alert","pct_change_bps":-731,"window":"close"},"severity":"medium","occurred_at":"2026-04-23T20:00:00Z","title":"TEM -7.31%","source":"fmp.quote",...}}
```

- 同一时间窗里 `web.log` 没有这些事件的 `sink delivered`，只有对应的 digest 入队：

```text
data/runtime/logs/web.log.2026-04-23:4883:[2026-04-24 04:02:46.968] INFO  digest queued
data/runtime/logs/web.log.2026-04-23:4884:[2026-04-24 04:02:46.995] INFO  digest queued
data/runtime/logs/web.log.2026-04-23:4885:[2026-04-24 04:02:47.016] INFO  digest queued
```

上述 `04:02` 本地时间对应 `2026-04-23 20:02Z` 的美股常规收盘后处理；对该 actor 来说，下一个 digest 窗口是北京时间 `09:00`，意味着这些 >6% 的持仓波动至少延后约 5 小时才会被看到。

## Hypothesis / Suspected Code Path

`crates/hone-event-engine/src/pollers/price.rs:103-129` 对 `PriceWindow::Close` 单独走 `closing_move_severity`，即使绝对涨跌幅已经达到 `high_pct`，也只会被标成 `Severity::Medium`：

```rust
if let Some(pct) = pct {
    let abs = pct.abs();
    if abs >= low_pct {
        let severity = if window == PriceWindow::Close {
            closing_move_severity(abs, high_pct)
        } else if abs >= high_pct {
            Severity::High
        } else {
            Severity::Low
        };
        let bps = (pct * 100.0).round() as i64;
        let direction = if pct >= 0.0 { "+" } else { "" };
        out.push(MarketEvent {
            id: format!("{}:{symbol}:{date_key}", window.price_id_prefix()),
            kind: EventKind::PriceAlert {
                pct_change_bps: bps,
                window: window.as_str().into(),
            },
            severity,
            symbols: vec![symbol.clone()],
```

`crates/hone-event-engine/src/pollers/price.rs:218-223` 进一步把这个规则钉死成“收盘异动永远不会是 High”：

```rust
fn closing_move_severity(abs_pct: f64, high_pct: f64) -> Severity {
    if abs_pct >= high_pct {
        Severity::Medium
    } else {
        Severity::Low
    }
}
```

`crates/hone-event-engine/src/router.rs:521-530` 又把 per-actor 的 `price_high_pct_override` 限定为只对 `window != "close"` 生效，因此用户把阈值调到 `4.0` 也无法把收盘异动升为即时推送：

```rust
if let Some(threshold_pct) = price_override_threshold(event, prefs) {
    if matches!(
        event.kind,
        EventKind::PriceAlert { ref window, .. } if window != "close"
    ) {
        let pct = event
            .payload
            .get("changesPercentage")
            .and_then(|v| v.as_f64());
        if let Some(p) = pct {
```

`crates/hone-event-engine/src/router.rs:2350-2398` 现有测试也明确把这种行为当成预期，断言收盘异动“不应被个人 price override 直推”：

```rust
let ev = MarketEvent {
    id: "price_close:AMD:2026-04-22".into(),
    kind: EventKind::PriceAlert {
        pct_change_bps: 667,
        window: "close".into(),
    },
    severity: Severity::Medium,
    symbols: vec!["AMD".into()],
    occurred_at: Utc::now(),
    title: "AMD +6.67%".into(),
    summary: String::new(),
    url: None,
    source: "fmp.quote".into(),
    payload: serde_json::json!({"changesPercentage": 6.67}),
};
let (sent, pending) = router.dispatch(&ev).await.unwrap();
assert_eq!(sent, 0, "收盘异动不应被个人 price override 直推");
assert_eq!(pending, 1);
assert!(sink.calls.lock().unwrap().is_empty());
```

综合来看，这不是偶发日志噪声，而是当前规则刻意把“收盘后大幅波动”排除在 High 即时链路之外。

## Evidence Gap

- 产品策略按本次修复收敛为：普通 `price_close` 继续 digest 化，超过系统高阈值或满足 actor 价格 override 直推条件的 close 才即时推送。
- 本轮巡检没有调用真实外部 API，因此只能依据本地持仓、SQLite delivery log、digest buffer 和源码规则判断；缺少用户侧实际阅读反馈，无法证明这 5 小时延迟已经造成了错过操作窗口。

## Fix Notes

- `crates/hone-event-engine/src/pollers/price.rs`
  - `closing_move_severity(abs_pct, high_pct)` 达到 high 阈值时返回 `Severity::High`，否则仍返回 `Severity::Low`。
  - 新增/调整 close quote 单测，锁定“高阈值 close 即 High、低于高阈值 close 仍 Low”。
- `crates/hone-event-engine/src/router.rs`
  - per-actor price override 适用于所有 `PriceAlert`，不再排除 `window="close"`。
  - 原先断言 close 不应直推的测试改为断言超过直推地板的 close 应直推。

## Verification

- `rtk cargo test -p hone-event-engine close_quote --lib`
- `rtk cargo test -p hone-event-engine per_actor_price_threshold_can_promote_closing_move --lib`
- `rtk cargo test -p hone-event-engine --lib`
- `rtk cargo fmt --all -- --check`

## Severity

`sev2`。理由：这会让持仓命中的大幅收盘异动稳定延后到下一个 digest 窗口，直接影响“重要价格信号是否能及时触达用户”；但问题范围目前局限在 `window="close"` 的价格事件，其它 High 即时链路和 sink 发送本身仍然正常。

## Date Observed

`2026-04-23T22:28:37Z`
