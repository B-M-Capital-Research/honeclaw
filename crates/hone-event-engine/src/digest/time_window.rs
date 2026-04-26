//! digest 时区 / HH:MM 窗口判定工具。
//!
//! 两层抽象:
//! - **自由函数**(`in_window` / `shift_hhmm_earlier` / `local_date_key`)是简单的
//!   `FixedOffset` 小时级封装,供 `engine` 和 `spawner::spawn_event_source` 内嵌
//!   的 cron-aligned 分支复用;
//! - **`EffectiveTz`** 是 per-actor 的调度器内部抽象:优先解析 IANA 名称(尊重
//!   DST/历史偏移),失败才退回 FixedOffset。这样用户在 prefs 里写
//!   `"timezone": "America/New_York"` 时能正确处理夏/冬令时切换。

use chrono::{DateTime, Datelike, FixedOffset, NaiveTime, TimeZone, Timelike, Utc};

/// 判断 `now` 对应的本地时间（按 `offset_hours` 解释）是否处于给定 HH:MM 的 60 秒窗口内。
pub fn in_window(now: DateTime<Utc>, hhmm: &str, offset_hours: i32) -> bool {
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    let Ok(target) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
        return false;
    };
    let now_t = NaiveTime::from_hms_opt(local.hour(), local.minute(), 0).unwrap();
    now_t == target
}

/// 把 `HH:MM` 形式的时间点向前偏移 `offset_mins` 分钟,用于 cron-align pollers
/// 计算"比 flush 窗口早 N 分钟去拉数据"的 target。
/// 非法输入按原样返回。跨日回绕取模处理(例如 "00:10" - 30min → "23:40")。
pub fn shift_hhmm_earlier(hhmm: &str, offset_mins: u32) -> String {
    let Ok(t) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
        return hhmm.into();
    };
    let total = t.hour() as i64 * 60 + t.minute() as i64;
    let shifted = (total - offset_mins as i64).rem_euclid(24 * 60);
    format!("{:02}:{:02}", shifted / 60, shifted % 60)
}

/// 当前本地日期（粗略）—— 用于 flush key 防止同一天重复触发。
pub fn local_date_key(now: DateTime<Utc>, offset_hours: i32) -> String {
    let offset =
        FixedOffset::east_opt(offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    format!(
        "{:04}-{:02}-{:02}",
        local.year(),
        local.month(),
        local.day()
    )
}

/// 调度器内部用的"有效时区"——优先 IANA 名称(尊重 DST/历史偏移),否则回到全局
/// FixedOffset。这层抽象让 actor 的 prefs.timezone 与全局 `digest.timezone` 共用同
/// 一套窗口/日期判断函数,不必双份实现。
#[derive(Debug, Clone)]
pub(super) enum EffectiveTz {
    Iana(chrono_tz::Tz),
    Fixed(FixedOffset),
}

impl EffectiveTz {
    pub(super) fn from_actor_prefs(prefs_tz: Option<&str>, fallback_offset_hours: i32) -> Self {
        if let Some(name) = prefs_tz {
            if let Ok(tz) = name.parse::<chrono_tz::Tz>() {
                return EffectiveTz::Iana(tz);
            }
            tracing::warn!(
                "actor prefs.timezone {name:?} 解析失败,回到全局 fallback_offset_hours={fallback_offset_hours}"
            );
        }
        let offset = FixedOffset::east_opt(fallback_offset_hours * 3600)
            .unwrap_or(FixedOffset::east_opt(0).unwrap());
        EffectiveTz::Fixed(offset)
    }

    fn local_hm(&self, now: DateTime<Utc>) -> (u32, u32) {
        match self {
            EffectiveTz::Iana(tz) => {
                let local = tz.from_utc_datetime(&now.naive_utc());
                (local.hour(), local.minute())
            }
            EffectiveTz::Fixed(off) => {
                let local = off.from_utc_datetime(&now.naive_utc());
                (local.hour(), local.minute())
            }
        }
    }

    pub(super) fn date_key(&self, now: DateTime<Utc>) -> String {
        let (y, m, d) = match self {
            EffectiveTz::Iana(tz) => {
                let local = tz.from_utc_datetime(&now.naive_utc());
                (local.year(), local.month(), local.day())
            }
            EffectiveTz::Fixed(off) => {
                let local = off.from_utc_datetime(&now.naive_utc());
                (local.year(), local.month(), local.day())
            }
        };
        format!("{y:04}-{m:02}-{d:02}")
    }

    pub(super) fn in_window(&self, now: DateTime<Utc>, hhmm: &str) -> bool {
        let Ok(target) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
            return false;
        };
        let (h, m) = self.local_hm(now);
        h == target.hour() && m == target.minute()
    }
}
