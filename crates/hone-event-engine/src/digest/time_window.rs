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
pub enum EffectiveTz {
    Iana(chrono_tz::Tz),
    Fixed(FixedOffset),
}

impl EffectiveTz {
    pub fn from_actor_prefs(prefs_tz: Option<&str>, fallback_offset_hours: i32) -> Self {
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

    pub fn local_hm(&self, now: DateTime<Utc>) -> (u32, u32) {
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

    pub fn in_window(&self, now: DateTime<Utc>, hhmm: &str) -> bool {
        let Ok(target) = NaiveTime::parse_from_str(hhmm, "%H:%M") else {
            return false;
        };
        let (h, m) = self.local_hm(now);
        h == target.hour() && m == target.minute()
    }

    /// 当前 `now` 对应本地时刻是否落在 `[from, to)` 区间内。`from > to` 视为跨午夜。
    /// `from == to` 视为空区间（永远 false），避免"全天静音"被表达成歧义形式。
    pub fn in_quiet_window(&self, now: DateTime<Utc>, from: &str, to: &str) -> bool {
        let Ok(from_t) = NaiveTime::parse_from_str(from, "%H:%M") else {
            return false;
        };
        let Ok(to_t) = NaiveTime::parse_from_str(to, "%H:%M") else {
            return false;
        };
        let (h, m) = self.local_hm(now);
        let now_min = h as i32 * 60 + m as i32;
        let from_min = from_t.hour() as i32 * 60 + from_t.minute() as i32;
        let to_min = to_t.hour() as i32 * 60 + to_t.minute() as i32;
        if from_min == to_min {
            return false;
        }
        if from_min < to_min {
            // 同日区间，如 13:00-15:00
            now_min >= from_min && now_min < to_min
        } else {
            // 跨午夜，如 23:00-07:00 → [23:00, 24:00) || [00:00, 07:00)
            now_min >= from_min || now_min < to_min
        }
    }

    /// 当前 `now` 对应本地时刻是否正好命中 `to` 这一分钟（用于触发 quiet_flush）。
    pub fn at_quiet_to_minute(&self, now: DateTime<Utc>, to: &str) -> bool {
        self.in_window(now, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc_at(h: u32, m: u32) -> DateTime<Utc> {
        // 用 2026-04-28 这天作为基准日,Asia/Shanghai 没有 DST,UTC+8 偏移稳定
        chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 4, 28, h, m, 0)
            .single()
            .unwrap()
    }

    #[test]
    fn quiet_window_crosses_midnight() {
        let tz = EffectiveTz::Fixed(FixedOffset::east_opt(8 * 3600).unwrap());
        // 23:00 - 07:00 跨午夜
        // UTC 15:00 = CST 23:00 → 在区间内
        assert!(tz.in_quiet_window(utc_at(15, 0), "23:00", "07:00"));
        // UTC 18:00 = CST 02:00 → 在区间内
        assert!(tz.in_quiet_window(utc_at(18, 0), "23:00", "07:00"));
        // UTC 22:59 = CST 06:59 → 在区间内
        assert!(tz.in_quiet_window(utc_at(22, 59), "23:00", "07:00"));
        // UTC 23:00 = CST 07:00 → 不在(`< to` 严格)
        assert!(!tz.in_quiet_window(utc_at(23, 0), "23:00", "07:00"));
        // UTC 12:00 = CST 20:00 → 不在
        assert!(!tz.in_quiet_window(utc_at(12, 0), "23:00", "07:00"));
        // UTC 14:59 = CST 22:59 → 不在
        assert!(!tz.in_quiet_window(utc_at(14, 59), "23:00", "07:00"));
    }

    #[test]
    fn quiet_window_same_day() {
        let tz = EffectiveTz::Fixed(FixedOffset::east_opt(8 * 3600).unwrap());
        // 13:00 - 15:00 同日区间 (CST)
        // UTC 05:30 = CST 13:30 → 在区间内
        assert!(tz.in_quiet_window(utc_at(5, 30), "13:00", "15:00"));
        // UTC 05:00 = CST 13:00 → 在区间内 (`>= from`)
        assert!(tz.in_quiet_window(utc_at(5, 0), "13:00", "15:00"));
        // UTC 07:00 = CST 15:00 → 不在(`< to`)
        assert!(!tz.in_quiet_window(utc_at(7, 0), "13:00", "15:00"));
        // UTC 04:59 = CST 12:59 → 不在
        assert!(!tz.in_quiet_window(utc_at(4, 59), "13:00", "15:00"));
    }

    #[test]
    fn quiet_window_empty_range_returns_false() {
        let tz = EffectiveTz::Fixed(FixedOffset::east_opt(8 * 3600).unwrap());
        assert!(!tz.in_quiet_window(utc_at(0, 0), "07:00", "07:00"));
        assert!(!tz.in_quiet_window(utc_at(12, 0), "07:00", "07:00"));
    }

    #[test]
    fn quiet_window_invalid_time_returns_false() {
        let tz = EffectiveTz::Fixed(FixedOffset::east_opt(8 * 3600).unwrap());
        assert!(!tz.in_quiet_window(utc_at(0, 0), "bogus", "07:00"));
        assert!(!tz.in_quiet_window(utc_at(0, 0), "23:00", "29:99"));
    }

    #[test]
    fn at_quiet_to_minute_matches_only_at_to_minute() {
        let tz = EffectiveTz::Fixed(FixedOffset::east_opt(8 * 3600).unwrap());
        // UTC 23:00 = CST 07:00 → 命中
        assert!(tz.at_quiet_to_minute(utc_at(23, 0), "07:00"));
        // UTC 23:01 = CST 07:01 → 不命中
        assert!(!tz.at_quiet_to_minute(utc_at(23, 1), "07:00"));
        // UTC 22:59 = CST 06:59 → 不命中
        assert!(!tz.at_quiet_to_minute(utc_at(22, 59), "07:00"));
    }

    #[test]
    fn iana_tz_quiet_window_with_dst_zone() {
        // America/New_York 有 DST, 但 quiet_window 是按"本地分钟"判断,不依赖 DST 偏移漂移
        let tz = EffectiveTz::Iana("America/New_York".parse().unwrap());
        // 2026-04-28 是 EDT (UTC-4)。UTC 03:00 = EDT 23:00 → 在 23:00-07:00 内
        assert!(tz.in_quiet_window(utc_at(3, 0), "23:00", "07:00"));
        // UTC 12:00 = EDT 08:00 → 不在
        assert!(!tz.in_quiet_window(utc_at(12, 0), "23:00", "07:00"));
    }
}
