//! 问卷回收存储 — cloud 模式写 PG `survey_responses`，local 模式追加 JSONL。
//!
//! 问卷是**无登录**入口，因此这里有两条硬约束：
//! 1. 永远不落原始 IP / UA，只落盐化摘要，够用来发现同一台机器灌数据，
//!    不足以反推回具体的人。
//! 2. 答案本体是不透明 JSON。题库只在前端定义，后端只做白名单与长度校验，
//!    这样改题目不需要跟着改后端和数据库。

use hone_core::cloud_runtime::{CloudPgRuntime, CloudSurveyResponseRecord};
use hone_core::{HoneError, HoneResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

/// 当前投放中的问卷。换一版问卷时改这个 id，历史回收数据不会被混进新版统计。
pub const ACTIVE_SURVEY_ID: &str = "hone-user-research-2026-08";

/// 同一客户端摘要在该窗口内允许的提交次数。进程内限流器重启就清零、也不跨副本，
/// 真正兜住重复灌入的是这条落库计数。
pub const SURVEY_CLIENT_WINDOW_HOURS: i32 = 24;
pub const SURVEY_CLIENT_WINDOW_LIMIT: i64 = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyResponse {
    #[serde(default)]
    pub response_id: i64,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub answers: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    #[serde(default)]
    pub submitted_at: String,
}

impl From<CloudSurveyResponseRecord> for SurveyResponse {
    fn from(record: CloudSurveyResponseRecord) -> Self {
        Self {
            response_id: record.response_id,
            locale: record.locale,
            answers: record.answers,
            contact: record.contact,
            submitted_at: record.submitted_at,
        }
    }
}

/// 落盘时才带上的字段。`client_digest` 不进 [`SurveyResponse`]，所以任何把回收
/// 结果读出来展示的路径都不可能顺手把它带到前端。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredSurveyResponse {
    #[serde(flatten)]
    response: SurveyResponse,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    client_digest: Option<String>,
}

static CLOUD_SURVEY_STORAGE: OnceLock<RwLock<Option<CloudPgRuntime>>> = OnceLock::new();

pub fn configure_cloud_survey_storage(postgres: Option<CloudPgRuntime>) {
    let lock = CLOUD_SURVEY_STORAGE.get_or_init(|| RwLock::new(None));
    match lock.write() {
        Ok(mut guard) => *guard = postgres,
        Err(error) => tracing::warn!("survey cloud runtime lock poisoned: {error}"),
    }
}

fn cloud_survey_storage() -> Option<CloudPgRuntime> {
    CLOUD_SURVEY_STORAGE
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()))
}

pub struct SurveyStorage {
    data_dir: PathBuf,
    cloud: Option<CloudPgRuntime>,
}

impl SurveyStorage {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let dir = data_dir.as_ref().to_path_buf();
        let cloud = cloud_survey_storage();
        if cloud.is_none() {
            std::fs::create_dir_all(&dir).ok();
        }
        Self {
            data_dir: dir,
            cloud,
        }
    }

    fn log_path(&self, survey_id: &str) -> PathBuf {
        self.data_dir
            .join(format!("survey_{}.jsonl", sanitize_survey_id(survey_id)))
    }

    pub async fn submit(
        &self,
        survey_id: &str,
        locale: &str,
        answers: &Value,
        contact: Option<&str>,
        client_digest: Option<&str>,
    ) -> HoneResult<i64> {
        if let Some(postgres) = self.cloud.clone() {
            return postgres
                .insert_survey_response(survey_id, locale, answers, contact, client_digest)
                .await;
        }
        self.append_local(survey_id, locale, answers, contact, client_digest)
    }

    fn append_local(
        &self,
        survey_id: &str,
        locale: &str,
        answers: &Value,
        contact: Option<&str>,
        client_digest: Option<&str>,
    ) -> HoneResult<i64> {
        let path = self.log_path(survey_id);
        // Line count is the id in local mode. It is only ever used for display
        // ordering, so a gap after a manual edit is harmless.
        let response_id = self.read_local(survey_id, usize::MAX)?.len() as i64 + 1;
        let record = StoredSurveyResponse {
            response: SurveyResponse {
                response_id,
                locale: locale.to_string(),
                answers: answers.clone(),
                contact: contact.map(str::to_string),
                submitted_at: now_rfc3339(),
            },
            client_digest: client_digest.map(str::to_string),
        };
        let line = serde_json::to_string(&record)
            .map_err(|err| HoneError::Config(format!("问卷序列化失败: {err}")))?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|err| HoneError::Config(format!("问卷写入失败: {err}")))?;
        writeln!(file, "{line}")
            .map_err(|err| HoneError::Config(format!("问卷写入失败: {err}")))?;
        Ok(response_id)
    }

    fn read_local(&self, survey_id: &str, limit: usize) -> HoneResult<Vec<StoredSurveyResponse>> {
        let path = self.log_path(survey_id);
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Ok(Vec::new());
        };
        let mut records = raw
            .lines()
            .filter(|line| !line.trim().is_empty())
            // A malformed line is dropped rather than failing the whole read:
            // one bad append must not make the rest of the responses
            // unreadable.
            .filter_map(|line| serde_json::from_str::<StoredSurveyResponse>(line).ok())
            .collect::<Vec<_>>();
        records.reverse();
        records.truncate(limit);
        Ok(records)
    }

    pub async fn list(&self, survey_id: &str, limit: usize) -> HoneResult<Vec<SurveyResponse>> {
        if let Some(postgres) = self.cloud.clone() {
            let records = postgres.list_survey_responses(survey_id, limit).await?;
            return Ok(records.into_iter().map(SurveyResponse::from).collect());
        }
        Ok(self
            .read_local(survey_id, limit)?
            .into_iter()
            .map(|record| record.response)
            .collect())
    }

    /// `true` 表示该客户端摘要在窗口内的提交次数已达上限。
    pub async fn client_window_exhausted(
        &self,
        survey_id: &str,
        client_digest: &str,
    ) -> HoneResult<bool> {
        if let Some(postgres) = self.cloud.clone() {
            let count = postgres
                .count_recent_survey_responses(survey_id, client_digest, SURVEY_CLIENT_WINDOW_HOURS)
                .await?;
            return Ok(count >= SURVEY_CLIENT_WINDOW_LIMIT);
        }
        let cutoff = now_rfc3339();
        let count = self
            .read_local(survey_id, usize::MAX)?
            .into_iter()
            .filter(|record| record.client_digest.as_deref() == Some(client_digest))
            .filter(|record| within_recent_hours(&record.response.submitted_at, &cutoff))
            .count() as i64;
        Ok(count >= SURVEY_CLIENT_WINDOW_LIMIT)
    }
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// The id reaches a filename, so it is restricted to characters that cannot
/// escape the data directory.
fn sanitize_survey_id(survey_id: &str) -> String {
    let cleaned = survey_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect::<String>();
    if cleaned.is_empty() {
        "survey".to_string()
    } else {
        cleaned
    }
}

/// Local mode compares RFC3339 strings rather than parsing: both timestamps are
/// produced by the same UTC formatter, so lexical order is chronological order.
fn within_recent_hours(submitted_at: &str, now: &str) -> bool {
    let (Some(submitted), Some(now)) = (
        chrono::DateTime::parse_from_rfc3339(submitted_at).ok(),
        chrono::DateTime::parse_from_rfc3339(now).ok(),
    ) else {
        return false;
    };
    now.signed_duration_since(submitted).num_hours() < i64::from(SURVEY_CLIENT_WINDOW_HOURS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hone_survey_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[tokio::test]
    async fn local_submissions_round_trip_newest_first() {
        let dir = temp_dir("roundtrip");
        let storage = SurveyStorage::new(&dir);

        storage
            .submit(ACTIVE_SURVEY_ID, "zh", &json!({"q1": "weekly"}), None, None)
            .await
            .expect("first submit");
        storage
            .submit(
                ACTIVE_SURVEY_ID,
                "en",
                &json!({"q1": "daily"}),
                Some("a@example.com"),
                None,
            )
            .await
            .expect("second submit");

        let listed = storage.list(ACTIVE_SURVEY_ID, 10).await.expect("list");
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].locale, "en");
        assert_eq!(listed[0].contact.as_deref(), Some("a@example.com"));
        assert_eq!(listed[1].answers["q1"], "weekly");
        let _ = std::fs::remove_dir_all(dir);
    }

    /// The digest is a rate-limiting input, never something a reader can see.
    #[tokio::test]
    async fn listed_responses_never_expose_the_client_digest() {
        let dir = temp_dir("digest");
        let storage = SurveyStorage::new(&dir);
        storage
            .submit(
                ACTIVE_SURVEY_ID,
                "zh",
                &json!({"q1": "daily"}),
                None,
                Some("digest-abc"),
            )
            .await
            .expect("submit");

        let listed = storage.list(ACTIVE_SURVEY_ID, 10).await.expect("list");
        let serialized = serde_json::to_string(&listed).expect("serialize");
        assert!(!serialized.contains("digest-abc"), "{serialized}");
        assert!(!serialized.contains("client_digest"), "{serialized}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn one_client_cannot_flood_the_form() {
        let dir = temp_dir("flood");
        let storage = SurveyStorage::new(&dir);
        for _ in 0..SURVEY_CLIENT_WINDOW_LIMIT {
            assert!(
                !storage
                    .client_window_exhausted(ACTIVE_SURVEY_ID, "same-client")
                    .await
                    .expect("check"),
            );
            storage
                .submit(
                    ACTIVE_SURVEY_ID,
                    "zh",
                    &json!({"q1": "daily"}),
                    None,
                    Some("same-client"),
                )
                .await
                .expect("submit");
        }
        assert!(
            storage
                .client_window_exhausted(ACTIVE_SURVEY_ID, "same-client")
                .await
                .expect("check")
        );
        // A different client is unaffected by the first one's quota.
        assert!(
            !storage
                .client_window_exhausted(ACTIVE_SURVEY_ID, "other-client")
                .await
                .expect("check")
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn survey_ids_cannot_escape_the_data_directory() {
        assert_eq!(sanitize_survey_id("../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_survey_id(""), "survey");
        assert_eq!(sanitize_survey_id("hone-2026_08"), "hone-2026_08");
    }
}
