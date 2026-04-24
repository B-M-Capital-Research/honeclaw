//! 文本截断等共享工具,避免各 crate 各自实现。

/// 若 `s` 长度超过 `max_chars`,按字符数截断并返回前 `max_chars` 个字符。
/// `max_chars == 0` 视为不截断（与原 `session_compactor` 语义一致）。
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    if max_chars == 0 || s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect()
}

/// 若 `s` 长度超过 `max_chars`,取前 `max_chars` 个字符并追加 `suffix`;否则原样返回。
/// 结果总长度为 `max_chars + suffix.chars().count()`。
///
/// 若希望"suffix 占用预算"（总长 ≤ max_chars）,调用方自行传 `max_chars - 1`
/// 并保持 suffix 为单字符（例如 `…`）。
pub fn truncate_chars_append(s: &str, max_chars: usize, suffix: &str) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars).collect();
    out.push_str(suffix);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_returns_original_when_short() {
        assert_eq!(truncate_chars("hello", 10), "hello");
    }

    #[test]
    fn truncate_chars_cuts_to_max() {
        assert_eq!(truncate_chars("abcdef", 3), "abc");
    }

    #[test]
    fn truncate_chars_zero_means_no_truncation() {
        assert_eq!(truncate_chars("abc", 0), "abc");
    }

    #[test]
    fn truncate_chars_append_noop_when_short() {
        assert_eq!(truncate_chars_append("hi", 10, "..."), "hi");
    }

    #[test]
    fn truncate_chars_append_adds_suffix_beyond_budget() {
        assert_eq!(truncate_chars_append("abcdef", 3, "..."), "abc...");
    }

    #[test]
    fn truncate_chars_append_handles_multibyte() {
        assert_eq!(truncate_chars_append("你好世界hello", 4, "…"), "你好世界…");
    }
}
