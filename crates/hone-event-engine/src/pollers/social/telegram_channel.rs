//! Telegram 公开频道 web preview 抓取 (`https://t.me/s/<handle>`)。
//!
//! 这个地址无需 Bot Token / 登录就能返回最新约 20 条帖子,DOM 是稳定的
//! `.tgme_widget_message_wrap` 列表。我们只解析文本与时间,不渲染图片/视频。
//!
//! 产出 `EventKind::SocialPost`,severity 一律 Low, `payload.source_class="uncertain"`,
//! 让 router 的 `LlmNewsClassifier` 按"是否重要"升 Medium。

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde_json::{Map, Value};

use crate::event::{EventKind, MarketEvent, Severity};
use crate::source::{EventSource, SourceSchedule};

use super::{SOCIAL_SUMMARY_MAX_CHARS, SOCIAL_TITLE_MAX_CHARS};

pub struct TelegramChannelPoller {
    handle: String, // 频道用户名,如 "watcherguru"
    interval: Duration,
    extract_cashtags: bool,
    http: reqwest::Client,
    base_url: String,    // 默认 "https://t.me",测试时可换 mock
    name_cached: String, // 缓存 name() 返回的字符串,避免每次分配
}

impl TelegramChannelPoller {
    pub fn new(handle: impl Into<String>, interval: Duration, extract_cashtags: bool) -> Self {
        let handle = handle.into();
        let name_cached = format!("telegram.{handle}");
        Self {
            handle,
            interval,
            extract_cashtags,
            http: reqwest::Client::builder()
                .user_agent("honeclaw-bot/0.2 (+https://github.com/)")
                .timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client build"),
            base_url: "https://t.me".into(),
            name_cached,
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base_url = base.into();
        self
    }

    async fn fetch_html(&self) -> anyhow::Result<String> {
        let url = format!("{}/s/{}", self.base_url, self.handle);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("telegram preview HTTP {status} for {url}");
        }
        Ok(body)
    }
}

#[async_trait]
impl EventSource for TelegramChannelPoller {
    fn name(&self) -> &str {
        &self.name_cached
    }

    fn schedule(&self) -> SourceSchedule {
        SourceSchedule::FixedInterval(self.interval)
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let html = self.fetch_html().await?;
        Ok(parse_telegram_preview(
            &html,
            &self.handle,
            self.extract_cashtags,
        ))
    }
}

/// 把 Telegram channel web preview 的 HTML 解析成 `MarketEvent` 列表。
///
/// 纯函数——给定输入 HTML 永远得同一输出,便于单测。
pub fn parse_telegram_preview(
    html: &str,
    handle: &str,
    extract_cashtags: bool,
) -> Vec<MarketEvent> {
    let doc = Html::parse_document(html);
    let msg_sel = Selector::parse(".tgme_widget_message[data-post]").unwrap();
    let text_sel = Selector::parse(".tgme_widget_message_text").unwrap();
    let date_sel = Selector::parse(".tgme_widget_message_date").unwrap();
    let time_sel = Selector::parse("time[datetime]").unwrap();

    let mut out = Vec::new();
    for msg in doc.select(&msg_sel) {
        let data_post = match msg.value().attr("data-post") {
            Some(v) => v,
            None => continue,
        };
        // data-post 形如 "watcherguru/12345" → post_id=12345
        let post_id = data_post.split('/').next_back().unwrap_or("").to_string();
        if post_id.is_empty() {
            continue;
        }

        let text = msg
            .select(&text_sel)
            .next()
            .map(extract_text)
            .unwrap_or_default();
        let text = text.trim().to_string();
        if text.is_empty() {
            // 纯图片/视频帖子——跳过,没有文字内容就没有 LLM 判定的基础。
            continue;
        }

        let (url, occurred_at) = match msg.select(&date_sel).next() {
            Some(a) => {
                let href = a.value().attr("href").unwrap_or("").to_string();
                let ts = a
                    .select(&time_sel)
                    .next()
                    .and_then(|t| t.value().attr("datetime"))
                    .and_then(parse_iso_datetime)
                    .unwrap_or_else(Utc::now);
                (Some(href), ts)
            }
            None => (None, Utc::now()),
        };

        let title = summarize(&text, SOCIAL_TITLE_MAX_CHARS);
        let summary = truncate(&text, SOCIAL_SUMMARY_MAX_CHARS);
        let symbols = if extract_cashtags {
            extract_cashtag_symbols(&text)
        } else {
            Vec::new()
        };

        let mut payload = Map::new();
        payload.insert("channel".into(), Value::String(handle.into()));
        payload.insert("source_class".into(), Value::String("uncertain".into()));
        payload.insert("raw_text".into(), Value::String(text));
        payload.insert("post_id".into(), Value::String(post_id.clone()));

        out.push(MarketEvent {
            id: format!("telegram:{handle}:{post_id}"),
            kind: EventKind::SocialPost,
            severity: Severity::Low,
            symbols,
            occurred_at,
            title,
            summary,
            url,
            source: format!("telegram.{handle}"),
            payload: Value::Object(payload),
        });
    }
    out
}

fn extract_text(node: scraper::ElementRef) -> String {
    let mut buf = String::new();
    for chunk in node.text() {
        buf.push_str(chunk);
    }
    buf
}

fn parse_iso_datetime(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

fn summarize(text: &str, max_chars: usize) -> String {
    let first_line = text.lines().next().unwrap_or("").trim();
    truncate(first_line, max_chars)
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// 简单 `$TICKER` 提取:匹配 `$` + 1-5 位大写字母,去重保序。
fn extract_cashtag_symbols(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for token in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '$') {
        if let Some(rest) = token.strip_prefix('$') {
            if (1..=5).contains(&rest.len())
                && rest.chars().all(|c| c.is_ascii_uppercase())
                && !out.contains(&rest.to_string())
            {
                out.push(rest.to_string());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_HTML: &str = r#"
    <html><body>
    <div class="tgme_widget_message_wrap">
      <div class="tgme_widget_message js-widget_message" data-post="watcherguru/12345">
        <div class="tgme_widget_message_text js-message_text">
          BREAKING: $BTC surges to new all-time high, acquired by Apple Inc.
        </div>
        <div class="tgme_widget_message_footer">
          <a class="tgme_widget_message_date" href="https://t.me/watcherguru/12345">
            <time datetime="2026-04-20T12:34:56+00:00">3:45 PM</time>
          </a>
        </div>
      </div>
    </div>
    <div class="tgme_widget_message_wrap">
      <div class="tgme_widget_message" data-post="watcherguru/12346">
        <div class="tgme_widget_message_text">Photo only, no text content? Actually this has text.</div>
        <a class="tgme_widget_message_date" href="https://t.me/watcherguru/12346">
          <time datetime="2026-04-20T12:40:00+00:00">3:50 PM</time>
        </a>
      </div>
    </div>
    <div class="tgme_widget_message_wrap">
      <div class="tgme_widget_message" data-post="watcherguru/12347">
        <div class="tgme_widget_message_text">   </div>
      </div>
    </div>
    </body></html>
    "#;

    #[test]
    fn parses_messages_with_core_fields() {
        let events = parse_telegram_preview(SAMPLE_HTML, "watcherguru", true);
        assert_eq!(events.len(), 2, "empty-text message should be skipped");
        let e = &events[0];
        assert_eq!(e.kind, EventKind::SocialPost);
        assert_eq!(e.severity, Severity::Low);
        assert_eq!(e.source, "telegram.watcherguru");
        assert_eq!(e.id, "telegram:watcherguru:12345");
        assert_eq!(e.url.as_deref(), Some("https://t.me/watcherguru/12345"));
        assert_eq!(
            e.payload.get("source_class").and_then(|v| v.as_str()),
            Some("uncertain")
        );
        assert_eq!(
            e.payload.get("channel").and_then(|v| v.as_str()),
            Some("watcherguru")
        );
        assert!(e.title.contains("BREAKING"));
        assert!(e.symbols.contains(&"BTC".to_string()));
    }

    #[test]
    fn occurred_at_parses_iso() {
        let events = parse_telegram_preview(SAMPLE_HTML, "watcherguru", false);
        let e = events.iter().find(|e| e.id.ends_with("12345")).unwrap();
        assert_eq!(e.occurred_at.to_rfc3339(), "2026-04-20T12:34:56+00:00");
    }

    #[test]
    fn cashtag_extraction_toggle() {
        let events_on = parse_telegram_preview(SAMPLE_HTML, "watcherguru", true);
        let events_off = parse_telegram_preview(SAMPLE_HTML, "watcherguru", false);
        let e_on = events_on.iter().find(|e| e.id.ends_with("12345")).unwrap();
        let e_off = events_off.iter().find(|e| e.id.ends_with("12345")).unwrap();
        assert!(e_on.symbols.contains(&"BTC".to_string()));
        assert!(e_off.symbols.is_empty());
    }

    #[test]
    fn title_keeps_long_social_first_line_beyond_legacy_80_chars() {
        let text = "JUST IN: Polymarket to launch 24/7 perpetual futures trading for crypto, equities, commodities, and FX markets next quarter.";
        let html = format!(
            r#"<div class="tgme_widget_message" data-post="watcherguru/222">
                <div class="tgme_widget_message_text">{text}</div>
                <a class="tgme_widget_message_date" href="https://t.me/watcherguru/222">
                  <time datetime="2026-04-20T12:40:00+00:00">3:50 PM</time>
                </a>
              </div>"#
        );
        let events = parse_telegram_preview(&html, "watcherguru", false);
        assert_eq!(events[0].title, text);
        assert!(!events[0].title.ends_with('…'));
    }

    #[test]
    fn empty_or_no_data_post_is_skipped() {
        let html = r#"<div class="tgme_widget_message">no data-post attr</div>"#;
        assert!(parse_telegram_preview(html, "foo", false).is_empty());
    }
}
