//! Deterministic security-identifier parsing and provider canonicalization.
//!
//! This module deliberately knows syntax and a small, audited set of provider
//! dialect mappings. It does not decide whether an ordinary English acronym is
//! a security in context; that confidence decision belongs to the caller.

const MAX_IDENTIFIER_BYTES: usize = 24;
const CRYPTO_QUOTE_CURRENCIES: &[&str] =
    &["USD", "USDT", "USDC", "EUR", "GBP", "JPY", "BTC", "ETH"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SecurityIdentifierKind {
    Bare,
    Cashtag,
    ExchangeQualified,
    ShareClass,
    Index,
    CryptoPair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecurityIdentifier {
    pub raw: String,
    pub normalized: String,
    pub start: usize,
    pub end: usize,
    pub kind: SecurityIdentifierKind,
}

/// Scan ASCII identifier-shaped spans in source order. A composite token is
/// always consumed as one lexical unit before validation. This is important:
/// an unsupported `605259.XY` may be rejected, but `.XY` must never be scanned
/// again as an unrelated US ticker.
pub(crate) fn scan_security_identifiers(input: &str) -> Vec<SecurityIdentifier> {
    let bytes = input.as_bytes();
    let excluded_ranges = non_security_source_ranges(input);
    let mut identifiers = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if let Some((_, end)) = excluded_ranges
            .iter()
            .find(|(start, end)| *start <= index && index < *end)
        {
            index = *end;
            continue;
        }
        let byte = bytes[index];
        let starts_identifier = byte.is_ascii_alphanumeric()
            || ((byte == b'$' || byte == b'^')
                && bytes
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_alphanumeric() || *next == b'^'));
        if !starts_identifier {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        if byte == b'$' && bytes.get(index) == Some(&b'^') {
            index += 1;
        }
        while index < bytes.len()
            && (bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'.' | b'-' | b'/'))
        {
            index += 1;
        }
        let mut end = index;
        while end > start && matches!(bytes[end - 1], b'.' | b'-' | b'/') {
            end -= 1;
        }
        if end <= start {
            continue;
        }
        push_scanned_token(input, start, end, &mut identifiers);
    }
    identifiers
}

/// URLs, email addresses and source-code/file paths are opaque source spans,
/// not comparison expressions. Excluding the whole span here prevents a
/// later slash/dot fallback from turning `example.com/news` or `src/lib.rs`
/// into securities when unrelated market words appear nearby.
fn non_security_source_ranges(input: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let lower = input.to_ascii_lowercase();
    for scheme in ["https://", "http://", "ftp://"] {
        let mut cursor = 0;
        while let Some(relative) = lower[cursor..].find(scheme) {
            let start = cursor + relative;
            let end = input[start..]
                .char_indices()
                .find(|(_, character)| {
                    character.is_whitespace()
                        || !character.is_ascii()
                        || matches!(
                            character,
                            ',' | ';' | '!' | '?' | ')' | ']' | '}' | '>' | '"' | '\''
                        )
                })
                .map_or(input.len(), |(offset, _)| start + offset);
            ranges.push((start, end));
            cursor = end.max(start + scheme.len());
            if cursor >= input.len() {
                break;
            }
        }
    }

    let bytes = input.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !is_source_token_byte(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && is_source_token_byte(bytes[index]) {
            index += 1;
        }
        let token = &input[start..index];
        let lower = token.to_ascii_lowercase();
        let email = lower.contains('@')
            && lower
                .rsplit_once('@')
                .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'));
        let explicit_path = lower.starts_with('/')
            || lower.starts_with("./")
            || lower.starts_with("../")
            || lower.starts_with("~/")
            || lower.contains('\\');
        let syntactic_identifier = normalize_and_classify(token).is_some();
        let raw_letters_are_uppercase = token
            .chars()
            .filter(|character| character.is_ascii_alphabetic())
            .all(|character| character.is_ascii_uppercase());
        let common_domain = [
            ".com", ".org", ".net", ".io", ".dev", ".app", ".cn", ".co.uk",
        ]
        .iter()
        .any(|suffix| lower.ends_with(suffix))
            && !(syntactic_identifier && raw_letters_are_uppercase);
        let common_file_suffix = [
            ".rs", ".md", ".toml", ".json", ".yaml", ".yml", ".js", ".jsx", ".ts", ".tsx", ".py",
            ".go", ".java", ".c", ".cc", ".cpp", ".h", ".hpp", ".html", ".css", ".scss", ".sh",
            ".zsh", ".fish", ".lock", ".log", ".txt", ".csv", ".pdf", ".doc", ".docx", ".xls",
            ".xlsx",
        ]
        .iter()
        .any(|suffix| lower.ends_with(suffix));
        let special_file_name = ["readme", "changelog", "license", "makefile", "dockerfile"]
            .iter()
            .any(|name| lower == *name || lower.starts_with(&format!("{name}.")));
        let provider_symbol_file_collision =
            lower.rsplit_once('.').is_some_and(|(base, suffix)| {
                (base.chars().all(|character| character.is_ascii_digit()) && suffix == "sh")
                    || (base.len() <= 3
                        && base
                            .chars()
                            .all(|character| character.is_ascii_alphabetic())
                        && matches!(suffix, "a" | "b" | "c"))
            });
        let looks_like_file = common_file_suffix
            && (token
                .chars()
                .any(|character| character.is_ascii_lowercase())
                || special_file_name)
            && !provider_symbol_file_collision;
        let repository_path = lower.contains('/')
            && ([
                "src/", "lib/", "docs/", "crates/", "apps/", "tests/", "scripts/",
            ]
            .iter()
            .any(|prefix| lower.starts_with(prefix))
                || looks_like_file);
        if email || explicit_path || common_domain || repository_path || looks_like_file {
            ranges.push((start, index));
        }
    }
    ranges.sort_unstable();
    ranges
}

fn is_source_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'/' | b'\\' | b'-' | b'~' | b'@')
}

fn push_scanned_token(
    input: &str,
    start: usize,
    end: usize,
    identifiers: &mut Vec<SecurityIdentifier>,
) {
    let raw = &input[start..end];
    if let Some((normalized, kind)) = normalize_and_classify(raw) {
        push_identifier(identifiers, raw, normalized, start, end, kind);
        return;
    }

    // A slash may be a crypto-pair separator or a human comparison separator.
    // The valid whole-token case returned above; otherwise parse each side as
    // a complete identifier so `600519.SH/NVDA`, `BRK-B/NVDA`, and
    // `BTC-USD/ETH-USD` retain their composite components.
    if raw.contains('/') {
        push_delimited_segments(raw, start, '/', identifiers);
        return;
    }
    // Likewise, a failed hyphenated whole token may be a compact comparison.
    if raw.contains('-') {
        push_delimited_segments(raw, start, '-', identifiers);
    }
}

fn push_delimited_segments(
    raw: &str,
    start: usize,
    separator: char,
    identifiers: &mut Vec<SecurityIdentifier>,
) {
    let mut cursor = 0;
    for segment in raw.split(separator) {
        let segment_start = start + cursor;
        let segment_end = segment_start + segment.len();
        if let Some((normalized, kind)) = normalize_and_classify(segment) {
            push_identifier(
                identifiers,
                segment,
                normalized,
                segment_start,
                segment_end,
                kind,
            );
        }
        cursor += segment.len() + separator.len_utf8();
    }
}

fn push_identifier(
    identifiers: &mut Vec<SecurityIdentifier>,
    raw: &str,
    normalized: String,
    start: usize,
    end: usize,
    kind: SecurityIdentifierKind,
) {
    identifiers.push(SecurityIdentifier {
        raw: raw.to_string(),
        normalized,
        start,
        end,
        kind,
    });
}

pub(crate) fn normalize_security_identifier(value: &str) -> Option<String> {
    normalize_and_classify(value.trim()).map(|(normalized, _)| normalized)
}

pub(crate) fn provider_lookup_variants(value: &str) -> Vec<String> {
    hone_core::provider_lookup_variants(value)
}

pub(crate) fn provider_canonical_key(value: &str) -> Option<String> {
    hone_core::provider_canonical_key(value)
}

/// Exact provider equivalence is intentionally bounded to audited provider
/// dialects. Bare numeric identifiers are never globally equivalent to a
/// suffixed symbol; they require a separate closed-market candidate probe.
pub(crate) fn provider_symbols_equivalent(requested: &str, candidate: &str) -> bool {
    hone_core::provider_symbols_equivalent(requested, candidate)
}

fn normalize_and_classify(value: &str) -> Option<(String, SecurityIdentifierKind)> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || !value.is_ascii() {
        return None;
    }
    let cashtag = value.starts_with('$');
    let core = value.strip_prefix('$').unwrap_or(value);
    if core.is_empty() || core.len() > MAX_IDENTIFIER_BYTES - usize::from(cashtag) {
        return None;
    }
    let normalized = core.to_ascii_uppercase();
    let classified = classify_normalized(&normalized)?;
    Some((
        normalized,
        if cashtag {
            SecurityIdentifierKind::Cashtag
        } else {
            classified
        },
    ))
}

fn classify_normalized(value: &str) -> Option<SecurityIdentifierKind> {
    if let Some(index) = value.strip_prefix('^') {
        return (!index.is_empty()
            && index.len() <= 12
            && index
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
            && index
                .chars()
                .any(|character| character.is_ascii_alphabetic()))
        .then_some(SecurityIdentifierKind::Index);
    }

    if value.contains('/') {
        let (base, suffix) = split_once_exact(value, '/')?;
        if is_crypto_pair(base, suffix) {
            return Some(SecurityIdentifierKind::CryptoPair);
        }
        return is_share_class(base, suffix).then_some(SecurityIdentifierKind::ShareClass);
    }
    if value.contains('-') {
        let (base, suffix) = split_once_exact(value, '-')?;
        if is_crypto_pair(base, suffix) {
            return Some(SecurityIdentifierKind::CryptoPair);
        }
        return is_share_class(base, suffix).then_some(SecurityIdentifierKind::ShareClass);
    }
    if value.contains('.') {
        let (base, suffix) = split_once_exact(value, '.')?;
        if base.chars().all(|character| character.is_ascii_digit())
            && is_exchange_suffix_shape(suffix)
        {
            return Some(SecurityIdentifierKind::ExchangeQualified);
        }
        if is_share_class(base, suffix) {
            return Some(SecurityIdentifierKind::ShareClass);
        }
        if base
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
            && base
                .chars()
                .any(|character| character.is_ascii_alphabetic())
            && is_exchange_suffix_shape(suffix)
        {
            return Some(SecurityIdentifierKind::ExchangeQualified);
        }
        return None;
    }

    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric())
    {
        return None;
    }
    let has_letter = value
        .chars()
        .any(|character| character.is_ascii_alphabetic());
    let numeric_identifier =
        value.chars().all(|character| character.is_ascii_digit()) && (3..=6).contains(&value.len());
    (has_letter || numeric_identifier).then_some(SecurityIdentifierKind::Bare)
}

fn split_once_exact(value: &str, separator: char) -> Option<(&str, &str)> {
    let mut parts = value.split(separator);
    let left = parts.next()?;
    let right = parts.next()?;
    (parts.next().is_none() && !left.is_empty() && !right.is_empty()).then_some((left, right))
}

fn is_share_class(base: &str, class: &str) -> bool {
    !base.is_empty()
        && base.len() <= 8
        && base
            .chars()
            .all(|character| character.is_ascii_alphabetic())
        && matches!(class, "A" | "B" | "C")
}

fn is_crypto_pair(base: &str, quote: &str) -> bool {
    (2..=10).contains(&base.len())
        && base
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
        && CRYPTO_QUOTE_CURRENCIES.contains(&quote)
}

fn is_exchange_suffix_shape(suffix: &str) -> bool {
    (1..=4).contains(&suffix.len())
        && suffix
            .chars()
            .all(|character| character.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        SecurityIdentifierKind, normalize_security_identifier, provider_lookup_variants,
        provider_symbols_equivalent, scan_security_identifiers,
    };

    #[test]
    fn scanner_covers_supported_identifier_families_in_source_order() {
        let input = "$RKLB, BRK.B 600519.SH 0700.HK 7203.T ^GSPC BTC-USD BTC/USD";
        let found = scan_security_identifiers(input);
        assert_eq!(
            found
                .iter()
                .map(|identifier| identifier.normalized.as_str())
                .collect::<Vec<_>>(),
            [
                "RKLB",
                "BRK.B",
                "600519.SH",
                "0700.HK",
                "7203.T",
                "^GSPC",
                "BTC-USD",
                "BTC/USD",
            ]
        );
        assert_eq!(found[0].kind, SecurityIdentifierKind::Cashtag);
        assert_eq!(found[1].kind, SecurityIdentifierKind::ShareClass);
        assert_eq!(found[2].kind, SecurityIdentifierKind::ExchangeQualified);
        assert_eq!(found[5].kind, SecurityIdentifierKind::Index);
        assert_eq!(found[6].kind, SecurityIdentifierKind::CryptoPair);
    }

    #[test]
    fn sentence_punctuation_is_not_part_of_a_cashtag() {
        let found = scan_security_identifiers("看一下 $AAPL. 然后 $^GSPC。");
        assert_eq!(
            found
                .iter()
                .map(|identifier| (identifier.raw.as_str(), identifier.normalized.as_str()))
                .collect::<Vec<_>>(),
            [("$AAPL", "AAPL"), ("$^GSPC", "^GSPC")]
        );
    }

    #[test]
    fn urls_emails_domains_and_source_paths_are_opaque() {
        let input = "行情 https://example.com/AAPL user@market.com src/lib.rs README.md config.rs main.py foo.ts AAPL/NVDA；参考https://example.com/news，看AAPL股价；https://x.com,MSFT股价；https://x.com看TSLA；请读README.md，然后看NVDA；600519.sh brk.c ABC.CN";
        let found = scan_security_identifiers(input)
            .into_iter()
            .map(|identifier| identifier.normalized)
            .collect::<Vec<_>>();
        assert_eq!(
            found,
            [
                "AAPL",
                "NVDA",
                "AAPL",
                "MSFT",
                "TSLA",
                "NVDA",
                "600519.SH",
                "BRK.C",
                "ABC.CN",
            ]
        );
    }

    #[test]
    fn digit_leading_composite_is_consumed_without_suffix_rescan() {
        for input in ["605259.SH", "600519.SS", "2026.Q1", "605259.XY"] {
            let found = scan_security_identifiers(input);
            assert!(
                !found.iter().any(|identifier| matches!(
                    identifier.normalized.as_str(),
                    "SH" | "SS" | "Q1" | "XY"
                )),
                "{input}: {found:?}"
            );
        }
        assert_eq!(
            scan_security_identifiers("605259.SH")[0].normalized,
            "605259.SH"
        );
        assert!(scan_security_identifiers("2026.Q1").is_empty());
    }

    #[test]
    fn comparison_separators_split_only_after_composite_validation_fails() {
        let found = scan_security_identifiers(
            "RKLB/NVDA BRK-B BTC/USD 600519.SH/NVDA BRK-B/NVDA BTC-USD/ETH-USD $BRK.B/NVDA",
        );
        assert_eq!(
            found
                .iter()
                .map(|identifier| identifier.normalized.as_str())
                .collect::<Vec<_>>(),
            [
                "RKLB",
                "NVDA",
                "BRK-B",
                "BTC/USD",
                "600519.SH",
                "NVDA",
                "BRK-B",
                "NVDA",
                "BTC-USD",
                "ETH-USD",
                "BRK.B",
                "NVDA",
            ]
        );
    }

    #[test]
    fn scanner_keeps_bare_ticker_before_chinese_heartbeat_suffix() {
        let found = scan_security_identifiers("ORCL 大事件监控");
        assert_eq!(
            found
                .iter()
                .map(|identifier| identifier.normalized.as_str())
                .collect::<Vec<_>>(),
            ["ORCL"]
        );
    }

    #[test]
    fn syntactically_valid_exchange_suffixes_are_preserved_for_exact_lookup() {
        for symbol in ["SAN.MC", "TEF.MC", "EDP.LS", "DELTA.BK", "NICE.TA"] {
            assert_eq!(
                normalize_security_identifier(symbol).as_deref(),
                Some(symbol),
                "{symbol}"
            );
        }
        assert!(normalize_security_identifier("2026.Q1").is_none());
    }

    #[test]
    fn provider_variants_are_bounded_ordered_and_idempotent() {
        for (input, expected) in [
            ("$RKLB", vec!["RKLB"]),
            ("BRK.B", vec!["BRK-B", "BRK.B"]),
            ("BRK-B", vec!["BRK-B"]),
            ("600519.SH", vec!["600519.SS", "600519.SH"]),
            ("600519.SS", vec!["600519.SS"]),
            ("700.HK", vec!["0700.HK", "700.HK"]),
            ("00700.HK", vec!["0700.HK", "00700.HK"]),
            ("09988.HK", vec!["9988.HK", "09988.HK"]),
            ("AAPL.US", vec!["AAPL", "AAPL.US"]),
            ("^GSPC", vec!["^GSPC"]),
            ("GSPC", vec!["^GSPC", "GSPC"]),
            ("IXIC", vec!["^IXIC", "IXIC"]),
            ("DJI", vec!["^DJI", "DJI"]),
            ("RUT", vec!["^RUT", "RUT"]),
            ("VIX", vec!["^VIX", "VIX"]),
            ("BTC-USD", vec!["BTCUSD", "BTC-USD"]),
            ("BTC/USD", vec!["BTCUSD", "BTC/USD"]),
        ] {
            assert_eq!(provider_lookup_variants(input), expected, "{input}");
            let canonical = &expected[0];
            assert_eq!(
                provider_lookup_variants(canonical)[0],
                *canonical,
                "{input}"
            );
        }
    }

    #[test]
    fn provider_equivalence_accepts_only_audited_dialects() {
        for (left, right) in [
            ("BRK.B", "BRK-B"),
            ("600519.SH", "600519.SS"),
            ("700.HK", "0700.HK"),
            ("09988.HK", "9988.HK"),
            ("GSPC", "^GSPC"),
            ("IXIC", "^IXIC"),
            ("DJI", "^DJI"),
            ("RUT", "^RUT"),
            ("VIX", "^VIX"),
            ("BTC-USD", "BTCUSD"),
            ("BTC/USD", "BTCUSD"),
            ("AAPL.US", "AAPL"),
        ] {
            assert!(
                provider_symbols_equivalent(left, right),
                "{left} != {right}"
            );
            assert!(
                provider_symbols_equivalent(right, left),
                "{right} != {left}"
            );
        }
        for (left, right) in [
            ("600519.SH", "600519.SZ"),
            ("RKLB", "RKLX"),
            ("BRK.B", "BRKC"),
            ("0700.HK", "3700.HK"),
            ("BTCUSD", "ETHUSD"),
            ("FOO", "^FOO"),
            ("0700", "0700.HK"),
            ("000001", "000001.SS"),
            ("000001", "000001.SZ"),
            ("0700", "000700.KS"),
        ] {
            assert!(
                !provider_symbols_equivalent(left, right),
                "{left} == {right}"
            );
            assert!(
                !provider_symbols_equivalent(right, left),
                "{right} == {left}"
            );
        }
    }

    #[test]
    fn invalid_identifiers_do_not_normalize() {
        for value in ["", "../", "A?B", "2026.Q1", "你好", "AAPL#x"] {
            assert!(normalize_security_identifier(value).is_none(), "{value}");
        }
    }

    #[test]
    fn scanner_spans_are_ordered_non_overlapping_and_round_trip() {
        let input = "比较 $RKLB、600519.SH 和 RKLB/NVDA";
        let found = scan_security_identifiers(input);
        for pair in found.windows(2) {
            assert!(pair[0].end <= pair[1].start, "{found:?}");
        }
        for identifier in found {
            assert_eq!(&input[identifier.start..identifier.end], identifier.raw);
        }
        let unique = found_symbols("RKLB RKLB $RKLB");
        assert_eq!(unique, ["RKLB"]);
    }

    fn found_symbols(input: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        scan_security_identifiers(input)
            .into_iter()
            .filter_map(|identifier| {
                seen.insert(identifier.normalized.clone())
                    .then_some(identifier.normalized)
            })
            .collect()
    }
}
