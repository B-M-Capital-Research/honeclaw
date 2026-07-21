//! Canonical Hone tool-effect classification shared by runners and retry
//! boundaries. Keeping this in `hone-core` prevents a function-calling Agent
//! and its outer session from disagreeing about whether a failed call may have
//! mutated durable state.

use serde_json::Value;

const PERSISTENT_TOOL_NAMES: &[&str] = &[
    "cron_job",
    "deep_research",
    "portfolio",
    "portfolio_tool",
    "notification_prefs",
    "restart_hone",
    "skill_tool",
];

const KNOWN_READ_ONLY_TOOL_NAMES: &[&str] = &[
    "data_fetch",
    "discover_skills",
    "load_skill",
    "local_list_files",
    "local_read_file",
    "local_search_files",
    "missed_events",
    "web_search",
];

fn normalized_runner_tool_name(name: &str) -> String {
    let mut normalized = name.trim();
    if normalized
        .as_bytes()
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"tool:"))
    {
        normalized = normalized.get(5..).unwrap_or_default().trim();
    }
    normalized.to_ascii_lowercase()
}

fn runner_tool_name_matches(normalized: &str, candidate: &str) -> bool {
    normalized == candidate
        || normalized == format!("hone/{candidate}")
        || normalized == format!("hone_{candidate}")
        || normalized == format!("mcp__hone__{candidate}")
        || normalized == format!("mcp_hone_{candidate}")
}

/// Return the canonical Hone tool name for direct, MCP, and runner aliases.
pub fn canonical_hone_tool_name(name: &str) -> Option<&'static str> {
    let normalized = normalized_runner_tool_name(name);
    PERSISTENT_TOOL_NAMES
        .iter()
        .chain(KNOWN_READ_ONLY_TOOL_NAMES)
        .copied()
        .find(|candidate| runner_tool_name_matches(&normalized, candidate))
}

fn tool_action(arguments: &Value) -> Option<&str> {
    arguments
        .get("action")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

/// Whether a concrete tool invocation can mutate persistent user/system state.
pub fn tool_call_has_persistent_side_effect(name: &str, arguments: &Value) -> bool {
    match canonical_hone_tool_name(name) {
        Some("cron_job") => !matches!(tool_action(arguments), Some("list")),
        Some("deep_research") => true,
        Some("portfolio") | Some("portfolio_tool") => {
            !matches!(tool_action(arguments), Some("view"))
        }
        Some("notification_prefs") => {
            !matches!(tool_action(arguments), Some("get" | "get_overview"))
        }
        Some("restart_hone") => true,
        Some("skill_tool") => arguments
            .get("execute_script")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        _ => false,
    }
}

/// Whether a concrete invocation is explicitly known to be read-only.
/// Unknown tools intentionally return false.
pub fn tool_call_is_known_read_only(name: &str, arguments: &Value) -> bool {
    match canonical_hone_tool_name(name) {
        Some("cron_job") => matches!(tool_action(arguments), Some("list")),
        Some("deep_research") => false,
        Some("portfolio") | Some("portfolio_tool") => {
            matches!(tool_action(arguments), Some("view"))
        }
        Some("notification_prefs") => {
            matches!(tool_action(arguments), Some("get" | "get_overview"))
        }
        Some("restart_hone") => false,
        // Loading a skill may update invoked-skill Session metadata, and
        // execute_script=true can perform arbitrary declared script effects.
        // Treat it as unknown/non-read-only even when script execution is off;
        // the persistent classifier above specifically blocks the executable
        // form at finance read-only boundaries.
        Some("skill_tool") => false,
        _ => {
            let normalized = normalized_runner_tool_name(name);
            KNOWN_READ_ONLY_TOOL_NAMES
                .iter()
                .any(|candidate| runner_tool_name_matches(&normalized, candidate))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn persistent_and_read_only_actions_share_one_classifier() {
        assert!(tool_call_has_persistent_side_effect(
            "mcp__hone__portfolio",
            &json!({"action":"watch"})
        ));
        assert!(!tool_call_is_known_read_only(
            "mcp__hone__portfolio",
            &json!({"action":"watch"})
        ));
        assert!(!tool_call_has_persistent_side_effect(
            "Tool: hone/portfolio",
            &json!({"action":"view"})
        ));
        assert!(tool_call_is_known_read_only(
            "Tool: hone/portfolio",
            &json!({"action":"view"})
        ));
        assert!(tool_call_is_known_read_only(
            "data_fetch",
            &json!({"data_type":"quote","ticker":"CRWV"})
        ));
        assert!(!tool_call_is_known_read_only(
            "external_unknown_tool",
            &json!({})
        ));
        assert!(tool_call_has_persistent_side_effect(
            "hone/skill_tool",
            &json!({"skill":"stock_research","execute_script":true})
        ));
        assert!(!tool_call_has_persistent_side_effect(
            "mcp__hone__skill_tool",
            &json!({"skill":"stock_research","execute_script":false})
        ));
        assert!(!tool_call_is_known_read_only(
            "skill_tool",
            &json!({"skill":"stock_research"})
        ));
    }
}
