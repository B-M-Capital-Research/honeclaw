//! Telegram OutboundSink —— 直接调 Bot API `sendMessage`。
//!
//! - 私聊:`chat_id = actor.user_id`(就是 Telegram 用户数字 id)
//! - 群聊:`actor.channel_scope = Some("chat_<chat_id>")`,剥掉 `chat_` 前缀后
//!   就是 Telegram 的负数 chat_id
//!
//! 不做消息分段;真超长 Telegram 会返 400,外层看日志就能发现。

use async_trait::async_trait;
use hone_core::ActorIdentity;

use crate::digest::{DigestItem, DigestPayload, group_by_kind_bucket};
use crate::event::Severity;
use crate::renderer::RenderFormat;
use crate::router::OutboundSink;

pub struct TelegramSink {
    bot_token: String,
    client: reqwest::Client,
}

impl TelegramSink {
    pub fn new(bot_token: impl Into<String>) -> Self {
        Self {
            bot_token: bot_token.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
        }
    }

    fn chat_id_for(actor: &ActorIdentity) -> String {
        match actor.channel_scope.as_deref() {
            Some(scope) if scope != "direct" => {
                scope.strip_prefix("chat_").unwrap_or(scope).to_string()
            }
            _ => actor.user_id.clone(),
        }
    }
}

impl TelegramSink {
    async fn post_html(&self, actor: &ActorIdentity, text: &str) -> anyhow::Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let chat_id = Self::chat_id_for(actor);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text,
                "disable_web_page_preview": true,
                "parse_mode": "HTML",
            }))
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let detail = resp.text().await.unwrap_or_default();
            anyhow::bail!("telegram sendMessage {status}: {detail}");
        }
        Ok(())
    }
}

#[async_trait]
impl OutboundSink for TelegramSink {
    async fn send(&self, actor: &ActorIdentity, body: &str) -> anyhow::Result<()> {
        self.post_html(actor, body).await
    }

    fn format(&self) -> RenderFormat {
        RenderFormat::TelegramHtml
    }

    /// digest 走 bucket 分组化的 HTML 文本——比扁平 bullet 更易扫读。链接用
    /// `<a href>` 短锚文本(🔗),`disable_web_page_preview=true` 抑制自动卡片。
    /// 主色块用 emoji 球(🔴/🟡/🔵)在标题前缀语义化 severity。
    async fn send_digest(
        &self,
        actor: &ActorIdentity,
        payload: &DigestPayload,
        _fallback_body: &str,
    ) -> anyhow::Result<()> {
        let html = build_telegram_digest_html(payload);
        self.post_html(actor, &html).await
    }
}

/// 把 `DigestPayload` 渲染成 Telegram `parse_mode=HTML` 文本。bucket 分组,
/// 标题加 severity 球,链接用 `<a href>🔗</a>` 锚。
pub(crate) fn build_telegram_digest_html(payload: &DigestPayload) -> String {
    let total = payload.total();
    let severity_dot = match payload.max_severity {
        Severity::High => "🔴",
        Severity::Medium => "🟡",
        Severity::Low => "🔵",
    };
    let title = if total > 1 {
        format!("{} 📬 {} · {} 条", severity_dot, payload.label, total)
    } else {
        format!("{} 📬 {}", severity_dot, payload.label)
    };
    let mut out = format!("<b>{}</b>", escape_html(&title));
    let grouped = group_by_kind_bucket(&payload.items);
    for (bucket, items) in grouped {
        out.push_str("\n\n");
        out.push_str(&format!(
            "<b>{}</b>",
            escape_html(&format!("{} · {}", bucket.header_label(), items.len()))
        ));
        for it in items {
            out.push('\n');
            out.push_str(&render_telegram_line(it));
        }
    }
    if payload.cap_overflow > 0 {
        out.push_str(&format!(
            "\n\n<i>{}</i>",
            escape_html(&format!(
                "另 {} 条因数量上限未展示,发送 /missed 查看完整清单",
                payload.cap_overflow
            ))
        ));
    }
    out
}

fn render_telegram_line(it: &DigestItem) -> String {
    let mut out = String::from("• ");
    if let Some(sym) = &it.primary_symbol {
        out.push_str(&format!("<b>${}</b> ", escape_html(sym)));
    }
    out.push_str(&escape_html(it.headline.trim()));
    if let Some(url) = &it.url {
        out.push_str(&format!(" <a href=\"{}\">🔗</a>", escape_html_attr(url)));
    }
    out
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn escape_html_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::ActorIdentity;

    #[test]
    fn chat_id_for_direct_uses_user_id() {
        let actor = ActorIdentity::new("telegram", "8039067465", None::<String>).unwrap();
        assert_eq!(TelegramSink::chat_id_for(&actor), "8039067465");
    }

    #[test]
    fn chat_id_for_group_strips_chat_prefix() {
        let actor =
            ActorIdentity::new("telegram", "8039067465", Some("chat_-1002012381143")).unwrap();
        assert_eq!(TelegramSink::chat_id_for(&actor), "-1002012381143");
    }

    #[test]
    fn chat_id_for_group_without_prefix_passes_through() {
        let actor = ActorIdentity::new("telegram", "8039067465", Some("-1001234567890")).unwrap();
        assert_eq!(TelegramSink::chat_id_for(&actor), "-1001234567890");
    }

    use crate::digest::DigestItem;
    use crate::event::EventKind;
    use chrono::Utc;

    fn item(
        kind: EventKind,
        sev: Severity,
        sym: &str,
        headline: &str,
        url: Option<&str>,
    ) -> DigestItem {
        DigestItem {
            id: format!("id:{kind:?}:{sym}:{headline}"),
            kind,
            severity: sev,
            primary_symbol: if sym.is_empty() {
                None
            } else {
                Some(sym.into())
            },
            headline: headline.into(),
            url: url.map(String::from),
            occurred_at: Utc::now(),
            origin: crate::unified_digest::ItemOrigin::Buffered,
            floor: None,
            comment: None,
            thesis_relation: None,
        }
    }

    fn payload_with(
        items: Vec<DigestItem>,
        max_sev: Severity,
        cap_overflow: usize,
    ) -> DigestPayload {
        DigestPayload {
            label: "盘前摘要 · 08:30".into(),
            items,
            cap_overflow,
            max_severity: max_sev,
            generated_at: Utc::now(),
        }
    }

    #[test]
    fn html_digest_starts_with_severity_dot_and_title() {
        let p = payload_with(
            vec![item(
                EventKind::NewsCritical,
                Severity::High,
                "AAPL",
                "x",
                None,
            )],
            Severity::High,
            0,
        );
        let html = build_telegram_digest_html(&p);
        assert!(html.starts_with("<b>🔴 📬 盘前摘要"), "html = {html}");
    }

    #[test]
    fn html_digest_uses_anchor_emoji_link() {
        let p = payload_with(
            vec![item(
                EventKind::NewsCritical,
                Severity::High,
                "MU",
                "Memory rally",
                Some("https://example.com/path"),
            )],
            Severity::High,
            0,
        );
        let html = build_telegram_digest_html(&p);
        assert!(
            html.contains(r#"<a href="https://example.com/path">🔗</a>"#),
            "html = {html}"
        );
        assert!(html.contains("<b>$MU</b>"));
    }

    #[test]
    fn html_digest_groups_by_bucket() {
        let items = vec![
            item(EventKind::NewsCritical, Severity::High, "AAPL", "n1", None),
            item(
                EventKind::EarningsUpcoming,
                Severity::Medium,
                "GOOGL",
                "e1",
                None,
            ),
        ];
        let p = payload_with(items, Severity::High, 0);
        let html = build_telegram_digest_html(&p);
        assert!(html.contains("📰 新闻公告 · 1"));
        assert!(html.contains("📅 财报 · 1"));
    }

    #[test]
    fn html_digest_escapes_dangerous_chars() {
        let p = payload_with(
            vec![item(
                EventKind::NewsCritical,
                Severity::High,
                "AAPL",
                "AT&T <hack>",
                None,
            )],
            Severity::High,
            0,
        );
        let html = build_telegram_digest_html(&p);
        assert!(html.contains("AT&amp;T &lt;hack&gt;"));
        assert!(!html.contains("<hack>"));
    }

    #[test]
    fn html_digest_includes_overflow_italic() {
        let p = payload_with(
            vec![item(
                EventKind::NewsCritical,
                Severity::High,
                "AAPL",
                "n",
                None,
            )],
            Severity::High,
            5,
        );
        let html = build_telegram_digest_html(&p);
        assert!(html.contains("<i>另 5 条"));
        assert!(html.contains("/missed"));
    }
}
