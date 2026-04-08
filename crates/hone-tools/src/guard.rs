//! Tool execution guard
//!
//! 在工具执行前对参数进行安全检查，用于阻拦高风险指令。

use hone_core::HoneError;
use hone_core::config::ToolGuardConfig;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGuardMode {
    Block,
    Audit,
}

impl ToolGuardMode {
    fn from_str(mode: &str) -> Self {
        match mode.trim().to_lowercase().as_str() {
            "audit" | "log" | "warn" => ToolGuardMode::Audit,
            _ => ToolGuardMode::Block,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolExecutionGuard {
    enabled: bool,
    mode: ToolGuardMode,
    apply_tools: Vec<String>,
    deny_patterns: Vec<String>,
}

impl ToolExecutionGuard {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            mode: ToolGuardMode::Block,
            apply_tools: Vec::new(),
            deny_patterns: Vec::new(),
        }
    }

    pub fn from_config(cfg: &ToolGuardConfig) -> Self {
        Self {
            enabled: cfg.enabled,
            mode: ToolGuardMode::from_str(&cfg.mode),
            apply_tools: cfg.apply_tools.clone(),
            deny_patterns: cfg.deny_patterns.iter().map(|s| s.to_lowercase()).collect(),
        }
    }

    fn applies_to_tool(&self, tool_name: &str) -> bool {
        if self.apply_tools.is_empty() {
            return true;
        }
        let tool_lower = tool_name.to_lowercase();
        let mut matched = false;
        for raw in &self.apply_tools {
            let item = raw.trim();
            if item.is_empty() {
                continue;
            }
            if let Some(exclude) = item.strip_prefix('!') {
                let exclude = exclude.trim().to_lowercase();
                if exclude == "*" || tool_lower.contains(&exclude) {
                    return false;
                }
                continue;
            }
            let include = item.to_lowercase();
            if include == "*" || tool_lower.contains(&include) {
                matched = true;
            }
        }
        matched
    }

    fn args_hit_deny_pattern(&self, args: &Value) -> Option<String> {
        if self.deny_patterns.is_empty() {
            return None;
        }
        let Ok(raw) = serde_json::to_string(args) else {
            return None;
        };
        let haystack = raw.to_lowercase();
        for pattern in &self.deny_patterns {
            if pattern.is_empty() {
                continue;
            }
            if haystack.contains(pattern) {
                return Some(pattern.clone());
            }
        }
        None
    }

    pub fn check(&self, tool_name: &str, args: &Value) -> Result<(), HoneError> {
        if !self.enabled {
            return Ok(());
        }
        if !self.applies_to_tool(tool_name) {
            return Ok(());
        }
        if let Some(pattern) = self.args_hit_deny_pattern(args) {
            let message = format!("tool_guard_blocked tool={tool_name} pattern={pattern}");
            match self.mode {
                ToolGuardMode::Audit => {
                    tracing::warn!("[ToolGuard] {message}");
                    Ok(())
                }
                ToolGuardMode::Block => {
                    tracing::warn!("[ToolGuard] {message}");
                    Err(HoneError::Tool(format!(
                        "工具调用被安全策略阻止（pattern={pattern}）"
                    )))
                }
            }
        } else {
            Ok(())
        }
    }
}

impl Default for ToolExecutionGuard {
    fn default() -> Self {
        Self::disabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::config::ToolGuardConfig;
    use serde_json::json;

    #[test]
    fn disabled_guard_allows_calls() {
        let guard = ToolExecutionGuard::disabled();
        assert!(guard.check("exec_command", &json!({"cmd": "rm -rf /"})).is_ok());
    }

    #[test]
    fn block_mode_rejects_matching_patterns_case_insensitively() {
        let guard = ToolExecutionGuard::from_config(&ToolGuardConfig {
            enabled: true,
            mode: "block".to_string(),
            apply_tools: vec!["exec".to_string()],
            deny_patterns: vec!["RM -RF".to_string()],
        });

        let err = guard
            .check("exec_command", &json!({"cmd": "sudo rm -rf /tmp/demo"}))
            .expect_err("guard should block");
        assert!(err.to_string().contains("pattern=rm -rf"));
    }

    #[test]
    fn audit_mode_logs_but_allows_calls() {
        let guard = ToolExecutionGuard::from_config(&ToolGuardConfig {
            enabled: true,
            mode: "audit".to_string(),
            apply_tools: vec!["*".to_string()],
            deny_patterns: vec!["shutdown -r".to_string()],
        });

        assert!(guard
            .check("exec_command", &json!({"cmd": "shutdown -r now"}))
            .is_ok());
    }

    #[test]
    fn excluded_tools_override_wildcard_inclusions() {
        let guard = ToolExecutionGuard::from_config(&ToolGuardConfig {
            enabled: true,
            mode: "block".to_string(),
            apply_tools: vec!["*".to_string(), "!web_search".to_string()],
            deny_patterns: vec!["drop table".to_string()],
        });

        assert!(guard
            .check("web_search", &json!({"query": "drop table users"}))
            .is_ok());
        assert!(guard
            .check("exec_command", &json!({"cmd": "drop table users"}))
            .is_err());
    }
}
