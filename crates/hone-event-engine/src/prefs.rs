//! 用户通知偏好 — 允许运行时（无需重启）控制"给哪个 actor 推什么"。
//!
//! 存储：每 actor 一个 JSON 文件，路径形如
//! `{prefs_dir}/{channel}__{scope}__{user_id}.json`。
//! 读盘粒度：**每事件、每命中 actor** 读一次——文件 I/O 廉价，换来真正的
//! 运行时可改。不缓存 mtime，用户编辑文件后下一条事件就生效。
//!
//! 默认行为：文件缺失 → `NotificationPrefs::default()`（全部放行），
//! 维持向后兼容——接入 prefs 前的部署行为不变。
//!
//! 用法示例（用户不想收消息）：
//! ```json
//! { "enabled": false }
//! ```
//!
//! 只要持仓相关：
//! ```json
//! { "portfolio_only": true }
//! ```
//!
//! 只要 High 严重度且只看财报 / SEC：
//! ```json
//! {
//!   "min_severity": "high",
//!   "allow_kinds": ["earnings_released", "sec_filing"]
//! }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};

use crate::event::{EventKind, MarketEvent, Severity};
use crate::unified_digest::DigestSlot;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationPrefs {
    /// 总开关。false → 本 actor 完全不收任何消息。
    pub enabled: bool,
    /// 只推命中用户持仓的事件（`event.symbols` 非空）。宏观等无 symbol 的事件会被过滤。
    pub portfolio_only: bool,
    /// 最低严重度，低于此的事件不推。默认 Low（全通过）。
    pub min_severity: Severity,
    /// 白名单（`kind_tag` 形式，如 `"earnings_released"`）。`None` 表示不启用白名单。
    pub allow_kinds: Option<Vec<String>>,
    /// 黑名单。与白名单叠加时，黑名单优先生效。
    pub blocked_kinds: Vec<String>,
    /// 不确定来源新闻的"重要性"短语。Router 把每条 `source_class=uncertain` 的
    /// `NewsCritical` 与此 prompt 一起送给 LLM 仲裁器,LLM 判 yes 即升 Medium。
    /// `None` → 走 `EventEngineConfig.news_importance_prompt` 全局默认。
    pub news_importance_prompt: Option<String>,
    /// 用户所在 IANA 时区,如 `"Asia/Shanghai"`、`"America/New_York"`。
    /// `None` → 沿用全局 `digest.timezone`。仅影响 digest 窗口的本地时刻解释。
    pub timezone: Option<String>,
    /// Unified digest 的触发槽位列表。每条 slot = 一次推送。
    /// `None` → 走全局默认 `event_engine.digest.default_slots`;`Some(vec![])` = 完全关 digest。
    pub digest_slots: Option<Vec<DigestSlot>>,
    /// 价格异动即时推阈值(百分点,绝对值)。`None` → 沿用全局
    /// `thresholds.price_alert_high_pct`(目前 6.0)。例如 `Some(3.5)` = 任何
    /// `|pct| >= 3.5%` 的 PriceAlert 在本 actor 路由阶段升 High。
    pub price_high_pct_override: Option<f64>,
    /// 强制升 High 即时推的 kind tag 列表(用 `kind_tag()` 字符串)。
    /// `None` / 空 → 不做任何 kind 强升;命中元素 → router 在本 actor 路径升 High。
    /// 校验复用 `first_invalid_kind_tag()`。
    pub immediate_kinds: Option<Vec<String>>,
    /// 少打扰模式：只保留财报 / SEC / 够大的持仓价格异动即时推送，其它 High 默认降级
    /// 进 digest。过滤仍由 `should_deliver` 执行，降级在 router 阶段完成。
    pub quiet_mode: bool,
    /// source 白名单 / 黑名单。元素按大小写无关的子串或前缀匹配
    /// `event.source`，例如 `"watcherguru"`、`"fmp.stock_news:globenewswire.com"`。
    pub allow_sources: Option<Vec<String>>,
    pub blocked_sources: Vec<String>,
    /// 价格即时推的方向性覆盖。未设置时回落到 `price_high_pct_override`。
    /// 正数价格变动优先用 `price_high_pct_up_override`，负数优先用 down。
    pub price_high_pct_up_override: Option<f64>,
    pub price_high_pct_down_override: Option<f64>,
    /// 当 router 能从事件 payload 读到 portfolio_weight / portfolio_weight_pct 时，
    /// 高仓位标的允许使用更敏感的用户阈值直推；低仓位仍受系统最小直推阈值保护。
    pub large_position_weight_pct: Option<f64>,
    /// 全局 digest Pass 2 personalize 时使用的"投资风格"自由文本。
    /// 例如:"长期叙事派,重视行业结构性叙事,轻视短期估值/技术形态/分析师评级"。
    /// LLM 会按此风格剔除用户视角下的噪音。`None` → 走 baseline 排序,不做风格过滤。
    #[serde(alias = "investment_global_style")]
    pub mainline_style: Option<String>,
    /// 每个 ticker 的投资主线。LLM 在 personalize 时按此重排:印证主线的优先,
    /// 证伪保留并标注,主线视角下的噪音剔除。例如 `MU → "看 NAND/DRAM 长期
    /// 稀缺性,噪音是估值过热/单日大涨大跌"`。`None` / 空 map → 不做 per-ticker 重排。
    #[serde(alias = "investment_theses")]
    pub mainline_by_ticker: Option<HashMap<String, String>>,
    /// **系统蒸馏元数据**(2026-04-26 起):`mainline_by_ticker` / `mainline_style`
    /// 由后台 cron 周扫用户 sandbox `company_profiles/*/profile.md` 自动蒸馏写入,
    /// 用户不再通过 NL tool 直接编辑。本字段是 RFC3339 时间戳记录最近一次蒸馏成功时刻,
    /// 让前端可以展示"上次更新"和判断是否需要手动刷一次。`None` = 还没蒸过(老数据兼容)。
    #[serde(alias = "last_thesis_distilled_at")]
    pub last_mainline_distilled_at: Option<String>,
    /// 蒸馏过程中跳过的 ticker(无 profile / LLM 失败 / 画像没有 ticker 标识)。
    /// 用于前端提示"这些持仓还没有画像或最近一次蒸馏失败"。
    #[serde(default, alias = "thesis_distill_skipped")]
    pub mainline_distill_skipped: Vec<String>,
    /// 勿扰时段 —— 用户希望"晚 X 点后别推、早 Y 点合并发我"。`None` = 不启用。
    /// 区间内：所有 immediate sink 推送被 hold 写 `delivery_log.status='quiet_held'`，
    /// digest fire 也跳过；`to` 时刻触发 `quiet_flush` 把 hold 住的事件 + buffer 里
    /// 累积的 Medium/Low 合并成一条早间合集，过保鲜期事件直接 drop。
    /// 跨午夜（from > to）由 EffectiveTz::in_quiet_window 处理。
    pub quiet_hours: Option<QuietHours>,
}

/// 勿扰时段配置。本地时刻按 `NotificationPrefs.timezone` 解释（缺省走全局 digest tz）。
/// 实际定义在 `hone_core::quiet::QuietHours`，这里 re-export 让 hone-channels 等
/// 不依赖 event-engine 的 crate 也能使用。
pub use hone_core::quiet::QuietHours;

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            enabled: true,
            portfolio_only: false,
            min_severity: Severity::Low,
            allow_kinds: None,
            blocked_kinds: Vec::new(),
            news_importance_prompt: None,
            timezone: None,
            digest_slots: None,
            price_high_pct_override: None,
            immediate_kinds: None,
            quiet_mode: false,
            allow_sources: None,
            blocked_sources: Vec::new(),
            price_high_pct_up_override: None,
            price_high_pct_down_override: None,
            large_position_weight_pct: None,
            mainline_style: None,
            mainline_by_ticker: None,
            last_mainline_distilled_at: None,
            mainline_distill_skipped: Vec::new(),
            quiet_hours: None,
        }
    }
}

impl NotificationPrefs {
    /// 按偏好判断是否应推送该事件。
    pub fn should_deliver(&self, event: &MarketEvent) -> bool {
        if !self.enabled {
            return false;
        }
        if event.severity.rank() < self.min_severity.rank() {
            return false;
        }
        if self.portfolio_only && event.symbols.is_empty() {
            return false;
        }
        if self.source_blocked(&event.source) {
            return false;
        }
        if let Some(allow) = &self.allow_sources {
            if !allow.iter().any(|pat| source_matches(&event.source, pat)) {
                return false;
            }
        }
        let tag = kind_tag(&event.kind);
        if self.blocked_kinds.iter().any(|k| k == tag) {
            return false;
        }
        if let Some(allow) = &self.allow_kinds {
            if !allow.iter().any(|k| k == tag) {
                return false;
            }
        }
        true
    }

    /// 取最终 slot 列表:`Some(slots)` 用 actor 自定义,`None` → 走全局默认。
    /// `Some(vec![])` = 用户关掉 digest。
    pub fn effective_digest_slots(&self) -> Option<Vec<DigestSlot>> {
        self.digest_slots.clone()
    }

    pub fn source_blocked(&self, source: &str) -> bool {
        self.blocked_sources
            .iter()
            .any(|pat| source_matches(source, pat))
    }
}

fn source_matches(source: &str, pattern: &str) -> bool {
    let source = source.trim().to_ascii_lowercase();
    let pattern = pattern.trim().to_ascii_lowercase();
    !pattern.is_empty()
        && (source == pattern || source.starts_with(&pattern) || source.contains(&pattern))
}

/// `EventKind` 的稳定字符串标签——用于 allow/block 列表匹配，
/// 与 `serde(rename_all = "snake_case")` 保持一致。
pub fn kind_tag(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::EarningsUpcoming => "earnings_upcoming",
        EventKind::EarningsReleased => "earnings_released",
        EventKind::EarningsCallTranscript => "earnings_call_transcript",
        EventKind::NewsCritical => "news_critical",
        EventKind::PriceAlert { .. } => "price_alert",
        EventKind::Weekly52High => "weekly52_high",
        EventKind::Weekly52Low => "weekly52_low",
        EventKind::Dividend => "dividend",
        EventKind::Split => "split",
        EventKind::SecFiling { .. } => "sec_filing",
        EventKind::AnalystGrade => "analyst_grade",
        EventKind::MacroEvent => "macro_event",
        EventKind::SocialPost => "social_post",
    }
}

/// 所有合法的 `kind_tag()` 输出。`allow_kinds` / `blocked_kinds` /
/// `disabled_kinds` 校验都以此为权威清单；新增 `EventKind` 变体需同步更新。
pub const ALL_KIND_TAGS: &[&str] = &[
    "earnings_upcoming",
    "earnings_released",
    "earnings_call_transcript",
    "news_critical",
    "price_alert",
    "weekly52_high",
    "weekly52_low",
    "dividend",
    "split",
    "sec_filing",
    "analyst_grade",
    "macro_event",
    "social_post",
];

/// 校验一串 kind tag 是否全部合法；返回第一个非法 tag（调用方据此构造错误消息）。
pub fn first_invalid_kind_tag<'a, I>(tags: I) -> Option<&'a str>
where
    I: IntoIterator<Item = &'a str>,
{
    tags.into_iter().find(|t| !ALL_KIND_TAGS.contains(t))
}

/// Prefs 加载抽象——router / scheduler 按 actor 查。
pub trait PrefsProvider: Send + Sync {
    fn load(&self, actor: &ActorIdentity) -> NotificationPrefs;
    /// 可选保存；文件/数据库后端可实现，内存 stub 可返回 `Err`。
    fn save(&self, _actor: &ActorIdentity, _prefs: &NotificationPrefs) -> anyhow::Result<()> {
        anyhow::bail!("this PrefsProvider is read-only")
    }
}

/// 默认放行所有事件。用于未配置 prefs 目录时的 fallback。
pub struct AllowAllPrefs;

impl PrefsProvider for AllowAllPrefs {
    fn load(&self, _actor: &ActorIdentity) -> NotificationPrefs {
        NotificationPrefs::default()
    }
}

/// 目录 = 根，每 actor 一个 JSON 文件。每次 `load` 重读；真正的运行时配置。
pub struct FilePrefsStorage {
    dir: PathBuf,
}

impl FilePrefsStorage {
    pub fn new(dir: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path_for(&self, actor: &ActorIdentity) -> PathBuf {
        self.dir.join(format!("{}.json", actor_slug(actor)))
    }
}

impl PrefsProvider for FilePrefsStorage {
    fn load(&self, actor: &ActorIdentity) -> NotificationPrefs {
        let path = self.path_for(actor);
        match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<NotificationPrefs>(&text) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        "notif prefs parse failed: {e}; falling back to default"
                    );
                    NotificationPrefs::default()
                }
            },
            Err(_) => NotificationPrefs::default(),
        }
    }

    fn save(&self, actor: &ActorIdentity, prefs: &NotificationPrefs) -> anyhow::Result<()> {
        let path = self.path_for(actor);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(prefs)?;
        std::fs::write(&path, text)?;
        Ok(())
    }
}

fn actor_slug(a: &ActorIdentity) -> String {
    let scope = a.channel_scope.as_deref().unwrap_or("direct");
    format!(
        "{}__{}__{}",
        sanitize(&a.channel),
        sanitize(scope),
        sanitize(&a.user_id)
    )
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// 方便类型别名。
pub type SharedPrefs = Arc<dyn PrefsProvider>;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;

    fn actor() -> ActorIdentity {
        ActorIdentity::new("telegram", "u1", None::<&str>).unwrap()
    }

    fn ev(kind: EventKind, sev: Severity, symbols: Vec<&str>) -> MarketEvent {
        MarketEvent {
            id: "x".into(),
            kind,
            severity: sev,
            symbols: symbols.into_iter().map(String::from).collect(),
            occurred_at: Utc::now(),
            title: "t".into(),
            summary: String::new(),
            url: None,
            source: "test".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn quiet_hours_serde_roundtrip_with_exempt_kinds() {
        let prefs = NotificationPrefs {
            quiet_hours: Some(QuietHours {
                from: "23:00".into(),
                to: "07:00".into(),
                exempt_kinds: vec!["earnings_released".into()],
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&prefs).expect("serialize");
        let loaded: NotificationPrefs = serde_json::from_str(&json).expect("deserialize");
        let qh = loaded.quiet_hours.expect("quiet_hours present");
        assert_eq!(qh.from, "23:00");
        assert_eq!(qh.to, "07:00");
        assert_eq!(qh.exempt_kinds, vec!["earnings_released".to_string()]);
    }

    #[test]
    fn old_prefs_without_quiet_hours_loads_with_none() {
        // 模拟老 JSON：没有 quiet_hours 字段
        let json = r#"{"enabled":true,"portfolio_only":false}"#;
        let loaded: NotificationPrefs = serde_json::from_str(json).expect("deserialize");
        assert!(loaded.quiet_hours.is_none());
        assert!(loaded.enabled);
    }

    #[test]
    fn default_prefs_allow_everything() {
        let p = NotificationPrefs::default();
        assert!(p.should_deliver(&ev(EventKind::NewsCritical, Severity::Low, vec!["AAPL"])));
        assert!(p.should_deliver(&ev(EventKind::MacroEvent, Severity::Low, vec![])));
    }

    #[test]
    fn disabled_blocks_all() {
        let p = NotificationPrefs {
            enabled: false,
            ..Default::default()
        };
        assert!(!p.should_deliver(&ev(
            EventKind::EarningsReleased,
            Severity::High,
            vec!["AAPL"]
        )));
    }

    #[test]
    fn portfolio_only_drops_symbol_less_events() {
        let p = NotificationPrefs {
            portfolio_only: true,
            ..Default::default()
        };
        assert!(p.should_deliver(&ev(EventKind::NewsCritical, Severity::Low, vec!["AAPL"])));
        assert!(!p.should_deliver(&ev(EventKind::MacroEvent, Severity::Low, vec![])));
    }

    #[test]
    fn min_severity_filters_lower_tiers() {
        let p = NotificationPrefs {
            min_severity: Severity::High,
            ..Default::default()
        };
        assert!(!p.should_deliver(&ev(EventKind::NewsCritical, Severity::Low, vec!["AAPL"])));
        assert!(!p.should_deliver(&ev(EventKind::NewsCritical, Severity::Medium, vec!["AAPL"])));
        assert!(p.should_deliver(&ev(EventKind::NewsCritical, Severity::High, vec!["AAPL"])));
    }

    #[test]
    fn allow_list_is_whitelist() {
        let p = NotificationPrefs {
            allow_kinds: Some(vec!["earnings_released".into()]),
            ..Default::default()
        };
        assert!(p.should_deliver(&ev(
            EventKind::EarningsReleased,
            Severity::High,
            vec!["AAPL"]
        )));
        assert!(!p.should_deliver(&ev(EventKind::NewsCritical, Severity::High, vec!["AAPL"])));
    }

    #[test]
    fn block_list_overrides_allow_list() {
        let p = NotificationPrefs {
            allow_kinds: Some(vec!["earnings_released".into(), "news_critical".into()]),
            blocked_kinds: vec!["news_critical".into()],
            ..Default::default()
        };
        assert!(p.should_deliver(&ev(
            EventKind::EarningsReleased,
            Severity::High,
            vec!["AAPL"]
        )));
        assert!(!p.should_deliver(&ev(EventKind::NewsCritical, Severity::High, vec!["AAPL"])));
    }

    #[test]
    fn file_storage_roundtrip() {
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let a = actor();
        // 缺失文件 → 默认
        let loaded = store.load(&a);
        assert!(loaded.enabled);
        // 写入 → 读回
        let p = NotificationPrefs {
            enabled: false,
            portfolio_only: true,
            min_severity: Severity::High,
            allow_kinds: Some(vec!["split".into()]),
            blocked_kinds: vec!["news_critical".into()],
            news_importance_prompt: None,
            timezone: Some("America/New_York".into()),
            digest_slots: Some(vec![DigestSlot::from_legacy_window("07:00")]),
            price_high_pct_override: Some(3.5),
            immediate_kinds: Some(vec!["weekly52_high".into(), "analyst_grade".into()]),
            quiet_mode: true,
            allow_sources: Some(vec!["fmp.stock_news:reuters.com".into()]),
            blocked_sources: vec!["watcherguru".into()],
            price_high_pct_up_override: Some(6.0),
            price_high_pct_down_override: Some(5.0),
            large_position_weight_pct: Some(20.0),
            mainline_style: Some("长期叙事派".into()),
            mainline_by_ticker: Some({
                let mut m = HashMap::new();
                m.insert("AAPL".into(), "看现金流 + 回购".into());
                m
            }),
            last_mainline_distilled_at: Some("2026-04-26T09:00:00Z".into()),
            mainline_distill_skipped: vec!["XYZ".into()],
            quiet_hours: Some(QuietHours {
                from: "23:00".into(),
                to: "07:00".into(),
                exempt_kinds: vec!["earnings_released".into()],
            }),
        };
        store.save(&a, &p).unwrap();
        let loaded = store.load(&a);
        assert!(!loaded.enabled);
        assert!(loaded.portfolio_only);
        assert_eq!(loaded.min_severity, Severity::High);
        assert_eq!(loaded.allow_kinds.as_deref(), Some(&["split".into()][..]));
        assert_eq!(loaded.timezone.as_deref(), Some("America/New_York"));
        assert_eq!(
            loaded
                .digest_slots
                .as_deref()
                .map(|s| s.iter().map(|x| x.time.clone()).collect::<Vec<_>>()),
            Some(vec!["07:00".to_string()])
        );
        assert_eq!(loaded.price_high_pct_override, Some(3.5));
        assert_eq!(
            loaded.immediate_kinds.as_deref(),
            Some(&["weekly52_high".to_string(), "analyst_grade".to_string()][..])
        );
        assert!(loaded.quiet_mode);
        assert_eq!(
            loaded.allow_sources.as_deref(),
            Some(&["fmp.stock_news:reuters.com".to_string()][..])
        );
        assert_eq!(loaded.blocked_sources, vec!["watcherguru".to_string()]);
        assert_eq!(loaded.price_high_pct_up_override, Some(6.0));
        assert_eq!(loaded.price_high_pct_down_override, Some(5.0));
        assert_eq!(loaded.large_position_weight_pct, Some(20.0));
        assert_eq!(loaded.mainline_style.as_deref(), Some("长期叙事派"));
        assert_eq!(
            loaded
                .mainline_by_ticker
                .as_ref()
                .and_then(|m| m.get("AAPL"))
                .map(String::as_str),
            Some("看现金流 + 回购")
        );
        assert_eq!(
            loaded.last_mainline_distilled_at.as_deref(),
            Some("2026-04-26T09:00:00Z")
        );
        assert_eq!(loaded.mainline_distill_skipped, vec!["XYZ".to_string()]);
    }

    #[test]
    fn new_per_actor_fields_default_to_none() {
        let p = NotificationPrefs::default();
        assert!(p.timezone.is_none());
        assert!(p.digest_slots.is_none());
        assert!(p.price_high_pct_override.is_none());
        assert!(p.immediate_kinds.is_none());
        assert!(!p.quiet_mode);
        assert!(p.allow_sources.is_none());
        assert!(p.blocked_sources.is_empty());
        assert!(p.price_high_pct_up_override.is_none());
        assert!(p.price_high_pct_down_override.is_none());
        assert!(p.large_position_weight_pct.is_none());
        assert!(p.mainline_style.is_none());
        assert!(p.mainline_by_ticker.is_none());
    }

    #[test]
    fn legacy_thesis_field_names_load_via_serde_alias() {
        // 老 prefs JSON 用 thesis 字段名,新 schema 必须经 #[serde(alias)] 兼容,
        // 否则线上已部署的 prefs 文件升级后读不出投资主线。
        let json = r#"{
            "enabled": true,
            "investment_global_style": "长期叙事派",
            "investment_theses": {"AAPL": "看现金流 + 回购"},
            "last_thesis_distilled_at": "2026-04-26T09:00:00Z",
            "thesis_distill_skipped": ["XYZ"]
        }"#;
        let p: NotificationPrefs =
            serde_json::from_str(json).expect("legacy prefs JSON should load");
        assert_eq!(p.mainline_style.as_deref(), Some("长期叙事派"));
        assert_eq!(
            p.mainline_by_ticker
                .as_ref()
                .and_then(|m| m.get("AAPL"))
                .map(String::as_str),
            Some("看现金流 + 回购")
        );
        assert_eq!(
            p.last_mainline_distilled_at.as_deref(),
            Some("2026-04-26T09:00:00Z")
        );
        assert_eq!(p.mainline_distill_skipped, vec!["XYZ".to_string()]);
    }

    #[test]
    fn new_per_actor_fields_missing_in_old_json_fall_back() {
        // 老 prefs 文件没有这 4 个字段;serde(default) 应让加载继续走默认。
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let a = actor();
        std::fs::write(
            store.path_for(&a),
            r#"{"enabled":true,"portfolio_only":false,"min_severity":"low","blocked_kinds":[]}"#,
        )
        .unwrap();
        let p = store.load(&a);
        assert!(p.timezone.is_none());
        assert!(p.digest_slots.is_none());
        assert!(p.price_high_pct_override.is_none());
        assert!(p.immediate_kinds.is_none());
        assert!(!p.quiet_mode);
        assert!(p.allow_sources.is_none());
        assert!(p.blocked_sources.is_empty());
        assert!(p.price_high_pct_up_override.is_none());
        assert!(p.price_high_pct_down_override.is_none());
        assert!(p.large_position_weight_pct.is_none());
    }

    #[test]
    fn source_allow_and_block_lists_filter_events() {
        let mut event = ev(EventKind::NewsCritical, Severity::High, vec!["AAPL"]);
        event.source = "fmp.stock_news:reuters.com".into();
        let p = NotificationPrefs {
            allow_sources: Some(vec!["reuters.com".into()]),
            ..Default::default()
        };
        assert!(p.should_deliver(&event));

        event.source = "telegram.channel:watcherguru".into();
        assert!(!p.should_deliver(&event));

        let p = NotificationPrefs {
            blocked_sources: vec!["watcherguru".into()],
            ..Default::default()
        };
        assert!(!p.should_deliver(&event));
    }

    #[test]
    fn file_storage_missing_fields_fall_back_to_default() {
        // 用户只写了 enabled=false，其他字段缺失；serde(default) 保证兼容。
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let a = actor();
        std::fs::write(store.path_for(&a), r#"{"enabled": false}"#).unwrap();
        let p = store.load(&a);
        assert!(!p.enabled);
        assert_eq!(p.min_severity, Severity::Low);
        assert!(!p.portfolio_only);
    }

    #[test]
    fn all_kind_tags_covers_every_variant() {
        // 保证 ALL_KIND_TAGS 与 kind_tag() 不漂移;所有 EventKind 变体都应能在清单里。
        use EventKind::*;
        let sample = [
            EarningsUpcoming,
            EarningsReleased,
            EarningsCallTranscript,
            NewsCritical,
            PriceAlert {
                pct_change_bps: 100,
                window: "5m".into(),
            },
            Weekly52High,
            Weekly52Low,
            Dividend,
            Split,
            SecFiling {
                form: String::new(),
            },
            AnalystGrade,
            MacroEvent,
            SocialPost,
        ];
        for k in &sample {
            let tag = kind_tag(k);
            assert!(
                ALL_KIND_TAGS.contains(&tag),
                "kind_tag {tag} 缺失于 ALL_KIND_TAGS"
            );
        }
    }

    #[test]
    fn first_invalid_kind_tag_catches_unknown() {
        assert!(first_invalid_kind_tag(["earnings_released", "news_critical"]).is_none());
        assert_eq!(
            first_invalid_kind_tag(["earnings_released", "not_a_tag"]),
            Some("not_a_tag")
        );
    }

    #[test]
    fn effective_digest_slots_returns_user_slots_when_set() {
        let p = NotificationPrefs {
            digest_slots: Some(vec![DigestSlot {
                id: "premarket".into(),
                time: "08:30".into(),
                label: Some("盘前".into()),
                floor_macro: Some(2),
            }]),
            ..Default::default()
        };
        let slots = p.effective_digest_slots().unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].id, "premarket");
        assert_eq!(slots[0].time, "08:30");
    }

    #[test]
    fn effective_digest_slots_returns_none_when_unset() {
        let p = NotificationPrefs::default();
        assert!(p.effective_digest_slots().is_none());
    }

    #[test]
    fn effective_digest_slots_preserves_empty_disable_semantics() {
        let p = NotificationPrefs {
            digest_slots: Some(vec![]),
            ..Default::default()
        };
        // Some([]) 必须保留 —— 用户主动关 digest 的语义。
        assert_eq!(p.effective_digest_slots(), Some(vec![]));
    }

    #[test]
    fn legacy_digest_windows_field_is_silently_ignored() {
        // 删字段后老 JSON 里残留的 digest_windows 应被 serde 默默忽略,不报错。
        let json = r#"{"enabled":true,"digest_windows":["07:00","19:00"]}"#;
        let loaded: NotificationPrefs = serde_json::from_str(json).expect("deserialize");
        assert!(loaded.digest_slots.is_none());
        assert!(loaded.enabled);
    }

    #[test]
    fn malformed_json_falls_back_without_panic() {
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let a = actor();
        std::fs::write(store.path_for(&a), "not json").unwrap();
        let p = store.load(&a);
        assert!(p.enabled, "解析失败时应回到默认（放行），不影响推送链路");
    }
}
