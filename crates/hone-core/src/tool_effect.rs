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
        // Loading a skill prompt and refreshing the same skill's invocation
        // metadata is idempotent bookkeeping, not a durable business
        // mutation. Executable skill scripts remain conservatively
        // persistent because repository-declared code may have arbitrary
        // effects.
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
        // A non-executable skill invocation returns repository instructions
        // and idempotently refreshes invoked-skill metadata. It is safe at a
        // read-only continuation/retry boundary. `execute_script=true` stays
        // unknown and persistent.
        Some("skill_tool") => !arguments
            .get("execute_script")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        _ => {
            let normalized = normalized_runner_tool_name(name);
            KNOWN_READ_ONLY_TOOL_NAMES
                .iter()
                .any(|candidate| runner_tool_name_matches(&normalized, candidate))
        }
    }
}

/// Return a safe read-after-write call for persistent mutations whose final
/// state can be observed through the same actor-scoped tool. This never
/// replays the mutation: callers execute the returned read only after an
/// ambiguous mutation failure, then let the Agent describe the observed state.
pub fn persistent_tool_reconciliation_call(
    name: &str,
    arguments: &Value,
) -> Option<(&'static str, Value)> {
    if !tool_call_has_persistent_side_effect(name, arguments) {
        return None;
    }
    match canonical_hone_tool_name(name) {
        Some("cron_job") => Some(("cron_job", serde_json::json!({"action": "list"}))),
        Some("portfolio") | Some("portfolio_tool") => {
            Some(("portfolio", serde_json::json!({"action": "view"})))
        }
        Some("notification_prefs") => Some((
            "notification_prefs",
            serde_json::json!({"action": "get_overview"}),
        )),
        // deep_research, restart_hone and executable skill scripts do not have
        // a side-effect-free actor-scoped state read that can prove completion.
        _ => None,
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
        assert!(tool_call_is_known_read_only(
            "skill_tool",
            &json!({"skill_name":"stock_research"})
        ));
        assert!(tool_call_is_known_read_only(
            "mcp__hone__skill_tool",
            &json!({"skill_name":"image_understanding","execute_script":false})
        ));
        assert!(!tool_call_is_known_read_only(
            "hone/skill_tool",
            &json!({"skill_name":"image_understanding","execute_script":true})
        ));
    }

    #[test]
    fn ambiguous_mutations_reconcile_with_reads_without_replaying_writes() {
        for (name, write, expected_name, expected_read) in [
            (
                "mcp__hone__portfolio",
                json!({"action":"add","ticker":"CRWV"}),
                "portfolio",
                json!({"action":"view"}),
            ),
            (
                "hone/cron_job",
                json!({"action":"delete","job_id":"job-1"}),
                "cron_job",
                json!({"action":"list"}),
            ),
            (
                "notification_prefs",
                json!({"action":"set","enabled":false}),
                "notification_prefs",
                json!({"action":"get_overview"}),
            ),
        ] {
            let (read_name, read_args) =
                persistent_tool_reconciliation_call(name, &write).expect("reconciliation read");
            assert_eq!(read_name, expected_name);
            assert_eq!(read_args, expected_read);
            assert!(tool_call_is_known_read_only(read_name, &read_args));
            assert!(!tool_call_has_persistent_side_effect(read_name, &read_args));
        }

        assert!(
            persistent_tool_reconciliation_call("portfolio", &json!({"action":"view"})).is_none()
        );
        assert!(
            persistent_tool_reconciliation_call("restart_hone", &json!({"action":"restart"}))
                .is_none()
        );
    }
}
