//! Shared macro-indicator vocabulary and source-span scanner.
//!
//! A match is context for confidence only. Callers must never use this module
//! to erase a security-shaped candidate because several indicator acronyms are
//! also real or plausible listing symbols.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroIndicator {
    pub canonical: &'static str,
    pub display: &'static str,
    pub agency: &'static str,
    pub aliases: &'static [&'static str],
    /// Whether an alias is also a real or plausibly colliding listing symbol.
    pub collides_with_listing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroMention {
    pub start: usize,
    pub end: usize,
    pub canonical: &'static str,
    pub collides_with_listing: bool,
}

pub const MACRO_INDICATORS: &[MacroIndicator] = &[
    MacroIndicator {
        canonical: "nonfarm_payrolls",
        display: "美国非农就业人数",
        agency: "bls.gov",
        aliases: &[
            "非农",
            "大非农",
            "nonfarm",
            "non-farm",
            "nonfarm payrolls",
            "NFP",
        ],
        collides_with_listing: false,
    },
    MacroIndicator {
        canonical: "adp_employment",
        display: "ADP 就业报告",
        agency: "adpemploymentreport.com",
        aliases: &["小非农", "ADP", "ADP就业", "ADP employment"],
        collides_with_listing: true,
    },
    MacroIndicator {
        canonical: "core_pce",
        display: "核心PCE物价指数",
        agency: "bea.gov",
        aliases: &[
            "核心PCE",
            "个人消费支出物价指数",
            "core PCE",
            "PCE",
            "personal consumption expenditures price index",
        ],
        collides_with_listing: true,
    },
    MacroIndicator {
        canonical: "cpi",
        display: "消费者物价指数",
        agency: "bls.gov",
        aliases: &["CPI", "消费者物价指数", "consumer price index"],
        collides_with_listing: true,
    },
    MacroIndicator {
        canonical: "ppi",
        display: "生产者物价指数",
        agency: "bls.gov",
        aliases: &["PPI", "生产者物价指数", "producer price index"],
        collides_with_listing: true,
    },
    MacroIndicator {
        canonical: "fomc_rate_decision",
        display: "FOMC 利率决议",
        agency: "federalreserve.gov",
        aliases: &[
            "FOMC",
            "联邦公开市场委员会",
            "议息会议",
            "利率决议",
            "interest rate decision",
            "rate decision",
            "fed interest rate",
            "federal funds",
        ],
        collides_with_listing: false,
    },
    MacroIndicator {
        canonical: "gdp",
        display: "国内生产总值",
        agency: "bea.gov",
        aliases: &["GDP", "国内生产总值", "gross domestic product"],
        collides_with_listing: true,
    },
    MacroIndicator {
        canonical: "ism_surveys",
        display: "ISM 采购经理指数",
        agency: "ismworld.org",
        aliases: &[
            "ISM",
            "ISM制造业",
            "ISM服务业",
            "ISM manufacturing",
            "ISM services",
        ],
        collides_with_listing: true,
    },
    MacroIndicator {
        canonical: "initial_jobless_claims",
        display: "首次申领失业金人数",
        agency: "dol.gov",
        aliases: &["初请失业金", "jobless claims", "initial jobless claims"],
        collides_with_listing: false,
    },
    MacroIndicator {
        canonical: "retail_sales",
        display: "零售销售",
        agency: "census.gov",
        aliases: &["零售销售", "retail sales"],
        collides_with_listing: false,
    },
    MacroIndicator {
        canonical: "unemployment_rate",
        display: "失业率",
        agency: "bls.gov",
        aliases: &["失业率", "unemployment rate"],
        collides_with_listing: false,
    },
    MacroIndicator {
        canonical: "consumer_confidence",
        display: "消费者信心指数",
        agency: "conference-board.org",
        aliases: &["消费者信心", "消费者信心指数", "consumer confidence"],
        collides_with_listing: false,
    },
];

#[derive(Clone, Copy)]
struct AliasMatch {
    start: usize,
    end: usize,
    canonical: &'static str,
    collides_with_listing: bool,
}

/// Scan macro-indicator aliases in source order.
///
/// Fully ASCII aliases are case-insensitive and require ASCII word boundaries.
/// Aliases containing non-ASCII text use direct substring matching. When
/// aliases overlap, only the longest match survives.
pub fn scan(input: &str) -> Vec<MacroMention> {
    let ascii_lower = input.to_ascii_lowercase();
    let mut matches = Vec::new();

    for indicator in MACRO_INDICATORS {
        for alias in indicator.aliases {
            if alias.is_ascii() {
                let needle = alias.to_ascii_lowercase();
                for (start, _) in ascii_lower.match_indices(&needle) {
                    let end = start + alias.len();
                    if has_ascii_word_boundaries(input, start, end) {
                        matches.push(AliasMatch {
                            start,
                            end,
                            canonical: indicator.canonical,
                            collides_with_listing: indicator.collides_with_listing,
                        });
                    }
                }
            } else {
                for (start, _) in input.match_indices(alias) {
                    matches.push(AliasMatch {
                        start,
                        end: start + alias.len(),
                        canonical: indicator.canonical,
                        collides_with_listing: indicator.collides_with_listing,
                    });
                }
            }
        }
    }

    matches.sort_unstable_by(|left, right| {
        let left_len = left.end - left.start;
        let right_len = right.end - right.start;
        right_len
            .cmp(&left_len)
            .then_with(|| left.start.cmp(&right.start))
            .then_with(|| left.canonical.cmp(right.canonical))
    });

    let mut selected: Vec<AliasMatch> = Vec::new();
    for candidate in matches {
        if selected
            .iter()
            .any(|existing| candidate.start < existing.end && existing.start < candidate.end)
        {
            continue;
        }
        selected.push(candidate);
    }
    selected.sort_unstable_by_key(|mention| (mention.start, mention.end));
    selected
        .into_iter()
        .map(|mention| MacroMention {
            start: mention.start,
            end: mention.end,
            canonical: mention.canonical,
            collides_with_listing: mention.collides_with_listing,
        })
        .collect()
}

fn has_ascii_word_boundaries(input: &str, start: usize, end: usize) -> bool {
    let before_is_word = input[..start]
        .chars()
        .next_back()
        .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
    let after_is_word = input[end..]
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_');
    !before_is_word && !after_is_word
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_indicator_dictionary_scanner_honors_alias_boundaries_and_longest_overlap() {
        let input = "PCEX core pce 与小非农后关注 adp employment，再看初请失业金";
        let mentions = scan(input);
        assert_eq!(
            mentions
                .iter()
                .map(|mention| (&input[mention.start..mention.end], mention.canonical))
                .collect::<Vec<_>>(),
            [
                ("core pce", "core_pce"),
                ("小非农", "adp_employment"),
                ("adp employment", "adp_employment"),
                ("初请失业金", "initial_jobless_claims"),
            ]
        );
        assert!(mentions[1].collides_with_listing);
        assert!(!mentions[3].collides_with_listing);
        assert!(scan("PCEX").is_empty(), "PCE needs an ASCII word boundary");
        assert!(
            MACRO_INDICATORS
                .iter()
                .find(|indicator| indicator.canonical == "adp_employment")
                .is_some_and(|indicator| indicator.collides_with_listing)
        );
        for non_listing in ["nonfarm_payrolls", "fomc_rate_decision"] {
            assert!(
                MACRO_INDICATORS
                    .iter()
                    .find(|indicator| indicator.canonical == non_listing)
                    .is_some_and(|indicator| !indicator.collides_with_listing),
                "{non_listing} is explicitly non-colliding"
            );
        }
    }
}
