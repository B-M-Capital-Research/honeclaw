//! Canonical tool-trace helpers used by retry and ACP failure handling.
//!
//! Tool names arrive in several runner-specific shapes (`portfolio`,
//! `hone/portfolio`, `mcp__hone__portfolio`, `Tool: hone/portfolio`).  Retry
//! safety must not depend on which ACP client happened to render the name.

use hone_core::agent::ToolCallMade;

pub(crate) const PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE: &str = "这次操作可能已经执行，但执行器在返回最终确认前中断，当前状态无法确定。为避免重复写入或重复启动研究任务，我没有自动重试；请先查看当前持仓、关注、提醒、研究任务或服务状态，再决定是否重试。";

pub(crate) const PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE: &str = "这次操作已经进入执行流程，但最终回复不完整。为避免重复修改持仓、关注、提醒、服务状态或重复启动研究任务，我没有自动重试；请先核对当前状态。";

pub(crate) const UNKNOWN_TOOL_EFFECT_NO_RETRY_MESSAGE: &str = "本轮分析调用了无法确认只读属性的工具，但最终回复不完整。为避免掩盖或重复外部操作，我没有自动修复或重试；请先核对相关状态。";

const PERSISTENT_TOOL_NAMES: &[&str] = &[
    "cron_job",
    "deep_research",
    "portfolio",
    "portfolio_tool",
    "notification_prefs",
    "restart_hone",
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

/// Return the canonical Hone tool name for runner/MCP aliases.
pub(crate) fn canonical_hone_tool_name(name: &str) -> Option<&'static str> {
    let normalized = normalized_runner_tool_name(name);

    PERSISTENT_TOOL_NAMES
        .iter()
        .copied()
        .find(|candidate| runner_tool_name_matches(&normalized, candidate))
}

fn tool_action(call: &ToolCallMade) -> Option<&str> {
    call.arguments
        .get("action")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

/// Whether a call can mutate persistent user/system state.
pub(crate) fn is_persistent_side_effect_call(call: &ToolCallMade) -> bool {
    match canonical_hone_tool_name(&call.name) {
        Some("cron_job") => !matches!(tool_action(call), Some("list")),
        Some("deep_research") => true,
        Some("portfolio") | Some("portfolio_tool") => !matches!(tool_action(call), Some("view")),
        Some("notification_prefs") => !matches!(tool_action(call), Some("get" | "get_overview")),
        Some("restart_hone") => true,
        _ => false,
    }
}

pub(crate) fn response_has_persistent_side_effect(tool_calls: &[ToolCallMade]) -> bool {
    tool_calls.iter().any(is_persistent_side_effect_call)
}

/// Whether every observed call is explicitly known to be read-only.
///
/// This is intentionally an allowlist rather than the inverse of the known
/// persistent tools. ACP clients and external MCP servers can expose arbitrary
/// write-capable tools, so an unknown name must never be discarded or replayed
/// during investment-response repair.
pub(crate) fn is_known_read_only_call(call: &ToolCallMade) -> bool {
    match canonical_hone_tool_name(&call.name) {
        Some("cron_job") => matches!(tool_action(call), Some("list")),
        Some("deep_research") => false,
        Some("portfolio") | Some("portfolio_tool") => {
            matches!(tool_action(call), Some("view"))
        }
        Some("notification_prefs") => {
            matches!(tool_action(call), Some("get" | "get_overview"))
        }
        Some("restart_hone") => false,
        _ => {
            let normalized = normalized_runner_tool_name(&call.name);
            KNOWN_READ_ONLY_TOOL_NAMES
                .iter()
                .any(|candidate| runner_tool_name_matches(&normalized, candidate))
        }
    }
}

pub(crate) fn response_has_only_known_read_only_calls(tool_calls: &[ToolCallMade]) -> bool {
    tool_calls.iter().all(is_known_read_only_call)
}

fn result_is_uncertain(result: &serde_json::Value) -> bool {
    result
        .get("status")
        .and_then(|value| value.as_str())
        .is_some_and(|status| {
            status == "failed"
                || status == "unknown_after_acp_failure"
                || status == "unknown_after_missing_acp_result"
        })
        || result
            .get("isError")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
}

pub(crate) fn persistent_side_effect_state_is_uncertain(tool_calls: &[ToolCallMade]) -> bool {
    tool_calls
        .iter()
        .any(|call| is_persistent_side_effect_call(call) && result_is_uncertain(&call.result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn call(name: &str, action: Option<&str>, result: serde_json::Value) -> ToolCallMade {
        ToolCallMade {
            name: name.to_string(),
            arguments: action.map_or_else(|| json!({}), |action| json!({"action": action})),
            result,
            tool_call_id: Some("call_1".to_string()),
        }
    }

    #[test]
    fn canonicalizes_real_acp_and_mcp_tool_names() {
        for name in [
            "portfolio",
            "hone/portfolio",
            "hone_portfolio",
            "mcp__hone__portfolio",
            "mcp_hone_portfolio",
            "Tool: hone/portfolio",
        ] {
            assert_eq!(canonical_hone_tool_name(name), Some("portfolio"), "{name}");
        }
        assert_eq!(canonical_hone_tool_name("工具：portfolio"), None);
    }

    #[test]
    fn distinguishes_read_only_and_persistent_actions() {
        assert!(!is_persistent_side_effect_call(&call(
            "cron_job",
            Some("list"),
            json!({})
        )));
        assert!(!is_persistent_side_effect_call(&call(
            "hone/portfolio",
            Some("view"),
            json!({})
        )));
        assert!(!is_persistent_side_effect_call(&call(
            "notification_prefs",
            Some("get_overview"),
            json!({})
        )));
        assert!(is_persistent_side_effect_call(&call(
            "mcp__hone__cron_job",
            Some("add"),
            json!({})
        )));
        assert!(is_persistent_side_effect_call(&call(
            "mcp__hone__deep_research",
            None,
            json!({"task_id":"research-1"})
        )));
        assert!(is_persistent_side_effect_call(&call(
            "Tool: hone/portfolio",
            Some("watch"),
            json!({})
        )));
        assert!(is_persistent_side_effect_call(&call(
            "restart_hone",
            None,
            json!({})
        )));
    }

    #[test]
    fn recognizes_uncertain_persistent_results() {
        let calls = vec![call(
            "portfolio",
            Some("add"),
            json!({"status":"unknown_after_acp_failure", "isError":true}),
        )];
        assert!(response_has_persistent_side_effect(&calls));
        assert!(persistent_side_effect_state_is_uncertain(&calls));
    }

    #[test]
    fn investment_repair_read_only_allowlist_fails_closed_for_unknown_tools() {
        for read_only in [
            call("data_fetch", None, json!({})),
            call("mcp__hone__web_search", None, json!({})),
            call("portfolio", Some("view"), json!({})),
            call("notification_prefs", Some("get_overview"), json!({})),
            call("cron_job", Some("list"), json!({})),
        ] {
            assert!(is_known_read_only_call(&read_only), "{}", read_only.name);
        }

        for unsafe_call in [
            call("portfolio", Some("add"), json!({})),
            call("deep_research", None, json!({"task_id":"research-1"})),
            call("skill_tool", None, json!({})),
            call("mcp__filesystem__write_file", None, json!({})),
            call("external/create_order", None, json!({})),
        ] {
            assert!(
                !is_known_read_only_call(&unsafe_call),
                "{} must fail closed",
                unsafe_call.name
            );
        }
        assert!(response_has_only_known_read_only_calls(&[]));
        assert!(!response_has_only_known_read_only_calls(&[
            call("data_fetch", None, json!({})),
            call("mcp__filesystem__write_file", None, json!({})),
        ]));
    }
}
