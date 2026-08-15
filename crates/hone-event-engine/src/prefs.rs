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

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::quiet::local_time_in_quiet_window;
use hone_core::{ActorIdentity, HoneError, HoneResult};
use serde::{Deserialize, Serialize};

use crate::event::{EventKind, MarketEvent, Severity};
use crate::unified_digest::DigestSlot;
static CLOUD_NOTIFICATION_PREFS: OnceLock<RwLock<Option<CloudPgRuntime>>> = OnceLock::new();

pub fn configure_cloud_notification_prefs(postgres: Option<CloudPgRuntime>) {
    let lock = CLOUD_NOTIFICATION_PREFS.get_or_init(|| RwLock::new(None));
    match lock.write() {
        Ok(mut guard) => *guard = postgres,
        Err(error) => tracing::warn!("notification prefs cloud runtime lock poisoned: {error}"),
    }
}

fn cloud_notification_prefs() -> Option<CloudPgRuntime> {
    CLOUD_NOTIFICATION_PREFS
        .get()
        .and_then(|lock| lock.read().ok().and_then(|guard| guard.clone()))
}

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
    /// 盘中价格首次达到即时推阈值后，再次即时提醒所需的最小前进步长（百分点）。
    /// `None` → 继承系统 `thresholds.price_band_min_advance_pct`。
    /// 例如首次阈值 8%、本字段 4% 时，系统候选档会形成 8% / 12% / 16%…
    /// 的 actor 级提醒阶梯；不足 4 个百分点的中间档进入摘要。
    pub price_realert_step_pct_override: Option<f64>,
    /// 当 router 能从事件 payload 读到 portfolio_weight / portfolio_weight_pct 时，
    /// 高仓位标的允许使用更敏感的用户阈值直推；低仓位仍受系统最小直推阈值保护。
    pub large_position_weight_pct: Option<f64>,
    /// 全局 digest Pass 2 personalize 时使用的"投资风格"自由文本。
    /// 例如:"长期叙事派,重视行业结构性叙事,轻视短期估值/技术形态/分析师评级"。
    /// LLM 会按此风格剔除用户视角下的噪音。`None` → 走 baseline 排序,不做风格过滤。
    #[serde(alias = "investment_global_style")]
    pub mainline_style: Option<String>,
    /// 用户在「我的 · 设置」里手写的投资风格。存在时**优先于**系统蒸馏出来的
    /// `mainline_style`（见 [`NotificationPrefs::effective_mainline_style`]），
    /// 且后台蒸馏不会覆盖它 —— 用户显式表达的偏好不该被自动流程改写。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mainline_style_user: Option<String>,
    /// 每个 ticker 的投资主线。LLM 在 personalize 时按此重排:印证主线的优先,
    /// 证伪保留并标注,主线视角下的噪音剔除。例如 `MU → "看 NAND/DRAM 长期
    /// 稀缺性,噪音是估值过热/单日大涨大跌"`。`None` / 空 map → 不做 per-ticker 重排。
    #[serde(alias = "investment_theses")]
    pub mainline_by_ticker: Option<HashMap<String, String>>,
    /// **系统蒸馏元数据**(2026-04-26 起):`mainline_by_ticker` / `mainline_style`
    /// 由后台 cron 按"缺失持仓主线优先、覆盖完整后每周刷新"策略读取用户 sandbox
    /// `company_profiles/*/profile.md` 自动蒸馏写入,用户不再通过 NL tool 直接编辑。
    /// 本字段是 RFC3339 时间戳记录最近一次蒸馏成功时刻,让前端可以展示"上次更新"
    /// 和判断是否需要手动刷一次。`None` = 还没蒸过(老数据兼容)。
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
/// 实际定义在 `hone_core::quiet::QuietHours`；这里 re-export 是为了保留
/// `hone_event_engine::prefs::QuietHours` 的既有导入路径。
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
            price_realert_step_pct_override: None,
            large_position_weight_pct: None,
            mainline_style: None,
            mainline_style_user: None,
            mainline_by_ticker: None,
            last_mainline_distilled_at: None,
            mainline_distill_skipped: Vec::new(),
            quiet_hours: None,
        }
    }
}

/// 对一个可继承的 actor 偏好字段做增量更新。
///
/// `Keep` 用于一次请求里不碰该字段；`Inherit` 清除 actor override；
/// `Set` 写入具体值。该三态不能用 `Option<T>` 表达，因为 `None` 已经是
/// “恢复继承”的业务含义。
#[derive(Debug, Clone, PartialEq)]
pub enum PreferenceUpdate<T> {
    Keep,
    Inherit,
    Set(T),
}

impl<T> Default for PreferenceUpdate<T> {
    fn default() -> Self {
        Self::Keep
    }
}

/// 可由普通 Agent 对话或管理 API 调整的确定性通知字段。
///
/// 提示词、模型、分类策略不属于此补丁；它们继续由系统配置和专用管理流程维护。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NotificationDeliveryPatch {
    pub timezone: PreferenceUpdate<String>,
    pub digest_slots: PreferenceUpdate<Vec<DigestSlot>>,
    pub price_high_pct_override: PreferenceUpdate<f64>,
    pub price_high_pct_up_override: PreferenceUpdate<f64>,
    pub price_high_pct_down_override: PreferenceUpdate<f64>,
    pub price_realert_step_pct_override: PreferenceUpdate<f64>,
    pub large_position_weight_pct: PreferenceUpdate<f64>,
    pub quiet_hours: PreferenceUpdate<QuietHours>,
}

/// 价格候选事件与即时推策略的系统默认值。
///
/// PricePoller 负责按 `candidate_first_pct + N * candidate_step_pct` 产生全局候选档；
/// router 再用 actor 偏好计算首次直推阈值和重复步长。把这组输入封装成值对象，确保
/// 路由执行、聊天概览与管理 API 使用同一份策略解析逻辑。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct PriceAlertPolicyDefaults {
    pub candidate_first_pct: f64,
    pub candidate_step_pct: f64,
    pub repeat_step_pct: f64,
    pub min_direct_pct: f64,
    pub large_position_weight_pct: f64,
    pub close_direct_enabled: bool,
}

impl Default for PriceAlertPolicyDefaults {
    fn default() -> Self {
        Self {
            candidate_first_pct: 6.0,
            candidate_step_pct: 2.0,
            repeat_step_pct: 2.0,
            min_direct_pct: 6.0,
            large_position_weight_pct: 20.0,
            close_direct_enabled: false,
        }
    }
}

impl From<&hone_core::config::EventEngineThresholds> for PriceAlertPolicyDefaults {
    fn from(thresholds: &hone_core::config::EventEngineThresholds) -> Self {
        Self {
            candidate_first_pct: thresholds.price_alert_high_pct,
            candidate_step_pct: thresholds.price_realert_step_pct,
            repeat_step_pct: thresholds.price_band_min_advance_pct,
            min_direct_pct: thresholds.price_min_direct_pct,
            large_position_weight_pct: thresholds.large_position_weight_pct,
            close_direct_enabled: thresholds.price_close_direct_enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PricePolicySource {
    System,
    ActorCommon,
    ActorDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectivePriceDirectionPolicy {
    /// 用户表达或系统继承得到的首次阈值，尚未应用非大仓位系统直推地板。
    pub configured_first_pct: f64,
    pub configured_first_source: PricePolicySource,
    /// 普通仓位实际参与路由判断的首次阈值。
    pub first_direct_pct: f64,
    /// 用户配置低于系统非大仓位直推地板时为 true；保留原始来源的同时，
    /// 让解释层明确最终阈值并非原样采用用户输入。
    pub system_floor_applied: bool,
    /// 达到大仓位权重门槛时实际参与路由判断的首次阈值。
    pub large_position_first_direct_pct: f64,
    /// 全局候选 band 网格中第一条不低于普通仓位阈值的可观测档。
    pub first_candidate_band_pct: f64,
    /// 全局候选 band 网格中第一条不低于大仓位阈值的可观测档。
    pub large_position_first_candidate_band_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectivePriceAlertPolicy {
    pub up: EffectivePriceDirectionPolicy,
    pub down: EffectivePriceDirectionPolicy,
    pub repeat_step_pct: f64,
    pub repeat_step_source: PricePolicySource,
    pub candidate_first_pct: f64,
    pub candidate_step_pct: f64,
    pub min_direct_pct: f64,
    pub large_position_weight_pct: f64,
    pub close_direct_enabled: bool,
}

impl EffectivePriceAlertPolicy {
    pub fn direction_for_change(&self, change_pct: f64) -> &EffectivePriceDirectionPolicy {
        if change_pct >= 0.0 {
            &self.up
        } else {
            &self.down
        }
    }

    pub fn first_direct_pct(&self, change_pct: f64, is_large_position: bool) -> f64 {
        let direction = self.direction_for_change(change_pct);
        if is_large_position {
            direction.large_position_first_direct_pct
        } else {
            direction.first_direct_pct
        }
    }

    /// 返回从首次可观测候选档开始的前 `count` 个实际提醒示例。
    ///
    /// 候选档由全局 poller 网格产生；actor 重复步长只是最小前进量。因此当两者
    /// 不整除时，下一档向上落到第一条满足最小前进量的候选档，查询结果不会承诺
    /// source 根本不会产生的精度。
    pub fn sample_candidate_bands(&self, upward: bool, count: usize) -> Vec<f64> {
        let direction = if upward { &self.up } else { &self.down };
        let mut bands = Vec::with_capacity(count);
        let mut current = direction.first_candidate_band_pct;
        let effective_advance = if self.repeat_step_pct > 0.0 {
            self.repeat_step_pct
        } else {
            self.candidate_step_pct
        };
        for _ in 0..count {
            bands.push(round_pct(current));
            current = next_candidate_band_at_or_above(
                current + effective_advance,
                self.candidate_first_pct,
                self.candidate_step_pct,
            );
        }
        bands
    }
}

impl NotificationPrefs {
    /// 个性化推送实际使用的投资风格：用户手写的优先，其次是系统蒸馏的。
    pub fn effective_mainline_style(&self) -> Option<&str> {
        self.mainline_style_user
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                self.mainline_style
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
            })
    }

    /// 按偏好判断是否应推送该事件。
    pub fn should_deliver(&self, event: &MarketEvent) -> bool {
        self.should_deliver_with_severity(event, event.severity)
    }

    /// 按路由已经为当前 actor 解析出的最终 severity 过滤。
    ///
    /// Router 可能因价格阈值或 immediate_kinds 对共享事件升/降级；此时不能再用
    /// 原始事件 severity 判断 min_severity，否则会出现“已降为 Medium 仍穿过 High
    /// 过滤”或“已升为 High 却被原始 Low 过滤”的执行分叉。
    pub fn should_deliver_with_severity(
        &self,
        event: &MarketEvent,
        effective_severity: Severity,
    ) -> bool {
        if !self.enabled {
            return false;
        }
        if effective_severity.rank() < self.min_severity.rank() {
            return false;
        }
        if self.portfolio_only && event.symbols.is_empty() {
            return false;
        }
        if self.source_blocked(&event.source) {
            return false;
        }
        if let Some(allow) = &self.allow_sources
            && !allow.iter().any(|pat| source_matches(&event.source, pat))
        {
            return false;
        }
        let tag = kind_tag(&event.kind);
        if self.blocked_kinds.iter().any(|k| k == tag) {
            return false;
        }
        if let Some(allow) = &self.allow_kinds
            && !allow.iter().any(|k| k == tag)
        {
            return false;
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

    /// 在副本上应用确定性通知补丁并整体校验；只有全部合法时才替换当前值。
    ///
    /// 这保证一条自然语言指令即使同时修改多个关联字段，也不会留下部分生效状态。
    pub fn apply_delivery_patch(&mut self, patch: NotificationDeliveryPatch) -> HoneResult<()> {
        let mut candidate = self.clone();
        apply_optional_update(&mut candidate.timezone, patch.timezone);
        apply_optional_update(&mut candidate.digest_slots, patch.digest_slots);
        apply_optional_update(
            &mut candidate.price_high_pct_override,
            patch.price_high_pct_override,
        );
        apply_optional_update(
            &mut candidate.price_high_pct_up_override,
            patch.price_high_pct_up_override,
        );
        apply_optional_update(
            &mut candidate.price_high_pct_down_override,
            patch.price_high_pct_down_override,
        );
        apply_optional_update(
            &mut candidate.price_realert_step_pct_override,
            patch.price_realert_step_pct_override,
        );
        apply_optional_update(
            &mut candidate.large_position_weight_pct,
            patch.large_position_weight_pct,
        );
        apply_optional_update(&mut candidate.quiet_hours, patch.quiet_hours);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// 校验一份可持久化通知偏好。Agent 与 HTTP API 必须共用此入口。
    pub fn validate(&self) -> HoneResult<()> {
        validate_kind_tags("blocked_kinds", &self.blocked_kinds)?;
        if let Some(allow_kinds) = &self.allow_kinds {
            validate_kind_tags("allow_kinds", allow_kinds)?;
        }
        if let Some(immediate_kinds) = &self.immediate_kinds {
            validate_kind_tags("immediate_kinds", immediate_kinds)?;
        }

        if let Some(timezone) = &self.timezone {
            let timezone = timezone.trim();
            if timezone.is_empty() || timezone.parse::<chrono_tz::Tz>().is_err() {
                return Err(HoneError::Config(format!(
                    "未知 IANA 时区 {timezone:?};示例:Asia/Shanghai、America/New_York、Europe/London"
                )));
            }
        }

        if let Some(slots) = &self.digest_slots {
            let mut slot_ids = HashSet::with_capacity(slots.len());
            let mut slot_times = HashSet::with_capacity(slots.len());
            for slot in slots {
                let slot_id = slot.id.trim();
                if slot_id.is_empty() {
                    return Err(HoneError::Config(
                        "digest_slots 的 id 不能为空;示例:premarket、postmarket".into(),
                    ));
                }
                if !slot_ids.insert(slot_id) {
                    return Err(HoneError::Config(format!(
                        "digest_slots 含重复 id {slot_id:?};每个槽位必须有稳定且唯一的 id"
                    )));
                }
                validate_hhmm("digest_slots.time", &slot.time)?;
                if !slot_times.insert(slot.time.as_str()) {
                    return Err(HoneError::Config(format!(
                        "digest_slots 含重复时刻 {:?};同一时刻只保留一个具名槽位",
                        slot.time
                    )));
                }
                if let Some(label) = &slot.label {
                    let label = label.trim();
                    if label.is_empty() {
                        return Err(HoneError::Config(
                            "digest_slots.label 不能为空字符串;不需要名称时请传 null".into(),
                        ));
                    }
                    if label.chars().count() > 64 {
                        return Err(HoneError::Config(
                            "digest_slots.label 最多 64 个字符".into(),
                        ));
                    }
                }
            }
        }

        validate_optional_percentage(
            "price_high_pct_override",
            self.price_high_pct_override,
            50.0,
        )?;
        validate_optional_percentage(
            "price_high_pct_up_override",
            self.price_high_pct_up_override,
            50.0,
        )?;
        validate_optional_percentage(
            "price_high_pct_down_override",
            self.price_high_pct_down_override,
            50.0,
        )?;
        validate_optional_percentage(
            "price_realert_step_pct_override",
            self.price_realert_step_pct_override,
            50.0,
        )?;
        validate_optional_percentage(
            "large_position_weight_pct",
            self.large_position_weight_pct,
            100.0,
        )?;

        if let Some(quiet_hours) = &self.quiet_hours {
            validate_hhmm("quiet_hours.from", &quiet_hours.from)?;
            validate_hhmm("quiet_hours.to", &quiet_hours.to)?;
            if quiet_hours.from == quiet_hours.to {
                return Err(HoneError::Config(
                    "quiet_hours 的 from 与 to 不能相等(空区间);若想全天关闭推送请使用总开关"
                        .into(),
                ));
            }
            validate_kind_tags("quiet_hours.exempt_kinds", &quiet_hours.exempt_kinds)?;
            if let Some(slots) = &self.digest_slots {
                let overlapping_slots = slots
                    .iter()
                    .filter(|slot| local_time_in_quiet_window(&slot.time, quiet_hours))
                    .map(|slot| format!("{} ({})", slot.id, slot.time))
                    .collect::<Vec<_>>();
                if !overlapping_slots.is_empty() {
                    return Err(HoneError::Config(format!(
                        "quiet_hours {}–{} 会吞掉 digest slot [{}];请调整槽位或勿扰时段",
                        quiet_hours.from,
                        quiet_hours.to,
                        overlapping_slots.join(", ")
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn effective_price_alert_policy(
        &self,
        defaults: PriceAlertPolicyDefaults,
    ) -> EffectivePriceAlertPolicy {
        let candidate_first_pct = sanitize_positive(defaults.candidate_first_pct, 6.0);
        let candidate_step_pct = sanitize_positive(defaults.candidate_step_pct, 2.0);
        let repeat_step_pct = self
            .price_realert_step_pct_override
            .unwrap_or(defaults.repeat_step_pct)
            .max(0.0);
        let min_direct_pct = defaults.min_direct_pct.max(0.0);
        let common_first = self.price_high_pct_override;
        let up = effective_direction_policy(
            self.price_high_pct_up_override,
            common_first,
            candidate_first_pct,
            candidate_step_pct,
            min_direct_pct,
        );
        let down = effective_direction_policy(
            self.price_high_pct_down_override,
            common_first,
            candidate_first_pct,
            candidate_step_pct,
            min_direct_pct,
        );
        EffectivePriceAlertPolicy {
            up,
            down,
            repeat_step_pct,
            repeat_step_source: if self.price_realert_step_pct_override.is_some() {
                PricePolicySource::ActorCommon
            } else {
                PricePolicySource::System
            },
            candidate_first_pct,
            candidate_step_pct,
            min_direct_pct,
            large_position_weight_pct: self
                .large_position_weight_pct
                .unwrap_or(defaults.large_position_weight_pct)
                .max(0.0),
            close_direct_enabled: defaults.close_direct_enabled,
        }
    }
}

fn effective_direction_policy(
    direction_override: Option<f64>,
    common_override: Option<f64>,
    candidate_first_pct: f64,
    candidate_step_pct: f64,
    min_direct_pct: f64,
) -> EffectivePriceDirectionPolicy {
    let (configured_first_pct, configured_first_source) =
        match (direction_override, common_override) {
            (Some(value), _) => (value, PricePolicySource::ActorDirection),
            (None, Some(value)) => (value, PricePolicySource::ActorCommon),
            (None, None) => (candidate_first_pct, PricePolicySource::System),
        };
    let first_direct_pct = configured_first_pct.max(min_direct_pct);
    EffectivePriceDirectionPolicy {
        configured_first_pct,
        configured_first_source,
        first_direct_pct,
        system_floor_applied: first_direct_pct > configured_first_pct + 1e-9,
        large_position_first_direct_pct: configured_first_pct,
        first_candidate_band_pct: next_candidate_band_at_or_above(
            first_direct_pct,
            candidate_first_pct,
            candidate_step_pct,
        ),
        large_position_first_candidate_band_pct: next_candidate_band_at_or_above(
            configured_first_pct,
            candidate_first_pct,
            candidate_step_pct,
        ),
    }
}

fn next_candidate_band_at_or_above(target: f64, first: f64, step: f64) -> f64 {
    if target <= first {
        return round_pct(first);
    }
    let lanes = ((target - first) / step - 1e-9).ceil().max(0.0);
    round_pct(first + lanes * step)
}

fn sanitize_positive(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn round_pct(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn apply_optional_update<T>(target: &mut Option<T>, update: PreferenceUpdate<T>) {
    match update {
        PreferenceUpdate::Keep => {}
        PreferenceUpdate::Inherit => *target = None,
        PreferenceUpdate::Set(value) => *target = Some(value),
    }
}

fn validate_kind_tags(field: &str, tags: &[String]) -> HoneResult<()> {
    if let Some(invalid_tag) = first_invalid_kind_tag(tags.iter().map(String::as_str)) {
        return Err(HoneError::Config(format!(
            "{field} 含未知 tag '{invalid_tag}';合法清单:{}",
            ALL_KIND_TAGS.join(", ")
        )));
    }
    Ok(())
}

fn validate_hhmm(field: &str, value: &str) -> HoneResult<()> {
    chrono::NaiveTime::parse_from_str(value, "%H:%M")
        .map(|_| ())
        .map_err(|_| HoneError::Config(format!("{field} 必须是 HH:MM (24h),收到 {value:?}")))
}

fn validate_optional_percentage(field: &str, value: Option<f64>, max: f64) -> HoneResult<()> {
    if let Some(value) = value
        && !(value.is_finite() && value > 0.0 && value <= max)
    {
        return Err(HoneError::Config(format!(
            "{field} 必须在 (0, {max}] 范围,收到 {value}"
        )));
    }
    Ok(())
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
    cloud: Option<CloudPgRuntime>,
}

impl FilePrefsStorage {
    pub fn new(dir: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let dir = dir.into();
        let cloud = cloud_notification_prefs();
        if cloud.is_none() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(Self { dir, cloud })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path_for(&self, actor: &ActorIdentity) -> PathBuf {
        self.dir.join(format!("{}.json", actor_slug(actor)))
    }

    pub fn load_many(&self, actors: &[ActorIdentity]) -> Vec<NotificationPrefs> {
        if let Some(postgres) = self.cloud.clone() {
            let actor_storage_keys = actors.iter().map(actor_slug).collect::<Vec<_>>();
            let query_keys = actor_storage_keys.clone();
            match run_cloud_notification_prefs(async move {
                postgres
                    .get_notification_prefs_many_cached(&query_keys)
                    .await
            }) {
                Ok(records) => {
                    return actor_storage_keys
                        .iter()
                        .map(|key| {
                            records
                                .get(key)
                                .cloned()
                                .and_then(|value| {
                                    serde_json::from_value::<NotificationPrefs>(value)
                                        .map_err(|err| {
                                            tracing::warn!(
                                                actor_storage_key = %key,
                                                "cloud notif prefs parse failed in batch: {err}"
                                            );
                                            err
                                        })
                                        .ok()
                                })
                                .unwrap_or_default()
                        })
                        .collect();
                }
                Err(err) => {
                    tracing::warn!(
                        "cloud notif prefs batch load failed: {err}; falling back to per-actor load"
                    );
                }
            }
        }

        actors.iter().map(|actor| self.load(actor)).collect()
    }
}

impl PrefsProvider for FilePrefsStorage {
    fn load(&self, actor: &ActorIdentity) -> NotificationPrefs {
        if let Some(postgres) = self.cloud.clone() {
            let actor_storage_key = actor_slug(actor);
            match run_cloud_notification_prefs(async move {
                postgres.get_notification_prefs(&actor_storage_key).await
            }) {
                Ok(Some(value)) => match serde_json::from_value::<NotificationPrefs>(value) {
                    Ok(prefs) => return prefs,
                    Err(err) => {
                        tracing::warn!(
                            "cloud notif prefs parse failed actor_storage_key={}: {err}; falling back to default",
                            actor_slug(actor)
                        );
                    }
                },
                Ok(None) => return NotificationPrefs::default(),
                Err(err) => {
                    tracing::warn!(
                        "cloud notif prefs load failed actor_storage_key={}: {err}; falling back to default",
                        actor_slug(actor)
                    );
                }
            }
            return NotificationPrefs::default();
        }

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
        if let Some(postgres) = self.cloud.clone() {
            let actor_storage_key = actor_slug(actor);
            let value = serde_json::to_value(prefs)?;
            return run_cloud_notification_prefs(async move {
                postgres
                    .upsert_notification_prefs(&actor_storage_key, value)
                    .await
            })
            .map_err(anyhow::Error::from);
        }

        let path = self.path_for(actor);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(prefs)?;
        std::fs::write(&path, text)?;
        Ok(())
    }
}

/// 通知偏好读写是推送链路的热路径,和 memory/ 那七个模块用同一个共享长驻 runtime。
///
/// 此前这里是「每次调用 `std::thread::spawn` + 内部再 `Runtime::new()`」的反模式,
/// 与 `62d0c889` 修掉的 cloud cron 完全同形——生产实测那个写法让进程 26 分钟烧掉
/// 47 CPU 分钟。多 agent 审计只扫了 `memory/`,漏掉了 event-engine 这一处。
fn run_cloud_notification_prefs<T, F>(future: F) -> HoneResult<T>
where
    T: Send + 'static,
    F: Future<Output = HoneResult<T>> + Send + 'static,
{
    hone_core::cloud_sync::run_cloud_sync(future, None, "cloud notification prefs operation")
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

    fn actor_identity_fixture() -> ActorIdentity {
        ActorIdentity::new("telegram", "u1", None::<&str>).unwrap()
    }

    fn market_event_fixture(
        kind: EventKind,
        severity: Severity,
        symbols: Vec<&str>,
    ) -> MarketEvent {
        MarketEvent {
            id: "x".into(),
            kind,
            severity,
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
        let prefs = NotificationPrefs::default();
        assert!(prefs.should_deliver(&market_event_fixture(
            EventKind::NewsCritical,
            Severity::Low,
            vec!["AAPL"]
        )));
        assert!(prefs.should_deliver(&market_event_fixture(
            EventKind::MacroEvent,
            Severity::Low,
            vec![]
        )));
    }

    #[test]
    fn disabled_blocks_all() {
        let prefs = NotificationPrefs {
            enabled: false,
            ..Default::default()
        };
        assert!(!prefs.should_deliver(&market_event_fixture(
            EventKind::EarningsReleased,
            Severity::High,
            vec!["AAPL"]
        )));
    }

    #[test]
    fn portfolio_only_drops_symbol_less_events() {
        let prefs = NotificationPrefs {
            portfolio_only: true,
            ..Default::default()
        };
        assert!(prefs.should_deliver(&market_event_fixture(
            EventKind::NewsCritical,
            Severity::Low,
            vec!["AAPL"]
        )));
        assert!(!prefs.should_deliver(&market_event_fixture(
            EventKind::MacroEvent,
            Severity::Low,
            vec![]
        )));
    }

    #[test]
    fn min_severity_filters_lower_tiers() {
        let prefs = NotificationPrefs {
            min_severity: Severity::High,
            ..Default::default()
        };
        assert!(!prefs.should_deliver(&market_event_fixture(
            EventKind::NewsCritical,
            Severity::Low,
            vec!["AAPL"]
        )));
        assert!(!prefs.should_deliver(&market_event_fixture(
            EventKind::NewsCritical,
            Severity::Medium,
            vec!["AAPL"]
        )));
        assert!(prefs.should_deliver(&market_event_fixture(
            EventKind::NewsCritical,
            Severity::High,
            vec!["AAPL"]
        )));
    }

    #[test]
    fn allow_list_is_whitelist() {
        let prefs = NotificationPrefs {
            allow_kinds: Some(vec!["earnings_released".into()]),
            ..Default::default()
        };
        assert!(prefs.should_deliver(&market_event_fixture(
            EventKind::EarningsReleased,
            Severity::High,
            vec!["AAPL"]
        )));
        assert!(!prefs.should_deliver(&market_event_fixture(
            EventKind::NewsCritical,
            Severity::High,
            vec!["AAPL"]
        )));
    }

    #[test]
    fn block_list_overrides_allow_list() {
        let prefs = NotificationPrefs {
            allow_kinds: Some(vec!["earnings_released".into(), "news_critical".into()]),
            blocked_kinds: vec!["news_critical".into()],
            ..Default::default()
        };
        assert!(prefs.should_deliver(&market_event_fixture(
            EventKind::EarningsReleased,
            Severity::High,
            vec!["AAPL"]
        )));
        assert!(!prefs.should_deliver(&market_event_fixture(
            EventKind::NewsCritical,
            Severity::High,
            vec!["AAPL"]
        )));
    }

    #[test]
    fn file_storage_roundtrip() {
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let actor_id = actor_identity_fixture();
        // 缺失文件 → 默认
        let loaded = store.load(&actor_id);
        assert!(loaded.enabled);
        // 写入 → 读回
        let prefs = NotificationPrefs {
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
            price_realert_step_pct_override: Some(4.0),
            large_position_weight_pct: Some(20.0),
            mainline_style: Some("长期叙事派".into()),
            mainline_style_user: None,
            mainline_by_ticker: Some({
                let mut mainlines_by_ticker = HashMap::new();
                mainlines_by_ticker.insert("AAPL".into(), "看现金流 + 回购".into());
                mainlines_by_ticker
            }),
            last_mainline_distilled_at: Some("2026-04-26T09:00:00Z".into()),
            mainline_distill_skipped: vec!["XYZ".into()],
            quiet_hours: Some(QuietHours {
                from: "23:00".into(),
                to: "07:00".into(),
                exempt_kinds: vec!["earnings_released".into()],
            }),
        };
        store.save(&actor_id, &prefs).unwrap();
        let loaded = store.load(&actor_id);
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
        assert_eq!(loaded.price_realert_step_pct_override, Some(4.0));
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
        let prefs = NotificationPrefs::default();
        assert!(prefs.timezone.is_none());
        assert!(prefs.digest_slots.is_none());
        assert!(prefs.price_high_pct_override.is_none());
        assert!(prefs.immediate_kinds.is_none());
        assert!(!prefs.quiet_mode);
        assert!(prefs.allow_sources.is_none());
        assert!(prefs.blocked_sources.is_empty());
        assert!(prefs.price_high_pct_up_override.is_none());
        assert!(prefs.price_high_pct_down_override.is_none());
        assert!(prefs.price_realert_step_pct_override.is_none());
        assert!(prefs.large_position_weight_pct.is_none());
        assert!(prefs.mainline_style.is_none());
        assert!(prefs.mainline_by_ticker.is_none());
    }

    #[test]
    fn effective_price_policy_resolves_actor_ladder_against_system_candidates() {
        let prefs = NotificationPrefs {
            price_high_pct_override: Some(8.0),
            price_realert_step_pct_override: Some(4.0),
            ..Default::default()
        };
        let policy = prefs.effective_price_alert_policy(PriceAlertPolicyDefaults {
            candidate_first_pct: 6.0,
            candidate_step_pct: 2.0,
            repeat_step_pct: 2.0,
            min_direct_pct: 6.0,
            ..Default::default()
        });

        assert_eq!(policy.up.first_direct_pct, 8.0);
        assert_eq!(policy.down.first_direct_pct, 8.0);
        assert_eq!(
            policy.up.configured_first_source,
            PricePolicySource::ActorCommon
        );
        assert_eq!(policy.repeat_step_pct, 4.0);
        assert_eq!(policy.repeat_step_source, PricePolicySource::ActorCommon);
        assert_eq!(
            policy.sample_candidate_bands(true, 3),
            vec![8.0, 12.0, 16.0]
        );
        assert_eq!(
            policy.sample_candidate_bands(false, 3),
            vec![8.0, 12.0, 16.0]
        );
    }

    #[test]
    fn effective_price_policy_reports_observable_grid_without_fake_precision() {
        let prefs = NotificationPrefs {
            price_high_pct_up_override: Some(7.0),
            price_realert_step_pct_override: Some(3.0),
            ..Default::default()
        };
        let policy = prefs.effective_price_alert_policy(PriceAlertPolicyDefaults::default());

        assert_eq!(policy.up.first_direct_pct, 7.0);
        assert_eq!(policy.up.first_candidate_band_pct, 8.0);
        assert_eq!(
            policy.sample_candidate_bands(true, 3),
            vec![8.0, 12.0, 16.0]
        );
        assert_eq!(policy.down.first_direct_pct, 6.0);
        assert_eq!(
            policy.down.configured_first_source,
            PricePolicySource::System
        );
    }

    #[test]
    fn effective_price_policy_keeps_system_floor_except_for_large_positions() {
        let prefs = NotificationPrefs {
            price_high_pct_override: Some(4.0),
            large_position_weight_pct: Some(25.0),
            ..Default::default()
        };
        let policy = prefs.effective_price_alert_policy(PriceAlertPolicyDefaults::default());

        assert_eq!(policy.up.first_direct_pct, 6.0);
        assert!(policy.up.system_floor_applied);
        assert_eq!(policy.up.large_position_first_direct_pct, 4.0);
        assert_eq!(policy.large_position_weight_pct, 25.0);
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
        let prefs: NotificationPrefs =
            serde_json::from_str(json).expect("legacy prefs JSON should load");
        assert_eq!(prefs.mainline_style.as_deref(), Some("长期叙事派"));
        assert_eq!(
            prefs
                .mainline_by_ticker
                .as_ref()
                .and_then(|m| m.get("AAPL"))
                .map(String::as_str),
            Some("看现金流 + 回购")
        );
        assert_eq!(
            prefs.last_mainline_distilled_at.as_deref(),
            Some("2026-04-26T09:00:00Z")
        );
        assert_eq!(prefs.mainline_distill_skipped, vec!["XYZ".to_string()]);
    }

    #[test]
    fn new_per_actor_fields_missing_in_old_json_fall_back() {
        // 老 prefs 文件没有这 4 个字段;serde(default) 应让加载继续走默认。
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let actor_id = actor_identity_fixture();
        std::fs::write(
            store.path_for(&actor_id),
            r#"{"enabled":true,"portfolio_only":false,"min_severity":"low","blocked_kinds":[]}"#,
        )
        .unwrap();
        let loaded = store.load(&actor_id);
        assert!(loaded.timezone.is_none());
        assert!(loaded.digest_slots.is_none());
        assert!(loaded.price_high_pct_override.is_none());
        assert!(loaded.immediate_kinds.is_none());
        assert!(!loaded.quiet_mode);
        assert!(loaded.allow_sources.is_none());
        assert!(loaded.blocked_sources.is_empty());
        assert!(loaded.price_high_pct_up_override.is_none());
        assert!(loaded.price_high_pct_down_override.is_none());
        assert!(loaded.price_realert_step_pct_override.is_none());
        assert!(loaded.large_position_weight_pct.is_none());
    }

    #[test]
    fn source_allow_and_block_lists_filter_events() {
        let mut event = market_event_fixture(EventKind::NewsCritical, Severity::High, vec!["AAPL"]);
        event.source = "fmp.stock_news:reuters.com".into();
        let prefs = NotificationPrefs {
            allow_sources: Some(vec!["reuters.com".into()]),
            ..Default::default()
        };
        assert!(prefs.should_deliver(&event));

        event.source = "telegram.channel:watcherguru".into();
        assert!(!prefs.should_deliver(&event));

        let prefs = NotificationPrefs {
            blocked_sources: vec!["watcherguru".into()],
            ..Default::default()
        };
        assert!(!prefs.should_deliver(&event));
    }

    #[test]
    fn file_storage_missing_fields_fall_back_to_default() {
        // 用户只写了 enabled=false，其他字段缺失；serde(default) 保证兼容。
        let dir = tempdir().unwrap();
        let store = FilePrefsStorage::new(dir.path()).unwrap();
        let actor_id = actor_identity_fixture();
        std::fs::write(store.path_for(&actor_id), r#"{"enabled": false}"#).unwrap();
        let loaded = store.load(&actor_id);
        assert!(!loaded.enabled);
        assert_eq!(loaded.min_severity, Severity::Low);
        assert!(!loaded.portfolio_only);
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
        let prefs = NotificationPrefs {
            digest_slots: Some(vec![DigestSlot {
                id: "premarket".into(),
                time: "08:30".into(),
                label: Some("盘前".into()),
                floor_macro: Some(2),
            }]),
            ..Default::default()
        };
        let slots = prefs.effective_digest_slots().unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].id, "premarket");
        assert_eq!(slots[0].time, "08:30");
    }

    #[test]
    fn effective_digest_slots_returns_none_when_unset() {
        let prefs = NotificationPrefs::default();
        assert!(prefs.effective_digest_slots().is_none());
    }

    #[test]
    fn effective_digest_slots_preserves_empty_disable_semantics() {
        let prefs = NotificationPrefs {
            digest_slots: Some(vec![]),
            ..Default::default()
        };
        // Some([]) 必须保留 —— 用户主动关 digest 的语义。
        assert_eq!(prefs.effective_digest_slots(), Some(vec![]));
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
        let actor_id = actor_identity_fixture();
        std::fs::write(store.path_for(&actor_id), "not json").unwrap();
        let loaded = store.load(&actor_id);
        assert!(
            loaded.enabled,
            "解析失败时应回到默认（放行），不影响推送链路"
        );
    }

    #[test]
    fn delivery_patch_distinguishes_keep_inherit_and_explicit_empty() {
        let original_slots = vec![DigestSlot {
            id: "premarket".into(),
            time: "08:30".into(),
            label: Some("盘前要闻".into()),
            floor_macro: Some(1),
        }];
        let mut prefs = NotificationPrefs {
            timezone: Some("Asia/Shanghai".into()),
            digest_slots: Some(original_slots.clone()),
            price_high_pct_override: Some(4.0),
            ..Default::default()
        };

        prefs
            .apply_delivery_patch(NotificationDeliveryPatch {
                timezone: PreferenceUpdate::Keep,
                digest_slots: PreferenceUpdate::Set(Vec::new()),
                price_high_pct_override: PreferenceUpdate::Inherit,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(prefs.timezone.as_deref(), Some("Asia/Shanghai"));
        assert_eq!(prefs.digest_slots, Some(Vec::new()));
        assert_eq!(prefs.price_high_pct_override, None);

        prefs
            .apply_delivery_patch(NotificationDeliveryPatch {
                digest_slots: PreferenceUpdate::Inherit,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(prefs.digest_slots, None);
    }

    #[test]
    fn invalid_delivery_patch_is_atomic() {
        let original_slots = vec![DigestSlot {
            id: "premarket".into(),
            time: "08:30".into(),
            label: Some("盘前要闻".into()),
            floor_macro: None,
        }];
        let mut prefs = NotificationPrefs {
            timezone: Some("Asia/Shanghai".into()),
            digest_slots: Some(original_slots.clone()),
            price_high_pct_override: Some(4.0),
            ..Default::default()
        };

        let error = prefs
            .apply_delivery_patch(NotificationDeliveryPatch {
                timezone: PreferenceUpdate::Set("America/New_York".into()),
                price_high_pct_override: PreferenceUpdate::Set(99.0),
                ..Default::default()
            })
            .unwrap_err();
        assert!(error.to_string().contains("(0, 50]"));
        assert_eq!(prefs.timezone.as_deref(), Some("Asia/Shanghai"));
        assert_eq!(prefs.digest_slots, Some(original_slots));
        assert_eq!(prefs.price_high_pct_override, Some(4.0));
    }

    #[test]
    fn validate_rejects_duplicate_digest_slot_id_and_time() {
        let duplicate_id = NotificationPrefs {
            digest_slots: Some(vec![
                DigestSlot {
                    id: "market".into(),
                    time: "08:30".into(),
                    label: None,
                    floor_macro: None,
                },
                DigestSlot {
                    id: "market".into(),
                    time: "19:00".into(),
                    label: None,
                    floor_macro: None,
                },
            ]),
            ..Default::default()
        };
        assert!(
            duplicate_id
                .validate()
                .unwrap_err()
                .to_string()
                .contains("重复 id")
        );

        let duplicate_time = NotificationPrefs {
            digest_slots: Some(vec![
                DigestSlot {
                    id: "premarket".into(),
                    time: "08:30".into(),
                    label: None,
                    floor_macro: None,
                },
                DigestSlot {
                    id: "morning".into(),
                    time: "08:30".into(),
                    label: None,
                    floor_macro: None,
                },
            ]),
            ..Default::default()
        };
        assert!(
            duplicate_time
                .validate()
                .unwrap_err()
                .to_string()
                .contains("重复时刻")
        );
    }

    #[test]
    fn quiet_window_end_boundary_can_share_digest_slot() {
        let prefs = NotificationPrefs {
            digest_slots: Some(vec![DigestSlot {
                id: "postmarket".into(),
                time: "07:30".into(),
                label: Some("盘后要闻".into()),
                floor_macro: Some(1),
            }]),
            quiet_hours: Some(QuietHours {
                from: "23:00".into(),
                to: "07:30".into(),
                exempt_kinds: Vec::new(),
            }),
            ..Default::default()
        };
        prefs
            .validate()
            .expect("quiet end uses an exclusive boundary");
    }

    #[test]
    fn validate_checks_directional_and_large_position_percentages() {
        let invalid_direction = NotificationPrefs {
            price_high_pct_up_override: Some(50.1),
            ..Default::default()
        };
        assert!(
            invalid_direction
                .validate()
                .unwrap_err()
                .to_string()
                .contains("price_high_pct_up_override")
        );

        let valid = NotificationPrefs {
            price_high_pct_up_override: Some(6.0),
            price_high_pct_down_override: Some(5.0),
            price_realert_step_pct_override: Some(4.0),
            large_position_weight_pct: Some(20.0),
            ..Default::default()
        };
        valid.validate().unwrap();

        let invalid_step = NotificationPrefs {
            price_realert_step_pct_override: Some(0.0),
            ..Default::default()
        };
        assert!(
            invalid_step
                .validate()
                .unwrap_err()
                .to_string()
                .contains("price_realert_step_pct_override")
        );
    }
}
