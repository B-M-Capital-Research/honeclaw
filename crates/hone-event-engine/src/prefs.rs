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

use std::path::{Path, PathBuf};
use std::sync::Arc;

use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};

use crate::event::{EventKind, MarketEvent, Severity};

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
}

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            enabled: true,
            portfolio_only: false,
            min_severity: Severity::Low,
            allow_kinds: None,
            blocked_kinds: Vec::new(),
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
}

/// `EventKind` 的稳定字符串标签——用于 allow/block 列表匹配，
/// 与 `serde(rename_all = "snake_case")` 保持一致。
pub fn kind_tag(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::EarningsUpcoming => "earnings_upcoming",
        EventKind::EarningsReleased => "earnings_released",
        EventKind::NewsCritical => "news_critical",
        EventKind::PressRelease => "press_release",
        EventKind::PriceAlert { .. } => "price_alert",
        EventKind::Weekly52High => "weekly52_high",
        EventKind::Weekly52Low => "weekly52_low",
        EventKind::VolumeSpike => "volume_spike",
        EventKind::Dividend => "dividend",
        EventKind::Split => "split",
        EventKind::Buyback => "buyback",
        EventKind::SecFiling { .. } => "sec_filing",
        EventKind::AnalystGrade => "analyst_grade",
        EventKind::MacroEvent => "macro_event",
        EventKind::PortfolioPreMarket => "portfolio_pre_market",
        EventKind::PortfolioPostMarket => "portfolio_post_market",
    }
}

/// 所有合法的 `kind_tag()` 输出。`allow_kinds` / `blocked_kinds` /
/// `disabled_kinds` 校验都以此为权威清单；新增 `EventKind` 变体需同步更新。
pub const ALL_KIND_TAGS: &[&str] = &[
    "earnings_upcoming",
    "earnings_released",
    "news_critical",
    "press_release",
    "price_alert",
    "weekly52_high",
    "weekly52_low",
    "volume_spike",
    "dividend",
    "split",
    "buyback",
    "sec_filing",
    "analyst_grade",
    "macro_event",
    "portfolio_pre_market",
    "portfolio_post_market",
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
        assert!(!p.should_deliver(&ev(EventKind::EarningsReleased, Severity::High, vec!["AAPL"])));
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
        assert!(p.should_deliver(&ev(EventKind::EarningsReleased, Severity::High, vec!["AAPL"])));
        assert!(!p.should_deliver(&ev(EventKind::NewsCritical, Severity::High, vec!["AAPL"])));
    }

    #[test]
    fn block_list_overrides_allow_list() {
        let p = NotificationPrefs {
            allow_kinds: Some(vec!["earnings_released".into(), "news_critical".into()]),
            blocked_kinds: vec!["news_critical".into()],
            ..Default::default()
        };
        assert!(p.should_deliver(&ev(EventKind::EarningsReleased, Severity::High, vec!["AAPL"])));
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
        };
        store.save(&a, &p).unwrap();
        let loaded = store.load(&a);
        assert!(!loaded.enabled);
        assert!(loaded.portfolio_only);
        assert_eq!(loaded.min_severity, Severity::High);
        assert_eq!(loaded.allow_kinds.as_deref(), Some(&["split".into()][..]));
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
            NewsCritical,
            PressRelease,
            PriceAlert {
                pct_change_bps: 100,
                window: "5m".into(),
            },
            Weekly52High,
            Weekly52Low,
            VolumeSpike,
            Dividend,
            Split,
            Buyback,
            SecFiling {
                form: String::new(),
            },
            AnalystGrade,
            MacroEvent,
            PortfolioPreMarket,
            PortfolioPostMarket,
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
    fn malformed_json_falls_back_without_panic() {
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let a = actor();
        std::fs::write(store.path_for(&a), "not json").unwrap();
        let p = store.load(&a);
        assert!(p.enabled, "解析失败时应回到默认（放行），不影响推送链路");
    }
}
