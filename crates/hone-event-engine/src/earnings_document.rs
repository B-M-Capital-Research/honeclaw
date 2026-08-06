//! 财报新闻稿 / SEC exhibit 的共享身份边界。
//!
//! SEC poller、earnings surprise poller、router 和 digest buffer 都需要对
//! “这是不是同一份财报材料”做一致判断。这里只处理文档身份，
//! 不判断财报结论或投资影响。

use crate::event::{EventKind, MarketEvent};

pub(crate) const EARNINGS_DOCUMENT_FLAG: &str = "hone_earnings_release_document";
pub(crate) const EARNINGS_DOCUMENT_KEY: &str = "hone_earnings_release_document_key";

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
}
