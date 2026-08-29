use hone_llm::CreatedLlmProvider;
use serde::{Deserialize, Serialize};

pub(crate) const POLICY_VERSION: &str = "hone-model-analysis-health-v1-fail-closed";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ModelAnalysisHealth {
    pub policy_version: String,
    pub status: String,
    pub provider_name: Option<String>,
    pub profile_name: Option<String>,
    pub model: Option<String>,
    pub requested_items: usize,
    pub analyzed_items: usize,
    pub failed_items: usize,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    pub decision_use_allowed: bool,
}

impl Default for ModelAnalysisHealth {
    fn default() -> Self {
        Self {
            policy_version: POLICY_VERSION.to_string(),
            status: "unknown_legacy".to_string(),
            provider_name: None,
            profile_name: None,
            model: None,
            requested_items: 0,
            analyzed_items: 0,
            failed_items: 0,
            failure_reasons: vec!["legacy_snapshot_without_analysis_health".to_string()],
            decision_use_allowed: false,
        }
    }
}

pub(crate) fn build<'a>(
    analyzer: Option<&CreatedLlmProvider>,
    requested_items: usize,
    analyzed_items: usize,
    failure_reasons: impl IntoIterator<Item = &'a str>,
    status: &str,
) -> ModelAnalysisHealth {
    let mut failure_reasons = failure_reasons
        .into_iter()
        .filter(|reason| !reason.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    failure_reasons.sort();
    failure_reasons.dedup();
    let failed_items = requested_items.saturating_sub(analyzed_items);
    let decision_use_allowed = matches!(status, "healthy" | "not_required")
        && failed_items == 0
        && failure_reasons.is_empty();
    ModelAnalysisHealth {
        policy_version: POLICY_VERSION.to_string(),
        status: status.to_string(),
        provider_name: analyzer.map(|value| value.provider_name.clone()),
        profile_name: analyzer.and_then(|value| value.profile_name.clone()),
        model: analyzer.map(|value| value.model.clone()),
        requested_items,
        analyzed_items,
        failed_items,
        failure_reasons,
        decision_use_allowed,
    }
}

pub(crate) fn scope(
    parent: &ModelAnalysisHealth,
    requested_items: usize,
    analyzed_items: usize,
) -> ModelAnalysisHealth {
    if requested_items == 0 {
        return ModelAnalysisHealth {
            policy_version: POLICY_VERSION.to_string(),
            status: "not_required".to_string(),
            provider_name: parent.provider_name.clone(),
            profile_name: parent.profile_name.clone(),
            model: parent.model.clone(),
            requested_items: 0,
            analyzed_items: 0,
            failed_items: 0,
            failure_reasons: Vec::new(),
            decision_use_allowed: true,
        };
    }
    let status = if analyzed_items == requested_items {
        "healthy"
    } else if analyzed_items == 0 {
        "unavailable"
    } else {
        "partial"
    };
    ModelAnalysisHealth {
        policy_version: POLICY_VERSION.to_string(),
        status: status.to_string(),
        provider_name: parent.provider_name.clone(),
        profile_name: parent.profile_name.clone(),
        model: parent.model.clone(),
        requested_items,
        analyzed_items,
        failed_items: requested_items.saturating_sub(analyzed_items),
        failure_reasons: if status == "healthy" {
            Vec::new()
        } else {
            parent.failure_reasons.clone()
        },
        decision_use_allowed: status == "healthy",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_and_legacy_health_fail_closed() {
        let partial = build(None, 3, 2, ["invalid_output_contract"], "partial");
        assert_eq!(partial.failed_items, 1);
        assert!(!partial.decision_use_allowed);

        let legacy = ModelAnalysisHealth::default();
        assert_eq!(legacy.status, "unknown_legacy");
        assert!(!legacy.decision_use_allowed);
    }

    #[test]
    fn only_complete_or_not_required_work_is_decision_usable() {
        assert!(build(None, 3, 3, std::iter::empty(), "healthy").decision_use_allowed);
        assert!(build(None, 0, 0, std::iter::empty(), "not_required").decision_use_allowed);
        assert!(!build(None, 3, 3, ["upstream_request_failed"], "healthy").decision_use_allowed);
    }

    #[test]
    fn report_scope_ignores_unrelated_failures_but_preserves_relevant_ones() {
        let parent = build(None, 10, 5, ["analysis_timeout"], "partial");
        let empty = scope(&parent, 0, 0);
        assert_eq!(empty.status, "not_required");
        assert!(empty.decision_use_allowed);
        let partial = scope(&parent, 2, 1);
        assert_eq!(partial.failure_reasons, vec!["analysis_timeout"]);
        assert!(!partial.decision_use_allowed);
    }
}
