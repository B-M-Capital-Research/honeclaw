//! 日志初始化（tracing）

use chrono::Local;
use serde_json::json;
use std::net::UdpSocket;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

struct UdpLogLayer {
    socket: UdpSocket,
    target_port: u16,
}

impl UdpLogLayer {
    fn new(udp_port: Option<u16>) -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind UDP socket for logging");
        socket.set_nonblocking(true).unwrap_or_default();
        let target_port = udp_port.unwrap_or(18118);
        Self {
            socket,
            target_port,
        }
    }
}

impl<S: tracing::Subscriber> Layer<S> for UdpLogLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let level = meta.level().to_string().to_uppercase();

        let mut payload = serde_json::Map::new();
        struct FieldVisitor<'a>(&'a mut serde_json::Map<String, serde_json::Value>);

        impl<'a> tracing::field::Visit for FieldVisitor<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                self.0
                    .insert(field.name().to_string(), json!(format!("{value:?}")));
            }
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                self.0
                    .insert(field.name().to_string(), json!(value.to_string()));
            }
            fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
                self.0.insert(field.name().to_string(), json!(value));
            }
            fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
                self.0.insert(field.name().to_string(), json!(value));
            }
            fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
                self.0.insert(field.name().to_string(), json!(value));
            }
            fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
                self.0.insert(field.name().to_string(), json!(value));
            }
        }

        let mut visitor = FieldVisitor(&mut payload);
        event.record(&mut visitor);

        let message = payload
            .remove("message")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        let mut log_obj = serde_json::Map::new();
        log_obj.insert("timestamp".to_string(), json!(timestamp));
        log_obj.insert("level".to_string(), json!(level));
        log_obj.insert("target".to_string(), json!(meta.target()));
        log_obj.insert("message".to_string(), json!(message));

        if let Some(file) = meta.file() {
            log_obj.insert("file".to_string(), json!(file));
        }
        if let Some(line) = meta.line() {
            log_obj.insert("line".to_string(), json!(line));
        }

        for (k, v) in payload {
            log_obj.insert(k, v);
        }

        if let Ok(bytes) = serde_json::to_vec(&log_obj) {
            let _ = self
                .socket
                .send_to(&bytes, format!("127.0.0.1:{}", self.target_port));
        }
    }
}

/// 初始化全局日志系统
///
/// 支持通过环境变量 `HONE_LOG_LEVEL` 或配置文件设置日志级别。
pub fn setup_logging(config: &crate::config::LoggingConfig) {
    let filter =
        EnvFilter::try_from_env("HONE_LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new(&config.level));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true);

    let udp_layer = UdpLogLayer::new(config.udp_port);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(udp_layer)
        .init();
}
