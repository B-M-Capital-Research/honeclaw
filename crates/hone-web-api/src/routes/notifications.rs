//! 管理端"推送日志"路由 — `GET /api/admin/notifications`。
//!
//! 跨任务读取 cron 执行记录(SQLite `cron_job_runs`),让管理人员可以排查:
//! - 谁在什么时间收到了什么推送(扁平的时间线表)
//! - 失败/拦截/未命中是不是集中在某个时段(24h 小时直方图)
//! - 单次推送的完整上下文(完整的 record 对象,含 detail / error / response)
//!
//! 所有写入仍由 `record_execution_event` 在 cron 触发末尾完成,本路由只读。

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use chrono::{DateTime, Duration as ChronoDuration, Timelike};
use serde::{Deserialize, Serialize};

use hone_core::beijing_offset;
use hone_memory::cron_job::{CronJobExecutionRecord, ExecutionFilter};

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 1000;
const HISTOGRAM_HOURS: i64 = 24;

#[derive(Deserialize)]
pub(crate) struct NotificationsQuery {
    /// 起始时间(东八区 RFC3339)。缺省 = 24 小时前。
    since: Option<String>,
    /// 终止时间(东八区 RFC3339)。缺省 = 现在。
    until: Option<String>,
    channel: Option<String>,
    user_id: Option<String>,
    job_id: Option<String>,
    /// 执行状态:`completed` / `noop` / `execution_failed`
    execution_status: Option<String>,
    /// 发送状态:`sent` / `skipped_noop` / `skipped_error` / `send_failed` /
    /// `target_resolution_failed` / `duplicate_suppressed`
    message_send_status: Option<String>,
    heartbeat_only: Option<bool>,
    limit: Option<usize>,
}

#[derive(Serialize)]
pub(crate) struct NotificationsResponse {
    /// 满足过滤条件的执行记录,按 executed_at 倒序。
    records: Vec<CronJobExecutionRecord>,
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
    let now_bj = chrono::Utc::now().with_timezone(&beijing_offset());
    let default_since = (now_bj - ChronoDuration::hours(HISTOGRAM_HOURS)).to_rfc3339();
    let since = q.since.clone().unwrap_or(default_since);

    // 主查询:严格按用户的过滤条件 + limit
    let filter = ExecutionFilter {
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
    let records = storage.list_recent_executions(&filter).unwrap_or_default();

    // 24h 直方图 / summary:固定窗口 24h、不受 status/user/channel 过滤影响,
    // 这样直方图反映的是"全局节奏",而不是某次过滤后的子集——更符合排查直觉。
    let histogram_filter = ExecutionFilter {
        since: Some((now_bj - ChronoDuration::hours(HISTOGRAM_HOURS)).to_rfc3339()),
        limit: MAX_LIMIT,
        ..ExecutionFilter::default()
    };
    let histogram_records = storage
        .list_recent_executions(&histogram_filter)
        .unwrap_or_default();

    let histogram_24h = build_histogram(&histogram_records, now_bj);
    let summary_24h = build_summary(&histogram_records);

    Json(NotificationsResponse {
        records,
        histogram_24h,
        summary_24h,
    })
}

fn build_histogram(
    records: &[CronJobExecutionRecord],
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

fn build_summary(records: &[CronJobExecutionRecord]) -> NotificationsSummary {
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

/// 把 (execution_status, message_send_status) 映射成 sent / failed / skipped 三类,
/// 与 task-detail.tsx 的 sendStatusLabel 对齐:
/// - `sent` = 真正送达
/// - `failed` = 任何"想发但发不出去"的失败(send_failed / target_resolution_failed
///   / execution_failed / skipped_error)
/// - `skipped` = 主动跳过(skipped_noop / duplicate_suppressed)
fn classify(record: &CronJobExecutionRecord, mut emit: impl FnMut(&str)) {
    let send = record.message_send_status.as_str();
    let exec = record.execution_status.as_str();
    let kind = match send {
        "sent" => "sent",
        "send_failed" | "target_resolution_failed" | "skipped_error" => "failed",
        "duplicate_suppressed" | "skipped_noop" => "skipped",
        _ => match exec {
            "execution_failed" => "failed",
            "noop" => "skipped",
            _ => "skipped",
        },
    };
    emit(kind);
}
