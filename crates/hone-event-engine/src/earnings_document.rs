//! 财报新闻稿 / SEC exhibit 的共享身份边界。
//!
//! SEC poller、earnings surprise poller、router 和 digest buffer 都需要对
//! “这是不是同一份财报材料”做一致判断。这里只处理文档身份，
//! 不判断财报结论或投资影响。

use crate::event::{EventKind, MarketEvent};

pub(crate) const EARNINGS_DOCUMENT_FLAG: &str = "hone_earnings_release_document";
pub(crate) const EARNINGS_DOCUMENT_KEY: &str = "hone_earnings_release_document_key";
/// 同一季度的 release / transcript / 10-Q(或 10-K) 共用的研究对象键。
///
/// 第一份结构化财报卡以已核验的新闻稿文档键作为对象键；后续材料由 store
/// 按 ticker + 临近披露窗口挂到该对象。这样无需凭模型猜 fiscal quarter，且
/// 仍能在画像层把每份原始材料保留为独立 append-only 事件。
pub(crate) const EARNINGS_RESEARCH_OBJECT_KEY: &str = "hone_earnings_research_object_key";

pub(crate) fn earnings_research_material_kind(event: &MarketEvent) -> Option<&'static str> {
    match &event.kind {
        EventKind::EarningsReleased
            if event
                .payload
                .get("earnings_quality_review_applied")
                .and_then(|value| value.as_bool())
                == Some(true) =>
        {
            Some("earnings_release")
        }
        EventKind::EarningsCallTranscript => Some("earnings_call_transcript"),
        EventKind::SecFiling { form }
            if form.eq_ignore_ascii_case("10-Q") || form.eq_ignore_ascii_case("10-K") =>
        {
            Some("formal_filing")
        }
        _ => None,
    }
}

pub(crate) fn earnings_research_object_key_for_event(event: &MarketEvent) -> Option<String> {
    if let Some(key) = event
        .payload
        .get(EARNINGS_RESEARCH_OBJECT_KEY)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(key.to_string());
    }
    earnings_document_key_for_event(event)
}

pub(crate) fn is_earnings_release_document_url(url: &str) -> bool {
    let normalized = canonical_earnings_document_key(url)
        .unwrap_or_default()
        .replace(['_', '-'], "");
    [
        "pressrelease",
        "earnings",
        "financialresults",
        "quarterlyresults",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

pub(crate) fn canonical_earnings_document_key(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_fragment = trimmed.split('#').next().unwrap_or(trimmed);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment)
        .trim_end_matches('/');
    if without_query.is_empty() {
        return None;
    }
    Some(without_query.to_ascii_lowercase())
}

pub(crate) fn earnings_document_key_for_event(event: &MarketEvent) -> Option<String> {
    if let Some(key) = event
        .payload
        .get(EARNINGS_DOCUMENT_KEY)
        .and_then(|value| value.as_str())
        .and_then(canonical_earnings_document_key)
    {
        return Some(key);
    }
    let eligible = match event.kind {
        EventKind::EarningsReleased => {
            event
                .payload
                .get("earnings_quality_review_applied")
                .and_then(|value| value.as_bool())
                == Some(true)
        }
        EventKind::SecFiling { .. } => {
            event
                .payload
                .get(EARNINGS_DOCUMENT_FLAG)
                .and_then(|value| value.as_bool())
                == Some(true)
        }
        _ => false,
    };
    eligible
        .then(|| event.url.as_deref())
        .flatten()
        .and_then(canonical_earnings_document_key)
}

pub(crate) fn is_earnings_release_document_event(event: &MarketEvent) -> bool {
    matches!(event.kind, EventKind::SecFiling { .. })
        && earnings_document_key_for_event(event).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_key_ignores_tracking_query_fragment_and_case() {
        assert_eq!(
            canonical_earnings_document_key(
                "HTTPS://SEC.GOV/Archives/Q4-PressRelease.htm?source=fmp#top"
            )
            .as_deref(),
            Some("https://sec.gov/archives/q4-pressrelease.htm")
        );
    }

    #[test]
    fn classifier_covers_sndk_and_common_earnings_names() {
        for url in [
            "https://sec.gov/sndkq4-26ex991xpressrelease.htm",
            "https://sec.gov/q2-earnings.htm",
            "https://sec.gov/quarterly_results.htm",
            "https://sec.gov/financial-results.htm",
        ] {
            assert!(is_earnings_release_document_url(url), "{url}");
        }
        assert!(!is_earnings_release_document_url(
            "https://sec.gov/unrelated8k.htm"
        ));
    }

    #[test]
    fn research_material_classifier_keeps_release_transcript_and_formal_filing_together() {
        let base = MarketEvent {
            id: "sample".into(),
            kind: EventKind::EarningsCallTranscript,
            severity: crate::event::Severity::Low,
            symbols: vec!["SNDK".into()],
            occurred_at: chrono::Utc::now(),
            title: "SNDK earnings call transcript".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::json!({}),
        };
        assert_eq!(
            earnings_research_material_kind(&base),
            Some("earnings_call_transcript")
        );
        let mut filing = base.clone();
        filing.kind = EventKind::SecFiling {
            form: "10-Q".into(),
        };
        assert_eq!(
            earnings_research_material_kind(&filing),
            Some("formal_filing")
        );
        filing.kind = EventKind::SecFiling { form: "8-K".into() };
        assert_eq!(earnings_research_material_kind(&filing), None);
    }
}
