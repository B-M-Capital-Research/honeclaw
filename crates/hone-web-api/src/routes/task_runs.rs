//! 管理端只读路由 —— `GET /api/admin/task-runs`。
//!
//! 暴露 `data/runtime/task_runs.YYYY-MM-DD.jsonl` 的内容供前端 task-health 页
//! 渲染。**仅读不改**——所有周期任务的写入由各 task 自己在 tick 末尾
//! `hone_core::task_observer::record_*` 完成,本路由只做扫盘 + 汇总。
//!
//! 路径规约:
//! - `?days=N`:扫最近 N 天的 jsonl,默认 1,最大 14(对齐 `TASK_RUNS_RETENTION_DAYS`)
//! - `?limit=N`:返回 runs 数组上限,默认 500,硬上限 2000
//! - `?task=X`:只返回 task 字段等于 X 的 runs(汇总 summary 仍包含全部 task)

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

const DEFAULT_DAYS: i64 = 1;
const MAX_DAYS: i64 = 14;
const DEFAULT_LIMIT: usize = 500;
const MAX_LIMIT: usize = 2000;
const SUMMARY_WINDOW_HOURS: i64 = 24;
/// 计算 24h 汇总时一次最多扫多少行,避免老用户的 jsonl 文件超大时拖慢响应。
const SUMMARY_SCAN_LIMIT: usize = 5000;

#[derive(Deserialize)]
pub(crate) struct TaskRunsQuery {
    days: Option<i64>,
    limit: Option<usize>,
    task: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TaskRunsResponse {
    /// 倒序(最近的在前)。受 `task` 过滤影响。
    runs: Vec<hone_core::TaskRunRecord>,
    /// 全 task 维度的 24h 汇总,不受 `task` 过滤影响——前端可以用它做"任务清单"。
    summary_by_task: HashMap<String, TaskSummary>,
    /// 本次扫盘的目录,便于排查"为啥没数据"。
    runtime_dir: String,
}

#[derive(Serialize, Default)]
pub(crate) struct TaskSummary {
    last_seen_at: Option<DateTime<Utc>>,
    runs_24h: u32,
    ok_24h: u32,
    skipped_24h: u32,
    failed_24h: u32,
    /// 最近一次失败的错误简述(取 24h 窗口内最近一次)。
    last_error: Option<String>,
    last_failure_at: Option<DateTime<Utc>>,
}

pub(crate) async fn handle_task_runs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<TaskRunsQuery>,
) -> Json<TaskRunsResponse> {
    let dir = hone_core::task_observer::task_runs_dir(&state.core.config);
    let days = q.days.unwrap_or(DEFAULT_DAYS).clamp(0, MAX_DAYS);
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);

    // 24h summary: 单独扫一遍(更大 limit),不受用户 task filter 影响。
    let summary = build_summary(&dir);

    let mut runs = hone_core::task_observer::read_recent_task_runs(&dir, days, limit);
    if let Some(task) = q.task.as_deref() {
        runs.retain(|r| r.task == task);
    }

    Json(TaskRunsResponse {
        runs,
        summary_by_task: summary,
        runtime_dir: dir.display().to_string(),
    })
}

fn build_summary(dir: &std::path::Path) -> HashMap<String, TaskSummary> {
    let cutoff = Utc::now() - chrono::Duration::hours(SUMMARY_WINDOW_HOURS);
    let scan = hone_core::task_observer::read_recent_task_runs(dir, 1, SUMMARY_SCAN_LIMIT);
    let mut out: HashMap<String, TaskSummary> = HashMap::new();
    for run in &scan {
        if run.started_at < cutoff {
            continue;
        }
        let entry = out.entry(run.task.clone()).or_default();
        entry.runs_24h += 1;
        if entry.last_seen_at.is_none_or(|t| run.started_at > t) {
            entry.last_seen_at = Some(run.started_at);
        }
        match run.outcome {
            hone_core::TaskOutcome::Ok => entry.ok_24h += 1,
            hone_core::TaskOutcome::Skipped => entry.skipped_24h += 1,
            hone_core::TaskOutcome::Failed => {
                entry.failed_24h += 1;
                if entry.last_failure_at.is_none_or(|t| run.started_at > t) {
                    entry.last_failure_at = Some(run.started_at);
                    entry.last_error = run.error.clone();
                }
            }
        }
    }
    out
}
