//! `DigestPayload` —— digest 推送的渠道无关结构化中间表示。
//!
//! `render_digest()` 早期把字符串当唯一产出,Discord/Feishu/Telegram 想用 embed /
//! interactive card / disable_web_page_preview 这些渠道原生富文本就只能 parse 字符串
//! 反推,脆弱不可靠。本模块把"哪些事件、属于哪个大类、严重度多高"这些结构化信号
//! 提到一份 `DigestPayload` 里,让 sink 自己根据自家能力渲染。
//!
//! 6 个 `KindBucket` 把 17 种 `EventKind` 折叠成可视化分组——14 种 short label
//! ([价格]/[新闻]/[财报] 等)在 digest 视图里互相挤,bucket 化后每组单独成块。

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::event::{EventKind, MarketEvent, Severity};
use crate::unified_digest::{FloorTag, ItemOrigin, MainlineRelation};

/// 单条 digest 条目的结构化表示。`headline` 是已经过 `digest_event_title()`
/// 处理(SocialPost 取首行)的展示标题;`primary_symbol` 取 `symbols.first()`
/// 大写化后的 cashtag 内容(不含 `$` 前缀)。
///
/// `origin` / `floor` / `comment` / `mainline_relation` 是 unified digest pipeline
/// commit 1 加入的字段——旧 `DigestScheduler` 产物默认 `Buffered + None`,
/// commit 3 起由 `unified_digest::collector` / `curator` 填充。
#[derive(Debug, Clone)]
pub struct DigestItem {
    pub id: String,
    pub kind: EventKind,
    pub severity: Severity,
    pub primary_symbol: Option<String>,
    pub headline: String,
    pub url: Option<String>,
    pub occurred_at: DateTime<Utc>,
    /// 来源标签——决定渲染层 emoji / 排序。旧 pipeline 全部 `Buffered`。
    pub origin: ItemOrigin,
    /// 命中 floor 时填充,绕过 LLM 排序、永远 prepend。
    pub floor: Option<FloorTag>,
    /// Pass 2 personalize 产出的中文短评;LLM 失败或未走 unified pipeline 时为 `None`。
    pub comment: Option<String>,
    /// 该条与用户投资主线的关系——渲染层据此打 ✅/❌ 标记。
    pub mainline_relation: Option<MainlineRelation>,
}

/// 一批 digest 推送的载荷。`items` 已 dedup 且保留 scheduler 排好的顺序,
/// `cap_overflow` 是被 `max_items_per_batch` 截掉的尾巴(footer 提示用)。
/// `max_severity` 给 embed/card 主色块用。
#[derive(Debug, Clone)]
pub struct DigestPayload {
    pub label: String,
    pub items: Vec<DigestItem>,
    pub cap_overflow: usize,
    pub max_severity: Severity,
    pub generated_at: DateTime<Utc>,
}

impl DigestPayload {
    /// `items.len() + cap_overflow` —— 标题里 "· N 条" 的 N。
    pub fn total(&self) -> usize {
        self.items.len() + self.cap_overflow
    }
}

/// 6 大可视化分组。order(声明顺序)同时是渲染顺序——价格异动放最上,社交放最下。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KindBucket {
    Price,
    Earnings,
    NewsFiling,
    CorpAction,
    Macro,
    Social,
}

impl KindBucket {
    pub fn from_kind(kind: &EventKind) -> Self {
        match kind {
            EventKind::PriceAlert { .. } | EventKind::Weekly52High | EventKind::Weekly52Low => {
                KindBucket::Price
            }
            EventKind::EarningsUpcoming
            | EventKind::EarningsReleased
            | EventKind::EarningsCallTranscript => KindBucket::Earnings,
            EventKind::NewsCritical | EventKind::SecFiling { .. } => KindBucket::NewsFiling,
            EventKind::Dividend | EventKind::Split | EventKind::AnalystGrade => {
                KindBucket::CorpAction
            }
            EventKind::MacroEvent => KindBucket::Macro,
            EventKind::SocialPost => KindBucket::Social,
        }
    }

    /// 渲染时的小标题前缀(emoji + 中文)。Discord embed 的 field name、
    /// Feishu card 的 div 标题、Telegram HTML 的 section header 都用它。
    pub fn header_label(self) -> &'static str {
        match self {
            KindBucket::Price => "💹 价格异动",
            KindBucket::Earnings => "📅 财报",
            KindBucket::NewsFiling => "📰 新闻公告",
            KindBucket::CorpAction => "🏢 公司行动",
            KindBucket::Macro => "🌐 宏观",
            KindBucket::Social => "🗣 社交",
        }
    }
}

/// 按 bucket 分组 items,保留传入顺序(BTreeMap 的 KindBucket 序 = 渲染序)。
/// 返回引用,不复制 items —— 调用方按需要遍历。
pub fn group_by_kind_bucket(items: &[DigestItem]) -> BTreeMap<KindBucket, Vec<&DigestItem>> {
    let mut out: BTreeMap<KindBucket, Vec<&DigestItem>> = BTreeMap::new();
    for it in items {
        out.entry(KindBucket::from_kind(&it.kind))
            .or_default()
            .push(it);
    }
    out
}

/// 从单条 `MarketEvent` 投影出 `DigestItem`。`headline` 由 caller 传(因为
/// SocialPost 的截首行逻辑住在 render.rs 里),其它字段直接 clone。
pub(crate) fn item_from_event(event: &MarketEvent, headline: String) -> DigestItem {
    let primary_symbol = event
        .symbols
        .iter()
        .find(|s| !s.is_empty())
        .map(|s| s.to_uppercase());
    DigestItem {
        id: event.id.clone(),
        kind: event.kind.clone(),
        severity: event.severity,
        primary_symbol,
        headline,
        url: event
            .url
            .as_deref()
            .filter(|u| !u.is_empty())
            .map(String::from),
        occurred_at: event.occurred_at,
        origin: ItemOrigin::Buffered,
        floor: None,
        comment: None,
        mainline_relation: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: EventKind, sev: Severity) -> DigestItem {
        DigestItem {
            id: format!("id:{kind:?}"),
            kind,
            severity: sev,
            primary_symbol: Some("AAPL".into()),
            headline: "h".into(),
            url: None,
            occurred_at: Utc::now(),
            origin: ItemOrigin::Buffered,
            floor: None,
            comment: None,
            mainline_relation: None,
        }
    }

    #[test]
    fn bucket_classifies_all_kinds() {
        assert_eq!(
            KindBucket::from_kind(&EventKind::PriceAlert {
                pct_change_bps: 100,
                window: "1d".into()
            }),
            KindBucket::Price
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::Weekly52High),
            KindBucket::Price
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::EarningsUpcoming),
            KindBucket::Earnings
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::NewsCritical),
            KindBucket::NewsFiling
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::SecFiling { form: "8-K".into() }),
            KindBucket::NewsFiling
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::Dividend),
            KindBucket::CorpAction
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::AnalystGrade),
            KindBucket::CorpAction
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::MacroEvent),
            KindBucket::Macro
        );
        assert_eq!(
            KindBucket::from_kind(&EventKind::SocialPost),
            KindBucket::Social
        );
    }

    #[test]
    fn group_orders_by_bucket_declaration() {
        let items = vec![
            item(EventKind::SocialPost, Severity::Low),
            item(EventKind::NewsCritical, Severity::High),
            item(
                EventKind::PriceAlert {
                    pct_change_bps: 100,
                    window: "1d".into(),
                },
                Severity::Medium,
            ),
            item(EventKind::EarningsUpcoming, Severity::Medium),
        ];
        let grouped = group_by_kind_bucket(&items);
        let order: Vec<_> = grouped.keys().copied().collect();
        // BTreeMap 排序 = enum 声明顺序
        assert_eq!(
            order,
            vec![
                KindBucket::Price,
                KindBucket::Earnings,
                KindBucket::NewsFiling,
                KindBucket::Social,
            ]
        );
    }

    #[test]
    fn payload_total_sums_items_and_overflow() {
        let p = DigestPayload {
            label: "x".into(),
            items: vec![item(EventKind::NewsCritical, Severity::Low)],
            cap_overflow: 3,
            max_severity: Severity::Low,
            generated_at: Utc::now(),
        };
        assert_eq!(p.total(), 4);
    }
}
