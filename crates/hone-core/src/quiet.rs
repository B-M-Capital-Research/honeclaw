//! `QuietHours` —— 推送勿扰时段配置。
//!
//! 单独提到 hone-core 是为了让 `hone-channels` 的 cron 调度入口能直接复用，
//! 不引入对 hone-event-engine 的依赖。`hone-event-engine::prefs::QuietHours` 是
//! 此处的 re-export，所有字段语义、JSON 兼容性都以这里为准。
//!
//! 区间语义：本地 `[from, to)`。`from > to` 视为跨午夜（如 `23:00 → 07:00`）。
//! `from == to` 表示空区间（无效配置，调用方拦下）。

use chrono::{DateTime, FixedOffset, NaiveTime, TimeZone, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct QuietHours {
    /// 起始时刻 `"HH:MM"`，如 `"23:00"`。
    pub from: String,
    /// 结束时刻 `"HH:MM"`，如 `"07:00"`。`from > to` 视为跨午夜。
    pub to: String,
    /// 即使在 quiet 区间内也要立即推送的 kind tag 列表（用 `kind_tag()` 字符串）。
    /// 默认空 = 区间内一切 immediate 都 hold。
    pub exempt_kinds: Vec<String>,
}

/// 判断 `now` 是否落在按 `tz_name`(IANA) 解释的 `[from, to)` quiet 区间内。
/// `tz_name` 解析失败回落到 `fallback_offset_hours` 的固定偏移。
pub fn quiet_window_active(
    tz_name: Option<&str>,
    fallback_offset_hours: i32,
    from: &str,
    to: &str,
    now: DateTime<Utc>,
) -> bool {
    let (h, m) = local_hm(tz_name, fallback_offset_hours, now);
    let now_min = h as i32 * 60 + m as i32;
    let Ok(from_t) = NaiveTime::parse_from_str(from, "%H:%M") else {
        return false;
    };
    let Ok(to_t) = NaiveTime::parse_from_str(to, "%H:%M") else {
        return false;
    };
    let from_min = from_t.hour() as i32 * 60 + from_t.minute() as i32;
    let to_min = to_t.hour() as i32 * 60 + to_t.minute() as i32;
    if from_min == to_min {
        return false;
    }
    if from_min < to_min {
        now_min >= from_min && now_min < to_min
    } else {
        // 跨午夜
        now_min >= from_min || now_min < to_min
    }
}

/// kind tag 是否在 exempt_kinds 列表内（精确匹配，区分大小写——kind_tag 永远 snake_case）。
pub fn is_kind_exempt(exempt_kinds: &[String], kind_tag: &str) -> bool {
    exempt_kinds.iter().any(|t| t == kind_tag)
}

fn local_hm(tz_name: Option<&str>, fallback_offset_hours: i32, now: DateTime<Utc>) -> (u32, u32) {
    if let Some(name) = tz_name {
        if let Ok(tz) = name.parse::<chrono_tz::Tz>() {
            let local = tz.from_utc_datetime(&now.naive_utc());
            return (local.hour(), local.minute());
        }
    }
    let offset =
        FixedOffset::east_opt(fallback_offset_hours * 3600).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let local = offset.from_utc_datetime(&now.naive_utc());
    (local.hour(), local.minute())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 28, h, m, 0).single().unwrap()
    }

    #[test]
    fn cross_midnight_window() {
        // CST (+8): UTC 15:00 = 23:00 → 在 23:00-07:00 内
        assert!(quiet_window_active(None, 8, "23:00", "07:00", at(15, 0)));
        // UTC 22:59 = CST 06:59 → 在
        assert!(quiet_window_active(None, 8, "23:00", "07:00", at(22, 59)));
        // UTC 23:00 = CST 07:00 → 不在 (`< to`)
        assert!(!quiet_window_active(None, 8, "23:00", "07:00", at(23, 0)));
        // UTC 12:00 = CST 20:00 → 不在
        assert!(!quiet_window_active(None, 8, "23:00", "07:00", at(12, 0)));
    }

    #[test]
    fn same_day_window() {
        // CST (+8): UTC 05:30 = CST 13:30 → 在 13:00-15:00 内
        assert!(quiet_window_active(None, 8, "13:00", "15:00", at(5, 30)));
        // UTC 07:00 = CST 15:00 → 不在
        assert!(!quiet_window_active(None, 8, "13:00", "15:00", at(7, 0)));
    }

    #[test]
    fn empty_window_returns_false() {
        assert!(!quiet_window_active(None, 8, "07:00", "07:00", at(0, 0)));
    }

    #[test]
    fn invalid_time_returns_false() {
        assert!(!quiet_window_active(None, 8, "bogus", "07:00", at(0, 0)));
        assert!(!quiet_window_active(None, 8, "23:00", "29:99", at(0, 0)));
    }

    #[test]
    fn iana_timezone_works() {
        // America/New_York EDT (UTC-4) on 2026-04-28
        // UTC 03:00 = EDT 23:00 → 在 23:00-07:00 内
        assert!(quiet_window_active(
            Some("America/New_York"),
            8,
            "23:00",
            "07:00",
            at(3, 0)
        ));
        // UTC 12:00 = EDT 08:00 → 不在
        assert!(!quiet_window_active(
            Some("America/New_York"),
            8,
            "23:00",
            "07:00",
            at(12, 0)
        ));
    }

    #[test]
    fn invalid_iana_falls_back_to_offset() {
        // 解析失败 → 用 fallback_offset_hours
        assert!(quiet_window_active(
            Some("Mars/Olympus"),
            8,
            "23:00",
            "07:00",
            at(15, 0)
        ));
    }

    #[test]
    fn is_kind_exempt_basic() {
        let kinds = vec!["earnings_released".to_string(), "sec_filing".to_string()];
        assert!(is_kind_exempt(&kinds, "earnings_released"));
        assert!(is_kind_exempt(&kinds, "sec_filing"));
        assert!(!is_kind_exempt(&kinds, "price_alert"));
        assert!(!is_kind_exempt(&[], "earnings_released"));
    }
}
