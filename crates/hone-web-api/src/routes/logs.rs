use std::collections::HashSet;
use std::convert::Infallible;
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

    if cleaned.len() >= 24 {
        let candidate = &cleaned[..24.min(cleaned.len())];
        if NaiveDateTime::parse_from_str(
            candidate.trim_end_matches('Z').replace('T', " ").as_str(),
            "%Y-%m-%d %H:%M:%S%.f",
        )
        .is_ok()
        {
            let message = cleaned[24..].trim().to_string();
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
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
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

/// GET /api/logs — 返回最近的日志条目（最多 500 条）
pub(crate) async fn handle_logs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let log_buffer = state.log_buffer.clone();
    let mut logs = {
        let buf = log_buffer.buffer.lock().unwrap();
        let len = buf.len();
        let start = len.saturating_sub(LOG_RESPONSE_MAX);
        buf.iter().skip(start).cloned().collect::<Vec<_>>()
    };

    let runtime_base = PathBuf::from(&state.core.config.storage.sessions_dir)
        .parent()
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./data"));
    logs.extend(collect_runtime_file_logs(&runtime_base.to_string_lossy()));

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
