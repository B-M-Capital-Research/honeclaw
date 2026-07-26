//! Shared classification for provider/model context-window failures.
//!
//! Keep this below runner/channel layers so recovery policy can use one marker
//! set without exposing provider wording to users.

/// Returns whether an internal error reports that the current model request no
/// longer fits the configured context window.
pub fn is_context_overflow_error(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    normalized.contains("context window exceeds limit")
        || normalized.contains("context window overflow")
        || normalized.contains("context_window_exceeded")
        || normalized.contains("context_window_will_overflow")
        || normalized.contains("context length exceeded")
        || normalized.contains("maximum context length")
        || normalized.contains("prompt is too long")
        || normalized.contains("too many tokens")
        || normalized.contains("request entity too large")
        || normalized.contains("当前会话上下文过长")
        || normalized.contains("会话上下文过长")
}

#[cfg(test)]
mod tests {
    use super::is_context_overflow_error;

    #[test]
    fn recognizes_supported_provider_context_markers() {
        for error in [
            "context window exceeds limit (2013)",
            "context_window_exceeded",
            "maximum context length exceeded",
            "prompt is too long",
            "too many tokens",
            "request entity too large (2013)",
            "当前会话上下文过长",
        ] {
            assert!(is_context_overflow_error(error), "{error}");
        }
        assert!(!is_context_overflow_error(
            "provider temporarily unavailable"
        ));
    }
}
