//! Unified digest 的"源"层 —— 把 buffer drain / synth 倒计时 / global news
//! 统一抽象成 `UnifiedCandidate` 流。三个 source + 组合器由
//! `UnifiedDigestScheduler` 在每个 slot 触发时编排。

pub mod buffer;
pub mod global;
pub mod synth;

use chrono::{DateTime, Utc};

use crate::event::MarketEvent;
use crate::pollers::news::NewsSourceClass;
use crate::unified_digest::ItemOrigin;

/// 一条尚未走 LLM / 渲染的候选事件,携带 `origin` 与(global 源专用的)富文本元数据。
///
/// `BufferSource` / `SynthSource` 产出时:`fmp_text` / `site` / `source_class` 全为 `None`。
/// `GlobalNewsSource` 产出时:三者由 EventStore 的 payload 读出,Pass 1 可直接吃。
#[derive(Debug, Clone)]
pub struct UnifiedCandidate {
    pub event: MarketEvent,
    pub origin: ItemOrigin,
    /// 候选事件被本 source 看到的时刻 —— 用于 floor 排序与 fired-today 防抖。
    /// `Buffered` 取 enqueued_at,`Synth` 取 now,`Global` 取 occurred_at。
    pub seen_at: DateTime<Utc>,
    /// 仅 `Global` 源填充:`payload.fmp.text` 浅拷贝,Pass 1 LLM 直接读。
    pub fmp_text: Option<String>,
    /// 仅 `Global` 源填充:`payload.fmp.site`(`reuters.com` / `rss:bloomberg_markets` 等)。
    pub site: Option<String>,
    /// 仅 `Global` 源填充:trusted / pr_wire / opinion_blog / uncertain。
    pub source_class: Option<NewsSourceClass>,
}

impl UnifiedCandidate {
    /// 包一条 buffer drain 出来的 `MarketEvent`,无 global 元数据。
    pub fn from_buffered(event: MarketEvent, seen_at: DateTime<Utc>) -> Self {
        Self {
            event,
            origin: ItemOrigin::Buffered,
            seen_at,
            fmp_text: None,
            site: None,
            source_class: None,
        }
    }

    /// 包一条 synth 倒计时 `MarketEvent`,id 形如 `synth:earnings:GOOGL:...`。
    pub fn from_synth(event: MarketEvent, seen_at: DateTime<Utc>) -> Self {
        Self {
            event,
            origin: ItemOrigin::Synth,
            seen_at,
            fmp_text: None,
            site: None,
            source_class: None,
        }
    }
}

pub use buffer::BufferSource;
pub use global::GlobalNewsSource;
pub use synth::SynthSource;
