//! Digest 精选逻辑:输入一批 Medium/Low 事件,按**多维度 cap + 话题去重 +
//! 时间窗口过滤**产出 kept/omitted 两个桶。
//!
//! 为什么独立一个文件:
//! - 这一层是 digest 的「噪声控制系统」,有大量 cap/阈值常量(每 symbol/source/
//!   domain/topic 的上限)和多个配套 helper(`digest_score` / `digest_topic_tokens` /
//!   `token_jaccard` / stopwords…),混在 `scheduler.rs` 的 tick 循环里会
//!   把已经足够复杂的 flush 流程搞得没法读。
//! - High severity 事件不走 curation(立刻推,不受 cap 约束),所以 caller
//!   要知道这是「Medium/Low 专用」,把它定在独立 module 里语义更直白。

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use crate::event::{EventKind, MarketEvent, Severity, is_noop_analyst_grade};

pub(super) const DIGEST_MAX_SOCIAL_ITEMS: usize = 3;
pub(super) const DIGEST_MAX_ITEMS_PER_SYMBOL: usize = 4;
pub(super) const DIGEST_MAX_ITEMS_PER_SOURCE: usize = 3;
pub(super) const DIGEST_MAX_ITEMS_PER_DOMAIN: usize = 2;
pub(super) const DIGEST_MACRO_LOOKAHEAD_HOURS: i64 = 48;

#[derive(Debug)]
pub(super) struct DigestCuration {
    pub(super) kept: Vec<MarketEvent>,
    pub(super) omitted: Vec<MarketEvent>,
}

impl DigestCuration {
    fn kept(events: Vec<MarketEvent>) -> Self {
        Self {
            kept: events,
            omitted: Vec::new(),
        }
    }
}

pub(crate) fn curate_digest_events_with_omitted_at(
    events: Vec<MarketEvent>,
    now: DateTime<Utc>,
) -> DigestCuration {
    let mut kept = Vec::with_capacity(events.len());
    let mut omitted = Vec::new();
    let mut social_count = 0usize;
    let mut by_symbol: HashMap<String, usize> = HashMap::new();
    let mut by_source: HashMap<String, usize> = HashMap::new();
    let mut by_domain: HashMap<String, usize> = HashMap::new();
    let mut title_keys: HashSet<String> = HashSet::new();
    let mut topic_tokens: Vec<(String, HashSet<String>)> = Vec::new();

    for event in events {
        let is_high = event.severity.rank() >= crate::event::Severity::High.rank();
        if !is_high {
            if should_omit_from_digest(&event, now) {
                omitted.push(event);
                continue;
            }
            if matches!(event.kind, EventKind::SocialPost) {
                if social_count >= DIGEST_MAX_SOCIAL_ITEMS {
                    omitted.push(event);
                    continue;
                }
            }
            if let Some(symbol) = primary_symbol_key(&event) {
                if by_symbol.get(&symbol).copied().unwrap_or(0) >= DIGEST_MAX_ITEMS_PER_SYMBOL {
                    omitted.push(event);
                    continue;
                }
            }
            if !event.source.is_empty()
                && by_source.get(&event.source).copied().unwrap_or(0) >= DIGEST_MAX_ITEMS_PER_SOURCE
            {
                omitted.push(event);
                continue;
            }
            if let Some(domain) = event_domain_key(&event) {
                if by_domain.get(&domain).copied().unwrap_or(0) >= DIGEST_MAX_ITEMS_PER_DOMAIN {
                    omitted.push(event);
                    continue;
                }
            }
            if let Some(title_key) = digest_title_dedupe_key(&event) {
                if !title_keys.insert(title_key) {
                    omitted.push(event);
                    continue;
                }
            }
            if let Some((topic_key, tokens)) = digest_topic_tokens(&event) {
                if topic_tokens
                    .iter()
                    .any(|(key, seen)| key == &topic_key && token_jaccard(seen, &tokens) >= 0.55)
                {
                    omitted.push(event);
                    continue;
                }
                topic_tokens.push((topic_key, tokens));
            }
        }

        if matches!(event.kind, EventKind::SocialPost) {
            social_count += 1;
        }
        if let Some(symbol) = primary_symbol_key(&event) {
            *by_symbol.entry(symbol).or_default() += 1;
        }
        if !event.source.is_empty() {
            *by_source.entry(event.source.clone()).or_default() += 1;
        }
        if let Some(domain) = event_domain_key(&event) {
            *by_domain.entry(domain).or_default() += 1;
        }
        kept.push(event);
    }

    DigestCuration { kept, omitted }
}

pub(crate) fn suppress_recent_digest_topics_with_omitted(
    actor_key: &str,
    events: Vec<MarketEvent>,
    store: &crate::store::EventStore,
    now: DateTime<Utc>,
) -> DigestCuration {
    let since = now - chrono::Duration::hours(24);
    let Ok(recent) = store.list_recent_digest_item_events(actor_key, since) else {
        return DigestCuration::kept(events);
    };
    let recent_topics: Vec<(String, HashSet<String>)> =
        recent.iter().filter_map(digest_topic_tokens).collect();
    if recent_topics.is_empty() {
        return DigestCuration::kept(events);
    }

    let mut kept = Vec::with_capacity(events.len());
    let mut omitted = Vec::new();
    for event in events {
        if event.severity == Severity::High {
            kept.push(event);
            continue;
        }
        let Some((topic_key, tokens)) = digest_topic_tokens(&event) else {
            kept.push(event);
            continue;
        };
        let duplicate = recent_topics
            .iter()
            .any(|(key, seen)| key == &topic_key && token_jaccard(seen, &tokens) >= 0.55);
        if duplicate {
            tracing::info!(
                actor = %actor_key,
                event_id = %event.id,
                topic = %topic_key,
                "digest topic suppressed by recent memory"
            );
            omitted.push(event);
        } else {
            kept.push(event);
        }
    }
    DigestCuration { kept, omitted }
}

pub(crate) fn digest_score(event: &MarketEvent) -> i32 {
    let mut score = match event.severity {
        Severity::High => 300,
        Severity::Medium => 200,
        Severity::Low => 100,
    };
    score += match event.kind {
        EventKind::EarningsReleased | EventKind::SecFiling { .. } => 50,
        EventKind::EarningsCallTranscript => 15,
        EventKind::PriceAlert { ref window, .. } if window != "close" => 35,
        EventKind::Dividend | EventKind::Split | EventKind::Buyback => 30,
        EventKind::MacroEvent => 20,
        EventKind::NewsCritical => 10,
        EventKind::SocialPost => -35,
        _ => 0,
    };
    if matches!(
        event.payload.get("source_class").and_then(|v| v.as_str()),
        Some("trusted")
    ) {
        score += 20;
    }
    if matches!(
        event.payload.get("source_class").and_then(|v| v.as_str()),
        Some("pr_wire" | "opinion_blog")
    ) {
        score -= 35;
    }
    if is_low_quality_social_source(event) {
        score -= 30;
    }
    if matches!(event.kind, EventKind::EarningsUpcoming) {
        let days_until = (event.occurred_at.date_naive() - Utc::now().date_naive()).num_days();
        if days_until > 7 {
            score -= 40;
        } else if days_until <= 3 {
            score += 25;
        }
    }
    score
}

fn should_omit_from_digest(event: &MarketEvent, now: DateTime<Utc>) -> bool {
    if event.severity == Severity::High {
        return false;
    }
    if is_noop_analyst_grade(event) {
        return true;
    }
    match event.kind {
        EventKind::MacroEvent => {
            event.severity == Severity::Low
                || event.occurred_at > now + chrono::Duration::hours(DIGEST_MACRO_LOOKAHEAD_HOURS)
        }
        EventKind::NewsCritical => {
            event.severity == Severity::Low
                || matches!(
                    event.payload.get("source_class").and_then(|v| v.as_str()),
                    Some("opinion_blog" | "pr_wire")
                )
        }
        EventKind::SocialPost => {
            event.severity == Severity::Low && is_low_quality_social_source(event)
        }
        _ => false,
    }
}

fn primary_symbol_key(event: &MarketEvent) -> Option<String> {
    event
        .symbols
        .iter()
        .find(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_ascii_uppercase())
}

fn event_domain_key(event: &MarketEvent) -> Option<String> {
    if let Some(url) = event.url.as_deref() {
        let without_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        let host = without_scheme.split('/').next().unwrap_or_default();
        let host = host.strip_prefix("www.").unwrap_or(host).trim();
        if !host.is_empty() {
            return Some(host.to_ascii_lowercase());
        }
    }
    event
        .source
        .split_once(':')
        .map(|(_, domain)| domain.trim().to_ascii_lowercase())
        .filter(|domain| !domain.is_empty())
}

fn digest_title_dedupe_key(event: &MarketEvent) -> Option<String> {
    if !matches!(
        event.kind,
        EventKind::NewsCritical | EventKind::PressRelease | EventKind::SocialPost
    ) {
        return None;
    }
    let title = super::render::digest_event_title(event);
    let normalized: Vec<String> = title
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|token| {
            let token = token.trim().to_ascii_lowercase();
            (token.len() > 2).then_some(token)
        })
        .take(10)
        .collect();
    if normalized.is_empty() {
        return None;
    }
    let symbol = primary_symbol_key(event).unwrap_or_else(|| "-".into());
    Some(format!("{symbol}:{}", normalized.join(" ")))
}

fn digest_topic_tokens(event: &MarketEvent) -> Option<(String, HashSet<String>)> {
    // 把 MacroEvent 也纳入话题去重:加拿大零售销售(`Retail Sales MoM` 与
    // `Retail Sales MoM (Mar)` 与 `Retail Sales Ex Autos MoM (Feb)`)三条
    // 标题 jaccard 相似度都 ≥ 0.55,以前不去重会让 digest 顶端被同主题
    // 宏观噪音占满。Earnings/PriceAlert 等事件依然不进话题去重——它们的
    // 标题模式化太强(`AAPL earnings tomorrow`),容易误判成同主题。
    if !matches!(
        event.kind,
        EventKind::NewsCritical
            | EventKind::PressRelease
            | EventKind::SocialPost
            | EventKind::MacroEvent
    ) {
        return None;
    }
    let tokens: HashSet<String> = super::render::digest_event_title(event)
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter_map(|token| {
            let token = token.trim().to_ascii_lowercase();
            if token.len() <= 2 || DIGEST_STOPWORDS.contains(&token.as_str()) {
                None
            } else {
                Some(token)
            }
        })
        .collect();
    if tokens.len() < 3 {
        return None;
    }
    let symbol = primary_symbol_key(event).unwrap_or_else(|| "-".into());
    Some((format!("{symbol}:{}", kind_topic_tag(&event.kind)), tokens))
}

fn token_jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn kind_topic_tag(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::SocialPost => "social",
        EventKind::PressRelease => "press",
        EventKind::MacroEvent => "macro",
        _ => "news",
    }
}

fn is_low_quality_social_source(event: &MarketEvent) -> bool {
    let source = event.source.to_ascii_lowercase();
    matches!(event.kind, EventKind::SocialPost) && source.contains("watcherguru")
}

const DIGEST_STOPWORDS: &[&str] = &[
    "the",
    "and",
    "for",
    "with",
    "from",
    "that",
    "this",
    "after",
    "before",
    "into",
    "over",
    "under",
    "says",
    "said",
    "stock",
    "stocks",
    "shares",
    "share",
    "inc",
    "corp",
    "ltd",
    "company",
    "announces",
    "announced",
    "update",
    "market",
];
