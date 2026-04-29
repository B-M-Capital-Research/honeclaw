//! Unified digest pipeline —— personal `DigestScheduler` 与 `GlobalDigestScheduler`
//! 合并后的单一推送链路。commit 1 仅落地类型基座(`types`),后续 commit 增量补
//! collector / scheduler / curator。
//!
//! 设计要点见 `/Users/bytedance/.claude/plans/global-digest-soft-pumpkin.md`。

pub mod types;

pub use types::{DigestSlot, FloorTag, ItemOrigin, ThesisRelation};
