//! Floor 分类器 —— 把一个 `MarketEvent` 映射到 `Option<FloorTag>`,floor 条目
//! 在 unified scheduler 里**绕过 LLM** 直接 prepend 到 payload 顶部,且永远不会
//! 被 `max_items_per_batch` 挤掉(只占配额)。
//!
//! 优先级(同一事件命中多条规则时取第一条):
//! 1. `Severity::High` → `HighSeverity`
//! 2. event id 形如 `synth:earnings:*:countdown:*` → `Countdown`
//!    (复用 `pollers::earnings::synthesize_countdowns` 的 id 约定)
//! 3. `prefs.immediate_kinds` 命中本事件 `kind_tag` → `UserImmediate`
//!
//! `MacroFloor` 不在此处分类,由 curator Pass 2 personalize 输出
//! `PickCategory::MacroFloor` 时由 scheduler 直接打 floor 标签。

use crate::event::{MarketEvent, Severity, is_noop_analyst_grade};
use crate::prefs::{NotificationPrefs, kind_tag};
use crate::unified_digest::FloorTag;

pub fn classify_floor(event: &MarketEvent, prefs: &NotificationPrefs) -> Option<FloorTag> {
    if event.severity.rank() >= Severity::High.rank() {
        return Some(FloorTag::HighSeverity);
    }
    if event.id.starts_with("synth:earnings:") && event.id.contains(":countdown:") {
        return Some(FloorTag::Countdown);
    }
    if let Some(immediates) = &prefs.immediate_kinds {
        let tag = kind_tag(&event.kind);
        if is_noop_analyst_grade(event) {
            return None;
        }
        if immediates.iter().any(|t| t == tag) {
            return Some(FloorTag::UserImmediate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventKind;
    use chrono::Utc;

    fn ev(id: &str, kind: EventKind, sev: Severity) -> MarketEvent {
        MarketEvent {
            id: id.into(),
            kind,
            severity: sev,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "t".into(),
            summary: String::new(),
            url: None,
            source: "x".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn high_severity_wins() {
        let e = ev("e1", EventKind::NewsCritical, Severity::High);
        assert_eq!(
            classify_floor(&e, &NotificationPrefs::default()),
            Some(FloorTag::HighSeverity)
        );
    }

    #[test]
    fn synth_countdown_id_matches() {
        let e = ev(
            "synth:earnings:GOOGL:2026-04-29:countdown:2026-04-27",
            EventKind::EarningsUpcoming,
            Severity::Medium,
        );
        assert_eq!(
            classify_floor(&e, &NotificationPrefs::default()),
            Some(FloorTag::Countdown)
        );
    }

    #[test]
    fn immediate_kinds_match_uses_kind_tag() {
        let mut prefs = NotificationPrefs::default();
        prefs.immediate_kinds = Some(vec!["sec_filing".into()]);
        let e = ev(
            "id1",
            EventKind::SecFiling { form: "8-K".into() },
            Severity::Medium,
        );
        assert_eq!(classify_floor(&e, &prefs), Some(FloorTag::UserImmediate));
    }

    #[test]
    fn immediate_kinds_does_not_floor_noop_analyst_grade() {
        let mut prefs = NotificationPrefs::default();
        prefs.immediate_kinds = Some(vec!["analyst_grade".into()]);
        let mut e = ev("grade-1", EventKind::AnalystGrade, Severity::Low);
        e.source = "fmp.upgrades_downgrades".into();
        e.summary = "Overweight → Overweight".into();
        e.payload = serde_json::json!({
            "action": "hold",
            "previousGrade": "Overweight",
            "newGrade": "Overweight"
        });

        assert_eq!(classify_floor(&e, &prefs), None);
    }

    #[test]
    fn medium_without_match_returns_none() {
        let e = ev("id1", EventKind::NewsCritical, Severity::Medium);
        assert_eq!(classify_floor(&e, &NotificationPrefs::default()), None);
    }

    #[test]
    fn high_beats_synth_id() {
        let e = ev(
            "synth:earnings:X:2026-04-29:countdown:2026-04-27",
            EventKind::EarningsUpcoming,
            Severity::High,
        );
        assert_eq!(
            classify_floor(&e, &NotificationPrefs::default()),
            Some(FloorTag::HighSeverity)
        );
    }
}
