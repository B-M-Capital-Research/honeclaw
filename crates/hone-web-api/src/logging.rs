use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing_subscriber::Layer;

const LOG_CHANNEL_CAPACITY: usize = 256;
const LOG_BUFFER_MAX: usize = 2000;

/// 单条日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 共享日志缓冲区 + 广播发送端
#[derive(Clone)]
pub struct LogBuffer {
    pub buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    pub tx: broadcast::Sender<LogEntry>,
}

impl LogBuffer {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(LOG_CHANNEL_CAPACITY);
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(LOG_BUFFER_MAX))),
            tx,
        }
    }

    pub fn push(&self, entry: LogEntry) {
        {
            let mut buf = self.buffer.lock().unwrap();
            if buf.len() >= LOG_BUFFER_MAX {
                buf.pop_front();
            }
            buf.push_back(entry.clone());
        }
        // 广播给所有 SSE 订阅者（忽略无接收者错误）
        let _ = self.tx.send(entry);
    }
}

/// 自定义 tracing Layer，捕获所有日志事件
pub struct LogCaptureLayer {
    log_buffer: LogBuffer,
}

impl LogCaptureLayer {
    pub fn new(log_buffer: LogBuffer) -> Self {
        Self { log_buffer }
    }
}

impl<S: tracing::Subscriber> Layer<S> for LogCaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let level = meta.level().to_string().to_uppercase();

        // 从事件字段中提取消息
        struct MsgVisitor(String);
        impl tracing::field::Visit for MsgVisitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = format!("{value:?}");
                }
            }
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    self.0 = value.to_string();
                }
            }
        }

        let mut visitor = MsgVisitor(String::new());
        event.record(&mut visitor);
        let message = visitor.0;

        // 显式使用 UTC+8（GMT+8 / Asia/Shanghai），不依赖系统时区
        let tz = chrono::FixedOffset::east_opt(8 * 3600).expect("valid tz");
        let timestamp = chrono::Utc::now()
            .with_timezone(&tz)
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();

        let entry = LogEntry {
            timestamp,
            level,
            target: meta.target().to_string(),
            message,
            file: meta.file().map(str::to_string),
            line: meta.line(),
            extra: HashMap::new(),
        };

        self.log_buffer.push(entry);
    }
}
