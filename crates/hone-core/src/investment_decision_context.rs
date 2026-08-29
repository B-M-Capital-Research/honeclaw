//! Shared, read-only projection of HONE's point-in-time investment decision.
//!
//! The Web API owns the full decision state and its validation.  Channel
//! runners must not deserialize that large internal object or reconstruct an
//! action from ratings, model memory, or historical research cards.  Instead,
//! the Web API writes this bounded sidecar after the full snapshot passes its
//! point-in-time validator, and every chat surface can load the same contract.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub const INVESTMENT_DECISION_CHAT_CONTEXT_SCHEMA_VERSION: &str =
    "hone-investment-decision-chat-context-v1";
pub const INVESTMENT_DECISION_SOURCE_SCHEMA_VERSION: &str = "hone-investment-decision-v1";
pub const INVESTMENT_DECISION_CHAT_CONTEXT_FRESH_HOURS: i64 = 36;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvestmentDecisionValuationContext {
    pub as_of: String,
    pub currency: String,
    pub bear_case: f64,
    pub base_case: f64,
    pub bull_case: f64,
    pub current_price: f64,
    pub expected_upside_percent: f64,
    pub method_count: usize,
    pub confidence: String,
    pub current_position: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvestmentDecisionChatContext {
    pub schema_version: String,
    pub source_schema_version: String,
    pub revision_id: String,
    pub symbol: String,
    pub company_name: String,
    pub theme: String,
    pub decision_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
    pub source_rating_methodology: String,
    pub source_rating_score: f64,
    pub source_rating_light: String,
    pub data_status: String,
    pub zone: String,
    pub action: String,
    pub confidence: String,
    pub rationale: Vec<String>,
    pub business_model: String,
    pub value_chain: String,
    pub moat_hypothesis: String,
    pub thesis: String,
    pub first_principles_model_id: Option<String>,
    pub first_principles_version: Option<String>,
    pub first_principles_status: Option<String>,
    pub financial_policy_version: String,
    pub financial_status: String,
    pub financial_as_of: Option<String>,
    pub financial_highlights: Vec<String>,
    pub financial_quality_warnings: Vec<String>,
    pub financial_missing_checks: Vec<String>,
    pub valuation: Option<InvestmentDecisionValuationContext>,
    pub valuation_missing_reason: String,
    pub crowding_policy_version: String,
    pub crowding_status: String,
    pub crowding_score: Option<f64>,
    pub crowding_label: String,
    pub crowding_observations: Vec<String>,
    pub crowding_missing_checks: Vec<String>,
    pub market_regime_status: String,
    pub market_regime_label: String,
    pub macro_score: Option<f64>,
    pub market_data_cutoff: Option<String>,
    pub completeness_policy_version: String,
    pub completeness_status: String,
    pub completeness_passed_checks: usize,
    pub completeness_total_checks: usize,
    pub directional_research_ready: bool,
    pub portfolio_decision_ready: bool,
    pub completeness_gaps: Vec<String>,
    pub hari_policy_version: String,
    pub confirmed_logic_ids: Vec<String>,
    pub hari_rule_statuses: Vec<String>,
    pub candidate_logic_used: bool,
    pub increase_candidate_authorized: bool,
    pub portfolio_action_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub trading_authorized: bool,
    pub falsifiers: Vec<String>,
    pub next_checks: Vec<String>,
    pub scope: String,
}

impl InvestmentDecisionChatContext {
    pub fn validate_at(&self, now: DateTime<Utc>) -> Result<(), String> {
        if self.schema_version != INVESTMENT_DECISION_CHAT_CONTEXT_SCHEMA_VERSION {
            return Err("unsupported investment decision chat-context schema".to_string());
        }
        if self.source_schema_version != INVESTMENT_DECISION_SOURCE_SCHEMA_VERSION {
            return Err("unsupported source investment decision schema".to_string());
        }
        if !valid_symbol(&self.symbol) {
            return Err("invalid investment decision context symbol".to_string());
        }
        for (label, value, maximum) in [
            ("revision_id", self.revision_id.as_str(), 240usize),
            ("company_name", self.company_name.as_str(), 160usize),
            ("theme", self.theme.as_str(), 160usize),
            (
                "source_rating_methodology",
                self.source_rating_methodology.as_str(),
                160usize,
            ),
            (
                "source_rating_light",
                self.source_rating_light.as_str(),
                32usize,
            ),
            ("data_status", self.data_status.as_str(), 48usize),
            ("zone", self.zone.as_str(), 48usize),
            ("action", self.action.as_str(), 48usize),
            ("confidence", self.confidence.as_str(), 32usize),
            ("business_model", self.business_model.as_str(), 1_200usize),
            ("value_chain", self.value_chain.as_str(), 800usize),
            ("moat_hypothesis", self.moat_hypothesis.as_str(), 1_200usize),
            ("thesis", self.thesis.as_str(), 1_200usize),
            (
                "financial_policy_version",
                self.financial_policy_version.as_str(),
                160usize,
            ),
            (
                "crowding_policy_version",
                self.crowding_policy_version.as_str(),
                160usize,
            ),
            (
                "completeness_policy_version",
                self.completeness_policy_version.as_str(),
                160usize,
            ),
            (
                "hari_policy_version",
                self.hari_policy_version.as_str(),
                160usize,
            ),
            ("scope", self.scope.as_str(), 1_600usize),
        ] {
            if value.trim().is_empty() || value.chars().count() > maximum {
                return Err(format!("invalid investment decision context {label}"));
            }
        }
        if !self.source_rating_score.is_finite()
            || !(0.0..=100.0).contains(&self.source_rating_score)
        {
            return Err("invalid source rating score".to_string());
        }
        if !matches!(
            self.zone.as_str(),
            "opportunity" | "hold" | "risk" | "insufficient_data"
        ) || !matches!(
            self.action.as_str(),
            "increase_candidate" | "maintain" | "reduce_candidate" | "research_only"
        ) || !matches!(self.confidence.as_str(), "low" | "medium" | "high")
        {
            return Err("invalid decision zone, action, or confidence".to_string());
        }
        if self.decision_at > now + Duration::minutes(5)
            || self.fresh_until
                != self.decision_at + Duration::hours(INVESTMENT_DECISION_CHAT_CONTEXT_FRESH_HOURS)
        {
            return Err("invalid investment decision context time window".to_string());
        }
        if self.completeness_total_checks == 0
            || self.completeness_passed_checks > self.completeness_total_checks
        {
            return Err("invalid decision completeness counts".to_string());
        }
        if self.candidate_logic_used
            || self.portfolio_action_authorized
            || self.shadow_portfolio_authorized
            || self.trading_authorized
        {
            return Err(
                "investment decision context exceeds company research authority".to_string(),
            );
        }
        if self.increase_candidate_authorized
            != (self.action == "increase_candidate" && self.zone == "opportunity")
        {
            return Err("increase-candidate authorization is inconsistent".to_string());
        }
        for value in [self.crowding_score, self.macro_score]
            .into_iter()
            .flatten()
        {
            if !value.is_finite() || !(0.0..=100.0).contains(&value) {
                return Err("invalid bounded decision context score".to_string());
            }
        }
        if let Some(valuation) = &self.valuation {
            if valuation.as_of.trim().is_empty()
                || valuation.currency.trim().is_empty()
                || valuation.confidence.trim().is_empty()
                || valuation.current_position.trim().is_empty()
                || valuation.method_count < 2
                || [
                    valuation.bear_case,
                    valuation.base_case,
                    valuation.bull_case,
                    valuation.current_price,
                ]
                .into_iter()
                .any(|value| !value.is_finite() || value <= 0.0)
                || !valuation.expected_upside_percent.is_finite()
                || !(valuation.bear_case < valuation.base_case
                    && valuation.base_case < valuation.bull_case)
            {
                return Err("invalid investment decision valuation context".to_string());
            }
        } else if self.valuation_missing_reason.trim().is_empty()
            || self.valuation_missing_reason.chars().count() > 1_200
        {
            return Err("missing investment decision valuation reason".to_string());
        }
        for (label, values, maximum_items, maximum_chars) in [
            ("rationale", &self.rationale, 8usize, 1_200usize),
            (
                "financial_highlights",
                &self.financial_highlights,
                16usize,
                500usize,
            ),
            (
                "financial_quality_warnings",
                &self.financial_quality_warnings,
                12usize,
                800usize,
            ),
            (
                "financial_missing_checks",
                &self.financial_missing_checks,
                16usize,
                800usize,
            ),
            (
                "crowding_observations",
                &self.crowding_observations,
                16usize,
                800usize,
            ),
            (
                "crowding_missing_checks",
                &self.crowding_missing_checks,
                16usize,
                800usize,
            ),
            (
                "completeness_gaps",
                &self.completeness_gaps,
                16usize,
                800usize,
            ),
            (
                "confirmed_logic_ids",
                &self.confirmed_logic_ids,
                12usize,
                80usize,
            ),
            (
                "hari_rule_statuses",
                &self.hari_rule_statuses,
                12usize,
                160usize,
            ),
            ("falsifiers", &self.falsifiers, 12usize, 800usize),
            ("next_checks", &self.next_checks, 16usize, 800usize),
        ] {
            validate_bounded_strings(label, values, maximum_items, maximum_chars)?;
        }
        Ok(())
    }

    pub fn is_fresh_at(&self, now: DateTime<Utc>) -> bool {
        self.validate_at(now).is_ok() && now <= self.fresh_until
    }
}

fn valid_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol.len() <= 16
        && symbol == symbol.to_ascii_uppercase()
        && symbol
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || b".-".contains(&byte))
}

fn validate_bounded_strings(
    label: &str,
    values: &[String],
    maximum_items: usize,
    maximum_chars: usize,
) -> Result<(), String> {
    if values.len() > maximum_items
        || values
            .iter()
            .any(|value| value.trim().is_empty() || value.chars().count() > maximum_chars)
    {
        return Err(format!("invalid investment decision context {label}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_context() -> InvestmentDecisionChatContext {
        let decision_at = DateTime::parse_from_rfc3339("2026-08-14T01:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        InvestmentDecisionChatContext {
            schema_version: INVESTMENT_DECISION_CHAT_CONTEXT_SCHEMA_VERSION.to_string(),
            source_schema_version: INVESTMENT_DECISION_SOURCE_SCHEMA_VERSION.to_string(),
            revision_id: "SNDK-revision".to_string(),
            symbol: "SNDK".to_string(),
            company_name: "Sandisk".to_string(),
            theme: "存储".to_string(),
            decision_at,
            fresh_until: decision_at
                + Duration::hours(INVESTMENT_DECISION_CHAT_CONTEXT_FRESH_HOURS),
            source_rating_methodology: "hone-company-rating-v8".to_string(),
            source_rating_score: 74.4,
            source_rating_light: "yellow".to_string(),
            data_status: "partial".to_string(),
            zone: "insufficient_data".to_string(),
            action: "research_only".to_string(),
            confidence: "medium".to_string(),
            rationale: vec!["估值仍缺失".to_string()],
            business_model: "NAND 与 SSD".to_string(),
            value_chain: "NAND/控制器/固件".to_string(),
            moat_hypothesis: "认证与固件".to_string(),
            thesis: "AI 存储需求增长".to_string(),
            first_principles_model_id: Some("ai-storage-demand-supply".to_string()),
            first_principles_version: Some("v1".to_string()),
            first_principles_status: Some("measurement_incomplete".to_string()),
            financial_policy_version: "financial-v1".to_string(),
            financial_status: "partially_measured".to_string(),
            financial_as_of: Some("2026-05-02".to_string()),
            financial_highlights: vec!["收入同比 +20.0%".to_string()],
            financial_quality_warnings: vec!["需核对口径".to_string()],
            financial_missing_checks: vec!["自由现金流".to_string()],
            valuation: None,
            valuation_missing_reason: "缺少两种可复算方法".to_string(),
            crowding_policy_version: "crowding-v1".to_string(),
            crowding_status: "partially_measured".to_string(),
            crowding_score: Some(41.2),
            crowding_label: "中性".to_string(),
            crowding_observations: vec!["价格路径已测量".to_string()],
            crowding_missing_checks: vec!["社交情绪".to_string()],
            market_regime_status: "observed".to_string(),
            market_regime_label: "balanced".to_string(),
            macro_score: Some(61.0),
            market_data_cutoff: Some("2026-08-13".to_string()),
            completeness_policy_version: "completeness-v1".to_string(),
            completeness_status: "research_incomplete".to_string(),
            completeness_passed_checks: 3,
            completeness_total_checks: 8,
            directional_research_ready: false,
            portfolio_decision_ready: false,
            completeness_gaps: vec!["估值缺失".to_string()],
            hari_policy_version: "hari-gate-v1".to_string(),
            confirmed_logic_ids: vec!["LOG-V0001".to_string()],
            hari_rule_statuses: vec!["LOG-V0001:passed".to_string()],
            candidate_logic_used: false,
            increase_candidate_authorized: false,
            portfolio_action_authorized: false,
            shadow_portfolio_authorized: false,
            trading_authorized: false,
            falsifiers: vec!["需求转弱".to_string()],
            next_checks: vec!["复核订单".to_string()],
            scope: "只读研究状态，不授权交易。".to_string(),
        }
    }

    #[test]
    fn current_context_validates_but_stale_context_is_not_fresh() {
        let context = fixture_context();
        let current = context.decision_at + Duration::hours(12);
        assert!(context.validate_at(current).is_ok());
        assert!(context.is_fresh_at(current));
        assert!(!context.is_fresh_at(context.fresh_until + Duration::seconds(1)));
    }

    #[test]
    fn context_rejects_hidden_authority_and_inconsistent_increase() {
        let mut context = fixture_context();
        context.trading_authorized = true;
        assert!(context.validate_at(context.decision_at).is_err());

        let mut context = fixture_context();
        context.increase_candidate_authorized = true;
        assert!(context.validate_at(context.decision_at).is_err());
    }
}
