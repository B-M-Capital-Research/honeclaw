//! 管理端"推送日志"路由 — `GET /api/admin/notifications`。
//!
//! 合并两类推送审计:
//! - cron 定时任务执行记录(SQLite `cron_job_runs`)
//! - event-engine 主动推送出口记录(SQLite `delivery_log`)
//!
//! 这样管理端能同时排查自定义 cron 任务、Discord/Telegram/Feishu/iMessage
//! 上真实送达的事件推送、静音 hold、偏好过滤、digest 排队与失败。

use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use chrono::{DateTime, Duration as ChronoDuration, TimeZone, Timelike, Utc};
use hone_event_engine::{DeliveryLogFilter, DeliveryLogRecord, EventStore};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use hone_core::beijing_offset;
use hone_memory::cron_job::{CronJobExecutionRecord, ExecutionFilter};

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 1000;
const EVENT_LOG_FETCH_LIMIT: usize = MAX_LIMIT * 5;
const HISTOGRAM_HOURS: i64 = 24;

#[derive(Deserialize)]
pub(crate) struct NotificationsQuery {
    /// 起始时间(东八区 RFC3339)。缺省 = 24 小时前。
    since: Option<String>,
    /// 终止时间(东八区 RFC3339)。缺省 = 现在。
    until: Option<String>,
    channel: Option<String>,
    user_id: Option<String>,
    channel_scope: Option<String>,
    job_id: Option<String>,
    /// 执行状态:`completed` / `noop` / `execution_failed`
    execution_status: Option<String>,
    /// 发送状态。cron 使用 `message_send_status`; event-engine 使用原始
    /// `delivery_log.status`。
    message_send_status: Option<String>,
    heartbeat_only: Option<bool>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NotificationRecord {
    run_id: i64,
    record_source: String,
    job_id: String,
    job_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    event_kind: Option<String>,
    channel: String,
    user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    channel_scope: Option<String>,
    channel_target: String,
    heartbeat: bool,
    executed_at: String,
    execution_status: String,
    message_send_status: String,
    should_deliver: bool,
    delivered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    response_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_message: Option<String>,
    #[serde(default)]
    detail: Value,
}

#[derive(Serialize)]
pub(crate) struct NotificationsResponse {
    /// 满足过滤条件的执行记录,按 executed_at 倒序。
    records: Vec<NotificationRecord>,
    /// 24h 按小时分桶的直方图(始终覆盖最近 24 小时,不受 since/until 影响),
    /// 让前端能直接画出推送频率。
    histogram_24h: Vec<HistogramBucket>,
    /// 同窗口下的合计指标,顶部数字卡片直接用。
    summary_24h: NotificationsSummary,
}

#[derive(Serialize)]
pub(crate) struct HistogramBucket {
    /// 桶的开始时刻(东八区 RFC3339,小时对齐)。
    bucket_start: String,
    total: u32,
    sent: u32,
    failed: u32,
    skipped: u32,
}

#[derive(Serialize, Default)]
pub(crate) struct NotificationsSummary {
    total: u32,
    sent: u32,
    failed: u32,
    skipped: u32,
    duplicate_suppressed: u32,
    distinct_users: u32,
}

pub(crate) async fn handle_notifications(
    State(state): State<Arc<AppState>>,
    Query(q): Query<NotificationsQuery>,
) -> Json<NotificationsResponse> {
    let storage = state.core.cron_job_storage();

    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let now_bj = Utc::now().with_timezone(&beijing_offset());
    let default_since = (now_bj - ChronoDuration::hours(HISTOGRAM_HOURS)).to_rfc3339();
    let since = q.since.clone().unwrap_or(default_since);

    let cron_filter = ExecutionFilter {
        since: Some(since.clone()),
        until: q.until.clone(),
        channel: q.channel.clone(),
        user_id: q.user_id.clone(),
        job_id: q.job_id.clone(),
        execution_status: q.execution_status.clone(),
        message_send_status: q.message_send_status.clone(),
        heartbeat_only: q.heartbeat_only,
        limit,
    };
    let mut records: Vec<NotificationRecord> = storage
        .list_recent_executions(&cron_filter)
        .unwrap_or_default()
        .into_iter()
        .map(record_from_cron)
        .collect();

    if q.heartbeat_only != Some(true) {
        records.extend(list_event_delivery_records(&state, &q, &since, limit));
    }

    records.retain(|record| record_matches_query(record, &q));
    records.sort_by(|a, b| compare_records_desc(a, b));
    records.truncate(limit);

    // 24h 直方图 / summary:固定窗口 24h、不受 status/user/channel 过滤影响,
    // 这样直方图反映的是"全局节奏",而不是某次过滤后的子集——更符合排查直觉。
    let histogram_since = (now_bj - ChronoDuration::hours(HISTOGRAM_HOURS)).to_rfc3339();
    let histogram_filter = ExecutionFilter {
        since: Some(histogram_since.clone()),
        limit: MAX_LIMIT,
        ..ExecutionFilter::default()
    };
    let mut histogram_records: Vec<NotificationRecord> = storage
        .list_recent_executions(&histogram_filter)
        .unwrap_or_default()
        .into_iter()
        .map(record_from_cron)
        .collect();
    histogram_records.extend(list_event_delivery_records(
        &state,
        &NotificationsQuery {
            since: Some(histogram_since),
            until: None,
            channel: None,
            user_id: None,
            channel_scope: None,
            job_id: None,
            execution_status: None,
            message_send_status: None,
            heartbeat_only: None,
            limit: Some(EVENT_LOG_FETCH_LIMIT),
        },
        "",
        EVENT_LOG_FETCH_LIMIT,
    ));

    let histogram_24h = build_histogram(&histogram_records, now_bj);
    let summary_24h = build_summary(&histogram_records);

    Json(NotificationsResponse {
        records,
        histogram_24h,
        summary_24h,
    })
}

fn list_event_delivery_records(
    state: &AppState,
    q: &NotificationsQuery,
    since: &str,
    limit: usize,
) -> Vec<NotificationRecord> {
    let path = event_store_path(state);
    if !path.exists() {
        return Vec::new();
    }
    let Ok(store) = EventStore::open(&path) else {
        return Vec::new();
    };
    let effective_since = if since.trim().is_empty() {
        q.since.as_deref().unwrap_or("")
    } else {
        since
    };
    let filter = DeliveryLogFilter {
        since_ts: parse_ts(effective_since),
        until_ts: q.until.as_deref().and_then(parse_ts),
        actor: exact_actor_key(q),
        actor_channel: q.channel.clone(),
        actor_user_id: q.user_id.clone(),
        event_id: q.job_id.clone(),
        status: None,
        delivery_channel: None,
        top_level_only: true,
        limit: limit.max(EVENT_LOG_FETCH_LIMIT),
    };
    store
        .list_recent_delivery_logs(&filter)
        .unwrap_or_default()
        .into_iter()
        .map(record_from_delivery)
        .collect()
}

fn event_store_path(state: &AppState) -> PathBuf {
    let session_db = PathBuf::from(&state.core.config.storage.session_sqlite_db_path);
    let base = session_db
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./data"));
    base.join("events.sqlite3")
}

fn exact_actor_key(q: &NotificationsQuery) -> Option<String> {
    let channel = q.channel.as_deref()?.trim();
    let user_id = q.user_id.as_deref()?.trim();
    if channel.is_empty() || user_id.is_empty() {
        return None;
    }
    let scope = q.channel_scope.as_deref().unwrap_or("").trim();
    Some(format!("{channel}::{scope}::{user_id}"))
}

fn parse_ts(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.timestamp())
}

fn record_from_cron(record: CronJobExecutionRecord) -> NotificationRecord {
    NotificationRecord {
        run_id: record.run_id,
        record_source: "cron_job".to_string(),
        job_id: record.job_id,
        job_name: record.job_name,
        event_kind: None,
        channel: record.channel,
        user_id: record.user_id,
        channel_scope: record.channel_scope,
        channel_target: record.channel_target,
        heartbeat: record.heartbeat,
        executed_at: record.executed_at,
        execution_status: record.execution_status,
        message_send_status: record.message_send_status,
        should_deliver: record.should_deliver,
        delivered: record.delivered,
        response_preview: record.response_preview,
        error_message: record.error_message,
        detail: record.detail,
    }
}

fn record_from_delivery(record: DeliveryLogRecord) -> NotificationRecord {
    let actor = parse_actor_key(&record.actor);
    let delivered = matches!(record.status.as_str(), "sent" | "dryrun");
    let execution_status = match record.status.as_str() {
        "failed" => "execution_failed",
        "sent" | "dryrun" => "completed",
        _ => "noop",
    };
    let job_name = record
        .event_title
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| event_log_fallback_name(&record));
    let preview = record
        .body
        .clone()
        .or_else(|| record.event_summary.clone())
        .or_else(|| record.event_title.clone())
        .map(|value| truncate_chars(&value, 240));

    NotificationRecord {
        run_id: record.id,
        record_source: "event_engine".to_string(),
        job_id: record.event_id.clone(),
        job_name,
        event_kind: record.event_kind.clone(),
        channel: actor
            .as_ref()
            .map(|value| value.channel.clone())
            .unwrap_or_else(|| record.actor.clone()),
        user_id: actor
            .as_ref()
            .map(|value| value.user_id.clone())
            .unwrap_or_else(|| record.actor.clone()),
        channel_scope: actor.and_then(|value| value.channel_scope),
        channel_target: record.channel.clone(),
        heartbeat: false,
        executed_at: Utc
            .timestamp_opt(record.sent_at_ts, 0)
            .single()
            .unwrap_or_else(Utc::now)
            .to_rfc3339(),
        execution_status: execution_status.to_string(),
        message_send_status: record.status.clone(),
        should_deliver: !matches!(record.status.as_str(), "filtered" | "omitted"),
        delivered,
        response_preview: preview,
        error_message: if record.status == "failed" {
            Some("event-engine delivery failed".to_string())
        } else {
            None
        },
        detail: json!({
            "record_source": "event_engine",
            "event_id": record.event_id,
            "actor": record.actor,
            "event_kind": record.event_kind,
            "delivery_channel": record.channel,
            "delivery_status": record.status,
            "severity": record.severity,
            "event_source": record.event_source,
            "symbols": record.event_symbols,
            "title": record.event_title,
            "summary": record.event_summary,
            "url": record.event_url,
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActorParts {
    channel: String,
    user_id: String,
    channel_scope: Option<String>,
}

fn parse_actor_key(key: &str) -> Option<ActorParts> {
    let parts: Vec<&str> = key.splitn(3, "::").collect();
    if parts.len() != 3 {
        return None;
    }
    let channel = parts[0].trim();
    let scope = parts[1].trim();
    let user_id = parts[2].trim();
    if channel.is_empty() || user_id.is_empty() {
        return None;
    }
    Some(ActorParts {
        channel: channel.to_string(),
        user_id: user_id.to_string(),
        channel_scope: if scope.is_empty() {
            None
        } else {
            Some(scope.to_string())
        },
    })
}

fn event_log_fallback_name(record: &DeliveryLogRecord) -> String {
    match record.channel.as_str() {
        "sink" => "事件即时推送".to_string(),
        "digest" => "Digest 推送".to_string(),
        "prefs" => "通知偏好过滤".to_string(),
        other => format!("Event Engine · {other}"),
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    for _ in 0..max_chars {
        let Some(ch) = chars.next() else {
            return out;
        };
        out.push(ch);
    }
    if chars.next().is_some() {
        out.push('…');
    }
    out
}

fn record_matches_query(record: &NotificationRecord, q: &NotificationsQuery) -> bool {
    if let Some(scope) = q.channel_scope.as_deref().filter(|value| !value.is_empty()) {
        if record.channel_scope.as_deref() != Some(scope) {
            return false;
        }
    }
    if let Some(status) = q
        .message_send_status
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        if !message_status_matches(&record.message_send_status, status) {
            return false;
        }
    }
    if let Some(status) = q
        .execution_status
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        if record.execution_status != status {
            return false;
        }
    }
    true
}

fn message_status_matches(actual: &str, expected: &str) -> bool {
    if actual == expected {
        return true;
    }
    match expected {
        "sent" => matches!(actual, "dryrun"),
        "send_failed" => matches!(actual, "failed"),
        "skipped_noop" => matches!(
            actual,
            "queued"
                | "filtered"
                | "quiet_held"
                | "capped"
                | "cooled_down"
                | "price_capped"
                | "price_cooled_down"
                | "omitted"
        ),
        _ => false,
    }
}

fn compare_records_desc(a: &NotificationRecord, b: &NotificationRecord) -> std::cmp::Ordering {
    let at = DateTime::parse_from_rfc3339(&a.executed_at)
        .ok()
        .map(|dt| dt.timestamp())
        .unwrap_or_default();
    let bt = DateTime::parse_from_rfc3339(&b.executed_at)
        .ok()
        .map(|dt| dt.timestamp())
        .unwrap_or_default();
    bt.cmp(&at).then_with(|| b.run_id.cmp(&a.run_id))
}

fn build_histogram(
    records: &[NotificationRecord],
    now_bj: DateTime<chrono::FixedOffset>,
) -> Vec<HistogramBucket> {
    // 用最近 24 小时,以"当前小时"为右端,共 24 个桶(从 23 小时前到现在)。
    let mut buckets: Vec<HistogramBucket> = Vec::with_capacity(HISTOGRAM_HOURS as usize);
    let current_hour = now_bj
        .with_minute(0)
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap_or(now_bj);

    for i in (0..HISTOGRAM_HOURS).rev() {
        let bucket_start = current_hour - ChronoDuration::hours(i);
        buckets.push(HistogramBucket {
            bucket_start: bucket_start.to_rfc3339(),
            total: 0,
            sent: 0,
            failed: 0,
            skipped: 0,
        });
    }

    for record in records {
        let Ok(executed) = DateTime::parse_from_rfc3339(&record.executed_at) else {
            continue;
        };
        let executed_bj = executed.with_timezone(&beijing_offset());
        let diff_hours = (current_hour - executed_bj).num_hours();
        if diff_hours < 0 || diff_hours >= HISTOGRAM_HOURS {
            continue;
        }
        let idx = (HISTOGRAM_HOURS - 1 - diff_hours) as usize;
        if let Some(bucket) = buckets.get_mut(idx) {
            bucket.total += 1;
            classify(record, |kind| match kind {
                "sent" => bucket.sent += 1,
                "failed" => bucket.failed += 1,
                "skipped" => bucket.skipped += 1,
                _ => {}
            });
        }
    }

    buckets
}

fn build_summary(records: &[NotificationRecord]) -> NotificationsSummary {
    use std::collections::HashSet;
    let mut summary = NotificationsSummary::default();
    let mut users: HashSet<(String, String)> = HashSet::new();
    for record in records {
        summary.total += 1;
        users.insert((record.channel.clone(), record.user_id.clone()));
        if record.message_send_status == "duplicate_suppressed" {
            summary.duplicate_suppressed += 1;
        }
        classify(record, |kind| match kind {
            "sent" => summary.sent += 1,
            "failed" => summary.failed += 1,
            "skipped" => summary.skipped += 1,
            _ => {}
        });
    }
    summary.distinct_users = users.len() as u32;
    summary
}

/// 把 (execution_status, message_send_status) 映射成 sent / failed / skipped 三类。
fn classify(record: &NotificationRecord, mut emit: impl FnMut(&str)) {
    let send = record.message_send_status.as_str();
    let exec = record.execution_status.as_str();
    let kind = match send {
        "sent" | "dryrun" => "sent",
        "send_failed" | "failed" | "target_resolution_failed" | "skipped_error" => "failed",
        "duplicate_suppressed"
        | "skipped_noop"
        | "queued"
        | "filtered"
        | "quiet_held"
        | "capped"
        | "cooled_down"
        | "price_capped"
        | "price_cooled_down"
        | "omitted" => "skipped",
        _ => match exec {
            "execution_failed" => "failed",
            "noop" => "skipped",
            _ => "skipped",
        },
    };
    emit(kind);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_actor_key_handles_direct_and_scoped() {
        assert_eq!(
            parse_actor_key("discord::::u1"),
            Some(ActorParts {
                channel: "discord".to_string(),
                user_id: "u1".to_string(),
                channel_scope: None,
            })
        );
        assert_eq!(
            parse_actor_key("discord::guild:channel::u1"),
            Some(ActorParts {
                channel: "discord".to_string(),
                user_id: "u1".to_string(),
                channel_scope: Some("guild:channel".to_string()),
            })
        );
    }

    #[test]
    fn event_delivery_status_can_match_existing_send_failed_filter() {
        assert!(message_status_matches("failed", "send_failed"));
        assert!(message_status_matches("queued", "skipped_noop"));
        assert!(message_status_matches("dryrun", "sent"));
        assert!(!message_status_matches("queued", "sent"));
    }

    #[test]
    fn event_record_exposes_business_event_kind() {
        let record = record_from_delivery(DeliveryLogRecord {
            id: 1,
            event_id: "ev1".to_string(),
            actor: "discord::::u1".to_string(),
            channel: "sink".to_string(),
            severity: "high".to_string(),
            sent_at_ts: 1_700_000_000,
            status: "sent".to_string(),
            body: Some("body".to_string()),
            event_title: Some("title".to_string()),
            event_summary: None,
            event_kind: Some("sec_filing".to_string()),
            event_source: Some("fmp.sec_filings".to_string()),
            event_url: None,
            event_symbols: vec!["AAPL".to_string()],
        });
        assert_eq!(record.event_kind.as_deref(), Some("sec_filing"));
        assert_eq!(
            record
                .detail
                .get("event_kind")
                .and_then(|value| value.as_str()),
            Some("sec_filing")
        );
    }
}
