//! DigestBuffer + DigestScheduler —— 按 actor 缓存 Medium/Low 事件,定时合并推送。
//!
//! 按职责切成 5 个 sibling module:
//!
//! | 子 module     | 职责 |
//! |---------------|------|
//! | `buffer`      | `DigestBuffer` per-actor JSONL 槽位 + price-latest 去重 |
//! | `time_window` | `in_window` / `shift_hhmm_earlier` / `local_date_key` + `EffectiveTz`(IANA+DST) |
//! | `curation`    | 多维度 cap + 话题去重 + `should_omit_from_digest` + `digest_score` |
//! | `render`      | `render_digest` + 飞书 post 特殊路径 + `digest_event_title`(social 取首行) |
//! | `scheduler`   | `DigestScheduler::tick_once` —— flush 时的 prefs → memory → curation → send 管线 |
//!
//! 公共符号(`DigestBuffer` / `DigestScheduler` / `render_digest` /
//! `in_window` / `shift_hhmm_earlier` / `local_date_key`)全部通过下面的
//! `pub use` 保持 `hone_event_engine::digest::*` 路径稳定。

mod buffer;
mod curation;
mod payload;
mod render;
mod scheduler;
pub(crate) mod time_window;

#[cfg(test)]
mod tests;

pub use buffer::DigestBuffer;
pub use payload::{DigestItem, DigestPayload, KindBucket, group_by_kind_bucket};
pub use render::{build_digest_payload, render_digest};
pub use scheduler::DigestScheduler;
pub use time_window::{in_window, local_date_key, shift_hhmm_earlier};
