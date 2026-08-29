//! Shared, source-bounded earnings-call transcript review.
//!
//! A full transcript is reviewed once into a compact public fact object. Actor
//! continuity can then compare that object with each A-tier profile without
//! repeatedly sending the copyrighted source body to the model or persistence.

use std::sync::Arc;

use async_trait::async_trait;
use hone_llm::{LlmProvider, Message};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::earnings_claim::EarningsClaimInput;
use crate::event::{EventKind, MarketEvent, Severity};
use crate::operating_kpi_claim::{
    OperatingKpiClaimInput, operating_kpi_input_is_supported_for_symbol,
    operating_kpi_input_is_verbatim_in_source, operating_kpi_prompt_for_symbol,
};

pub const DEFAULT_EARNINGS_TRANSCRIPT_SYSTEM_PROMPT: &str = r#"你是专业股票研究团队的财报电话会审计员。输入是一份公司官方投资者关系页面提供的完整电话会文字稿。只使用输入文字稿，不得补充模型记忆、市场传闻或外部事实。

你的工作不是复述整场电话会，而是提取会改变长期投资判断、能够在下季继续核验的增量证据。必须严格分开 prepared remarks 与 analyst Q&A：
1. prepared_findings 只记录管理层主动披露的关键经营变化、因果解释和前瞻口径。
2. qa_findings 要概括分析师真正追问的问题，并把回答质量标为 direct、partial、evaded 或 unclear。只有正面回答核心问题才是 direct；转换口径、只重复 prepared remarks、不给时间或数字时不能算 direct。
3. commitments 只记录管理层明确作出的未来可核验承诺、量化指引或有明确时间点的行动。愿景、目标市场、一般性信心和“我们相信”不是承诺。
4. contradictions 只记录本次不同于 prepared remarks、财报新闻稿或同场早先表述的直接冲突；没有冲突就输出空数组。
5. unresolved_questions 只保留会影响投资主线、但 Q&A 仍未回答的具体问题。
6. 保留原文数字、币种、B/M 和百分比单位；不得擅自换算数量级。证据字段是短摘录式转述，不要大段复制原文。
7. 所有中文字段要短、可核验。每个数组最多 4 项，evidence_zh 最多 80 个汉字；不要输出电话会版权页、主持流程或安全港声明。
8. claims 最多 12 项；没有明确期间、原始数值/口径、说话人或位置时不要提取。metric_basis 必须写 GAAP、non-GAAP 或公司明确给出的经营口径。metric_id 只能是 revenue、revenue_growth、gross_margin、operating_margin、free_cash_flow、capital_expenditure、inventory、accounts_receivable、accounts_payable、backlog、rpo、arr、orders、shipments、capacity、utilization、asp、market_share、customer_qualification、customers、retention、usage、tokens、context_length、power_capacity、product_mix、unit_cost、delivery_lead_time。numeric_value 必须逐字出现在 value_text 或 evidence_zh；unit 只能是 %、percentage_points、basis_points、USD、USD_millions、USD_billions、units、customers、days、ratio、MW、GW、GB、TB、PB、EB、tokens。disposition 默认为 active；只有原文明确修正旧口径时用 corrected，明确撤回时用 withdrawn。否则不要填数字或直接不提取该 claim。
9. operating_kpi_claims 最多 6 项，只能使用本次 Ticker 后附动态目录中的 kpi_id。issuer_metric_name 和 issuer_definition 必须逐字复制文字稿中的公司原始名称/定义，issuer_definition 还必须原样出现在 evidence_quote 或 value_text；找不到定义就不要提取。行业数据不得冒充公司实现值，机会管线不得冒充已签订单，送样/认证/量产必须分开。numeric_value 非空时 unit 只能是 %、percentage_points、basis_points、USD、USD_millions、USD_billions、units、customers、days、weeks、ratio、kW、MW、GW、GB、TB、PB、EB、bits、tokens、calls、modules、ports、wafers、workflows、milestone；不要写 percent。comparison_basis 只能是 year_over_year、sequential_quarter、point_in_time、period_total、period_average、period_end。definition_changed 只有公司明确宣布口径改变时才为 true。

输出一个 JSON object，不要 Markdown：
{
  "source_scope":"prepared_and_qa|prepared_only|qa_only|unclear",
  "management_tone":"more_confident|unchanged|more_cautious|mixed|unclear",
  "prepared_findings":[{"topic":"主题","finding_zh":"关键增量事实或口径","evidence_zh":"短证据","speaker":"姓名或职务"}],
  "qa_findings":[{"topic":"追问主题","question_zh":"分析师实际追问","answer_quality":"direct|partial|evaded|unclear","answer_zh":"管理层实际回答到哪里","evidence_zh":"短证据"}],
  "commitments":[{"statement_zh":"未来可核验承诺","due_at":"时间或材料","evidence_zh":"短证据"}],
  "claims":[{"claim_kind":"reported_fact|management_guidance|management_commentary","metric_id":"受支持的规范指标ID","metric_basis":"GAAP|non-GAAP|公司定义口径","period":"财季/日期范围","numeric_value":null,"unit":"原始规范单位；无数值时为空字符串","value_text":"原始值与口径","speaker":"姓名或职务；管理层主张必填","evidence_zh":"短原文证据","source_locator":"prepared remarks或Q&A及说话人","disposition":"active|corrected|withdrawn"}],
  "operating_kpi_claims":[{"claim_kind":"reported_fact|management_guidance|contract_milestone","kpi_id":"动态目录中的 KPI ID","issuer_metric_name":"公司原始指标名","issuer_definition":"文字稿中逐字定义","period":"明确期间","numeric_value":null,"unit":"原始规范单位","value_text":"原始值与口径","measurement_scope":"产品/分母/客户/期末或平均等边界","comparison_basis":"year_over_year|sequential_quarter|point_in_time|period_total|period_average|period_end","speaker":"管理层主张必填","evidence_quote":"文字稿中的短原文","source_locator":"prepared remarks或Q&A及说话人","definition_changed":false,"disposition":"active|corrected|withdrawn"}],
  "contradictions":[{"statement_zh":"冲突点","evidence_zh":"短证据"}],
  "unresolved_questions":["仍未回答的具体问题"]
}"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptPreparedFinding {
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub finding_zh: String,
    #[serde(default)]
    pub evidence_zh: String,
    #[serde(default)]
    pub speaker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptQaFinding {
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub question_zh: String,
    #[serde(default)]
    pub answer_quality: String,
    #[serde(default)]
    pub answer_zh: String,
    #[serde(default)]
    pub evidence_zh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptCommitment {
    #[serde(default)]
    pub statement_zh: String,
    #[serde(default)]
    pub due_at: String,
    #[serde(default)]
    pub evidence_zh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptContradiction {
    #[serde(default)]
    pub statement_zh: String,
    #[serde(default)]
    pub evidence_zh: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EarningsTranscriptReview {
    pub source_scope: String,
    pub management_tone: String,
    #[serde(default)]
    pub prepared_findings: Vec<TranscriptPreparedFinding>,
    #[serde(default)]
    pub qa_findings: Vec<TranscriptQaFinding>,
    #[serde(default)]
    pub commitments: Vec<TranscriptCommitment>,
    #[serde(default)]
    pub claims: Vec<EarningsClaimInput>,
    #[serde(default)]
    pub operating_kpi_claims: Vec<OperatingKpiClaimInput>,
    #[serde(default)]
    pub contradictions: Vec<TranscriptContradiction>,
    #[serde(default)]
    pub unresolved_questions: Vec<String>,
}

#[async_trait]
pub trait EarningsTranscriptReviewer: Send + Sync {
    async fn review(
        &self,
        event: &MarketEvent,
        transcript: &str,
    ) -> Option<EarningsTranscriptReview>;
}

pub struct LlmEarningsTranscriptReviewer {
    provider: Arc<dyn LlmProvider>,
    model: String,
    system_prompt: String,
}

impl LlmEarningsTranscriptReviewer {
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            model: model.into(),
            system_prompt: DEFAULT_EARNINGS_TRANSCRIPT_SYSTEM_PROMPT.to_string(),
        }
    }

    #[cfg(test)]
    fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }
}

#[async_trait]
impl EarningsTranscriptReviewer for LlmEarningsTranscriptReviewer {
    async fn review(
        &self,
        event: &MarketEvent,
        transcript: &str,
    ) -> Option<EarningsTranscriptReview> {
        if !matches!(event.kind, EventKind::EarningsCallTranscript)
            || transcript.trim().chars().count() < 2_000
        {
            return None;
        }
        let messages = build_review_messages(&self.system_prompt, event, transcript);
        let response = match self.provider.chat(&messages, Some(&self.model)).await {
            Ok(response) => response,
            Err(error) => {
                tracing::warn!(
                    event_id = %event.id,
                    model = %self.model,
                    degraded = true,
                    "earnings transcript review LLM failed: {error}"
                );
                return None;
            }
        };
        parse_review_response(&response.content)
            .map(normalize_review)
            .filter(valid_review)
            .or_else(|| {
                tracing::warn!(
                    event_id = %event.id,
                    model = %self.model,
                    degraded = true,
                    content_prefix = %response.content.chars().take(160).collect::<String>(),
                    "earnings transcript review returned invalid JSON"
                );
                None
            })
    }
}

pub fn apply_earnings_transcript_review(
    event: &mut MarketEvent,
    mut review: EarningsTranscriptReview,
    transcript_chars: usize,
) -> bool {
    // This compatibility path has no source body to verify against.  Preserve
    // the ordinary transcript review but fail closed for operating KPI rows.
    review.operating_kpi_claims.clear();
    apply_earnings_transcript_review_inner(event, review, transcript_chars, false)
}

pub fn apply_earnings_transcript_review_with_source(
    event: &mut MarketEvent,
    mut review: EarningsTranscriptReview,
    transcript: &str,
    transcript_chars: usize,
) -> bool {
    let symbol = event
        .symbols
        .first()
        .map(String::as_str)
        .unwrap_or_default();
    review.operating_kpi_claims.retain(|claim| {
        operating_kpi_input_is_supported_for_symbol(symbol, claim)
            && operating_kpi_input_is_verbatim_in_source(claim, transcript)
    });
    apply_earnings_transcript_review_inner(event, review, transcript_chars, true)
}

fn apply_earnings_transcript_review_inner(
    event: &mut MarketEvent,
    review: EarningsTranscriptReview,
    transcript_chars: usize,
    operating_kpi_source_verified: bool,
) -> bool {
    if !matches!(event.kind, EventKind::EarningsCallTranscript) || !valid_review(&review) {
        return false;
    }
    let summary = render_review_summary(&review);
    if summary.is_empty() {
        return false;
    }
    event.title = format!(
        "电话会：{}",
        tone_label(&review.management_tone).unwrap_or("新增经营证据")
    );
    event.summary = summary;
    event.severity = Severity::Medium;
    ensure_payload_object(&mut event.payload);
    if let Some(payload) = event.payload.as_object_mut() {
        payload.insert(
            "earnings_transcript_review".to_string(),
            serde_json::to_value(&review).unwrap_or(Value::Null),
        );
        payload.insert(
            "earnings_transcript_review_applied".to_string(),
            Value::Bool(true),
        );
        payload.insert(
            "earnings_transcript_source_chars".to_string(),
            Value::from(transcript_chars as u64),
        );
        payload.insert(
            "earnings_transcript_operating_kpi_source_verified".to_string(),
            Value::Bool(operating_kpi_source_verified),
        );
    }
    true
}

fn build_review_messages(
    system_prompt: &str,
    event: &MarketEvent,
    transcript: &str,
) -> Vec<Message> {
    let symbol = event.symbols.first().cloned().unwrap_or_default();
    let system_prompt = format!(
        "{}{}",
        system_prompt,
        operating_kpi_prompt_for_symbol(&symbol)
    );
    let user = format!(
        "Ticker: {}\nCall title: {}\nCall date: {}\nOfficial source URL: {}\n\nFull transcript:\n{}",
        symbol,
        event.title,
        event.occurred_at.to_rfc3339(),
        event.url.as_deref().unwrap_or("unavailable"),
        transcript.trim()
    );
    vec![
        Message {
            role: "system".to_string(),
            content: Some(system_prompt),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".to_string(),
            content: Some(user),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

fn parse_review_response(content: &str) -> Option<EarningsTranscriptReview> {
    let trimmed = content.trim();
    let candidate = if trimmed.starts_with("```") {
        trimmed
            .lines()
            .skip(1)
            .take_while(|line| !line.trim_start().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        trimmed[start..=end].to_string()
    } else {
        trimmed.to_string()
    };
    serde_json::from_str(&candidate).ok()
}

fn normalize_review(mut review: EarningsTranscriptReview) -> EarningsTranscriptReview {
    review.source_scope = review.source_scope.trim().to_ascii_lowercase();
    review.management_tone = review.management_tone.trim().to_ascii_lowercase();
    review.claims.truncate(12);
    review.operating_kpi_claims.truncate(6);
    review.prepared_findings = review
        .prepared_findings
        .into_iter()
        .filter_map(|mut finding| {
            finding.topic = truncate(&finding.topic, 40);
            finding.finding_zh = truncate(&finding.finding_zh, 160);
            finding.evidence_zh = truncate(&finding.evidence_zh, 80);
            finding.speaker = truncate(&finding.speaker, 60);
            (!finding.finding_zh.is_empty()).then_some(finding)
        })
        .take(4)
        .collect();
    review.qa_findings = review
        .qa_findings
        .into_iter()
        .filter_map(|mut finding| {
            finding.topic = truncate(&finding.topic, 40);
            finding.question_zh = truncate(&finding.question_zh, 140);
            finding.answer_quality = finding.answer_quality.trim().to_ascii_lowercase();
            finding.answer_zh = truncate(&finding.answer_zh, 180);
            finding.evidence_zh = truncate(&finding.evidence_zh, 80);
            matches!(
                finding.answer_quality.as_str(),
                "direct" | "partial" | "evaded" | "unclear"
            )
            .then_some(finding)
        })
        .take(4)
        .collect();
    review.commitments = review
        .commitments
        .into_iter()
        .filter_map(|mut commitment| {
            commitment.statement_zh = truncate(&commitment.statement_zh, 180);
            commitment.due_at = truncate(&commitment.due_at, 80);
            commitment.evidence_zh = truncate(&commitment.evidence_zh, 80);
            (!commitment.statement_zh.is_empty()).then_some(commitment)
        })
        .take(4)
        .collect();
    review.contradictions = review
        .contradictions
        .into_iter()
        .filter_map(|mut contradiction| {
            contradiction.statement_zh = truncate(&contradiction.statement_zh, 180);
            contradiction.evidence_zh = truncate(&contradiction.evidence_zh, 80);
            (!contradiction.statement_zh.is_empty()).then_some(contradiction)
        })
        .take(4)
        .collect();
    review.unresolved_questions = review
        .unresolved_questions
        .into_iter()
        .map(|value| truncate(&value, 180))
        .filter(|value| !value.is_empty())
        .take(4)
        .collect();
    review
}

fn valid_review(review: &EarningsTranscriptReview) -> bool {
    matches!(
        review.source_scope.as_str(),
        "prepared_and_qa" | "prepared_only" | "qa_only" | "unclear"
    ) && matches!(
        review.management_tone.as_str(),
        "more_confident" | "unchanged" | "more_cautious" | "mixed" | "unclear"
    ) && (!review.prepared_findings.is_empty() || !review.qa_findings.is_empty())
}

fn render_review_summary(review: &EarningsTranscriptReview) -> String {
    let mut lines = Vec::new();
    if let Some(label) = tone_label(&review.management_tone) {
        lines.push(format!("管理层口径：{label}"));
    }
    if !review.prepared_findings.is_empty() {
        lines.push(format!(
            "主动披露：{}",
            review
                .prepared_findings
                .iter()
                .take(3)
                .map(|item| item.finding_zh.as_str())
                .collect::<Vec<_>>()
                .join("；")
        ));
    }
    if !review.qa_findings.is_empty() {
        lines.push(format!(
            "分析师问答：{}",
            review
                .qa_findings
                .iter()
                .take(3)
                .map(|item| format!("{}（{}）", item.answer_zh, qa_label(&item.answer_quality)))
                .collect::<Vec<_>>()
                .join("；")
        ));
    }
    if !review.commitments.is_empty() {
        lines.push(format!(
            "明确承诺：{}",
            review
                .commitments
                .iter()
                .take(2)
                .map(|item| item.statement_zh.as_str())
                .collect::<Vec<_>>()
                .join("；")
        ));
    }
    if !review.unresolved_questions.is_empty() {
        lines.push(format!(
            "仍未回答：{}",
            review
                .unresolved_questions
                .iter()
                .take(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join("；")
        ));
    }
    lines.join("\n")
}

fn tone_label(value: &str) -> Option<&'static str> {
    match value {
        "more_confident" => Some("较此前更有信心"),
        "unchanged" => Some("核心判断未变"),
        "more_cautious" => Some("较此前更谨慎"),
        "mixed" => Some("信号分化"),
        "unclear" => Some("未形成清晰增量口径"),
        _ => None,
    }
}

fn qa_label(value: &str) -> &'static str {
    match value {
        "direct" => "直接回答",
        "partial" => "部分回答",
        "evaded" => "回避",
        _ => "不清晰",
    }
}

fn ensure_payload_object(value: &mut Value) {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.trim().chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Mutex;

    use chrono::Utc;
    use futures::stream::{self, BoxStream};
    use hone_core::HoneResult;
    use hone_llm::ChatResponse;
    use hone_llm::provider::ChatResult;

    use super::*;
    use crate::earnings_claim::EarningsClaimDisposition;
    use crate::operating_kpi_claim::{
        OPERATING_KPI_POLICY_STATUS, OperatingKpiClaimKind, OperatingKpiComparisonBasis,
    };

    struct StubProvider {
        response: String,
        prompt: Mutex<String>,
    }

    #[async_trait]
    impl LlmProvider for StubProvider {
        async fn chat(&self, messages: &[Message], _: Option<&str>) -> HoneResult<ChatResult> {
            *self.prompt.lock().unwrap() = messages
                .iter()
                .filter_map(|message| message.content.as_deref())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(ChatResult {
                content: self.response.clone(),
                usage: None,
            })
        }

        async fn chat_with_tools(
            &self,
            _: &[Message],
            _: &[Value],
            _: Option<&str>,
        ) -> HoneResult<ChatResponse> {
            unreachable!()
        }

        fn chat_stream<'a>(
            &'a self,
            _: &'a [Message],
            _: Option<&'a str>,
        ) -> BoxStream<'a, HoneResult<String>> {
            Box::pin(stream::empty())
        }
    }

    fn event() -> MarketEvent {
        MarketEvent {
            id: "earnings_call_transcript:official".into(),
            kind: EventKind::EarningsCallTranscript,
            severity: Severity::Low,
            symbols: vec!["ACME".into()],
            occurred_at: Utc::now(),
            title: "ACME Q2 earnings call transcript".into(),
            summary: String::new(),
            url: Some("https://ir.example/acme-q2.pdf".into()),
            source: "company_ir".into(),
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn official_transcript_fixture_covers_four_company_shapes_and_eight_calls() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/event_engine/earnings_transcript_baseline_2026-08-06.json"
        ))
        .expect("valid fixture JSON");
        assert_eq!(fixture["version"], "2026-08-06");
        let companies = fixture["companies"].as_array().expect("companies");
        assert_eq!(companies.len(), 4);
        let mut company_types = HashSet::new();
        let mut urls = HashSet::new();
        let mut calls = 0;
        for company in companies {
            assert!(company_types.insert(company["company_type"].as_str().unwrap()));
            assert!(!company["thesis"].as_str().unwrap_or_default().is_empty());
            assert!(
                company["focus_metrics"]
                    .as_array()
                    .is_some_and(|values| values.len() >= 4)
            );
            assert_eq!(company["seed_questions"].as_array().map(Vec::len), Some(2));
            let company_calls = company["calls"].as_array().expect("calls");
            assert_eq!(company_calls.len(), 2);
            for call in company_calls {
                let format = call["format"].as_str().expect("format");
                assert!(matches!(format, "pdf" | "docx"));
                let url = call["url"].as_str().expect("url");
                assert!(url.starts_with("https://"));
                assert!(urls.insert(url), "duplicate URL {url}");
                assert!(call.get("body").is_none());
                assert!(call.get("transcript").is_none());
                calls += 1;
            }
        }
        assert_eq!(company_types.len(), 4);
        assert_eq!(calls, 8);
    }

    #[tokio::test]
    async fn reviewer_separates_prepared_remarks_from_qa() {
        let provider = Arc::new(StubProvider {
            response: serde_json::json!({
                "source_scope": "prepared_and_qa",
                "management_tone": "more_confident",
                "prepared_findings": [{
                    "topic": "订单",
                    "finding_zh": "数据中心订单增长",
                    "evidence_zh": "订单创季度新高",
                    "speaker": "CFO"
                }],
                "qa_findings": [{
                    "topic": "持续性",
                    "question_zh": "订单是否可持续",
                    "answer_quality": "partial",
                    "answer_zh": "确认近期需求但未给出下半年规模",
                    "evidence_zh": "能见度只覆盖下一季度"
                }],
                "commitments": [],
                "contradictions": [],
                "unresolved_questions": ["下半年订单规模"]
            })
            .to_string(),
            prompt: Mutex::new(String::new()),
        });
        let reviewer = LlmEarningsTranscriptReviewer::new(provider.clone(), "test")
            .with_system_prompt(DEFAULT_EARNINGS_TRANSCRIPT_SYSTEM_PROMPT);
        let review = reviewer
            .review(&event(), &"full transcript ".repeat(200))
            .await
            .expect("review");
        assert_eq!(review.qa_findings[0].answer_quality, "partial");
        let prompt = provider.prompt.lock().unwrap();
        assert!(prompt.contains("prepared remarks"));
        assert!(prompt.contains("analyst Q&A"));
        assert!(prompt.contains("Full transcript"));
        assert!(prompt.contains("operating_kpi_claims 必须输出空数组"));
    }

    #[test]
    fn transcript_prompt_is_scoped_to_the_issuer_model() {
        let mut event = event();
        event.symbols = vec!["SNDK".into()];
        let messages = build_review_messages("base", &event, "transcript");
        let system = messages[0].content.as_deref().unwrap();
        assert!(system.contains("nand_asp_change"));
        assert!(!system.contains("token_or_call_volume"));
    }

    #[test]
    fn apply_review_writes_compact_fact_object_without_source_body() {
        let mut event = event();
        let review = EarningsTranscriptReview {
            source_scope: "prepared_and_qa".into(),
            management_tone: "mixed".into(),
            prepared_findings: vec![TranscriptPreparedFinding {
                topic: "增长".into(),
                finding_zh: "企业订单改善".into(),
                evidence_zh: "订单同比增长".into(),
                speaker: "CEO".into(),
            }],
            qa_findings: vec![TranscriptQaFinding {
                topic: "利润率".into(),
                question_zh: "利润率何时恢复".into(),
                answer_quality: "evaded".into(),
                answer_zh: "未给出恢复时间".into(),
                evidence_zh: "仅重申长期目标".into(),
            }],
            commitments: vec![],
            claims: vec![],
            operating_kpi_claims: vec![],
            contradictions: vec![],
            unresolved_questions: vec!["利润率恢复时间".into()],
        };
        assert!(apply_earnings_transcript_review(&mut event, review, 42_000));
        assert_eq!(event.severity, Severity::Medium);
        assert!(event.summary.contains("回避"));
        assert!(event.summary.contains("仍未回答"));
        assert_eq!(
            event
                .payload
                .get("earnings_transcript_source_chars")
                .and_then(Value::as_u64),
            Some(42_000)
        );
        assert!(!event.payload.to_string().contains("full transcript"));
    }

    #[test]
    fn applied_storage_review_yields_only_a_verbatim_training_kpi_claim() {
        let mut event = event();
        event.symbols = vec!["SNDK".into()];
        event.title = "SNDK FY2026 Q4 earnings call transcript".into();
        event.url = Some("https://investor.sandisk.com/fy2026-q4-transcript".into());
        let review = EarningsTranscriptReview {
            source_scope: "prepared_and_qa".into(),
            management_tone: "mixed".into(),
            prepared_findings: vec![TranscriptPreparedFinding {
                topic: "NAND ASP".into(),
                finding_zh: "公司披露 NAND 售价环比变化".into(),
                evidence_zh: "NAND average selling price increased 15% sequentially".into(),
                speaker: "CFO".into(),
            }],
            qa_findings: vec![],
            commitments: vec![],
            claims: vec![],
            operating_kpi_claims: vec![OperatingKpiClaimInput {
                claim_kind: OperatingKpiClaimKind::ReportedFact,
                kpi_id: "nand_asp_change".into(),
                issuer_metric_name: "NAND ASP".into(),
                issuer_definition: "NAND average selling price".into(),
                period: "FY2026 Q4".into(),
                numeric_value: Some(15.0),
                unit: "%".into(),
                value_text: "NAND average selling price increased 15% sequentially".into(),
                measurement_scope: "company NAND products; sequential quarter".into(),
                comparison_basis: OperatingKpiComparisonBasis::SequentialQuarter,
                speaker: "CFO".into(),
                evidence_quote: "NAND average selling price increased 15% sequentially".into(),
                source_locator: "prepared remarks · CFO".into(),
                definition_changed: false,
                disposition: EarningsClaimDisposition::Active,
            }],
            contradictions: vec![],
            unresolved_questions: vec![],
        };

        let source =
            "CFO prepared remarks: NAND ASP. NAND average selling price increased 15% sequentially";
        assert!(apply_earnings_transcript_review_with_source(
            &mut event, review, source, 42_000
        ));
        let claims = crate::operating_kpi_claims_from_event(&event);
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].symbol, "SNDK");
        assert_eq!(claims[0].kpi_id, "nand_asp_change");
        assert_eq!(claims[0].policy_status, OPERATING_KPI_POLICY_STATUS);
        assert_eq!(claims[0].issuer_definition, "NAND average selling price");
    }

    #[tokio::test]
    async fn reviewer_rejects_snippets_that_are_not_full_transcripts() {
        let provider = Arc::new(StubProvider {
            response: "{}".into(),
            prompt: Mutex::new(String::new()),
        });
        let reviewer = LlmEarningsTranscriptReviewer::new(provider.clone(), "test");
        assert!(reviewer.review(&event(), "short summary").await.is_none());
        assert!(provider.prompt.lock().unwrap().is_empty());
    }
}
