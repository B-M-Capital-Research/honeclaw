//! Digest 内核基座 —— `UnifiedDigestScheduler` 共享的 buffer / curation / render
//! 模块。从 commit 3 起,本目录不再有自己的 scheduler;`unified_digest::scheduler`
//! 在每个 slot 触发时复用这里的 `DigestBuffer` / `digest_score` / `render_digest`。
//!
//! 按职责切成 4 个 sibling module:
//!
//! | 子 module     | 职责 |
//! |---------------|------|
//! | `buffer`      | `DigestBuffer` per-actor JSONL 槽位 + price-latest 去重 |
//! | `time_window` | `in_window` / `shift_hhmm_earlier` / `local_date_key` + `EffectiveTz`(IANA+DST) |
//! | `curation`    | 多维度 cap + 话题去重 + `should_omit_from_digest` + `digest_score` |
//! | `render`      | `render_digest` + 飞书 post 特殊路径 + `digest_event_title`(social 取首行) |
//!
//! 公共符号(`DigestBuffer` / `render_digest` /
//! `in_window` / `shift_hhmm_earlier` / `local_date_key`)全部通过下面的
//! `pub use` 保持 `hone_event_engine::digest::*` 路径稳定。

mod buffer;
pub(crate) mod curation;
mod payload;
pub(crate) mod render;
pub(crate) mod time_window;

#[cfg(test)]
mod tests;

pub use buffer::DigestBuffer;
pub use payload::{DigestItem, DigestPayload, KindBucket, group_by_kind_bucket};
pub use render::{build_digest_payload, render_digest};
pub use time_window::{in_window, local_date_key, shift_hhmm_earlier};
