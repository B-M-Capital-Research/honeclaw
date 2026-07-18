//! Bounded, provider-audited security-symbol equivalence shared by routing layers.
//!
//! This module intentionally handles only syntax and known provider dialects.
//! It does not infer whether an arbitrary word is a ticker in user context.

const MAX_IDENTIFIER_BYTES: usize = 24;
const CRYPTO_QUOTE_CURRENCIES: &[&str] =
    &["USD", "USDT", "USDC", "EUR", "GBP", "JPY", "BTC", "ETH"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderSymbolKind {
    Bare,
    ExchangeQualified,
    ShareClass,
    Index,
    CryptoPair,
}

pub fn provider_lookup_variants(value: &str) -> Vec<String> {
    let Some((normalized, kind)) = normalize_and_classify(value.trim()) else {
        return Vec::new();
    };
    let mut variants = Vec::new();
    let provider = match kind {
        ProviderSymbolKind::CryptoPair => normalized.replace(['-', '/'], ""),
        ProviderSymbolKind::ShareClass if normalized.contains(['.', '/']) => {
            normalized.replace(['.', '/'], "-")
        }
        ProviderSymbolKind::ExchangeQualified => provider_exchange_symbol(&normalized),
        ProviderSymbolKind::Bare if is_known_unprefixed_index_alias(&normalized) => {
            format!("^{normalized}")
        }
        _ => normalized.clone(),
    };
    push_unique(&mut variants, provider);
    push_unique(&mut variants, normalized);
    variants
}

pub fn provider_canonical_key(value: &str) -> Option<String> {
    provider_lookup_variants(value).into_iter().next()
}

/// Exact provider equivalence is intentionally bounded to audited provider
/// dialects. Bare numeric identifiers are never globally equivalent to a
/// suffixed symbol; they require a separate closed-market candidate probe.
pub fn provider_symbols_equivalent(requested: &str, candidate: &str) -> bool {
    let requested_variants = provider_lookup_variants(requested);
    let candidate_variants = provider_lookup_variants(candidate);
    if requested_variants.is_empty() || candidate_variants.is_empty() {
        return false;
    }
    if requested_variants.iter().any(|left| {
        candidate_variants
            .iter()
            .any(|right| left.eq_ignore_ascii_case(right))
    }) {
        return true;
    }

    let requested = requested_variants[0].as_str();
    let candidate = candidate_variants[0].as_str();
    if requested.strip_prefix('^').is_some_and(|index| {
        index.eq_ignore_ascii_case(candidate) && is_known_unprefixed_index_alias(candidate)
    }) {
        return true;
    }
    candidate.strip_prefix('^').is_some_and(|index| {
        index.eq_ignore_ascii_case(requested) && is_known_unprefixed_index_alias(requested)
    })
}

fn normalize_and_classify(value: &str) -> Option<(String, ProviderSymbolKind)> {
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
    // `$` is only a user-facing marker. Keep the classified provider dialect
    // so `$BRK.B`, `$BTC/USD`, and `$600519.SH` canonicalize exactly like the
    // same identifier without a cashtag.
    Some((normalized, classified))
}

fn classify_normalized(value: &str) -> Option<ProviderSymbolKind> {
    if let Some(index) = value.strip_prefix('^') {
        return (!index.is_empty()
            && index.len() <= 12
            && index
                .chars()
                .all(|character| character.is_ascii_alphanumeric())
            && index
                .chars()
                .any(|character| character.is_ascii_alphabetic()))
        .then_some(ProviderSymbolKind::Index);
    }

    if value.contains('/') {
        let (base, suffix) = split_once_exact(value, '/')?;
        if is_crypto_pair(base, suffix) {
            return Some(ProviderSymbolKind::CryptoPair);
        }
        return is_share_class(base, suffix).then_some(ProviderSymbolKind::ShareClass);
    }
    if value.contains('-') {
        let (base, suffix) = split_once_exact(value, '-')?;
        if is_crypto_pair(base, suffix) {
            return Some(ProviderSymbolKind::CryptoPair);
        }
        return is_share_class(base, suffix).then_some(ProviderSymbolKind::ShareClass);
    }
    if value.contains('.') {
        let (base, suffix) = split_once_exact(value, '.')?;
        if base.chars().all(|character| character.is_ascii_digit())
            && is_exchange_suffix_shape(suffix)
        {
            return Some(ProviderSymbolKind::ExchangeQualified);
        }
        if is_share_class(base, suffix) {
            return Some(ProviderSymbolKind::ShareClass);
        }
        if base
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
            && base
                .chars()
                .any(|character| character.is_ascii_alphabetic())
            && is_exchange_suffix_shape(suffix)
        {
            return Some(ProviderSymbolKind::ExchangeQualified);
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
    (has_letter || numeric_identifier).then_some(ProviderSymbolKind::Bare)
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

fn is_known_unprefixed_index_alias(value: &str) -> bool {
    matches!(value, "GSPC" | "IXIC" | "DJI" | "RUT" | "VIX")
}

fn provider_exchange_symbol(value: &str) -> String {
    let Some((base, suffix)) = value.rsplit_once('.') else {
        return value.to_string();
    };
    if suffix == "US" {
        return base.to_string();
    }
    let suffix = if suffix == "SH" { "SS" } else { suffix };
    let base = if suffix == "HK" && base.chars().all(|character| character.is_ascii_digit()) {
        canonical_hong_kong_base(base)
    } else {
        base.to_string()
    };
    format!("{base}.{suffix}")
}

fn canonical_hong_kong_base(value: &str) -> String {
    let significant = value.trim_start_matches('0');
    let significant = if significant.is_empty() {
        "0"
    } else {
        significant
    };
    if significant.len() < 4 {
        format!("{significant:0>4}")
    } else {
        significant.to_string()
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_equivalence_is_bounded_to_audited_dialects() {
        for (left, right) in [
            ("BRK/B", "BRK-B"),
            ("BRK.B", "BRK-B"),
            ("$BRK.B", "BRK-B"),
            ("600519.SH", "600519.SS"),
            ("$600519.SH", "600519.SS"),
            ("700.HK", "0700.HK"),
            ("09988.HK", "9988.HK"),
            ("GSPC", "^GSPC"),
            ("BTC/USD", "BTCUSD"),
            ("$BTC/USD", "BTCUSD"),
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
            ("ABC.DEF", "ABC-DEF"),
            ("600519.SH", "600519.SZ"),
            ("RKLB", "RKLX"),
            ("0700", "0700.HK"),
            ("BTCUSD", "ETHUSD"),
        ] {
            assert!(
                !provider_symbols_equivalent(left, right),
                "{left} == {right}"
            );
        }
    }
}
