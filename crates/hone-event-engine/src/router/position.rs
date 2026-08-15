//! 仓位感知事件注入(2026-08,推送体检 item 2)。
//!
//! 推送一直只说「SNDK -8%」,从不说「你的 13 股今日 -$430,距成本 -6%」。
//! 本模块在 dispatch 的 per-actor 阶段把 actor 持仓上下文写进事件 payload
//! 的 actor 级克隆(原始事件与 store 不动),renderer 据此追加持仓行;
//! 同时注入 `portfolio_weight_pct`,让 `large_position_weight_pct` 大仓位
//! 阈值机制第一次真正拿到数据。
//!
//! 期权持仓与 tracking_only 自选不注入(registry 折算时已跳过)。

use serde_json::Value;

use crate::event::MarketEvent;
use crate::subscription::PositionSnapshot;

/// payload 注入字段名(renderer / 测试共用)。
pub(crate) const POSITION_SHARES_KEY: &str = "hone_position_shares";
pub(crate) const POSITION_AVG_COST_KEY: &str = "hone_position_avg_cost";
pub(crate) const POSITION_MARKET_VALUE_KEY: &str = "hone_position_market_value";
pub(crate) const POSITION_DAY_PNL_KEY: &str = "hone_position_day_pnl_usd";
pub(crate) const POSITION_COST_DISTANCE_KEY: &str = "hone_position_cost_distance_pct";
pub(crate) const PORTFOLIO_WEIGHT_KEY: &str = "portfolio_weight_pct";

/// 生成带仓位上下文的 actor 级事件克隆。事件 payload 非对象或无可注入内容时
/// 返回 None(调用方沿用原事件)。
pub(crate) fn position_annotated_event(
    event: &MarketEvent,
    position: &PositionSnapshot,
) -> Option<MarketEvent> {
    let mut annotated = event.clone();
    let obj = annotated.payload.as_object_mut()?;

    obj.insert(POSITION_SHARES_KEY.into(), json_number(position.shares)?);
    obj.insert(
        POSITION_AVG_COST_KEY.into(),
        json_number(position.avg_cost)?,
    );
    if let Some(weight) = position.weight_pct.filter(|w| w.is_finite() && *w > 0.0) {
        // 不覆盖上游已有的权重字段(未来若 poller 侧先写,以先写为准)。
        if !obj.contains_key(PORTFOLIO_WEIGHT_KEY) && !obj.contains_key("portfolio_weight") {
            obj.insert(PORTFOLIO_WEIGHT_KEY.into(), json_number(weight)?);
        }
    }

    if let Some(price) = event_price(event).filter(|p| *p > 0.0) {
        if let Some(value) = json_number(position.shares * price) {
            obj.insert(POSITION_MARKET_VALUE_KEY.into(), value);
        }
        if position.avg_cost > 0.0
            && let Some(distance) =
                json_number((price - position.avg_cost) / position.avg_cost * 100.0)
        {
            obj.insert(POSITION_COST_DISTANCE_KEY.into(), distance);
        }
        // 今日盈亏 = shares × (price − prev_close),prev_close 由 pct 反推:
        // prev = price / (1 + r) → pnl = shares × price × r / (1 + r)。
        if let Some(pct) = event_day_pct(event).filter(|r| r.is_finite() && *r > -100.0) {
            let ratio = pct / 100.0;
            if let Some(pnl) = json_number(position.shares * price * ratio / (1.0 + ratio)) {
                obj.insert(POSITION_DAY_PNL_KEY.into(), pnl);
            }
        }
    }

    Some(annotated)
}

/// 事件里的现价:价格类事件写 `hone_price`,extended/quote 原始字段是 `price`。
fn event_price(event: &MarketEvent) -> Option<f64> {
    event
        .payload
        .get("hone_price")
        .or_else(|| event.payload.get("price"))
        .and_then(Value::as_f64)
}

/// 事件里的当日涨跌 %:价格类事件写 `hone_price_pct`,quote/extended 是
/// `changesPercentage`(extended 的符号语义是净方向,幅度是振幅 —— 用它算
/// 美元影响会高估,因此 extended 事件只注入市值/距成本,不注入 day_pnl)。
fn event_day_pct(event: &MarketEvent) -> Option<f64> {
    if event.source == "fmp.extended_hours" {
        return None;
    }
    event
        .payload
        .get("hone_price_pct")
        .or_else(|| event.payload.get("changesPercentage"))
        .and_then(Value::as_f64)
}

fn json_number(n: f64) -> Option<Value> {
    serde_json::Number::from_f64(n).map(Value::Number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventKind, Severity};
    use chrono::Utc;

    fn price_event(payload: serde_json::Value) -> MarketEvent {
        MarketEvent {
            id: "price_band:SNDK:2026-08-14:down:800".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: -800,
                window: "day".into(),
            },
            severity: Severity::High,
            symbols: vec!["SNDK".into()],
            occurred_at: Utc::now(),
            title: "SNDK 跨过 -8% 档".into(),
            summary: "当前 380.00，日跌 -8.00%".into(),
            url: None,
            source: "fmp.quote".into(),
            payload,
        }
    }

    /// B1:真实持仓数值(SNDK 13 股 @ 404.19,现价 380,-8%)。
    #[test]
    fn injects_dollar_impact_and_cost_distance_for_real_holding() {
        let event = price_event(serde_json::json!({
            "hone_price": 380.0,
            "hone_price_pct": -8.0,
        }));
        let position = PositionSnapshot {
            shares: 13.0,
            avg_cost: 404.1907692307692,
            weight_pct: Some(11.4),
        };
        let annotated = position_annotated_event(&event, &position).unwrap();
        let payload = annotated.payload.as_object().unwrap();
        assert_eq!(payload[POSITION_SHARES_KEY].as_f64(), Some(13.0));
        let market_value = payload[POSITION_MARKET_VALUE_KEY].as_f64().unwrap();
        assert!((market_value - 4940.0).abs() < 0.01);
        // pnl = 13 × 380 × (−0.08/0.92) ≈ −429.57
        let pnl = payload[POSITION_DAY_PNL_KEY].as_f64().unwrap();
        assert!((pnl - (-429.565)).abs() < 0.01, "pnl={pnl}");
        // 距成本 = (380 − 404.19)/404.19 ≈ −5.985%
        let distance = payload[POSITION_COST_DISTANCE_KEY].as_f64().unwrap();
        assert!((distance - (-5.985)).abs() < 0.01, "distance={distance}");
        assert_eq!(payload[PORTFOLIO_WEIGHT_KEY].as_f64(), Some(11.4));
        // 原事件不被污染
        assert!(event.payload.get(POSITION_SHARES_KEY).is_none());
    }

    /// 非价格事件(无现价)仍注入 shares/成本/权重,跳过美元字段。
    #[test]
    fn non_price_event_gets_static_position_fields_only() {
        let mut event = price_event(serde_json::json!({"newGrade": "Sell"}));
        event.kind = EventKind::AnalystGrade;
        let position = PositionSnapshot {
            shares: 19.0,
            avg_cost: 432.55,
            weight_pct: Some(24.0),
        };
        let annotated = position_annotated_event(&event, &position).unwrap();
        let payload = annotated.payload.as_object().unwrap();
        assert_eq!(payload[POSITION_SHARES_KEY].as_f64(), Some(19.0));
        assert_eq!(payload[PORTFOLIO_WEIGHT_KEY].as_f64(), Some(24.0));
        assert!(payload.get(POSITION_DAY_PNL_KEY).is_none());
        assert!(payload.get(POSITION_COST_DISTANCE_KEY).is_none());
    }

    /// extended_hours 的 changesPercentage 是有符号振幅,不能当日涨跌算钱。
    #[test]
    fn extended_hours_amp_is_not_used_for_day_pnl() {
        let mut event = price_event(serde_json::json!({
            "price": 380.0,
            "changesPercentage": -9.0,
            "amp_pct": 9.0,
        }));
        event.source = "fmp.extended_hours".into();
        let position = PositionSnapshot {
            shares: 13.0,
            avg_cost: 404.19,
            weight_pct: None,
        };
        let annotated = position_annotated_event(&event, &position).unwrap();
        let payload = annotated.payload.as_object().unwrap();
        assert!(payload.get(POSITION_DAY_PNL_KEY).is_none());
        assert!(payload.get(POSITION_MARKET_VALUE_KEY).is_some());
    }

    /// 已有上游权重字段时不覆盖。
    #[test]
    fn existing_portfolio_weight_is_not_overwritten() {
        let event = price_event(serde_json::json!({
            "hone_price": 380.0,
            "portfolio_weight_pct": 33.0,
        }));
        let position = PositionSnapshot {
            shares: 13.0,
            avg_cost: 404.19,
            weight_pct: Some(11.4),
        };
        let annotated = position_annotated_event(&event, &position).unwrap();
        assert_eq!(
            annotated.payload["portfolio_weight_pct"].as_f64(),
            Some(33.0)
        );
    }
}
