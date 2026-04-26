//! 周期性任务的统一观测落盘 —— `data/runtime/task_runs.YYYY-MM-DD.jsonl`。
//!
//! 任何周期任务(EventSource poller / digest scheduler / daily_report /
//! thesis_cron / cleanup 等)在每次 tick 末尾调一次 [`record_task_run`],
//! 把一条结构化记录追加到当日的 jsonl 文件。文件每天切一个,通过启动时
//! 的清理保留 [`TASK_RUNS_RETENTION_DAYS`] 天。
//!
//! 这是给"机器/管理端只读视图"准备的全局任务维度账,跟下列已存在的观测**正交**
//! ——后者各有读者,合并会破坏各自优化:
//! - `cron_job_runs` SQLite 表(memory crate):终端用户视角的 cron job 历史
//! - `data/daily_reports/*.md`:人类可读的当日叙事
//! - `data/runtime/{channel}.heartbeat.{json,error}`:进程存活 sidecar
//! - `delivery_log` 表(event-engine):router 出口侧账
//!
//! 失败处理:落盘自身只 `tracing::warn!`,绝不上抛——观测层挂了不能影响主链路。
//! 调用方不需要 `?` 不需要返回值,fire-and-forget。

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;

use crate::config::HoneConfig;
use crate::heartbeat::runtime_heartbeat_dir;

/// 保留多少天的 task_runs.jsonl。启动时清理超过这个天数的文件。
pub const TASK_RUNS_RETENTION_DAYS: i64 = 14;

/// 单次 tick 的成败结果。
///
/// - `Ok` —— 命中并成功执行。
/// - `Skipped` —— 按业务策略主动跳过(thesis cron staleness 没到 / digest 不在窗口内
///   等),不是失败。
/// - `Failed` —— 业务函数返回 Err。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskOutcome {
    Ok,
    Skipped,
    Failed,
}

impl TaskOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskOutcome::Ok => "ok",
            TaskOutcome::Skipped => "skipped",
            TaskOutcome::Failed => "failed",
        }
    }
}

/// 一行 task_runs.jsonl 的结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRunRecord {
    /// 稳定标识,跟 tracing `task=` 字段一致。例:
    /// `poller.fmp.earnings` / `internal.daily_report` / `thesis_cron`。
    pub task: String,
    /// 本轮 tick 开始时刻 (UTC)。
    pub started_at: DateTime<Utc>,
    /// 本轮 tick 完成时刻 (UTC)。即使 outcome=Failed 也填,代表"执行结束的瞬间"。
    pub ended_at: DateTime<Utc>,
    pub outcome: TaskOutcome,
    /// 本次处理的条数(事件数 / 推送数 / 蒸馏对象数等),无意义就填 0。
    pub items: u64,
    /// 失败时的错误简述。其他 outcome 应为 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// 全局写锁:所有 task 共用同一把 Mutex 串行化文件 append,保证多 task 同时写入不撕裂行。
// task_runs.jsonl 写入频率粗略上限 = 总 task 数 × tick 频率 < 每秒 1 次,
// 串行化代价远低于一次 fs syscall,不需要 per-file lock。
static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// 计算 task_runs.jsonl 应该写在哪个目录。沿用 heartbeat 的 `data/runtime/`,
/// 跟 `.heartbeat.json` 同级。
pub fn task_runs_dir(config: &HoneConfig) -> PathBuf {
    runtime_heartbeat_dir(config)
}

/// `data/runtime/task_runs.YYYY-MM-DD.jsonl`,按 UTC 日期切。
///
/// 切日期不影响读取(API 一次读多天),但限制了单文件大小 + 简化清理。
pub fn task_runs_path(runtime_dir: &Path, date: chrono::NaiveDate) -> PathBuf {
    runtime_dir.join(format!("task_runs.{}.jsonl", date.format("%Y-%m-%d")))
}

/// 追加一行 task run 记录到当日 jsonl。所有错误只 warn 不上抛。
///
/// 调用方:在 tick 完成后(成功 or 失败 or 主动跳过)调一次。`started_at` 取
/// tick 开始时的 `Utc::now()`,`ended_at` 取写入前的 `Utc::now()`。
pub fn record_task_run(runtime_dir: &Path, record: &TaskRunRecord) {
    if let Err(e) = record_task_run_inner(runtime_dir, record) {
        warn!(
            task = %record.task,
            outcome = record.outcome.as_str(),
            "failed to write task_runs jsonl: {e:#}"
        );
    }
}

fn record_task_run_inner(runtime_dir: &Path, record: &TaskRunRecord) -> std::io::Result<()> {
    fs::create_dir_all(runtime_dir)?;
    let path = task_runs_path(runtime_dir, record.started_at.date_naive());
    let line = serde_json::to_string(record)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut f = OpenOptions::new().create(true).append(true).open(&path)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}

/// 删除 `runtime_dir` 下早于 `retention_days` 的 task_runs.*.jsonl 文件。
/// 启动时调一次即可。错误只 warn 不上抛。
pub fn purge_old_task_runs(runtime_dir: &Path, retention_days: i64) {
    if !runtime_dir.exists() {
        return;
    }
    let cutoff = Utc::now().date_naive() - chrono::Duration::days(retention_days);
    let read = match fs::read_dir(runtime_dir) {
        Ok(r) => r,
        Err(e) => {
            warn!("task_runs purge: read_dir failed: {e:#}");
            return;
        }
    };
    for entry in read.flatten() {
        let name = entry.file_name();
        let s = match name.to_str() {
            Some(s) => s,
            None => continue,
        };
        let date_str = match s
            .strip_prefix("task_runs.")
            .and_then(|s| s.strip_suffix(".jsonl"))
        {
            Some(d) => d,
            None => continue,
        };
        let date = match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };
        if date < cutoff {
            if let Err(e) = fs::remove_file(entry.path()) {
                warn!(
                    file = %entry.path().display(),
                    "task_runs purge: remove failed: {e:#}"
                );
            }
        }
    }
}

/// 读取最近 `days_back` 天的所有 task_runs 记录,按 started_at 倒序返回。
/// 给 web-api 的 admin 路由 `GET /api/admin/task-runs` 用。
///
/// 单次最多返回 `limit` 条;超过 limit 后跳出停止扫早期文件。
pub fn read_recent_task_runs(
    runtime_dir: &Path,
    days_back: i64,
    limit: usize,
) -> Vec<TaskRunRecord> {
    let mut out = Vec::new();
    if !runtime_dir.exists() || limit == 0 {
        return out;
    }
    let today = Utc::now().date_naive();
    for offset in 0..=days_back {
        let date = today - chrono::Duration::days(offset);
        let path = task_runs_path(runtime_dir, date);
        let raw = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue, // 文件可能不存在(那天没记录),跳过
        };
        // 文件内是按 append 顺序的,倒序读出来给 caller(由 caller 决定是否再排序)。
        let mut day_records: Vec<TaskRunRecord> = raw
            .lines()
            .filter(|l| !l.is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        day_records.reverse();
        for rec in day_records {
            out.push(rec);
            if out.len() >= limit {
                return out;
            }
        }
    }
    out
}

/// 给 caller 一个把 `started_at` / `ended_at` / `outcome` 一次塞好的 helper,
/// 减少调用点的样板。失败的 error 字符串截断到 `MAX_ERROR_LEN` 防止 jsonl 行过长。
const MAX_ERROR_LEN: usize = 500;

pub fn record_ok(runtime_dir: &Path, task: &str, started_at: DateTime<Utc>, items: u64) {
    record_task_run(
        runtime_dir,
        &TaskRunRecord {
            task: task.to_string(),
            started_at,
            ended_at: Utc::now(),
            outcome: TaskOutcome::Ok,
            items,
            error: None,
        },
    );
}

pub fn record_skipped(runtime_dir: &Path, task: &str, started_at: DateTime<Utc>) {
    record_task_run(
        runtime_dir,
        &TaskRunRecord {
            task: task.to_string(),
            started_at,
            ended_at: Utc::now(),
            outcome: TaskOutcome::Skipped,
            items: 0,
            error: None,
        },
    );
}

pub fn record_failed(runtime_dir: &Path, task: &str, started_at: DateTime<Utc>, error: &str) {
    let truncated = if error.len() > MAX_ERROR_LEN {
        let mut s = error.chars().take(MAX_ERROR_LEN).collect::<String>();
        s.push_str("…(truncated)");
        s
    } else {
        error.to_string()
    };
    record_task_run(
        runtime_dir,
        &TaskRunRecord {
            task: task.to_string(),
            started_at,
            ended_at: Utc::now(),
            outcome: TaskOutcome::Failed,
            items: 0,
            error: Some(truncated),
        },
    );
}

/// 仅在测试 / 自检时使用。把单条记录格式化成 jsonl 行,方便比对。
#[doc(hidden)]
pub fn render_record_for_debug(record: &TaskRunRecord) -> String {
    json!({
        "task": record.task,
        "started_at": record.started_at,
        "ended_at": record.ended_at,
        "outcome": record.outcome,
        "items": record.items,
        "error": record.error,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn append_creates_file_and_writes_one_line_per_record() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path();
        record_ok(dir, "poller.test", Utc::now(), 5);
        record_skipped(dir, "thesis_cron", Utc::now());
        record_failed(dir, "internal.cleanup", Utc::now(), "disk full");

        let path = task_runs_path(dir, Utc::now().date_naive());
        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 3, "应写入 3 行");

        let parsed: Vec<TaskRunRecord> = lines
            .iter()
            .map(|l| serde_json::from_str(l).expect("行可被反序列化"))
            .collect();
        assert_eq!(parsed[0].task, "poller.test");
        assert_eq!(parsed[0].outcome, TaskOutcome::Ok);
        assert_eq!(parsed[0].items, 5);
        assert_eq!(parsed[1].outcome, TaskOutcome::Skipped);
        assert_eq!(parsed[2].outcome, TaskOutcome::Failed);
        assert_eq!(parsed[2].error.as_deref(), Some("disk full"));
    }

    #[test]
    fn read_recent_returns_records_in_reverse_chrono() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path();
        let t0 = Utc::now() - chrono::Duration::seconds(3);
        let t1 = Utc::now() - chrono::Duration::seconds(2);
        let t2 = Utc::now() - chrono::Duration::seconds(1);
        record_ok(dir, "a", t0, 1);
        record_ok(dir, "b", t1, 1);
        record_ok(dir, "c", t2, 1);

        let recent = read_recent_task_runs(dir, 1, 10);
        assert_eq!(recent.len(), 3);
        // 倒序:最近写入的在最前
        assert_eq!(recent[0].task, "c");
        assert_eq!(recent[1].task, "b");
        assert_eq!(recent[2].task, "a");
    }

    #[test]
    fn read_recent_respects_limit() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path();
        for i in 0..5 {
            record_ok(dir, &format!("task_{i}"), Utc::now(), i);
        }
        let recent = read_recent_task_runs(dir, 0, 3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn purge_removes_files_older_than_cutoff() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path();
        // 模拟 30 天前一个文件
        let old_date = Utc::now().date_naive() - chrono::Duration::days(30);
        let old_path = task_runs_path(dir, old_date);
        std::fs::write(&old_path, "{}\n").unwrap();
        // 今天一个文件
        record_ok(dir, "today", Utc::now(), 1);

        purge_old_task_runs(dir, 14);

        assert!(!old_path.exists(), "30 天前的文件应被清理");
        let today_path = task_runs_path(dir, Utc::now().date_naive());
        assert!(today_path.exists(), "今天的文件应保留");
    }

    #[test]
    fn long_error_is_truncated() {
        let tmp = tempdir().unwrap();
        let long = "x".repeat(2000);
        record_failed(tmp.path(), "task", Utc::now(), &long);
        let recent = read_recent_task_runs(tmp.path(), 0, 1);
        let err = recent[0].error.as_deref().unwrap();
        assert!(err.len() < 1000, "超长错误应截断");
        assert!(err.ends_with("…(truncated)"));
    }
}
