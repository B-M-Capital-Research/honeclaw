//! 调度时间 / 日历 / 节假日 / 触发判定等纯函数。
//!
//! 本模块里所有函数都是纯函数（给定输入 -> 固定输出），不做 IO 也不依赖
//! `CronJobStorage` 自身状态。`storage.rs` 会基于这里的判定实现完整的
//! `get_due_jobs` / `add_job` / `validate` 管线。

use chrono::{Datelike, NaiveDate};
use hone_core::beijing_offset;

use super::types::CronJob;

/// 时间窗口容错（分钟）：允许将过去 5 分钟内的计划视为「当前到点」，
/// 用来抵消 scheduler 触发延迟以及 LLM 处理慢造成的错过。
pub(super) const DUE_WINDOW_MINUTES: i32 = 5;

impl CronJob {
    /// 是否为 heartbeat 类型：`schedule.repeat == "heartbeat"` 或 `tags` 里有 `heartbeat`。
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

pub(super) fn validate_schedule_date(repeat: &str, date: Option<&str>) -> Result<(), String> {
    let Some(date) = date.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if !repeat.eq_ignore_ascii_case("once") {
        return Err("date 仅支持 repeat=once 的一次性任务".to_string());
    }
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| format!("date 须为 YYYY-MM-DD，收到 {date}"))
}

pub(super) fn normalize_schedule_date(date: Option<String>) -> Option<String> {
    date.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
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

/// 周一到周五视为工作日（不考虑假期调休）。
pub(super) fn is_workday(day: NaiveDate) -> bool {
    day.weekday().num_days_from_monday() < 5
}

fn is_market_holiday(day: NaiveDate) -> bool {
    us_market_holidays(day.year()).contains(&day)
}

/// 美股交易日：工作日且不是美股节假日。
pub(super) fn is_trading_day(day: NaiveDate) -> bool {
    is_workday(day) && !is_market_holiday(day)
}

/// 非工作日或美股节假日（`trading_day` 的补集）。
pub(super) fn is_holiday(day: NaiveDate) -> bool {
    !is_workday(day) || is_market_holiday(day)
}

/// 把 `(day, hour, minute)` 解释成 +08:00 时区的具体时刻。
/// 专供 `get_due_jobs` 的 catch-up 判定和测试断言使用。
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

/// 任务是否在当天的计划时刻之前就已经存在：用于判断某个已经过了 `hh:mm`
/// 的任务是「今天漏跑需要补」还是「今天才刚创建，不补跑」。
/// 缺失 / 无法解析 `created_at` 时按「存在」处理，保持向后兼容。
pub(super) fn job_existed_before_slot(job: &CronJob, day: NaiveDate) -> bool {
    let Some(created_at) = job.created_at.as_deref() else {
        return true;
    };
    let Ok(created_dt) = chrono::DateTime::parse_from_rfc3339(created_at) else {
        return true;
    };
    created_dt <= beijing_slot_time(day, job.schedule.hour, job.schedule.minute)
}

/// 把落在周末的节假日挪到最近的工作日（周六→周五，周日→周一），
/// 匹配 NYSE 的 observed holiday 规则。
fn observed_holiday(base: NaiveDate) -> NaiveDate {
    match base.weekday().num_days_from_monday() {
        5 => base - chrono::Duration::days(1), // Saturday → Friday
        6 => base + chrono::Duration::days(1), // Sunday → Monday
        _ => base,
    }
}

/// 某月的第 n 个指定星期几（用于 MLK、总统日、劳动节、感恩节等）。
fn nth_weekday(year: i32, month: u32, weekday: u32, n: u32) -> NaiveDate {
    let mut current = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    while current.weekday().num_days_from_monday() != weekday {
        current += chrono::Duration::days(1);
    }
    current + chrono::Duration::days(((n - 1) * 7) as i64)
}

/// 某月的最后一个指定星期几（用于阵亡将士纪念日：5 月最后一个周一）。
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

/// 公元 `year` 的复活节日期（Anonymous Gregorian 算法）。
/// 美股耶稣受难日（Good Friday）是复活节前第二天，需要动态计算。
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
