//! Hone Telegram Bot 入口
//!
//! 使用 teloxide 实现 Telegram Bot。

mod handler;
mod listener;
mod markdown_v2;
mod scheduler;
mod types;

#[tokio::main]
async fn main() {
    handler::run().await;
}
