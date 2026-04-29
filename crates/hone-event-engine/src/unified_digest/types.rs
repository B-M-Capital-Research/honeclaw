//! Unified digest 类型基座 —— `UnifiedDigestScheduler` 跨 source / curator /
//! 渲染各层共享的小类型:
//!
//! - `ItemOrigin` — `DigestItem` 的来源标签,渲染层据此决定 emoji / 排序。
//! - `FloorTag` — High severity / earnings T-N / `immediate_kinds` 等绕过 LLM 的
//!   优先级标签;floor 条目永远 prepend 到 payload 顶部,不被 max_items_per_batch 挤掉。
//! - `ThesisRelation` — Pass 2 personalize 标记一条 item 与用户 thesis 的关系。
//! - `DigestSlot` — 用户自定义的 digest 触发槽位(替代旧 `digest_windows` 字符串数组)。

use serde::{Deserialize, Serialize};

/// 一条 `DigestItem` 的来源。`Buffered` 来自 per-actor `DigestBuffer`(持仓路由);
/// `Synth` 是 scheduler 现算的 earnings 倒计时;`Global` 是 LLM 精读的全球要闻。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemOrigin {
    Buffered,
    Synth,
    Global,
}

impl Default for ItemOrigin {
    fn default() -> Self {
        // 旧 pipeline 产出的 `DigestItem` 都来自 buffer;commit 1 不改行为,
        // 默认 Buffered 维持现有 fixture / 测试断言。
        ItemOrigin::Buffered
    }
}

/// Floor 标签 —— 强制保留在 payload 顶部,绕过 LLM personalize 排序。
/// 详见 plan: floor item 占 `max_items_per_batch` 配额但永不被挤掉。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloorTag {
    /// `event.severity == High` —— 即时推降级进 digest 的兜底。
    HighSeverity,
    /// Earnings synth countdown(T-3 / T-2 / T-1)。
    Countdown,
    /// 用户 `immediate_kinds` 命中(例如硬性"必收 SEC 8-K")。
    UserImmediate,
    /// `slot.floor_macro` 配额 —— LLM 试图全剔时仍保留的宏观底线。
    MacroFloor,
}

/// Pass 2 personalize 标记的"该条与用户 thesis 的关系"。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThesisRelation {
    /// 印证用户对该 ticker 的看多/看空 thesis。
    Aligned,
    /// 反证 thesis —— 强制保留并标注。
    Counter,
    /// 与任何 thesis 无明显关系(pure macro / 旁系)。
    Neutral,
}

/// 用户自定义的 digest 触发槽位。一条 slot = 一次 push;`time` 按
/// `prefs.timezone`(回退全局 unified_digest.timezone)解释为本地时刻。
///
/// 旧 `prefs.digest_windows: Vec<String>` 反序列化时自动 wrap 为
/// `Vec<DigestSlot>`,见 `crate::unified_digest::migration::slots_from_windows`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DigestSlot {
    /// 稳定 ID,用于在 NL tool 里指定要改哪个 slot(例 `"premarket"` / `"postmarket"`)。
    pub id: String,
    /// 本地时刻 `"HH:MM"`。
    pub time: String,
    /// 渲染 header 用的中文 label(例 `"盘前要闻"`)。`None` → 渲染时取 id。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 该 slot 的 macro floor 条数。`None` → 走 scheduler 兜底(`DEFAULT_FLOOR_MACRO_PICKS = 1`)。
    /// 即使 thesis 把所有宏观料剔除,Pass 2 personalize 至少保留这么多条 macro_floor,
    /// 让用户看到大盘背景。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_macro: Option<u32>,
}

impl DigestSlot {
    /// 默认 slot id —— 旧 `digest_windows: Vec<String>` 升级时附加,
    /// 单 slot 时直接用这个 id。
    pub const DEFAULT_ID: &'static str = "default";

    /// 从单一 `"HH:MM"` 字符串构造一个最小 slot(id = `default`,无 label / floor)。
    /// 仅由 migration 路径使用。
    pub fn from_legacy_window(time: impl Into<String>) -> Self {
        Self {
            id: Self::DEFAULT_ID.into(),
            time: time.into(),
            label: None,
            floor_macro: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_origin_default_is_buffered() {
        assert_eq!(ItemOrigin::default(), ItemOrigin::Buffered);
    }

    #[test]
    fn item_origin_serde_snake_case() {
        let json = serde_json::to_string(&ItemOrigin::Global).unwrap();
        assert_eq!(json, "\"global\"");
        let parsed: ItemOrigin = serde_json::from_str("\"synth\"").unwrap();
        assert_eq!(parsed, ItemOrigin::Synth);
    }

    #[test]
    fn floor_tag_serde_snake_case() {
        let json = serde_json::to_string(&FloorTag::HighSeverity).unwrap();
        assert_eq!(json, "\"high_severity\"");
        let parsed: FloorTag = serde_json::from_str("\"macro_floor\"").unwrap();
        assert_eq!(parsed, FloorTag::MacroFloor);
    }

    #[test]
    fn thesis_relation_serde_snake_case() {
        let json = serde_json::to_string(&ThesisRelation::Counter).unwrap();
        assert_eq!(json, "\"counter\"");
    }

    #[test]
    fn digest_slot_skips_none_label_and_floor() {
        let slot = DigestSlot {
            id: "premarket".into(),
            time: "08:30".into(),
            label: None,
            floor_macro: None,
        };
        let json = serde_json::to_string(&slot).unwrap();
        assert_eq!(json, r#"{"id":"premarket","time":"08:30"}"#);
    }

    #[test]
    fn digest_slot_round_trip_with_all_fields() {
        let slot = DigestSlot {
            id: "premarket".into(),
            time: "08:30".into(),
            label: Some("盘前要闻".into()),
            floor_macro: Some(2),
        };
        let json = serde_json::to_string(&slot).unwrap();
        let parsed: DigestSlot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, slot);
    }

    #[test]
    fn from_legacy_window_uses_default_id() {
        let slot = DigestSlot::from_legacy_window("19:00");
        assert_eq!(slot.id, DigestSlot::DEFAULT_ID);
        assert_eq!(slot.time, "19:00");
        assert!(slot.label.is_none());
        assert!(slot.floor_macro.is_none());
    }
}
