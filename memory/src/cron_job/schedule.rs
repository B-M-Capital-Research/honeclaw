//! 调度时间 / 日历 / 节假日 / 触发判定等纯函数。

use chrono::{Datelike, NaiveDate};
use hone_core::beijing_offset;

use super::types::CronJob;

/// 容错窗口（分钟）— 向过去看 5 分钟，覆盖 LLM 处理时间导致的时间窗口错过
pub(super) const DUE_WINDOW_MINUTES: i32 = 5;

impl CronJob {
    pub fn is_heartbeat(&self) -> bool {
        is_heartbeat_repeat_or_tags(&self.schedule.repeat, &self.tags)
    }
}

pub(super) fn validate_schedule(
    hour: Option<u32>,
    minute: Option<u32>,
    repeat: &str,
    weekday: Option<u32>,
) -> Result<(), String> {
    let normalized_repeat = normalized_repeat(repeat, &[]);
    if normalized_repeat != "heartbeat" {
        let Some(hour) = hour else {
            return Err("缺少 hour".to_string());
        };
        let Some(minute) = minute else {
            return Err("缺少 minute".to_string());
        };
        if hour > 23 {
            return Err(format!("小时须在 0-23 之间，收到 {hour}"));
        }
        if minute > 59 {
            return Err(format!("分钟须在 0-59 之间，收到 {minute}"));
        }
    }

    let valid_repeats = [
        "daily",
        "weekly",
        "once",
        "workday",
        "trading_day",
        "holiday",
        "heartbeat",
    ];
    if !valid_repeats.contains(&normalized_repeat) {
        return Err(format!(
            "repeat 须为 daily/weekly/once/workday/trading_day/holiday/heartbeat，收到 {repeat}"
        ));
    }
    if normalized_repeat == "weekly" && (weekday.is_none() || weekday.unwrap_or(7) > 6) {
        return Err("weekly 类型须指定 weekday (0-6)".to_string());
    }

    Ok(())
}

pub(super) fn normalized_tags(tags: Vec<String>, repeat: &str) -> Vec<String> {
    let mut out = Vec::new();
    for tag in tags {
        let tag = tag.trim().to_ascii_lowercase();
        if !tag.is_empty() && !out.contains(&tag) {
            out.push(tag);
        }
    }
    if repeat.trim().eq_ignore_ascii_case("heartbeat") && !out.iter().any(|t| t == "heartbeat") {
        out.push("heartbeat".to_string());
    }
    out
}

pub(super) fn is_heartbeat_repeat_or_tags(repeat: &str, tags: &[String]) -> bool {
    repeat.trim().eq_ignore_ascii_case("heartbeat")
        || tags.iter().any(|tag| tag.eq_ignore_ascii_case("heartbeat"))
}

pub(super) fn normalized_repeat<'a>(repeat: &'a str, tags: &[String]) -> &'a str {
    if is_heartbeat_repeat_or_tags(repeat, tags) {
        "heartbeat"
    } else {
        repeat
    }
}

pub(super) fn is_workday(day: NaiveDate) -> bool {
    day.weekday().num_days_from_monday() < 5
}

fn is_market_holiday(day: NaiveDate) -> bool {
    us_market_holidays(day.year()).contains(&day)
}

pub(super) fn is_trading_day(day: NaiveDate) -> bool {
    is_workday(day) && !is_market_holiday(day)
}

pub(super) fn is_holiday(day: NaiveDate) -> bool {
    !is_workday(day) || is_market_holiday(day)
}

pub(crate) fn beijing_slot_time(
    day: NaiveDate,
    hour: u32,
    minute: u32,
) -> chrono::DateTime<chrono::FixedOffset> {
    day.and_hms_opt(hour, minute, 0)
        .expect("valid cron slot time")
        .and_local_timezone(beijing_offset())
        .single()
        .expect("fixed offset slot")
}

pub(super) fn job_existed_before_slot(job: &CronJob, day: NaiveDate) -> bool {
    let Some(created_at) = job.created_at.as_deref() else {
        return true;
    };
    let Ok(created_dt) = chrono::DateTime::parse_from_rfc3339(created_at) else {
        return true;
    };
    created_dt <= beijing_slot_time(day, job.schedule.hour, job.schedule.minute)
}

fn observed_holiday(base: NaiveDate) -> NaiveDate {
    match base.weekday().num_days_from_monday() {
        5 => base - chrono::Duration::days(1), // Saturday → Friday
        6 => base + chrono::Duration::days(1), // Sunday → Monday
        _ => base,
    }
}

fn nth_weekday(year: i32, month: u32, weekday: u32, n: u32) -> NaiveDate {
    let mut current = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    while current.weekday().num_days_from_monday() != weekday {
        current += chrono::Duration::days(1);
    }
    current + chrono::Duration::days(((n - 1) * 7) as i64)
}

fn last_weekday(year: i32, month: u32, weekday: u32) -> NaiveDate {
    let mut current = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - chrono::Duration::days(1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap() - chrono::Duration::days(1)
    };
    while current.weekday().num_days_from_monday() != weekday {
        current -= chrono::Duration::days(1);
    }
    current
}

fn easter_date(year: i32) -> NaiveDate {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = ((h + l - 7 * m + 114) % 31) + 1;
    NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap()
}

fn us_market_holidays(year: i32) -> Vec<NaiveDate> {
    vec![
        observed_holiday(NaiveDate::from_ymd_opt(year, 1, 1).unwrap()), // New Year
        nth_weekday(year, 1, 0, 3),                                     // MLK Day
        nth_weekday(year, 2, 0, 3),                                     // Presidents Day
        easter_date(year) - chrono::Duration::days(2),                  // Good Friday
        last_weekday(year, 5, 0),                                       // Memorial Day
        observed_holiday(NaiveDate::from_ymd_opt(year, 6, 19).unwrap()), // Juneteenth
        observed_holiday(NaiveDate::from_ymd_opt(year, 7, 4).unwrap()), // Independence Day
        nth_weekday(year, 9, 0, 1),                                     // Labor Day
        nth_weekday(year, 11, 3, 4),                                    // Thanksgiving
        observed_holiday(NaiveDate::from_ymd_opt(year, 12, 25).unwrap()), // Christmas
    ]
}
