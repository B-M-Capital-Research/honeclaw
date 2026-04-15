use std::collections::HashSet;
use std::convert::Infallible;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::logging::LogEntry;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use chrono::NaiveDateTime;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::state::AppState;

const LOG_RESPONSE_MAX: usize = 500;
const LOG_TAIL_LINES_PER_FILE: usize = 120;

fn strip_ansi_codes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            let _ = chars.next();
            while let Some(code) = chars.next() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn parse_log_line(path: &Path, line: &str) -> Option<LogEntry> {
    let cleaned = strip_ansi_codes(line).trim().to_string();
    if cleaned.is_empty() {
        return None;
    }

    let target = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("runtime")
        .to_string();

    if let Some(rest) = cleaned.strip_prefix('[') {
        if let Some((timestamp, rest)) = rest.split_once("] ") {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('[') {
                if let Some((level, message)) = rest.split_once(']') {
                    return Some(LogEntry {
                        timestamp: timestamp.trim().to_string(),
                        level: level.trim().to_uppercase(),
                        target,
                        message: message.trim().to_string(),
                        file: Some(path.display().to_string()),
                        line: None,
                        extra: Default::default(),
                    });
                }
            }

            if let Some((level, message)) = rest.split_once(' ') {
                let normalized = level.trim().trim_end_matches(':').to_uppercase();
                if ["INFO", "WARN", "ERROR", "DEBUG"].contains(&normalized.as_str()) {
                    return Some(LogEntry {
                        timestamp: timestamp.trim().to_string(),
                        level: normalized,
                        target,
                        message: message.trim().to_string(),
                        file: Some(path.display().to_string()),
                        line: None,
                        extra: Default::default(),
                    });
                }
            }
        }
    }

    let candidate = cleaned.chars().take(24).collect::<String>();
    if candidate.chars().count() == 24 {
        if NaiveDateTime::parse_from_str(
            candidate.trim_end_matches('Z').replace('T', " ").as_str(),
            "%Y-%m-%d %H:%M:%S%.f",
        )
        .is_ok()
        {
            let message = cleaned
                .chars()
                .skip(24)
                .collect::<String>()
                .trim()
                .to_string();
            return Some(LogEntry {
                timestamp: candidate.trim_end_matches('Z').replace('T', " "),
                level: "INFO".to_string(),
                target,
                message,
                file: Some(path.display().to_string()),
                line: None,
                extra: Default::default(),
            });
        }
    }

    Some(LogEntry {
        timestamp: String::new(),
        level: "INFO".to_string(),
        target,
        message: cleaned,
        file: Some(path.display().to_string()),
        line: None,
        extra: Default::default(),
    })
}

fn recent_lines(path: &Path, limit: usize) -> Vec<String> {
    let Ok(content) = std::fs::read(path) else {
        return Vec::new();
    };
    let content = String::from_utf8_lossy(&content);
    let mut lines = content.lines().rev().take(limit).collect::<Vec<_>>();
    lines.reverse();
    lines.into_iter().map(|line| line.to_string()).collect()
}

fn runtime_log_files(base_path: &str) -> Vec<PathBuf> {
    let logs_dir = PathBuf::from(base_path).join("runtime").join("logs");
    let Ok(entries) = std::fs::read_dir(logs_dir) else {
        return Vec::new();
    };

    let mut files = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("log"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn collect_runtime_file_logs(base_path: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    for path in runtime_log_files(base_path) {
        for line in recent_lines(&path, LOG_TAIL_LINES_PER_FILE) {
            if let Some(entry) = parse_log_line(&path, &line) {
                entries.push(entry);
            }
        }
    }
    entries
}

fn snapshot_buffer_logs(log_buffer: &crate::logging::LogBuffer) -> Vec<LogEntry> {
    let buffer = match log_buffer.buffer.lock() {
        Ok(buffer) => buffer,
        Err(poisoned) => poisoned.into_inner(),
    };
    let len = buffer.len();
    let start = len.saturating_sub(LOG_RESPONSE_MAX);
    buffer.iter().skip(start).cloned().collect()
}

/// GET /api/logs — 返回最近的日志条目（最多 500 条）
pub(crate) async fn handle_logs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let logs = catch_unwind(AssertUnwindSafe(|| {
        let mut logs = snapshot_buffer_logs(&state.log_buffer);

        let runtime_base = PathBuf::from(&state.core.config.storage.sessions_dir)
            .parent()
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./data"));
        let runtime_logs = catch_unwind(AssertUnwindSafe(|| {
            collect_runtime_file_logs(&runtime_base.to_string_lossy())
        }))
        .unwrap_or_default();
        logs.extend(runtime_logs);

        let mut seen = HashSet::new();
        logs.retain(|entry| {
            seen.insert(format!(
                "{}|{}|{}|{}",
                entry.timestamp, entry.level, entry.target, entry.message
            ))
        });

        logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        if logs.len() > LOG_RESPONSE_MAX {
            let drain = logs.len() - LOG_RESPONSE_MAX;
            logs.drain(0..drain);
        }

        logs
    }))
    .unwrap_or_default();

    Json(serde_json::json!({ "logs": logs }))
}

/// GET /api/logs/stream — SSE 实时日志流
pub(crate) async fn handle_logs_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let log_buffer = state.log_buffer.clone();
    let rx = log_buffer.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(entry) => {
            let data = serde_json::to_string(&entry).unwrap_or_default();
            Some(Ok(Event::default().event("log").data(data)))
        }
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(n)) => {
            let data = serde_json::json!({
                "level": "WARN",
                "message": format!("[stream] 日志消费过慢，跳过了 {n} 条日志"),
            })
            .to_string();
            Some(Ok(Event::default().event("log").data(data)))
        }
    });

    // 先发送 connected 确认事件
    let init = tokio_stream::iter(vec![Ok::<_, Infallible>(
        Event::default().event("connected").data("{}"),
    )]);

    Sse::new(init.chain(stream)).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use uuid::Uuid;

    use super::*;
    use crate::logging::LogBuffer;

    fn temp_log_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hone-web-api-logs-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    fn sample_log_entry(message: &str) -> LogEntry {
        LogEntry {
            timestamp: "2026-04-15 11:00:00.000".to_string(),
            level: "INFO".to_string(),
            target: "runtime".to_string(),
            message: message.to_string(),
            file: None,
            line: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn recent_lines_tolerates_invalid_utf8() {
        let path = temp_log_path("runtime.log");
        fs::write(&path, b"first\nbroken:\xff\xfe\nlast\n").unwrap();

        let lines = recent_lines(&path, 10);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines.first().map(String::as_str), Some("first"));
        assert!(lines.iter().any(|line| line.starts_with("broken:")));
        assert_eq!(lines.last().map(String::as_str), Some("last"));
    }

    #[test]
    fn snapshot_buffer_logs_recovers_from_poisoned_mutex() {
        let log_buffer = LogBuffer::new();
        log_buffer.push(sample_log_entry("still available"));

        let cloned = log_buffer.clone();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = cloned.buffer.lock().unwrap();
            panic!("poison log buffer");
        }));

        let logs = snapshot_buffer_logs(&log_buffer);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].message, "still available");
    }

    #[test]
    fn parse_log_line_tolerates_multibyte_plaintext() {
        let entry = parse_log_line(
            Path::new("/tmp/runtime.log"),
            "中文日志也不能把 /api/logs 弄崩",
        )
        .unwrap();
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "中文日志也不能把 /api/logs 弄崩");
    }
}
