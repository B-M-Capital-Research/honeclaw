//! LLM-assisted quality review for `EarningsReleased` events.
//!
//! The earnings surprise feed only carries EPS actual vs estimate. That signal
//! is useful for detecting that earnings were released, but it is too narrow for
//! user-facing pushes on loss-making or near-zero EPS names. This module reviews
//! a selected earnings-release excerpt and decides whether the candidate should
//! be emitted as immediate, digest, or suppressed.

use std::sync::Arc;

use async_trait::async_trait;
use hone_llm::{LlmProvider, Message};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::warn;

use crate::event::{MarketEvent, Severity};

pub const DEFAULT_EARNINGS_QUALITY_SYSTEM_PROMPT: &str = r#"你是一个面向长期主线投资者的财报质量判断器。你会收到一个 EPS surprise 事件和对应 SEC 8-K / earnings release 的精选摘抄。请只根据摘抄做综合判断，不要补充外部事实。

必须综合看：收入增长、指引、backlog / RPO / 大客户订单、毛利率、经营利润率、GAAP 与 non-GAAP 利润、EBIT / EBITA / EBITDA、adjusted EBITDA、经营现金流、capex、债务/流动性、管理层措辞和明确风险。EPS 只是其中一个信号；对于亏损公司或接近 0 的 EPS，不要把 EPS surprise 百分比当成主要结论。

输出必须是单个 JSON object，不要 Markdown，不要解释：
{
  "conclusion": "positive|mixed_positive|neutral|mixed_negative|negative|unclear",
  "route": "immediate|digest|suppress",
  "confidence": 0.0,
  "headline_zh": "18字以内中文标题，不要重复ticker",
  "summary_zh": "一句中文综合判断",
  "evidence": ["最多3条短证据"],
  "risks": ["最多2条短风险"],
  "override_eps_only": true
}

route 规则：
- immediate：只有高置信、信息足以改变用户当日判断的显著正面或负面财报才使用。
- digest：混合、常规、仅 EPS 方向明显但综合信号不足，或需要等电话会/后续数据确认。
- suppress：摘抄没有足够业务/财务新信息，或只是 routine。

如果没有非 EPS 指标的 consensus，不要说“超预期/不及预期”；只能说公司披露的增长、改善、承压或风险。"#;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EarningsQualityReview {
    pub conclusion: String,
    pub route: String,
    pub confidence: f64,
    #[serde(default)]
    pub headline_zh: String,
    #[serde(default)]
    pub summary_zh: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub override_eps_only: bool,
}

#[async_trait]
pub trait EarningsQualityReviewer: Send + Sync {
    async fn review(&self, event: &MarketEvent, context: &str) -> Option<EarningsQualityReview>;
}

pub struct LlmEarningsQualityReviewer {
    provider: Arc<dyn LlmProvider>,
    model: String,
    system_prompt: String,
}

impl LlmEarningsQualityReviewer {
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            system_prompt: DEFAULT_EARNINGS_QUALITY_SYSTEM_PROMPT.to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }
}

#[async_trait]
impl EarningsQualityReviewer for LlmEarningsQualityReviewer {
    async fn review(&self, event: &MarketEvent, context: &str) -> Option<EarningsQualityReview> {
        let messages = build_review_messages(&self.system_prompt, event, context);
        let response = match self.provider.chat(&messages, Some(&self.model)).await {
            Ok(response) => response,
            Err(e) => {
                warn!(
                    event_id = %event.id,
                    model = %self.model,
                    degraded = true,
                    "earnings quality review LLM failed: {e}"
                );
                return None;
            }
        };
        parse_review_response(&response.content).or_else(|| {
            warn!(
                event_id = %event.id,
                model = %self.model,
                degraded = true,
                content_prefix = %response.content.chars().take(160).collect::<String>(),
                "earnings quality review returned unparsable JSON"
            );
            None
        })
    }
}

pub fn apply_earnings_quality_review(
    event: &mut MarketEvent,
    review: EarningsQualityReview,
    context_url: Option<String>,
    min_review_confidence: f64,
    min_immediate_confidence: f64,
) -> bool {
    let mut applied = false;
    let mut reason = None;
    let confidence = if review.confidence.is_finite() {
        review.confidence.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let route = normalized_route(&review.route);

    if confidence < min_review_confidence {
        reason = Some("low_confidence");
    } else if route.is_none() {
        reason = Some("invalid_route");
    } else {
        let route = route.expect("checked route");
        let ticker = event
            .symbols
            .first()
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string());

        let headline = review.headline_zh.trim();
        if !headline.is_empty() {
            event.title = format!("{ticker} 财报 {headline}");
        }

        let summary = review_summary(&review);
        if !summary.is_empty() {
            event.summary = summary;
        }

        match route {
            "immediate" if confidence >= min_immediate_confidence => {
                event.severity = Severity::High;
            }
            "immediate" | "digest" => {
                event.severity = Severity::Medium;
            }
            "suppress" => {
                event.severity = Severity::Low;
            }
            _ => {}
        }

        if let Some(url) = context_url.as_ref().filter(|url| !url.trim().is_empty()) {
            event.url = Some(url.clone());
        }
        applied = true;
    }

    ensure_payload_object(&mut event.payload);
    if let Some(obj) = event.payload.as_object_mut() {
        obj.insert(
            "earnings_quality_review".into(),
            serde_json::to_value(&review).unwrap_or(Value::Null),
        );
        obj.insert(
            "earnings_quality_review_applied".into(),
            Value::Bool(applied),
        );
        obj.insert(
            "earnings_quality_review_confidence".into(),
            Value::from(confidence),
        );
        if let Some(url) = context_url {
            obj.insert("earnings_quality_context_url".into(), Value::String(url));
        }
        if let Some(reason) = reason {
            obj.insert(
                "earnings_quality_review_skipped_reason".into(),
                Value::String(reason.into()),
            );
        }
    }

    applied
}

fn build_review_messages(system_prompt: &str, event: &MarketEvent, context: &str) -> Vec<Message> {
    let payload = serde_json::to_string(&event.payload).unwrap_or_else(|_| "{}".to_string());
    let user = format!(
        "Ticker: {}\nCandidate title: {}\nEPS trigger summary: {}\nRaw EPS payload: {}\n\nSEC earnings-release excerpt:\n{}",
        event.symbols.first().cloned().unwrap_or_default(),
        event.title,
        event.summary,
        payload,
        context
    );
    vec![
        Message {
            role: "system".into(),
            content: Some(system_prompt.to_string()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".into(),
            content: Some(user),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

fn parse_review_response(content: &str) -> Option<EarningsQualityReview> {
    let trimmed = strip_json_fence(content.trim());
    for candidate in [
        Some(trimmed.to_string()),
        extract_balanced_json_object(trimmed),
    ]
    .into_iter()
    .flatten()
    {
        let value: Value = match serde_json::from_str(&candidate) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let object = if let Some(first) = value.as_array().and_then(|arr| arr.first()).cloned() {
            first
        } else {
            value
        };
        if let Ok(review) = serde_json::from_value::<EarningsQualityReview>(object) {
            return Some(review);
        }
    }
    None
}

fn strip_json_fence(content: &str) -> &str {
    let content = content.trim();
    if !content.starts_with("```") {
        return content;
    }
    let without_open = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```JSON"))
        .or_else(|| content.strip_prefix("```"))
        .unwrap_or(content)
        .trim_start();
    without_open
        .strip_suffix("```")
        .unwrap_or(without_open)
        .trim()
}

fn extract_balanced_json_object(content: &str) -> Option<String> {
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;

    for (idx, ch) in content.char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if start.is_none() {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    let begin = start?;
                    let end = idx + ch.len_utf8();
                    return Some(content[begin..end].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn normalized_route(route: &str) -> Option<&'static str> {
    match route.trim().to_ascii_lowercase().as_str() {
        "immediate" => Some("immediate"),
        "digest" => Some("digest"),
        "suppress" => Some("suppress"),
        _ => None,
    }
}

fn review_summary(review: &EarningsQualityReview) -> String {
    let mut parts = Vec::new();
    let summary = review.summary_zh.trim();
    if !summary.is_empty() {
        parts.push(summary.to_string());
    } else {
        parts.extend(
            review
                .evidence
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .take(2)
                .map(str::to_string),
        );
    }
    if let Some(risk) = review
        .risks
        .iter()
        .map(|s| s.trim())
        .find(|s| !s.is_empty())
    {
        parts.push(format!("风险: {risk}"));
    }
    parts.join("；")
}

fn ensure_payload_object(value: &mut Value) {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_event() -> MarketEvent {
        MarketEvent {
            id: "earnings_surprise:AAOI:2026-05-08".into(),
            kind: crate::event::EventKind::EarningsReleased,
            severity: Severity::High,
            symbols: vec!["AAOI".into()],
            occurred_at: Utc::now(),
            title: "AAOI 财报 candidate 亏损多于预期 EPS差 -0.14".into(),
            summary: "EPS 实际 -0.19 / 预期 -0.05；差值 -0.14".into(),
            url: Some("https://finance.yahoo.com/quote/AAOI/press-releases/".into()),
            source: "fmp.earnings_surprises".into(),
            payload: serde_json::json!({"actualEarningResult": -0.19, "estimatedEarning": -0.05}),
        }
    }

    #[test]
    fn parses_json_fence_response() {
        let raw = r#"```json
        {"conclusion":"mixed_positive","route":"digest","confidence":0.82,"headline_zh":"营收增51%但仍亏损","summary_zh":"营收和指引改善，但亏损仍扩大","evidence":["收入增长51%"],"risks":["non-GAAP仍亏损"],"override_eps_only":true}
        ```"#;
        let review = parse_review_response(raw).expect("review");
        assert_eq!(review.route, "digest");
        assert_eq!(review.conclusion, "mixed_positive");
        assert!(review.override_eps_only);
    }

    #[test]
    fn applies_digest_review_by_demoting_eps_high() {
        let mut event = sample_event();
        let review = EarningsQualityReview {
            conclusion: "mixed_positive".into(),
            route: "digest".into(),
            confidence: 0.85,
            headline_zh: "营收增51%但仍亏损".into(),
            summary_zh: "营收和指引改善，但亏损仍扩大".into(),
            evidence: vec!["收入增长51%".into()],
            risks: vec!["non-GAAP仍亏损".into()],
            override_eps_only: true,
        };
        let applied = apply_earnings_quality_review(
            &mut event,
            review,
            Some("https://sec.gov/aaoi.htm".into()),
            0.65,
            0.9,
        );
        assert!(applied);
        assert_eq!(event.severity, Severity::Medium);
        assert_eq!(event.url.as_deref(), Some("https://sec.gov/aaoi.htm"));
        assert!(event.title.contains("营收增51%"));
        assert!(
            event
                .payload
                .get("earnings_quality_review_applied")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
    }

    #[test]
    fn promotes_high_confidence_immediate_review() {
        let mut event = sample_event();
        event.severity = Severity::Medium;
        let review = EarningsQualityReview {
            conclusion: "positive".into(),
            route: "immediate".into(),
            confidence: 0.95,
            headline_zh: "营收暴增毛利改善".into(),
            summary_zh: "收入和毛利率同时显著改善，现金流转正".into(),
            evidence: vec!["收入增长79%".into()],
            risks: vec![],
            override_eps_only: true,
        };
        assert!(apply_earnings_quality_review(
            &mut event, review, None, 0.65, 0.9
        ));
        assert_eq!(event.severity, Severity::High);
        assert!(event.summary.contains("现金流转正"));
    }

    #[test]
    fn low_confidence_review_is_recorded_but_not_applied() {
        let mut event = sample_event();
        let original_title = event.title.clone();
        let review = EarningsQualityReview {
            conclusion: "unclear".into(),
            route: "immediate".into(),
            confidence: 0.3,
            headline_zh: "不应覆盖".into(),
            summary_zh: "不应覆盖".into(),
            evidence: vec![],
            risks: vec![],
            override_eps_only: false,
        };
        assert!(!apply_earnings_quality_review(
            &mut event, review, None, 0.65, 0.9
        ));
        assert_eq!(event.title, original_title);
        assert_eq!(
            event
                .payload
                .get("earnings_quality_review_skipped_reason")
                .and_then(Value::as_str),
            Some("low_confidence")
        );
    }
}
