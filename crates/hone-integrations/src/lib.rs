//! Hone Integrations — 外部服务集成
//!
//! Feishu facade、NanoBanana 图片生成

pub mod feishu_facade;
pub mod nano_banana;

pub use feishu_facade::FeishuFacadeClient;
pub use nano_banana::NanoBananaClient;
