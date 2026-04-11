//! Hone Feishu 渠道入口
//!
//! Rust 侧承接所有业务逻辑；Go facade 仅负责官方 SDK 长连接与 API facade。

mod card;
mod client;
mod handler;
mod listener;
mod markdown;
mod outbound;
mod scheduler;
mod types;

#[tokio::main]
async fn main() {
    handler::run().await;
}
