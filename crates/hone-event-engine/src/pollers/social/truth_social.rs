//! Truth Social 公开账号时间线抓取。
//!
//! Truth Social 使用 Mastodon 兼容的公开 API:`/api/v1/accounts/:id/statuses`,
//! 无需认证即可读取公开发文。首次调用时若未配置 `account_id`,则通过
//! `/api/v2/search?q=@<username>&resolve=true` 查一次并缓存。
//!
//! 产出 `EventKind::SocialPost`,severity 一律 Low,`payload.source_class="uncertain"`,
//! router 的 LLM 仲裁器按重要性判断是否升 Medium。转发(reblog)跳过,避免冗余。

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use tokio::sync::RwLock;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::source::{EventSource, SourceSchedule};

use super::{SOCIAL_SUMMARY_MAX_CHARS, SOCIAL_TITLE_MAX_CHARS};

pub struct TruthSocialPoller {
    username: String,                   // "realDonaldTrump" (无 @)
    account_id: RwLock<Option<String>>, // 首次 resolve 后缓存
    interval: Duration,
    http: reqwest::Client,
    base_url: String, // 默认 "https://truthsocial.com"
    limit: u32,
    name_cached: String,
}

impl TruthSocialPoller {
    pub fn new(
        username: impl Into<String>,
        account_id: Option<String>,
        interval: Duration,
    ) -> Self {
        let username = username.into();
        let name_cached = format!("truth_social.{}", username.to_lowercase());
        Self {
            username,
            account_id: RwLock::new(account_id),
            interval,
            http: reqwest::Client::builder()
                .user_agent("honeclaw-bot/0.2 (+https://github.com/)")
                .timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client build"),
            base_url: "https://truthsocial.com".into(),
            limit: 20,
            name_cached,
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base_url = base.into();
        self
    }

    async fn resolve_account_id(&self) -> anyhow::Result<String> {
        if let Some(id) = self.account_id.read().await.clone() {
            return Ok(id);
        }
        let url = format!(
            "{}/api/v2/search?q=%40{}&resolve=true&type=accounts&limit=1",
            self.base_url, self.username
        );
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        let body: Value = resp.json().await?;
        if !status.is_success() {
            anyhow::bail!("truth_social search HTTP {status}: {body}");
        }
        // Mastodon /api/v2/search → { accounts: [{ id, username, ... }] }
        let id = body
            .get("accounts")
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|acc| acc.get("id"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "truth_social: resolve username={} returned no accounts",
                    self.username
                )
            })?;
        *self.account_id.write().await = Some(id.clone());
        Ok(id)
    }

    async fn fetch_statuses(&self) -> anyhow::Result<Vec<Value>> {
        let account_id = self.resolve_account_id().await?;
        let url = format!(
            "{}/api/v1/accounts/{}/statuses?limit={}&exclude_replies=true",
            self.base_url, account_id, self.limit
        );
        let resp = self.http.get(&url).send().await?;
        let status = resp.status();
        let body: Value = resp.json().await?;
        if !status.is_success() {
            anyhow::bail!("truth_social statuses HTTP {status}: {body}");
        }
        Ok(body.as_array().cloned().unwrap_or_default())
    }
}

#[async_trait]
impl EventSource for TruthSocialPoller {
    fn name(&self) -> &str {
        &self.name_cached
    }

    fn schedule(&self) -> SourceSchedule {
        SourceSchedule::FixedInterval(self.interval)
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let raw = self.fetch_statuses().await?;
        Ok(parse_statuses(&raw, &self.username))
    }
}

/// Mastodon-style `[status]` JSON → `MarketEvent` 列表。
pub fn parse_statuses(arr: &[Value], username: &str) -> Vec<MarketEvent> {
    let username_lc = username.to_lowercase();
    let mut out = Vec::new();
    for item in arr {
        // 跳过转发(reblog 非 null 表示本条是转发)
        if !matches!(item.get("reblog"), None | Some(Value::Null)) {
            continue;
        }
        let id = match item.get("id").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };
        let content_html = item
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let text = strip_html(&content_html);
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }
        let url = item.get("url").and_then(|v| v.as_str()).map(str::to_string);
        let occurred_at = item
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(parse_iso_datetime)
            .unwrap_or_else(Utc::now);

        let title = summarize(&text, SOCIAL_TITLE_MAX_CHARS);
        let summary = truncate(&text, SOCIAL_SUMMARY_MAX_CHARS);

        let mut payload = Map::new();
        payload.insert("username".into(), Value::String(username.into()));
        payload.insert("source_class".into(), Value::String("uncertain".into()));
        payload.insert("raw_text".into(), Value::String(text));
        payload.insert("status_id".into(), Value::String(id.clone()));

        out.push(MarketEvent {
            id: format!("truth_social:{username_lc}:{id}"),
            kind: EventKind::SocialPost,
            severity: Severity::Low,
            symbols: Vec::new(),
            occurred_at,
            title,
            summary,
            url,
            source: format!("truth_social.{username_lc}"),
            payload: Value::Object(payload),
        });
    }
    out
}

/// 非常基础的 HTML 标签剥离:只关心文字内容和换行语义(<br>、</p> 替换为 \n)。
/// 不做 entity decode 的全量实现,但常见 &amp;/&lt;/&gt;/&quot;/&#39; 覆盖。
fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_newline = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '<' {
            // 扫到 '>' 内部,顺便检查是否 <br / </p> / </div> 等块级 → 写换行
            let start = i;
            while i < bytes.len() && bytes[i] as char != '>' {
                i += 1;
            }
            let tag = &s[start..i.min(bytes.len())];
            let lower = tag.to_ascii_lowercase();
            if lower.starts_with("<br")
                || lower.starts_with("</p")
                || lower.starts_with("</div")
                || lower.starts_with("</li")
            {
                pending_newline = true;
            }
            i += 1;
            continue;
        }
        if pending_newline {
            out.push('\n');
            pending_newline = false;
        }
        out.push(c);
        i += 1;
    }
    decode_entities(&out)
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_status(id: &str, content: &str, created_at: &str) -> Value {
        json!({
            "id": id,
            "content": content,
            "created_at": created_at,
            "url": format!("https://truthsocial.com/@realDonaldTrump/posts/{id}"),
            "reblog": null,
        })
    }

    #[test]
    fn parses_basic_status() {
        let arr = vec![sample_status(
            "110001",
            "<p>Big news! Tariffs are coming. MAKE AMERICA GREAT AGAIN.</p>",
            "2026-04-20T18:00:00.000Z",
        )];
        let events = parse_statuses(&arr, "realDonaldTrump");
        assert_eq!(events.len(), 1);
        let e = &events[0];
        assert_eq!(e.kind, EventKind::SocialPost);
        assert_eq!(e.id, "truth_social:realdonaldtrump:110001");
        assert_eq!(e.source, "truth_social.realdonaldtrump");
        assert_eq!(
            e.payload.get("source_class").and_then(|v| v.as_str()),
            Some("uncertain")
        );
        assert!(e.summary.contains("Tariffs"));
        assert_eq!(e.occurred_at.to_rfc3339(), "2026-04-20T18:00:00+00:00");
    }

    #[test]
    fn skips_reblog() {
        let arr = vec![json!({
            "id": "x",
            "content": "<p>retweet</p>",
            "created_at": "2026-04-20T18:00:00.000Z",
            "reblog": { "id": "other" }
        })];
        assert!(parse_statuses(&arr, "realDonaldTrump").is_empty());
    }

    #[test]
    fn skips_empty_content() {
        let arr = vec![sample_status("1", "<p>   </p>", "2026-04-20T18:00:00.000Z")];
        assert!(parse_statuses(&arr, "a").is_empty());
    }

    #[test]
    fn title_keeps_long_social_first_line_beyond_legacy_80_chars() {
        let text = "JUST IN: President Trump says Saudi Arabia is helping the US on the Strait of Hormuz and energy market stability this week.";
        let arr = vec![sample_status(
            "110002",
            &format!("<p>{text}</p>"),
            "2026-04-20T18:00:00.000Z",
        )];
        let events = parse_statuses(&arr, "realDonaldTrump");
        assert_eq!(events[0].title, text);
        assert!(!events[0].title.ends_with('…'));
    }

    #[test]
    fn strip_html_preserves_text_and_decodes_entities() {
        let got = strip_html("<p>Hello &amp; world</p><p>line2</p>");
        assert!(got.contains("Hello & world"));
        assert!(got.contains("line2"));
    }
}
