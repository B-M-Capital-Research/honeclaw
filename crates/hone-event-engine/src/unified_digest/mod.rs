//! Unified digest pipeline —— personal `DigestScheduler` 与 `GlobalDigestScheduler`
//! 合并后的单一推送链路。commit 1 落地类型基座(`types`),commit 2 落地 source 抽象
//! (`sources` + `collector`),scheduler / curator 由后续 commit 接入。
//!
//! 设计要点见 `/Users/bytedance/.claude/plans/global-digest-soft-pumpkin.md`。

pub mod collector;
pub mod sources;
pub mod types;

pub use collector::UnifiedCollector;
pub use sources::{BufferSource, GlobalNewsSource, SynthSource, UnifiedCandidate};
pub use types::{DigestSlot, FloorTag, ItemOrigin, ThesisRelation};
