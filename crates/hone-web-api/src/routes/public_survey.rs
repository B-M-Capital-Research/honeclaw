//! 用户调研问卷 — 公开提交（无登录）+ 管理员读取。
//!
//! 设计取舍：题库只在前端定义，后端把答案当作不透明 JSON 存。后端只负责
//! **结构**校验（键的形状、条数、长度），不认识任何一个选项文本。这样改题、
//! 加题、调整选项都不需要动后端和数据库；代价是后端无法拒绝一个前端没发过的
//! 键，所以键名有白名单字符集与数量上限兜底。

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use crate::public_auth::PublicAuthLimitStatus;
use crate::state::AppState;
use hone_memory::{ACTIVE_SURVEY_ID, SurveyResponse, SurveyStorage};

/// 单份问卷的上限。放得比真实题量宽，但不足以被当成免费的 KV 存储。
const MAX_QUESTIONS: usize = 40;
const MAX_QUESTION_KEY_CHARS: usize = 48;
const MAX_CHOICES_PER_QUESTION: usize = 20;
const MAX_CHOICE_CHARS: usize = 120;
/// 开放题。够写清楚一个具体诉求，不够拿来传文件。
const MAX_TEXT_CHARS: usize = 2_000;
const MAX_CONTACT_CHARS: usize = 120;
const MAX_ADMIN_PAGE: usize = 1_000;

#[derive(Debug, Deserialize)]
pub(crate) struct PublicSurveyRequest {
    #[serde(default)]
    pub survey_id: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub answers: Map<String, Value>,
    #[serde(default)]
    pub contact: Option<String>,
}

#[derive(Debug, Serialize)]
struct PublicSurveySubmitResponse {
    ok: bool,
    response_id: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AdminSurveyQuery {
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    survey_id: Option<String>,
}

pub(crate) async fn handle_submit_survey(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<PublicSurveyRequest>,
) -> Response {
    let client_key = crate::routes::public::public_client_key(&headers);
    if let PublicAuthLimitStatus::Blocked { retry_after_secs } =
        state.public_auth_limiter.consume_survey_submit(&client_key)
    {
        return crate::routes::json_error(
            StatusCode::TOO_MANY_REQUESTS,
            format!("提交过于频繁，请在 {retry_after_secs} 秒后重试"),
        );
    }

    let survey_id = request
        .survey_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(ACTIVE_SURVEY_ID)
        .to_string();
    let locale = match request.locale.as_deref().map(str::trim) {
        Some("en") => "en",
        _ => "zh",
    };

    let answers = match sanitize_answers(&request.answers) {
        Ok(answers) => answers,
        Err(message) => return crate::routes::json_error(StatusCode::BAD_REQUEST, message),
    };
    if answers.is_empty() {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "问卷内容为空，请至少回答一题");
    }
    let contact = request
        .contact
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_chars(value, MAX_CONTACT_CHARS));

    let digest = client_digest(&client_key);
    let storage = SurveyStorage::new(crate::routes::public::public_survey_data_dir());
    match storage.client_window_exhausted(&survey_id, &digest).await {
        Ok(true) => {
            return crate::routes::json_error(
                StatusCode::TOO_MANY_REQUESTS,
                "这台设备今天已经提交过问卷了，感谢你的反馈",
            );
        }
        Ok(false) => {}
        // A counting failure must not eat a real response: the in-process
        // limiter above already ran, so degrade to accepting rather than
        // rejecting a user who did nothing wrong.
        Err(error) => tracing::warn!("survey duplicate check failed: {error}"),
    }

    match storage
        .submit(
            &survey_id,
            locale,
            &Value::Object(answers),
            contact.as_deref(),
            Some(&digest),
        )
        .await
    {
        Ok(response_id) => (
            StatusCode::OK,
            Json(PublicSurveySubmitResponse {
                ok: true,
                response_id,
            }),
        )
            .into_response(),
        Err(error) => {
            tracing::error!("survey submit failed: {error}");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "问卷保存失败，请稍后重试",
            )
        }
    }
}

pub(crate) async fn handle_admin_survey_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AdminSurveyQuery>,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    let survey_id = query
        .survey_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(ACTIVE_SURVEY_ID)
        .to_string();
    let limit = query
        .limit
        .unwrap_or(MAX_ADMIN_PAGE)
        .clamp(1, MAX_ADMIN_PAGE);

    let storage = SurveyStorage::new(crate::routes::public::public_survey_data_dir());
    match storage.list(&survey_id, limit).await {
        Ok(responses) => {
            let summary = aggregate(&responses);
            (
                StatusCode::OK,
                Json(json!({
                    "survey_id": survey_id,
                    "total": responses.len(),
                    "summary": summary,
                    "responses": responses,
                })),
            )
                .into_response()
        }
        Err(error) => crate::routes::json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("问卷读取失败: {error}"),
        ),
    }
}

/// Per-question option counts. Free-text answers are counted but never
/// aggregated into buckets — every one of them is a distinct sentence, and a
/// frequency table over sentences is noise.
fn aggregate(responses: &[SurveyResponse]) -> Value {
    let mut choice_counts: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut text_counts: BTreeMap<String, usize> = BTreeMap::new();
    for response in responses {
        let Some(answers) = response.answers.as_object() else {
            continue;
        };
        for (question, value) in answers {
            match value {
                Value::String(text) if !text.trim().is_empty() => {
                    *text_counts.entry(question.clone()).or_default() += 1;
                }
                Value::Array(choices) => {
                    let bucket = choice_counts.entry(question.clone()).or_default();
                    for choice in choices.iter().filter_map(Value::as_str) {
                        *bucket.entry(choice.to_string()).or_default() += 1;
                    }
                }
                _ => {}
            }
        }
    }
    json!({
        "choice_counts": choice_counts,
        "text_answer_counts": text_counts,
    })
}

/// Structural validation only. Single choice and free text arrive as strings,
/// multi-select as an array of strings; anything else is rejected rather than
/// silently coerced, so a malformed client cannot quietly poison the dataset.
fn sanitize_answers(answers: &Map<String, Value>) -> Result<Map<String, Value>, String> {
    if answers.len() > MAX_QUESTIONS {
        return Err(format!("题目数量超过上限（最多 {MAX_QUESTIONS} 题）"));
    }
    let mut sanitized = Map::new();
    for (question, value) in answers {
        let key = question.trim();
        if key.is_empty() || key.chars().count() > MAX_QUESTION_KEY_CHARS {
            return Err("题目标识不合法".to_string());
        }
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err("题目标识只允许字母、数字、下划线和连字符".to_string());
        }
        match value {
            Value::Null => {}
            Value::String(text) => {
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                sanitized.insert(
                    key.to_string(),
                    Value::String(truncate_chars(text, MAX_TEXT_CHARS)),
                );
            }
            Value::Array(items) => {
                if items.len() > MAX_CHOICES_PER_QUESTION {
                    return Err("单题选项数量超过上限".to_string());
                }
                let choices = items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|choice| !choice.is_empty())
                    .map(|choice| Value::String(truncate_chars(choice, MAX_CHOICE_CHARS)))
                    .collect::<Vec<_>>();
                if choices.is_empty() {
                    continue;
                }
                sanitized.insert(key.to_string(), Value::Array(choices));
            }
            _ => return Err("答案格式不合法".to_string()),
        }
    }
    Ok(sanitized)
}

fn truncate_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

/// A per-deployment secret salt. An unsalted hash of an IP is not anonymous —
/// the whole IPv4 space can be enumerated in seconds — so the salt has to be
/// secret and it has to survive restarts, otherwise the durable duplicate
/// check silently stops matching. `HONE_SURVEY_DIGEST_SALT` wins when set;
/// otherwise one is generated once and kept beside the responses.
fn survey_digest_salt() -> &'static str {
    static SALT: OnceLock<String> = OnceLock::new();
    SALT.get_or_init(|| {
        if let Ok(configured) = std::env::var("HONE_SURVEY_DIGEST_SALT")
            && !configured.trim().is_empty()
        {
            return configured;
        }
        let dir = crate::routes::public::public_survey_data_dir();
        let path = dir.join(".digest-salt");
        if let Ok(existing) = std::fs::read_to_string(&path)
            && !existing.trim().is_empty()
        {
            return existing.trim().to_string();
        }
        let generated = uuid::Uuid::new_v4().to_string();
        let _ = std::fs::create_dir_all(&dir);
        if let Err(error) = std::fs::write(&path, &generated) {
            // A salt that cannot be persisted still protects this process; it
            // only means the duplicate window resets on restart.
            tracing::warn!("survey digest salt could not be persisted: {error}");
        }
        generated
    })
}

/// Salted digest of the client key. What lands in storage cannot be walked
/// back to an address without the deployment's salt.
fn client_digest(client_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(survey_digest_salt().as_bytes());
    hasher.update(b":");
    hasher.update(client_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_choice_multi_choice_and_free_text_are_all_accepted() {
        let mut answers = Map::new();
        answers.insert("q1".into(), json!("weekly"));
        answers.insert("q2".into(), json!(["fundamentals", "supply_chain"]));
        answers.insert("q11".into(), json!("  希望能持续跟踪某家公司  "));

        let sanitized = sanitize_answers(&answers).expect("valid");

        assert_eq!(sanitized["q1"], json!("weekly"));
        assert_eq!(sanitized["q2"], json!(["fundamentals", "supply_chain"]));
        // Trimmed, so a stray space does not become part of the answer.
        assert_eq!(sanitized["q11"], json!("希望能持续跟踪某家公司"));
    }

    #[test]
    fn empty_answers_are_dropped_rather_than_stored_as_blanks() {
        let mut answers = Map::new();
        answers.insert("q1".into(), json!("   "));
        answers.insert("q2".into(), json!([]));
        answers.insert("q3".into(), Value::Null);
        answers.insert("q4".into(), json!(["ai"]));

        let sanitized = sanitize_answers(&answers).expect("valid");

        assert_eq!(sanitized.len(), 1);
        assert_eq!(sanitized["q4"], json!(["ai"]));
    }

    #[test]
    fn a_malformed_client_cannot_poison_the_dataset() {
        let reject = |key: &str, value: Value| {
            let mut answers = Map::new();
            answers.insert(key.into(), value);
            sanitize_answers(&answers).expect_err("must reject")
        };
        // Numbers and nested objects would break every downstream aggregate.
        reject("q1", json!(42));
        reject("q1", json!({"nested": true}));
        // Keys reach an aggregate map and a JSON column; keep them boring.
        reject("q 1", json!("a"));
        reject("../etc", json!("a"));
        reject(&"q".repeat(MAX_QUESTION_KEY_CHARS + 1), json!("a"));

        let mut too_many = Map::new();
        for index in 0..=MAX_QUESTIONS {
            too_many.insert(format!("q{index}"), json!("a"));
        }
        sanitize_answers(&too_many).expect_err("question cap");

        let mut too_many_choices = Map::new();
        too_many_choices.insert(
            "q1".into(),
            Value::Array(vec![json!("a"); MAX_CHOICES_PER_QUESTION + 1]),
        );
        sanitize_answers(&too_many_choices).expect_err("choice cap");
    }

    #[test]
    fn oversized_free_text_is_truncated_not_rejected() {
        // Someone writing a long, genuine answer should not lose it entirely.
        let mut answers = Map::new();
        answers.insert("q11".into(), json!("好".repeat(MAX_TEXT_CHARS + 500)));

        let sanitized = sanitize_answers(&answers).expect("valid");

        assert_eq!(
            sanitized["q11"].as_str().expect("text").chars().count(),
            MAX_TEXT_CHARS
        );
    }

    #[test]
    fn aggregate_counts_choices_per_question_and_leaves_prose_alone() {
        let responses = vec![
            SurveyResponse {
                response_id: 1,
                locale: "zh".into(),
                answers: json!({"q2": ["fundamentals", "sector"], "q11": "跟踪 NVDA"}),
                contact: None,
                submitted_at: "2026-08-06T00:00:00Z".into(),
            },
            SurveyResponse {
                response_id: 2,
                locale: "zh".into(),
                answers: json!({"q2": ["fundamentals"], "q11": "总结财报"}),
                contact: None,
                submitted_at: "2026-08-06T01:00:00Z".into(),
            },
        ];

        let summary = aggregate(&responses);

        assert_eq!(summary["choice_counts"]["q2"]["fundamentals"], 2);
        assert_eq!(summary["choice_counts"]["q2"]["sector"], 1);
        // Prose is counted, never bucketed by its text.
        assert_eq!(summary["text_answer_counts"]["q11"], 2);
        assert!(summary["choice_counts"].get("q11").is_none());
    }
}
