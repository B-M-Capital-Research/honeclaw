//! Severity 策略层:在事件 *已经* 被 classifier 升级 / 已经匹配到具体 actor 之后,
//! 还要按以下三条规则做最后一轮 severity 调整。
//!
//! - **`apply_system_event_policy`**:全局规则。律所软文降 Low、Macro 高优只在临近
//!   窗口生效、远期 EarningsUpcoming 提前预览不算 High。
//! - **`apply_per_actor_severity_override`**:用户偏好。允许通过 `price_*_override`
//!   或 `immediate_kinds` 把 Medium 升 High 即时推。
//! - **`apply_quiet_mode`**:夜间静默。除几类「真硬信号」(财报 / SEC filing /
//!   盘中价格 band)外,把 High 降回 Medium 进 digest。
//!
//! 同时这里收拢了一组共用的事件查询纯函数(`is_intraday_price_band_alert` /
//! `event_category` / …)—— 它们既被本文件用,也被 `dispatch.rs` 频繁引用,
//! 集中放这里能让 dispatch 主流程的 import 简短。

use chrono::{DateTime, FixedOffset, NaiveTime, TimeZone, Utc};

use crate::event::{EventKind, MarketEvent, Severity, is_noop_analyst_grade};
use crate::prefs::{NotificationPrefs, kind_tag};

use super::config::NotificationRouter;

impl NotificationRouter {
    /// 系统级事件策略：在 per-actor prefs 之前先把明显不该即时推的事件降级。
    /// 这里不丢事件，只调整 severity，让它们进入 digest 或保持低优先级。
    pub(super) fn apply_system_event_policy(&self, event: &MarketEvent) -> MarketEvent {
        let mut routed = event.clone();
        if is_legal_ad_event(&routed) && routed.severity != Severity::Low {
            routed.severity = Severity::Low;
            tracing::info!(
                event_id = %routed.id,
                source = %routed.source,
                "legal-ad news demoted to Low"
            );
        }
        if matches!(routed.kind, EventKind::MacroEvent) && routed.severity == Severity::High {
            let now = chrono::Utc::now();
            let earliest = now - chrono::Duration::hours(self.macro_immediate_grace_hours);
            let latest = now + chrono::Duration::hours(self.macro_immediate_lookahead_hours);
            if routed.occurred_at < earliest || routed.occurred_at > latest {
                routed.severity = Severity::Medium;
                tracing::info!(
                    event_id = %routed.id,
                    source = %routed.source,
                    occurred_at = %routed.occurred_at,
                    lookahead_hours = self.macro_immediate_lookahead_hours,
                    grace_hours = self.macro_immediate_grace_hours,
                    "macro high demoted to digest outside due window"
                );
            }
        }
        if matches!(routed.kind, EventKind::EarningsUpcoming) {
            let days_until =
                (routed.occurred_at.date_naive() - chrono::Utc::now().date_naive()).num_days();
            if days_until > 7 && routed.severity.rank() > Severity::Low.rank() {
                routed.severity = Severity::Low;
                tracing::info!(
                    event_id = %routed.id,
                    source = %routed.source,
                    days_until,
                    "far earnings preview demoted to Low"
                );
            }
        }
        routed
    }

    /// 按用户 prefs 重写 severity:价格阈值 / immediate_kinds。价格覆盖保留用户
    /// 敏感度，但默认不能低于系统最小直推阈值；大仓位标的可用用户阈值。
    pub(super) fn apply_per_actor_severity_override(
        &self,
        event: &MarketEvent,
        sev: Severity,
        prefs: &NotificationPrefs,
    ) -> Severity {
        if matches!(sev, Severity::High) {
            return sev;
        }
        if let Some(threshold_pct) = price_override_threshold(event, prefs) {
            if matches!(event.kind, EventKind::PriceAlert { .. }) {
                let pct = event
                    .payload
                    .get("changesPercentage")
                    .and_then(|v| v.as_f64());
                if let Some(p) = pct {
                    let min_direct = self.price_min_direct_pct.max(0.0);
                    let large_weight_threshold = prefs
                        .large_position_weight_pct
                        .unwrap_or(self.large_position_weight_pct);
                    let is_large_position = event_position_weight_pct(event)
                        .map(|w| w >= large_weight_threshold)
                        .unwrap_or(false);
                    let required = if is_large_position {
                        threshold_pct
                    } else {
                        threshold_pct.max(min_direct)
                    };
                    if p.abs() >= required {
                        return Severity::High;
                    }
                }
            }
        }
        if let Some(kinds) = prefs.immediate_kinds.as_deref() {
            let tag = kind_tag(&event.kind);
            if kinds.iter().any(|k| k == tag) {
                if matches!(
                    event.kind,
                    EventKind::NewsCritical | EventKind::PressRelease
                ) && matches!(sev, Severity::Low)
                {
                    tracing::info!(
                        event_id = %event.id,
                        kind = %tag,
                        source = %event.source,
                        "immediate_kinds override skipped for Low-signal news"
                    );
                    return sev;
                }
                if is_noop_analyst_grade(event) {
                    tracing::info!(
                        event_id = %event.id,
                        kind = %tag,
                        source = %event.source,
                        "immediate_kinds override skipped for no-op analyst grade"
                    );
                    return sev;
                }
                return Severity::High;
            }
        }
        sev
    }

    pub(super) fn apply_quiet_mode(
        &self,
        event: &MarketEvent,
        sev: Severity,
        prefs: &NotificationPrefs,
    ) -> Severity {
        if !prefs.quiet_mode || sev != Severity::High {
            return sev;
        }
        if quiet_mode_allows_immediate(event) {
            return sev;
        }
        tracing::info!(
            event_id = %event.id,
            kind = %kind_tag(&event.kind),
            source = %event.source,
            "quiet mode demoted High to digest"
        );
        Severity::Medium
    }
}

fn price_override_threshold(event: &MarketEvent, prefs: &NotificationPrefs) -> Option<f64> {
    let pct = event
        .payload
        .get("changesPercentage")
        .and_then(|v| v.as_f64());
    match pct {
        Some(p) if p >= 0.0 => prefs
            .price_high_pct_up_override
            .or(prefs.price_high_pct_override),
        Some(_) => prefs
            .price_high_pct_down_override
            .or(prefs.price_high_pct_override),
        None => prefs.price_high_pct_override,
    }
}

fn event_position_weight_pct(event: &MarketEvent) -> Option<f64> {
    let raw = event
        .payload
        .get("portfolio_weight_pct")
        .or_else(|| event.payload.get("portfolio_weight"))
        .and_then(|v| v.as_f64())?;
    Some(if raw <= 1.0 { raw * 100.0 } else { raw })
}

pub(super) fn is_price_close_alert(event: &MarketEvent) -> bool {
    matches!(&event.kind, EventKind::PriceAlert { window, .. } if window == "close")
}

pub(super) fn is_intraday_price_band_alert(event: &MarketEvent) -> bool {
    matches!(&event.kind, EventKind::PriceAlert { window, .. } if window != "close")
        && event.id.starts_with("price_band:")
}

pub(super) fn price_alert_symbol_direction(event: &MarketEvent) -> Option<(&str, &str)> {
    if !is_intraday_price_band_alert(event) {
        return None;
    }
    let symbol = event.symbols.first()?.as_str();
    let direction = event
        .payload
        .get("hone_price_direction")
        .and_then(|v| v.as_str())
        .or_else(|| {
            event
                .payload
                .get("changesPercentage")
                .and_then(|v| v.as_f64())
                .map(|pct| if pct >= 0.0 { "up" } else { "down" })
        })?;
    matches!(direction, "up" | "down").then_some((symbol, direction))
}

fn quiet_mode_allows_immediate(event: &MarketEvent) -> bool {
    match &event.kind {
        EventKind::EarningsReleased
        | EventKind::EarningsCallTranscript
        | EventKind::SecFiling { .. } => true,
        EventKind::PriceAlert { window, .. } if window != "close" => true,
        _ => false,
    }
}

fn is_legal_ad_event(event: &MarketEvent) -> bool {
    event
        .payload
        .get("legal_ad_template")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
        || {
            let text = format!("{} {}", event.title, event.summary).to_ascii_lowercase();
            [
                "shareholder alert",
                "investor alert",
                "class action lawsuit",
                "securities fraud class action",
                "law offices",
                "law firm",
            ]
            .iter()
            .any(|pat| text.contains(pat))
        }
}

pub(super) fn event_category(event: &MarketEvent) -> &'static str {
    match event.kind {
        EventKind::PriceAlert { .. }
        | EventKind::Weekly52High
        | EventKind::Weekly52Low
        | EventKind::VolumeSpike => "price",
        EventKind::NewsCritical | EventKind::PressRelease | EventKind::SocialPost => "news",
        EventKind::SecFiling { .. } => "filing",
        EventKind::EarningsUpcoming
        | EventKind::EarningsReleased
        | EventKind::EarningsCallTranscript => "earnings",
        EventKind::MacroEvent => "macro",
        EventKind::Dividend | EventKind::Split | EventKind::Buyback => "corp_action",
        EventKind::AnalystGrade => "analyst",
        EventKind::PortfolioPreMarket | EventKind::PortfolioPostMarket => "portfolio",
    }
}

/// 按给定 tz 偏移求本地当日 00:00 对应的 UTC 时刻。用作
/// `count_high_sent_since` 的 cutoff,保证跨时区一致。
pub(super) fn local_day_start(now: DateTime<Utc>, offset_hours: i32) -> DateTime<Utc> {
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let midnight = local
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    offset
        .from_local_datetime(&midnight)
        .single()
        .map(|l| l.with_timezone(&Utc))
        .unwrap_or(now)
}
