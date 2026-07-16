//! Canonical tool-trace helpers used by retry and ACP failure handling.
//!
//! Tool names arrive in several runner-specific shapes (`portfolio`,
//! `hone/portfolio`, `mcp__hone__portfolio`, `Tool: hone/portfolio`).  Retry
//! safety must not depend on which ACP client happened to render the name.

use hone_core::agent::ToolCallMade;

pub(crate) const PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE: &str = "这次操作可能已经执行，但执行器在返回最终确认前中断，当前状态无法确定。为避免重复写入，我没有自动重试；请先查看当前持仓、关注、提醒或服务状态，再决定是否重试。";

pub(crate) const PERSISTENT_SIDE_EFFECT_NO_RETRY_MESSAGE: &str = "这次操作已经进入执行流程，但最终回复不完整。为避免重复修改持仓、关注、提醒或服务状态，我没有自动重试；请先核对当前状态。";

const PERSISTENT_TOOL_NAMES: &[&str] = &[
    "cron_job",
    "portfolio",
    "portfolio_tool",
    "notification_prefs",
    "restart_hone",
];

/// Return the canonical Hone tool name for runner/MCP aliases.
pub(crate) fn canonical_hone_tool_name(name: &str) -> Option<&'static str> {
    let mut normalized = name.trim();
    if normalized
        .as_bytes()
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"tool:"))
    {
        normalized = normalized.get(5..).unwrap_or_default().trim();
    }
    let normalized = normalized.to_ascii_lowercase();

    PERSISTENT_TOOL_NAMES.iter().copied().find(|candidate| {
        normalized == *candidate
            || normalized == format!("hone/{candidate}")
            || normalized == format!("hone_{candidate}")
            || normalized == format!("mcp__hone__{candidate}")
            || normalized == format!("mcp_hone_{candidate}")
    })
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
        Some("portfolio") | Some("portfolio_tool") => !matches!(tool_action(call), Some("view")),
        Some("notification_prefs") => !matches!(tool_action(call), Some("get" | "get_overview")),
        Some("restart_hone") => true,
        _ => false,
    }
}

pub(crate) fn response_has_persistent_side_effect(tool_calls: &[ToolCallMade]) -> bool {
    tool_calls.iter().any(is_persistent_side_effect_call)
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
}
