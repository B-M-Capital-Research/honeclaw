//! Point-in-time claims extracted from company earnings materials.
//!
//! LLM review output is not a verified fact.  This module accepts only a
//! narrow, structured claim shape and deterministically binds it to the
//! original event.  Downstream decision systems may display and review these
//! claims, but the policy status deliberately remains training-only.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::event::{EventKind, MarketEvent};

pub const CLAIM_POLICY_STATUS: &str = "training_only_pending_human_review";
pub const LEGACY_UNSPECIFIED_METRIC_BASIS: &str = "unspecified_legacy";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EarningsClaimKind {
    ReportedFact,
    ManagementGuidance,
    ManagementCommentary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EarningsClaimDisposition {
    #[default]
    Active,
    Corrected,
    Withdrawn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EarningsClaimInput {
    pub claim_kind: EarningsClaimKind,
    pub metric_id: String,
    /// GAAP / non-GAAP / company-defined operating definition.  This prevents
    /// two similarly named but economically different metrics being compared.
    #[serde(default = "default_metric_basis")]
    pub metric_basis: String,
    pub period: String,
    #[serde(default)]
    pub numeric_value: Option<f64>,
    #[serde(default)]
    pub unit: String,
    pub value_text: String,
    #[serde(default)]
    pub speaker: String,
    pub evidence_zh: String,
    pub source_locator: String,
    #[serde(default)]
    pub disposition: EarningsClaimDisposition,
}

fn default_metric_basis() -> String {
    LEGACY_UNSPECIFIED_METRIC_BASIS.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EarningsSourceClaim {
    pub claim_id: String,
    pub source_event_id: String,
    pub symbol: String,
    pub claim_kind: EarningsClaimKind,
    pub metric_id: String,
    pub metric_basis: String,
    pub period: String,
    pub numeric_value: Option<f64>,
    pub unit: String,
    pub value_text: String,
    pub speaker: Option<String>,
    pub quote_excerpt: String,
    pub source_locator: String,
    pub source_document: String,
    pub source_name: String,
    pub source_url: String,
    pub published_at: DateTime<Utc>,
    pub source_tier: String,
    pub policy_status: String,
    pub disposition: EarningsClaimDisposition,
}

/// Reads only explicit structured arrays.  It never regex-parses prose from a
/// summary, title or transcript review, because doing so would erase metric,
/// period and provenance boundaries.
pub fn source_claims_from_event(event: &MarketEvent) -> Vec<EarningsSourceClaim> {
    let Some(symbol) = event
        .symbols
        .first()
        .map(|value| value.trim().trim_start_matches('$').to_ascii_uppercase())
        .filter(|value| !value.is_empty())
    else {
        return Vec::new();
    };
    let Some(source_url) = event
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| url.starts_with("https://"))
    else {
        return Vec::new();
    };
    let (payload_pointer, source_document, source_tier) = match &event.kind {
        EventKind::EarningsCallTranscript => (
            "/earnings_transcript_review/claims",
            "earnings_call_transcript",
            "company_primary",
        ),
        EventKind::EarningsReleased => (
            "/earnings_quality_review/claims",
            "earnings_release",
            "company_primary",
        ),
        EventKind::SecFiling { form }
            if form.eq_ignore_ascii_case("10-Q")
                || form.eq_ignore_ascii_case("10-K")
                || form.eq_ignore_ascii_case("20-F") =>
        {
            (
                "/earnings_filing_claims",
                "sec_filing",
                "regulatory_primary",
            )
        }
        _ => return Vec::new(),
    };
    let Some(values) = event
        .payload
        .pointer(payload_pointer)
        .and_then(|value| value.as_array())
    else {
        return Vec::new();
    };

    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let input = serde_json::from_value::<EarningsClaimInput>(value.clone()).ok()?;
            valid_claim_input(&input).then(|| EarningsSourceClaim {
                claim_id: format!("source-claim:{}:{index}", event.id),
                source_event_id: event.id.clone(),
                symbol: symbol.clone(),
                claim_kind: input.claim_kind,
                metric_id: input.metric_id.trim().to_ascii_lowercase(),
                metric_basis: truncate(input.metric_basis.trim(), 80),
                period: truncate(input.period.trim(), 80),
                numeric_value: input.numeric_value,
                unit: input.unit.trim().to_string(),
                value_text: truncate(input.value_text.trim(), 180),
                speaker: (!input.speaker.trim().is_empty())
                    .then(|| truncate(input.speaker.trim(), 80)),
                quote_excerpt: truncate(input.evidence_zh.trim(), 240),
                source_locator: truncate(input.source_locator.trim(), 120),
                source_document: source_document.to_string(),
                source_name: truncate(event.source.trim(), 120),
                source_url: source_url.to_string(),
                published_at: event.occurred_at,
                source_tier: source_tier.to_string(),
                policy_status: CLAIM_POLICY_STATUS.to_string(),
                disposition: input.disposition,
            })
        })
        .take(24)
        .collect()
}

fn valid_claim_input(input: &EarningsClaimInput) -> bool {
    let metric = input.metric_id.trim().to_ascii_lowercase();
    let unit = input.unit.trim();
    let numeric_valid = input.numeric_value.is_none_or(f64::is_finite);
    let management_speaker_valid = matches!(input.claim_kind, EarningsClaimKind::ReportedFact)
        || !input.speaker.trim().is_empty();
    allowed_metric(&metric)
        && !input.metric_basis.trim().is_empty()
        && !input.period.trim().is_empty()
        && !input.value_text.trim().is_empty()
        && !input.evidence_zh.trim().is_empty()
        && !input.source_locator.trim().is_empty()
        && numeric_valid
        && (input.numeric_value.is_none() || allowed_unit(unit))
        && input.numeric_value.is_none_or(|value| {
            numeric_value_appears_in_text(value, &input.value_text)
                || numeric_value_appears_in_text(value, &input.evidence_zh)
        })
        && management_speaker_valid
}

fn numeric_value_appears_in_text(value: f64, text: &str) -> bool {
    let normalized = text.replace([',', '，', ' '], "");
    let exact = value.to_string();
    let rounded_integer = (value.fract().abs() < f64::EPSILON).then(|| format!("{value:.0}"));
    normalized.contains(&exact)
        || rounded_integer
            .as_deref()
            .is_some_and(|candidate| normalized.contains(candidate))
}

fn allowed_metric(metric: &str) -> bool {
    matches!(
        metric,
        "revenue"
            | "revenue_growth"
            | "gross_margin"
            | "gross_profit"
            | "operating_margin"
            | "operating_income"
            | "operating_cash_flow"
            | "free_cash_flow"
            | "capital_expenditure"
            | "cash_and_equivalents"
            | "long_term_debt"
            | "inventory"
            | "accounts_receivable"
            | "accounts_payable"
            | "backlog"
            | "rpo"
            | "arr"
            | "orders"
            | "shipments"
            | "capacity"
            | "utilization"
            | "asp"
            | "market_share"
            | "customer_qualification"
            | "customers"
            | "retention"
            | "usage"
            | "tokens"
            | "context_length"
            | "power_capacity"
            | "product_mix"
            | "unit_cost"
            | "delivery_lead_time"
    )
}

fn allowed_unit(unit: &str) -> bool {
    matches!(
        unit,
        "%" | "percentage_points"
            | "basis_points"
            | "USD"
            | "USD_millions"
            | "USD_billions"
            | "EUR_millions"
            | "TWD_millions"
            | "GBP_millions"
            | "units"
            | "customers"
            | "days"
            | "ratio"
            | "MW"
            | "GW"
            | "GB"
            | "TB"
            | "PB"
            | "EB"
            | "tokens"
    )
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut output = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        output.push('…');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Severity;

    fn event(payload: serde_json::Value) -> MarketEvent {
        MarketEvent {
            id: "sndk-call-2026q4".into(),
            kind: EventKind::EarningsCallTranscript,
            severity: Severity::Medium,
            symbols: vec!["SNDK".into()],
            occurred_at: DateTime::parse_from_rfc3339("2026-08-06T21:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            title: "call".into(),
            summary: "summary".into(),
            url: Some("https://investor.sandisk.com/call".into()),
            source: "Sandisk investor relations".into(),
            payload,
        }
    }

    #[test]
    fn extracts_structured_claim_with_event_bound_provenance() {
        let claims = source_claims_from_event(&event(serde_json::json!({
            "earnings_transcript_review": {"claims": [{
                "claim_kind": "management_guidance",
                "metric_id": "gross_margin",
                "metric_basis": "non-GAAP",
                "period": "FY2027 Q1",
                "numeric_value": 48.5,
                "unit": "%",
                "value_text": "毛利率指引中值48.5%",
                "speaker": "CFO",
                "evidence_zh": "预计下一季度毛利率约48%至49%",
                "source_locator": "prepared remarks / CFO"
            }]}
        })));
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].symbol, "SNDK");
        assert_eq!(claims[0].metric_id, "gross_margin");
        assert_eq!(claims[0].source_event_id, "sndk-call-2026q4");
        assert_eq!(claims[0].policy_status, CLAIM_POLICY_STATUS);
        assert_eq!(claims[0].source_document, "earnings_call_transcript");
    }

    #[test]
    fn rejects_prose_fallback_unknown_metric_unit_and_missing_speaker() {
        let claims = source_claims_from_event(&event(serde_json::json!({
            "earnings_transcript_review": {
                "prepared_findings": [{"finding_zh": "收入大幅增长"}],
                "claims": [
                    {"claim_kind":"reported_fact","metric_id":"magic","metric_basis":"GAAP","period":"Q4","numeric_value":1,"unit":"times","value_text":"x","speaker":"","evidence_zh":"x","source_locator":"p1"},
                    {"claim_kind":"management_guidance","metric_id":"revenue","metric_basis":"GAAP","period":"Q1","numeric_value":1,"unit":"USD_billions","value_text":"x","speaker":"","evidence_zh":"x","source_locator":"p1"}
                ]
            }
        })));
        assert!(claims.is_empty());
    }

    #[test]
    fn rejects_non_https_and_non_finite_values() {
        let mut candidate = event(serde_json::json!({
            "earnings_transcript_review": {"claims": [{
                "claim_kind":"reported_fact","metric_id":"revenue","period":"Q4",
                "metric_basis":"GAAP",
                "numeric_value": 1.0,"unit":"USD_billions","value_text":"$1B",
                "speaker":"","evidence_zh":"收入$1B","source_locator":"release table"
            }]}
        }));
        candidate.url = Some("http://example.com".into());
        assert!(source_claims_from_event(&candidate).is_empty());

        let input = EarningsClaimInput {
            claim_kind: EarningsClaimKind::ReportedFact,
            metric_id: "revenue".into(),
            metric_basis: "GAAP".into(),
            period: "Q4".into(),
            numeric_value: Some(f64::NAN),
            unit: "USD_billions".into(),
            value_text: "$1B".into(),
            speaker: String::new(),
            evidence_zh: "收入$1B".into(),
            source_locator: "table".into(),
            disposition: EarningsClaimDisposition::Active,
        };
        assert!(!valid_claim_input(&input));

        let mismatched = EarningsClaimInput {
            claim_kind: EarningsClaimKind::ReportedFact,
            metric_id: "revenue".into(),
            metric_basis: "GAAP".into(),
            period: "Q4".into(),
            numeric_value: Some(13_335.0),
            unit: "USD_millions".into(),
            value_text: "$1,335M".into(),
            speaker: String::new(),
            evidence_zh: "收入为$1,335M".into(),
            source_locator: "table".into(),
            disposition: EarningsClaimDisposition::Active,
        };
        assert!(!valid_claim_input(&mismatched));
    }

    #[test]
    fn legacy_claim_without_metric_basis_remains_traceable_but_marked_unspecified() {
        let claims = source_claims_from_event(&event(serde_json::json!({
            "earnings_transcript_review": {"claims": [{
                "claim_kind":"reported_fact","metric_id":"revenue","period":"Q4",
                "numeric_value": 1.0,"unit":"USD_billions","value_text":"收入 1.0B",
                "speaker":"","evidence_zh":"收入为 1.0B","source_locator":"release table"
            }]}
        })));
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].metric_basis, LEGACY_UNSPECIFIED_METRIC_BASIS);
    }
}
