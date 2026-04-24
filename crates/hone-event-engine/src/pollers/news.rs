//! NewsPoller — 拉取 FMP `v3/stock_news`，产出 `NewsCritical` 事件。
//!
//! MVP 行为：
//! - `poll()` 从 FMP 拉一页最新新闻（可选 ticker 过滤，`None` 表示全局流）
//! - 默认 severity = Low；title/text 命中关键词库 → 升级为 High
//! - id 直接用文章 URL 做稳定去重；缺 URL 则回落到 "title+date" 组合
//! - 关键词库先内置一组保守的"高影响"词（破产、SEC 调查、召回、被起诉、CEO 辞任、收购等）

use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;

/// 标题反模板:律所/股东集体诉讼广告类的固定话术。命中即强制 Severity::Low,
/// 不再走关键词升级。覆盖 globenewswire 等 PR wire 发布的常见 SHAREHOLDER ALERT
/// / Bernstein Liebhard / Rosen Law / Schall Law 等模板。
const LEGAL_AD_TITLE_PATTERNS: &[&str] = &[
    "shareholder alert",
    "investor alert",
    "investor deadline",
    "deadline alert",
    "class action lawsuit has been filed",
    "securities fraud class action",
    "law firm",
    "law offices",
    "bernstein liebhard",
    "rosen law",
    "schall law",
    "pomerantz law",
    "bragar eagel",
    "kessler topaz",
    "robbins geller",
    "investors to act",
    "lost money",
    "investors who lost",
];

/// PR wire / press release 聚合域。这些域几乎只发布商业 PR/律所广告,
/// 关键词命中也不应直接升 High——给路由层判断或保持 Low。
const PR_WIRE_DOMAINS: &[&str] = &[
    "globenewswire.com",
    "prnewswire.com",
    "businesswire.com",
    "accesswire.com",
    "newsfile.io",
    "newsfilecorp.com",
    "einnews.com",
    "newswire.ca",
    "marketwired.com",
    "issuewire.com",
];

/// Opinion blog / financial content farm。这些域产 list / "5 stocks to buy" /
/// 估值评论 / YouTube 转录这类弱信号内容。即使关键字命中也保持 Low,且不进
/// LLM 仲裁——避免 LLM 把"5 AI Cloud Stocks That Will Make Investors a Fortune"
/// 这种 promo 文章误判为 important。
///
/// 设计原则:**真正的重大事件(CEO 退任/收购/破产)同时会被 trusted 媒体
/// (reuters/bloomberg/wsj/cnbc 等)报道**,所以即便把这些 opinion 域全部
/// 降级,actor 也不会漏掉重大新闻——只是少收一份重复包装。
const OPINION_BLOG_DOMAINS: &[&str] = &[
    "seekingalpha.com",
    "fool.com",
    "zacks.com",
    "forbes.com",
    "247wallst.com",
    "etftrends.com",
    "gurufocus.com",
    "benzinga.com",
    "investorplace.com",
    "thestreet.com",
    "youtube.com",
    "youtu.be",
    "investopedia.com",
    "fastcompany.com",
    "cnet.com",
    "tomsguide.com",
    "tomshardware.com",
    "macrumors.com",
    "9to5mac.com",
    "appleinsider.com",
    // 13F / 机构持仓变化博客 — 内容几乎只是"X Bank purchased N shares of Y",
    // 无公司基本面信号
    "defenseworld.net",
    // 交易策略 / 期权博客
    "schaeffersresearch.com",
    // 财经 promo / 大盘观点聚合 — 偶有研报转发但 90%+ 噪音
    "finbold.com",
    "proactiveinvestors.com",
    "invezz.com",
    // 注:marketbeat.com 故意不加 — 该域偶有真合同/研报新闻
    // (例:Broadcom-Meta AI Pact 2029),整域降级会丢真信号
];

/// 高可信主流媒体 / 权威发布源。只有这些源被允许通过纯关键词升 High。
const TRUSTED_NEWS_DOMAINS: &[&str] = &[
    "reuters.com",
    "bloomberg.com",
    "wsj.com",
    "ft.com",
    "ap.org",
    "apnews.com",
    "cnbc.com",
    "nytimes.com",
    "marketwatch.com",
    "barrons.com",
    "sec.gov",
    "fda.gov",
    "treasury.gov",
    "federalreserve.gov",
];

/// 来源信誉分类。
/// - `Trusted` = 主流媒体/SEC 等,关键字命中可升 High
/// - `PrWire` = PR 聚合,直接降级 Low,且不走 LLM 仲裁
/// - `OpinionBlog` = 财经博客 / list 文章 / YouTube,直接 Low,不走 LLM
/// - `Uncertain` = 其它,需 router 层 LLM 进一步判断
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewsSourceClass {
    Trusted,
    PrWire,
    OpinionBlog,
    Uncertain,
}

impl NewsSourceClass {
    pub fn as_str(self) -> &'static str {
        match self {
            NewsSourceClass::Trusted => "trusted",
            NewsSourceClass::PrWire => "pr_wire",
            NewsSourceClass::OpinionBlog => "opinion_blog",
            NewsSourceClass::Uncertain => "uncertain",
        }
    }
}

/// 简单子串匹配——`site` 字段从 FMP 直接拿。空 site → Uncertain。
pub fn classify_news_source(site: &str) -> NewsSourceClass {
    let s = site.trim().to_lowercase();
    if s.is_empty() {
        return NewsSourceClass::Uncertain;
    }
    if PR_WIRE_DOMAINS.iter().any(|d| s.contains(d)) {
        return NewsSourceClass::PrWire;
    }
    if TRUSTED_NEWS_DOMAINS.iter().any(|d| s.contains(d)) {
        return NewsSourceClass::Trusted;
    }
    if OPINION_BLOG_DOMAINS.iter().any(|d| s.contains(d)) {
        return NewsSourceClass::OpinionBlog;
    }
    NewsSourceClass::Uncertain
}

/// 标题是否命中律所广告模板。命中即强制 Low。
pub fn is_legal_ad_title(title: &str) -> bool {
    let lower = title.to_lowercase();
    LEGAL_AD_TITLE_PATTERNS.iter().any(|p| lower.contains(p))
}

/// Earnings call transcript 是稳定、可订阅的财报材料,但不应被当作
/// NewsCritical 送 LLM 仲裁。独立成 kind 后用户可以单独 allow/block。
pub fn is_earnings_call_transcript_title(title: &str) -> bool {
    let lower = title.to_lowercase();
    lower.contains("earnings call transcript") || lower.contains("earnings transcript")
}

/// 默认高影响关键词（小写匹配）。后续可从 config 注入覆盖。
const DEFAULT_CRITICAL_KEYWORDS: &[&str] = &[
    "bankruptcy",
    "bankrupt",
    "delist",
    "halt trading",
    "trading halted",
    "sec investigation",
    "sec probe",
    "sec charges",
    "sec settles",
    "recall",
    "fraud",
    "lawsuit",
    "class action",
    "short report",
    "short-seller",
    "hindenburg",
    "muddy waters",
    "guidance cut",
    "cuts guidance",
    "lowers guidance",
    "ceo resigns",
    "ceo steps down",
    "cfo resigns",
    "cfo steps down",
    "acquired by",
    "agrees to acquire",
    "merger",
    "buyout",
    "going private",
    "data breach",
    "cyberattack",
];

pub struct NewsPoller {
    client: FmpClient,
    tickers: Option<Vec<String>>,
    page_limit: u32,
    keywords: Vec<String>,
}

impl NewsPoller {
    pub fn new(client: FmpClient) -> Self {
        Self {
            client,
            tickers: None,
            page_limit: 50,
            keywords: DEFAULT_CRITICAL_KEYWORDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn with_tickers(mut self, tickers: Vec<String>) -> Self {
        self.tickers = if tickers.is_empty() {
            None
        } else {
            Some(tickers)
        };
        self
    }

    pub fn with_page_limit(mut self, limit: u32) -> Self {
        self.page_limit = limit;
        self
    }

    pub fn with_keywords(mut self, kws: Vec<String>) -> Self {
        if !kws.is_empty() {
            self.keywords = kws.into_iter().map(|s| s.to_lowercase()).collect();
        }
        self
    }

    pub async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let mut path = format!("/v3/stock_news?limit={}", self.page_limit);
        if let Some(ts) = &self.tickers {
            path.push_str("&tickers=");
            path.push_str(&ts.join(","));
        }
        let raw = self.client.get_json(&path).await?;
        Ok(events_from_stock_news(&raw, &self.keywords))
    }
}

/// FMP stock_news 响应 → MarketEvent 列表。
fn events_from_stock_news(raw: &Value, keywords: &[String]) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };

    arr.iter()
        .filter_map(|item| {
            let title = item.get("title")?.as_str()?.to_string();
            let published_raw = item.get("publishedDate")?.as_str()?.to_string();
            let occurred_at = parse_fmp_datetime(&published_raw).unwrap_or_else(Utc::now);

            let symbol = item
                .get("symbol")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let url = item.get("url").and_then(|v| v.as_str()).map(String::from);
            let text = item
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let site = item
                .get("site")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let source_class = classify_news_source(&site);
            let is_transcript = is_earnings_call_transcript_title(&title);
            let severity = if is_transcript {
                Severity::Low
            } else {
                classify_severity(&title, &text, keywords, source_class)
            };
            let id_prefix = if is_transcript {
                "earnings_call_transcript"
            } else {
                "news"
            };
            let id = match &url {
                Some(u) => format!("{id_prefix}:{u}"),
                None => format!("{id_prefix}:{published_raw}:{}", truncate(&title, 64)),
            };
            let symbols = symbol.map(|s| vec![s]).unwrap_or_default();
            let summary_snippet = truncate(&text, 240);

            // 把 source_class 与是否命中律所模板写进 payload,供 router-stage
            // LLM 仲裁器读出后做 per-actor 重要性判断。原始 FMP item 整体保留在
            // `payload.fmp` 下,避免下游消费方破坏现有字段路径。
            let mut enriched = serde_json::Map::new();
            enriched.insert(
                "source_class".into(),
                Value::String(source_class.as_str().into()),
            );
            enriched.insert(
                "legal_ad_template".into(),
                Value::Bool(is_legal_ad_title(&title)),
            );
            enriched.insert(
                "earnings_call_transcript".into(),
                Value::Bool(is_transcript),
            );
            enriched.insert("fmp".into(), item.clone());

            Some(MarketEvent {
                id,
                kind: if is_transcript {
                    EventKind::EarningsCallTranscript
                } else {
                    EventKind::NewsCritical
                },
                severity,
                symbols,
                occurred_at,
                title,
                summary: summary_snippet,
                url,
                source: if site.is_empty() {
                    "fmp.stock_news".into()
                } else {
                    format!("fmp.stock_news:{site}")
                },
                payload: Value::Object(enriched),
            })
        })
        .collect()
}

fn parse_fmp_datetime(s: &str) -> Option<chrono::DateTime<Utc>> {
    // FMP 格式如 "2026-04-20 14:30:00"（UTC）
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&ndt));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Some(Utc.from_utc_datetime(&ndt));
    }
    None
}

fn classify_severity(
    title: &str,
    text: &str,
    keywords: &[String],
    source_class: NewsSourceClass,
) -> Severity {
    // 律所模板标题——典型 SHAREHOLDER ALERT / class action lawsuit has been filed,
    // 不论关键词命中,直接强制 Low,断绝 PR wire 的"假高"链路。
    if is_legal_ad_title(title) {
        return Severity::Low;
    }
    // PR wire / opinion blog 整域降级:几乎只发广告 PR / list / 估值评论,
    // 关键词命中也维持 Low,避免占用 immediate sink + daily high cap,
    // 也避免 LLM 仲裁阶段对低质内容做无用功。
    if matches!(
        source_class,
        NewsSourceClass::PrWire | NewsSourceClass::OpinionBlog
    ) {
        return Severity::Low;
    }
    let t = title.to_lowercase();
    let body = text.to_lowercase();
    let matched = keywords
        .iter()
        .any(|kw| t.contains(kw) || body.contains(kw));
    if !matched {
        return Severity::Low;
    }
    // 主流媒体 / SEC 等可信源:关键词命中即可走 High。
    if matches!(source_class, NewsSourceClass::Trusted) {
        return Severity::High;
    }
    // 不确定来源:即使关键词命中也只升到 Low,等路由层 LLM 仲裁器按用户的
    // importance prompt 决定是否升级到 Medium。
    Severity::Low
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_kws() -> Vec<String> {
        DEFAULT_CRITICAL_KEYWORDS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn parses_typical_stock_news() {
        let raw = serde_json::json!([
            {
                "symbol": "AAPL",
                "publishedDate": "2026-04-21 08:15:00",
                "title": "Apple beats estimates on services strength",
                "image": "",
                "site": "reuters.com",
                "text": "Apple Inc reported Q2 results above expectations driven by services growth ...",
                "url": "https://example.com/apple-beats"
            },
            {
                "symbol": "TSLA",
                "publishedDate": "2026-04-21 09:00:00",
                "title": "Tesla faces SEC investigation over disclosures",
                "site": "wsj.com",
                "text": "The SEC has opened a probe into Tesla's ...",
                "url": "https://example.com/tsla-sec"
            }
        ]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].severity, Severity::Low);
        assert_eq!(events[1].severity, Severity::High);
        assert!(events[1].title.to_lowercase().contains("sec"));
        assert_eq!(events[0].id, "news:https://example.com/apple-beats");
        assert!(events[0].touches("AAPL"));
        // payload 应该携带 source_class 与 fmp 原 item
        assert_eq!(
            events[1]
                .payload
                .get("source_class")
                .and_then(|v| v.as_str()),
            Some("trusted")
        );
        assert!(events[1].payload.get("fmp").is_some());
    }

    #[test]
    fn legal_ad_template_forced_to_low_even_on_pr_wire_with_keywords() {
        let raw = serde_json::json!([{
            "symbol": "SNOW",
            "publishedDate": "2026-04-21 08:21:00",
            "title": "SHAREHOLDER ALERT Bernstein Liebhard LLP Announces A Securities Fraud Class Action Lawsuit Has Been Filed Against Snowflake Inc. (SNOW)",
            "site": "globenewswire.com",
            "text": "fraud lawsuit class action ...",
            "url": "https://www.globenewswire.com/news-release/x"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, Severity::Low);
        assert_eq!(
            events[0]
                .payload
                .get("source_class")
                .and_then(|v| v.as_str()),
            Some("pr_wire")
        );
        assert_eq!(
            events[0]
                .payload
                .get("legal_ad_template")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn pr_wire_source_caps_severity_at_low() {
        // 即便没有律所模板,PR wire 的 fraud / lawsuit 关键词也不应升 High。
        let raw = serde_json::json!([{
            "symbol": "ACME",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "ACME Announces New Product",
            "site": "prnewswire.com",
            "text": "There has been some lawsuit chatter recently, but ACME ...",
            "url": "https://www.prnewswire.com/x"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events[0].severity, Severity::Low);
    }

    #[test]
    fn opinion_blog_source_caps_severity_at_low_and_marked() {
        // fool.com / zacks.com 等 opinion 域:即使关键字命中也维持 Low,
        // payload 标 source_class=opinion_blog 让 router 跳过 LLM 仲裁。
        let raw = serde_json::json!([{
            "symbol": "AAPL",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Is Apple a Buy After Recent Acquired By Rumors?",
            "site": "fool.com",
            "text": "Some analysts speculate Apple may be acquired by ...",
            "url": "https://fool.com/x"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events[0].severity, Severity::Low);
        assert_eq!(
            events[0]
                .payload
                .get("source_class")
                .and_then(|v| v.as_str()),
            Some("opinion_blog")
        );
    }

    #[test]
    fn defenseworld_13f_classified_as_opinion_blog() {
        let raw = serde_json::json!([{
            "symbol": "TSLA",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Busey Bank Has $8.13 Million Stock Holdings in Tesla",
            "site": "defenseworld.net",
            "text": "Busey Bank reported holdings ...",
            "url": "https://defenseworld.net/x"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events[0].severity, Severity::Low);
        assert_eq!(
            events[0]
                .payload
                .get("source_class")
                .and_then(|v| v.as_str()),
            Some("opinion_blog")
        );
    }

    #[test]
    fn market_commentary_sites_are_classified_as_opinion_blog() {
        for site in [
            "seekingalpha.com",
            "forbes.com",
            "proactiveinvestors.com",
            "invezz.com",
        ] {
            assert_eq!(
                classify_news_source(site),
                NewsSourceClass::OpinionBlog,
                "{site} should skip LLM arbitration and stay low by default"
            );
        }
    }

    #[test]
    fn earnings_call_transcript_gets_dedicated_kind() {
        let raw = serde_json::json!([{
            "symbol": "GEV",
            "publishedDate": "2026-04-22 21:30:00",
            "title": "GE Vernova Inc. (GEV) Q1 2026 Earnings Call Transcript",
            "site": "seekingalpha.com",
            "text": "Prepared remarks and Q&A transcript ...",
            "url": "https://seekingalpha.com/article/gev-transcript"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::EarningsCallTranscript);
        assert_eq!(events[0].severity, Severity::Low);
        assert!(events[0].id.starts_with("earnings_call_transcript:"));
        assert_eq!(
            events[0]
                .payload
                .get("earnings_call_transcript")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn live_news_classifier_baseline_source_policy_is_stable() {
        let fixture = include_str!(
            "../../../../tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json"
        );
        let fixture: Value = serde_json::from_str(fixture).expect("fixture json");
        let items = fixture
            .get("items")
            .and_then(|v| v.as_array())
            .expect("items array");
        assert_eq!(items.len(), 43);

        let mut uncertain_llm_items = 0usize;
        for item in items {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let site = item.get("site").and_then(|v| v.as_str()).unwrap_or("");
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let expected_source = item
                .get("expected_source_class_after")
                .and_then(|v| v.as_str())
                .unwrap();
            let expected_kind = item
                .get("expected_kind_after")
                .and_then(|v| v.as_str())
                .unwrap();
            let expected_llm = item
                .get("expected_llm_after_engine")
                .and_then(|v| v.as_str());

            assert_eq!(
                classify_news_source(site).as_str(),
                expected_source,
                "{id}: source class drift for {site} / {title}"
            );
            assert_eq!(
                if is_earnings_call_transcript_title(title) {
                    "earnings_call_transcript"
                } else {
                    "news_critical"
                },
                expected_kind,
                "{id}: kind split drift for {title}"
            );

            if expected_llm.is_some() {
                uncertain_llm_items += 1;
                assert_eq!(
                    expected_source, "uncertain",
                    "{id}: only uncertain-source news should need LLM baseline reruns"
                );
                assert_eq!(
                    expected_kind, "news_critical",
                    "{id}: standalone events should not enter news LLM arbitration"
                );
            }
        }
        assert_eq!(uncertain_llm_items, 15);
    }

    #[test]
    fn marketbeat_kept_as_uncertain_to_preserve_real_signals() {
        // marketbeat 有真合同/研报新闻(如 Broadcom-Meta AI Pact),不能整域降级。
        let raw = serde_json::json!([{
            "symbol": "META",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Broadcom & Meta Extend AI Pact Into 2029",
            "site": "marketbeat.com",
            "text": "...",
            "url": "https://marketbeat.com/x"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(
            events[0]
                .payload
                .get("source_class")
                .and_then(|v| v.as_str()),
            Some("uncertain"),
            "marketbeat 应保持 uncertain 进入 LLM 仲裁,不能被 OpinionBlog 拦下"
        );
    }

    #[test]
    fn youtube_classified_as_opinion_blog() {
        let raw = serde_json::json!([{
            "symbol": "TSLA",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Tesla Stock Crashes Tomorrow!! (Don't Miss This)",
            "site": "youtube.com",
            "text": "...",
            "url": "https://youtube.com/watch?v=x"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events[0].severity, Severity::Low);
        assert_eq!(
            events[0]
                .payload
                .get("source_class")
                .and_then(|v| v.as_str()),
            Some("opinion_blog")
        );
    }

    #[test]
    fn uncertain_source_keeps_low_even_when_keyword_matches() {
        // 不确定源 + 关键词命中:维持 Low,等路由层 LLM 仲裁。
        let raw = serde_json::json!([{
            "symbol": "ACME",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "ACME Faces Major Recall",
            "site": "smallblog.io",
            "text": "ACME announced a recall of its flagship product ...",
            "url": "https://smallblog.io/acme-recall"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events[0].severity, Severity::Low);
        assert_eq!(
            events[0]
                .payload
                .get("source_class")
                .and_then(|v| v.as_str()),
            Some("uncertain")
        );
    }

    #[test]
    fn missing_url_falls_back_to_date_title_id() {
        let raw = serde_json::json!([{
            "symbol": "NVDA",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Nvidia announces chip partnership"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 1);
        assert!(events[0].id.starts_with("news:2026-04-21"));
        assert!(events[0].url.is_none());
    }

    #[test]
    fn keyword_match_is_case_insensitive_and_body_searched() {
        // 站点是 reuters.com (Trusted) → 关键词命中即可升 High。
        let raw = serde_json::json!([{
            "symbol": "ACME",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Acme hits new record",
            "site": "reuters.com",
            "text": "Despite the Hindenburg short report published today, Acme ..."
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events[0].severity, Severity::High);
    }

    #[test]
    fn custom_keywords_override_default() {
        let raw = serde_json::json!([{
            "symbol": "X",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Boring quarterly update",
            "site": "reuters.com",
            "text": "Nothing special happened."
        }]);
        // 默认关键词：Low
        let low = events_from_stock_news(&raw, &default_kws());
        assert_eq!(low[0].severity, Severity::Low);
        // 注入新词匹配 "boring"。reuters.com 是 trusted 源，关键词命中即升 High。
        let high = events_from_stock_news(&raw, &vec!["boring".into()]);
        assert_eq!(high[0].severity, Severity::High);
    }

    #[test]
    fn skips_entries_missing_title_or_published_date() {
        let raw = serde_json::json!([
            {"symbol": "X", "publishedDate": "2026-04-21 10:00:00"}, // 缺 title
            {"symbol": "Y", "title": "no date"},                      // 缺 publishedDate
            {"symbol": "Z", "publishedDate": "2026-04-21 10:00:00", "title": "ok"}
        ]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "ok");
    }

    /// 真实 FMP 烟测；默认忽略。
    /// `HONE_FMP_API_KEY=xxx cargo test -p hone-event-engine -- --ignored live_fmp_news_smoke --nocapture`
    #[tokio::test]
    #[ignore]
    async fn live_fmp_news_smoke() {
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&cfg);
        let poller = NewsPoller::new(client).with_page_limit(5);
        let events = poller.poll().await.expect("FMP poll failed");
        println!("news events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.id);
        }
        assert!(!events.is_empty());
    }
}
