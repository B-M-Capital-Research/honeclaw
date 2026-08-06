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

use crate::earnings_document::{EARNINGS_DOCUMENT_KEY, canonical_earnings_document_key};
use crate::event::{MarketEvent, Severity};

pub const DEFAULT_EARNINGS_QUALITY_SYSTEM_PROMPT: &str = r#"你是一个面向长期主线投资者的财报质量判断器。你会收到一个 EPS surprise 事件和对应 SEC 8-K / earnings release 的精选摘抄。请只根据摘抄做综合判断，不要补充外部事实。

必须综合看：收入增长、收入的量价/组合/并购/低基数来源、指引、backlog / RPO / 大客户订单、毛利率、经营利润率、GAAP 与 non-GAAP 利润、EBIT / EBITA / EBITDA、adjusted EBITDA、经营现金流、capex、债务/流动性、管理层措辞和明确风险。EPS 只是其中一个信号；对于亏损公司或接近 0 的 EPS，不要把 EPS surprise 百分比当成主要结论。

严格区分：摘抄已经确认的事实、你基于事实做的综合判断、摘抄尚未回答的问题。没有市场共识、用户自己的预期或上季承诺时，不得伪造比较结论。即使综合结论正面，也必须保留最强反向项和仍需核验的问题。

金额与单位是硬约束：优先原样保留摘抄中的 `$8.97B`、`$226.4M` 等 B/M 表示，不得擅自改写成“亿美元/万元”或改变小数点、数量级。若表头说明 `in millions`，裸数字 `13,335` 必须写成 `$13,335M`，绝不能写成 `$13,335B`；若要写 B 也必须先正确换算成 `$13.335B`，但仍优先保留原始 M 单位。只有 Raw EPS payload 同时给出 actual 与 estimate 时，才可对 EPS 使用“超预期/不及预期”；收入、利润率、指引等没有对应共识字段时不得说“全面超预期”。无法确认单位时保留原始写法并把疑问放入 unknowns，不得猜测换算。

输出必须是单个 JSON object，不要 Markdown，不要解释：
{
  "conclusion": "positive|mixed_positive|neutral|mixed_negative|negative|unclear",
  "route": "immediate|digest|suppress",
  "confidence": 0.0,
  "headline_zh": "28字以内中文标题，不要重复ticker",
  "summary_zh": "1到2句中文综合判断",
  "evidence": ["最多3条短证据"],
  "risks": ["最多2条短风险"],
  "unknowns": ["最多2条摘抄尚未确认、但会影响判断的问题"],
  "follow_ups": ["最多3条电话会、正式季报或下季必须继续核验的问题"],
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
    pub unknowns: Vec<String>,
    #[serde(default)]
    pub follow_ups: Vec<String>,
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
        parse_review_response(&response.content)
            .map(normalize_review_shape)
            .filter(|review| !has_implausible_billions_unit(review))
            .or_else(|| {
                warn!(
                    event_id = %event.id,
                    model = %self.model,
                    degraded = true,
                    content_prefix = %response.content.chars().take(160).collect::<String>(),
                    "earnings quality review returned invalid or numerically unsafe JSON"
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
    let earnings_document_key = context_url
        .as_deref()
        .and_then(canonical_earnings_document_key);
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
        let headline = review.headline_zh.trim();
        if !headline.is_empty() {
            // ticker 与事件类型已经由 renderer header 统一展示；title 只保留真正的
            // 研究结论，避免 `$SNDK · 财报发布 / SNDK 财报 ...` 重复占一行。
            event.title = headline.to_string();
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
        if applied && let Some(key) = earnings_document_key {
            obj.insert(EARNINGS_DOCUMENT_KEY.into(), Value::String(key));
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
        let object = if let Some(first) = value
            .as_array()
            .and_then(|response_items| response_items.first())
            .cloned()
        {
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

fn normalize_review_shape(mut review: EarningsQualityReview) -> EarningsQualityReview {
    review.headline_zh = truncate_review_text(&review.headline_zh, 28);
    review.summary_zh = truncate_review_text(&review.summary_zh, 600);
    normalize_review_list(&mut review.evidence, 3, 320);
    normalize_review_list(&mut review.risks, 2, 320);
    normalize_review_list(&mut review.unknowns, 2, 320);
    normalize_review_list(&mut review.follow_ups, 3, 320);
    review
}

fn normalize_review_list(values: &mut Vec<String>, max_items: usize, max_chars: usize) {
    *values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .take(max_items)
        .map(|value| truncate_review_text(value, max_chars))
        .collect();
}

fn truncate_review_text(value: &str, max_chars: usize) -> String {
    value.trim().chars().take(max_chars).collect()
}

fn has_implausible_billions_unit(review: &EarningsQualityReview) -> bool {
    static COMMA_BILLIONS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let regex = COMMA_BILLIONS.get_or_init(|| {
        regex::Regex::new(r"(?i)\$\s*[0-9]{1,3}(?:,[0-9]{3})+\s*B\b")
            .expect("valid comma billions regex")
    });
    std::iter::once(review.headline_zh.as_str())
        .chain(std::iter::once(review.summary_zh.as_str()))
        .chain(review.evidence.iter().map(String::as_str))
        .chain(review.risks.iter().map(String::as_str))
        .chain(review.unknowns.iter().map(String::as_str))
        .chain(review.follow_ups.iter().map(String::as_str))
        .any(|value| regex.is_match(value))
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
    let mut lines = Vec::new();
    let summary = review.summary_zh.trim();
    if !summary.is_empty() {
        lines.push(format!("结论：{summary}"));
    }
    push_review_list(&mut lines, "关键证据", &review.evidence, 3);
    push_review_list(&mut lines, "反向项", &review.risks, 2);
    push_review_list(&mut lines, "尚未确认", &review.unknowns, 2);
    push_review_list(&mut lines, "后续核验", &review.follow_ups, 3);
    if lines.is_empty() {
        return String::new();
    }
    lines.join("\n")
}

fn push_review_list(lines: &mut Vec<String>, label: &str, values: &[String], limit: usize) {
    let values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .take(limit)
        .collect::<Vec<_>>();
    if !values.is_empty() {
        lines.push(format!("{label}：{}", values.join("；")));
    }
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

    fn sample_aaoi_earnings_event() -> MarketEvent {
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
    fn prompt_preserves_source_units_and_limits_consensus_language() {
        let prompt = DEFAULT_EARNINGS_QUALITY_SYSTEM_PROMPT;
        assert!(prompt.contains("$8.97B"));
        assert!(prompt.contains("$226.4M"));
        assert!(prompt.contains("不得擅自改写成“亿美元/万元”"));
        assert!(prompt.contains("`13,335` 必须写成 `$13,335M`"));
        assert!(prompt.contains("只有 Raw EPS payload 同时给出 actual 与 estimate"));
        assert!(prompt.contains("不得说“全面超预期”"));
    }

    #[test]
    fn rejects_implausible_comma_billions_units() {
        let mut review = EarningsQualityReview {
            conclusion: "positive".into(),
            route: "digest".into(),
            confidence: 0.8,
            headline_zh: "现金流稳健".into(),
            summary_zh: "经营现金流$13,335B".into(),
            evidence: vec!["现金及等价物$14,161B".into()],
            risks: vec![],
            unknowns: vec![],
            follow_ups: vec![],
            override_eps_only: true,
        };
        assert!(has_implausible_billions_unit(&review));
        review.summary_zh = "经营现金流$13,335M".into();
        review.evidence = vec!["营收$41.46B".into()];
        assert!(!has_implausible_billions_unit(&review));
    }

    #[test]
    fn parses_json_fence_response() {
        let raw = r#"```json
        {"conclusion":"mixed_positive","route":"digest","confidence":0.82,"headline_zh":"营收增51%但仍亏损","summary_zh":"营收和指引改善，但亏损仍扩大","evidence":["收入增长51%"],"risks":["non-GAAP仍亏损"],"unknowns":["现金流口径未披露"],"follow_ups":["电话会核验利润率"],"override_eps_only":true}
        ```"#;
        let review = parse_review_response(raw).expect("review");
        assert_eq!(review.route, "digest");
        assert_eq!(review.conclusion, "mixed_positive");
        assert_eq!(review.unknowns, vec!["现金流口径未披露"]);
        assert_eq!(review.follow_ups, vec!["电话会核验利润率"]);
        assert!(review.override_eps_only);
    }

    #[test]
    fn normalizes_model_lists_to_the_product_contract() {
        let parsed = parse_review_response(
            r#"{"conclusion":"positive","route":"immediate","confidence":0.9,"headline_zh":"这是一个超过二十八个汉字并且不应该原样进入通知标题的财报结论标题","summary_zh":"结论","evidence":["一","二","三","四"],"risks":["一","二","三"],"unknowns":["一","二","三"],"follow_ups":["一","二","三","四"],"override_eps_only":true}"#,
        )
        .map(normalize_review_shape)
        .expect("review");
        assert_eq!(parsed.headline_zh.chars().count(), 28);
        assert_eq!(parsed.evidence, vec!["一", "二", "三"]);
        assert_eq!(parsed.risks, vec!["一", "二"]);
        assert_eq!(parsed.unknowns, vec!["一", "二"]);
        assert_eq!(parsed.follow_ups, vec!["一", "二", "三"]);
    }

    #[test]
    fn applies_digest_review_by_demoting_eps_high() {
        let mut event = sample_aaoi_earnings_event();
        let review = EarningsQualityReview {
            conclusion: "mixed_positive".into(),
            route: "digest".into(),
            confidence: 0.85,
            headline_zh: "营收增51%但仍亏损".into(),
            summary_zh: "营收和指引改善，但亏损仍扩大".into(),
            evidence: vec!["收入增长51%".into()],
            risks: vec!["non-GAAP仍亏损".into()],
            unknowns: vec!["增长的量价贡献未披露".into()],
            follow_ups: vec!["电话会核验订单能见度".into()],
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
        assert_eq!(
            event
                .payload
                .get(EARNINGS_DOCUMENT_KEY)
                .and_then(Value::as_str),
            Some("https://sec.gov/aaoi.htm")
        );
        assert!(event.title.contains("营收增51%"));
        assert!(event.summary.contains("关键证据：收入增长51%"));
        assert!(event.summary.contains("反向项：non-GAAP仍亏损"));
        assert!(event.summary.contains("尚未确认：增长的量价贡献未披露"));
        assert!(event.summary.contains("后续核验：电话会核验订单能见度"));
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
        let mut event = sample_aaoi_earnings_event();
        event.severity = Severity::Medium;
        let review = EarningsQualityReview {
            conclusion: "positive".into(),
            route: "immediate".into(),
            confidence: 0.95,
            headline_zh: "营收暴增毛利改善".into(),
            summary_zh: "收入和毛利率同时显著改善，现金流转正".into(),
            evidence: vec!["收入增长79%".into()],
            risks: vec![],
            unknowns: vec![],
            follow_ups: vec![],
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
        let mut event = sample_aaoi_earnings_event();
        let original_title = event.title.clone();
        let review = EarningsQualityReview {
            conclusion: "unclear".into(),
            route: "immediate".into(),
            confidence: 0.3,
            headline_zh: "不应覆盖".into(),
            summary_zh: "不应覆盖".into(),
            evidence: vec![],
            risks: vec![],
            unknowns: vec![],
            follow_ups: vec![],
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

    #[test]
    fn sndk_regression_keeps_full_research_card_fields_in_event() {
        let mut event = sample_aaoi_earnings_event();
        event.symbols = vec!["SNDK".into()];
        let review = EarningsQualityReview {
            conclusion: "mixed_positive".into(),
            route: "immediate".into(),
            confidence: 0.95,
            headline_zh: "数据中心强劲，消费端仍承压".into(),
            summary_zh: "数据中心订单和毛利率改善，但消费端下滑要求继续核验周期持续性。".into(),
            evidence: vec![
                "数据中心收入和订单增长".into(),
                "毛利率显著改善".into(),
                "下一季收入指引上调".into(),
            ],
            risks: vec!["消费端收入环比下降".into(), "历史比较含分拆口径".into()],
            unknowns: vec!["增长的量价与低基数贡献尚未拆分".into()],
            follow_ups: vec![
                "电话会核验企业级 SSD 客户采用".into(),
                "下季核验 NAND 供给纪律和毛利持续性".into(),
            ],
            override_eps_only: true,
        };

        assert!(apply_earnings_quality_review(
            &mut event,
            review,
            Some("https://sec.example.test/sndk-earnings.htm".into()),
            0.65,
            0.9,
        ));
        assert_eq!(event.title, "数据中心强劲，消费端仍承压");
        assert!(!event.title.contains("SNDK 财报"));
        for expected in [
            "关键证据：数据中心收入和订单增长",
            "毛利率显著改善",
            "反向项：消费端收入环比下降",
            "尚未确认：增长的量价与低基数贡献尚未拆分",
            "后续核验：电话会核验企业级 SSD 客户采用",
            "下季核验 NAND 供给纪律和毛利持续性",
        ] {
            assert!(
                event.summary.contains(expected),
                "missing {expected}: {}",
                event.summary
            );
        }
    }
}
