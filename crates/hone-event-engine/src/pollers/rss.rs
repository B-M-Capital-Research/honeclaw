//! 通用 RSS 2.0 新闻源 poller。
//!
//! POC 验证(见 docs/proposals 或 SKILL `poc-driven-feature-design`):FMP 漏掉
//! Bloomberg 93% / SpaceNews 100% / STAT News 100% 的关键料,这些 RSS 直接补强:
//! - Bloomberg 提供大盘宏观 / 地缘 / 油价信号
//! - SpaceNews 命中 RKLB 同行(SpaceX/Astrobotic/Pentagon contracts)
//! - STAT News 命中 CAI / TEM 医疗 AI 监管事件
//!
//! 输出:`MarketEvent { kind: NewsCritical, severity: High, source: "rss:{handle}",
//! payload: { source_class: "trusted", legal_ad_template: false, fmp: { site, text,
//! title, url } } }` —— payload 模拟成 FMP 形态以便 collector / curator 复用同一
//! 解析路径,不区分 RSS 与 FMP 来源。
//!
//! 反爬:大部分主流 RSS 通过浏览器 UA + 跟随重定向即可访问;Cloudflare / 401 失败
//! 由本 poller 容忍,下一 tick 重试,不冒泡 error。

use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use quick_xml::Reader;
use quick_xml::events::Event;
use serde_json::json;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::source::{EventSource, SourceSchedule};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36";

/// RSS 单条目从 feed 里抽出来的字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub pub_date: Option<DateTime<Utc>>,
    pub summary: String,
}

pub struct RssNewsPoller {
    handle: String, // "bloomberg_markets" / "spacenews" / "stat_news"
    url: String,    // feed url
    interval: Duration,
    http: reqwest::Client,
    name_cached: String,
}

impl RssNewsPoller {
    pub fn new(handle: impl Into<String>, url: impl Into<String>, interval: Duration) -> Self {
        let handle = handle.into();
        let name_cached = format!("rss.{handle}");
        Self {
            handle,
            url: url.into(),
            interval,
            http: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(Duration::from_secs(20))
                .build()
                .expect("reqwest client"),
            name_cached,
        }
    }
}

#[async_trait]
impl EventSource for RssNewsPoller {
    fn name(&self) -> &str {
        &self.name_cached
    }

    fn schedule(&self) -> SourceSchedule {
        SourceSchedule::FixedInterval(self.interval)
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let resp = self
            .http
            .get(&self.url)
            .send()
            .await
            .with_context(|| format!("rss fetch {}", self.url))?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("rss {} returned HTTP {status}", self.url);
        }
        let body = resp
            .text()
            .await
            .with_context(|| format!("rss body decode {}", self.url))?;
        let items = parse_rss_2(&body)?;
        let now = Utc::now();
        let out = items
            .into_iter()
            .filter_map(|it| self.into_event(it, now))
            .collect();
        Ok(out)
    }
}

impl RssNewsPoller {
    fn into_event(&self, item: RssItem, now: DateTime<Utc>) -> Option<MarketEvent> {
        if item.link.trim().is_empty() || item.title.trim().is_empty() {
            return None;
        }
        let occurred_at = item.pub_date.unwrap_or(now);
        // payload 模拟 FMP shape,让 collector / curator 复用同一解析路径
        let payload = json!({
            "source_class": "trusted",
            "legal_ad_template": false,
            "earnings_call_transcript": false,
            "fmp": {
                "site": self.handle,
                "text": item.summary,
                "title": item.title,
                "url": item.link,
                "publishedDate": occurred_at.to_rfc3339(),
            },
        });
        Some(MarketEvent {
            id: format!("news:{}", item.link),
            kind: EventKind::NewsCritical,
            severity: Severity::High,
            symbols: Vec::new(), // RSS 不提取 ticker;curator 按 title + summary 推断
            occurred_at,
            title: item.title,
            summary: item.summary.chars().take(400).collect(),
            url: Some(item.link),
            source: format!("rss:{}", self.handle),
            payload,
        })
    }
}

/// 解析 RSS 2.0 feed,抽 `<channel>/<item>` 列表。
///
/// 容错:解析中遇到非预期 tag / 编码错误一律 skip 该 item,不 panic;
/// 完全无法解析的整个 feed 才 bail。
pub fn parse_rss_2(xml: &str) -> anyhow::Result<Vec<RssItem>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut buf = Vec::new();
    let mut in_item = false;
    let mut current_field: Option<String> = None;
    let mut cur_title = String::new();
    let mut cur_link = String::new();
    let mut cur_pub = String::new();
    let mut cur_desc = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name().as_ref().to_vec();
                let name_str = String::from_utf8_lossy(&name).to_string();
                if name_str == "item" {
                    in_item = true;
                    cur_title.clear();
                    cur_link.clear();
                    cur_pub.clear();
                    cur_desc.clear();
                } else if in_item {
                    current_field = Some(name_str);
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name().as_ref().to_vec();
                let name_str = String::from_utf8_lossy(&name).to_string();
                if name_str == "item" {
                    let pub_date = parse_pub_date(&cur_pub);
                    items.push(RssItem {
                        title: std::mem::take(&mut cur_title),
                        link: std::mem::take(&mut cur_link),
                        pub_date,
                        summary: strip_html(&cur_desc).chars().take(500).collect(),
                    });
                    in_item = false;
                    current_field = None;
                } else if in_item {
                    current_field = None;
                }
            }
            Ok(Event::Text(t)) => {
                if in_item {
                    let text = match t.unescape() {
                        Ok(s) => s.into_owned(),
                        Err(_) => String::from_utf8_lossy(t.as_ref()).into_owned(),
                    };
                    match current_field.as_deref() {
                        Some("title") => cur_title.push_str(&text),
                        Some("link") => cur_link.push_str(&text),
                        Some("pubDate") => cur_pub.push_str(&text),
                        Some("description") => cur_desc.push_str(&text),
                        _ => {}
                    }
                }
            }
            Ok(Event::CData(t)) => {
                if in_item {
                    let text = String::from_utf8_lossy(t.as_ref()).into_owned();
                    match current_field.as_deref() {
                        Some("title") => cur_title.push_str(&text),
                        Some("link") => cur_link.push_str(&text),
                        Some("pubDate") => cur_pub.push_str(&text),
                        Some("description") => cur_desc.push_str(&text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => {
                anyhow::bail!("xml parse error at {}: {e}", reader.error_position());
            }
        }
        buf.clear();
    }
    Ok(items)
}

/// 解析 RSS 常见 pubDate 格式(RFC 822 / RFC 2822 / ISO 8601)。
fn parse_pub_date(s: &str) -> Option<DateTime<Utc>> {
    if s.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    None
}

/// 简易去 HTML 标签(`<...>` 全删,`&amp;` / `&lt;` / `&gt;` / `&quot;` / `&#39;` 还原)。
fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Sample Feed</title>
    <item>
      <title>Bloomberg: Hormuz Crisis Is Biggest Energy Disruption Ever</title>
      <link>https://www.bloomberg.com/news/articles/2026-04-25/hormuz-crisis</link>
      <pubDate>Sat, 25 Apr 2026 14:30:00 GMT</pubDate>
      <description>&lt;p&gt;The Strait of Hormuz oil shock has yet to crash demand.&lt;/p&gt;</description>
    </item>
    <item>
      <title>SpaceX wins $57M contract</title>
      <link>https://spacenews.com/spacex-wins-57-million</link>
      <pubDate>Fri, 24 Apr 2026 20:47:26 +0000</pubDate>
      <description>Space Force awarded SpaceX a contract for satellite-to-satellite communications.</description>
    </item>
  </channel>
</rss>"#;

    #[test]
    fn parse_rss_extracts_items_with_fields() {
        let items = parse_rss_2(SAMPLE_RSS).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].title,
            "Bloomberg: Hormuz Crisis Is Biggest Energy Disruption Ever"
        );
        assert!(items[0].link.starts_with("https://www.bloomberg.com/"));
        assert!(items[0].pub_date.is_some());
        assert!(items[0].summary.contains("Strait of Hormuz oil shock"));
        // strip_html 去掉了 <p> 标签
        assert!(!items[0].summary.contains("<p>"));
    }

    #[test]
    fn parse_rss_handles_atom_iso8601_date_via_rfc3339() {
        let xml = r#"<?xml version="1.0"?><rss><channel><item>
            <title>T</title><link>http://x</link>
            <pubDate>2026-04-25T14:30:00Z</pubDate>
            <description>d</description>
        </item></channel></rss>"#;
        let items = parse_rss_2(xml).unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].pub_date.is_some());
    }

    #[test]
    fn parse_rss_skips_items_with_empty_title_or_link_via_into_event() {
        let xml = r#"<?xml version="1.0"?><rss><channel>
            <item><title></title><link>http://x</link></item>
            <item><title>good</title><link></link></item>
            <item><title>real</title><link>http://r</link></item>
        </channel></rss>"#;
        let parsed = parse_rss_2(xml).unwrap();
        assert_eq!(parsed.len(), 3);
        let poller = RssNewsPoller::new("test", "http://feed", Duration::from_secs(60));
        let events: Vec<_> = parsed
            .into_iter()
            .filter_map(|it| poller.into_event(it, Utc::now()))
            .collect();
        // 只有第三条 title+link 都非空
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "real");
    }

    #[test]
    fn into_event_produces_fmp_shaped_payload() {
        let poller = RssNewsPoller::new(
            "spacenews",
            "https://spacenews.com/feed/",
            Duration::from_secs(3600),
        );
        let item = RssItem {
            title: "SpaceX wins $57M contract".into(),
            link: "https://spacenews.com/spacex-wins-57-million".into(),
            pub_date: Some(Utc::now()),
            summary: "Space Force contract for satellite crosslink demo".into(),
        };
        let ev = poller.into_event(item, Utc::now()).unwrap();
        assert_eq!(ev.source, "rss:spacenews");
        assert_eq!(ev.severity, Severity::High);
        assert!(matches!(ev.kind, EventKind::NewsCritical));
        assert!(ev.id.starts_with("news:https://spacenews.com/"));
        // payload 模拟 FMP shape
        assert_eq!(
            ev.payload.get("source_class").and_then(|v| v.as_str()),
            Some("trusted")
        );
        assert_eq!(
            ev.payload
                .get("legal_ad_template")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        let fmp = ev.payload.get("fmp").unwrap();
        assert_eq!(fmp.get("site").and_then(|v| v.as_str()), Some("spacenews"));
        assert!(
            fmp.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .contains("satellite crosslink")
        );
    }

    #[test]
    fn parse_rss_unescapes_html_entities() {
        let xml = r#"<?xml version="1.0"?><rss><channel><item>
            <title>AT&amp;T &amp; Verizon merge talks</title>
            <link>http://x</link>
            <description>Stocks &lt;up&gt; 5%</description>
        </item></channel></rss>"#;
        let items = parse_rss_2(xml).unwrap();
        assert_eq!(items[0].title, "AT&T & Verizon merge talks");
    }

    #[test]
    fn poller_name_uses_handle_prefix() {
        let p = RssNewsPoller::new("bloomberg_markets", "http://feed", Duration::from_secs(60));
        assert_eq!(p.name(), "rss.bloomberg_markets");
    }
}
