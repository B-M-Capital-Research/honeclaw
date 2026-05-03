//! Unified digest pipeline —— per-actor digest 与全球要闻 digest 合并后的单一推送链路。
//!
//! 各子模块职责:
//! - `types` — `ItemOrigin` / `FloorTag` / `DigestSlot` / `MainlineRelation` 跨层共享类型
//! - `sources` — `BufferSource` / `SynthSource` / `GlobalNewsSource` 抽象 `UnifiedCandidate` 流
//! - `collector` — 三 source 编排 + per-actor 关联 + price-latest 去重保留
//! - `floor` — High severity / earnings T-N / `immediate_kinds` 绕过 LLM 的 prepend 队
//! - `scheduler` — 60s tick + 共享 Pass 1/baseline + per-actor fan-out personalize

pub mod collector;
pub mod floor;
pub mod scheduler;
pub mod sources;
pub mod types;

pub use collector::UnifiedCollector;
pub use floor::classify_floor;
pub use scheduler::UnifiedDigestScheduler;
pub use sources::{BufferSource, GlobalNewsSource, SynthSource, UnifiedCandidate};
pub use types::{DigestSlot, FloorTag, ItemOrigin, MainlineRelation};
