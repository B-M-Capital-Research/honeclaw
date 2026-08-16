use std::cmp::Ordering;
use std::sync::{LazyLock, RwLock};

use chrono::{
    DateTime, Datelike, FixedOffset, Local, LocalResult, NaiveDateTime, Offset, TimeZone, Utc,
};

/// A process-wide timezone selected from configuration, environment, or the host.
///
/// IANA zones retain daylight-saving transitions. `Fixed` is used only when the
/// host cannot report an IANA name and `chrono::Local` is the best available
/// source. UTC is the final fallback; no geographic timezone is implicit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeTimezone {
    Iana(chrono_tz::Tz),
    Fixed(FixedOffset),
}

impl RuntimeTimezone {
    pub fn fixed_offset_seconds(seconds: i32) -> Self {
        Self::Fixed(FixedOffset::east_opt(seconds).unwrap_or_else(|| Utc.fix()))
    }

    pub fn parse_iana(name: &str) -> Result<Self, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("timezone 不能为空".to_string());
        }
        trimmed
            .parse::<chrono_tz::Tz>()
            .map(Self::Iana)
            .map_err(|_| format!("timezone {trimmed:?} 不是合法 IANA 时区名"))
    }

    pub fn at_utc(&self, now: DateTime<Utc>) -> DateTime<FixedOffset> {
        match self {
            Self::Iana(timezone) => now.with_timezone(timezone).fixed_offset(),
            Self::Fixed(offset) => now.with_timezone(offset),
        }
    }

    pub fn from_local_datetime(&self, local: &NaiveDateTime) -> LocalResult<DateTime<FixedOffset>> {
        match self {
            Self::Iana(timezone) => timezone
                .from_local_datetime(local)
                .map(|value| value.fixed_offset()),
            Self::Fixed(offset) => offset.from_local_datetime(local),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Iana(timezone) => timezone.name().to_string(),
            Self::Fixed(offset) if offset.local_minus_utc() == 0 => "UTC".to_string(),
            Self::Fixed(offset) => format!("UTC{offset}"),
        }
    }

    pub fn date_key(&self, now: DateTime<Utc>) -> String {
        let local = self.at_utc(now);
        format!(
            "{:04}-{:02}-{:02}",
            local.year(),
            local.month(),
            local.day()
        )
    }

    pub fn local_day_start_utc(&self, now: DateTime<Utc>) -> DateTime<Utc> {
        let local = self.at_utc(now);
        let midnight = local
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("midnight is valid");
        match self.from_local_datetime(&midnight) {
            LocalResult::Single(value) | LocalResult::Ambiguous(value, _) => {
                value.with_timezone(&Utc)
            }
            LocalResult::None => (1..=180)
                .find_map(|minutes| {
                    self.from_local_datetime(&(midnight + chrono::Duration::minutes(minutes)))
                        .earliest()
                })
                .map(|value| value.with_timezone(&Utc))
                .unwrap_or(now),
        }
    }
}

static CONFIGURED_RUNTIME_TIMEZONE: LazyLock<RwLock<Option<RuntimeTimezone>>> =
    LazyLock::new(|| RwLock::new(None));

/// Validate a configured IANA timezone without changing process state.
pub fn validate_timezone_name(name: &str) -> Result<(), String> {
    RuntimeTimezone::parse_iana(name).map(|_| ())
}

/// Resolve and activate the process timezone.
///
/// Precedence is intentionally explicit:
/// 1. top-level `timezone` configuration;
/// 2. `HONE_TIMEZONE` when the config value is absent;
/// 3. the host IANA timezone, then the host's current local offset;
/// 4. UTC.
pub fn configure_runtime_timezone(configured: Option<&str>) -> Result<RuntimeTimezone, String> {
    let timezone = resolve_runtime_timezone(configured)?;
    let mut selected = CONFIGURED_RUNTIME_TIMEZONE
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *selected = Some(timezone.clone());
    Ok(timezone)
}

/// Return the active timezone. Before runtime configuration is loaded this uses
/// the same environment/host/UTC fallback chain without permanently caching it,
/// so a later config load can still become authoritative.
pub fn runtime_timezone() -> RuntimeTimezone {
    if let Some(timezone) = CONFIGURED_RUNTIME_TIMEZONE
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
    {
        return timezone;
    }
    resolve_runtime_timezone(None).unwrap_or_else(|_| RuntimeTimezone::Fixed(Utc.fix()))
}

pub fn runtime_timezone_name() -> String {
    runtime_timezone().name()
}

pub fn local_offset() -> FixedOffset {
    *local_now().offset()
}

pub fn local_now() -> DateTime<FixedOffset> {
    runtime_timezone().at_utc(Utc::now())
}

pub fn local_now_rfc3339() -> String {
    local_now().to_rfc3339()
}

pub fn local_time_at(now: DateTime<Utc>) -> DateTime<FixedOffset> {
    runtime_timezone().at_utc(now)
}

/// Compare RFC 3339 timestamps by instant, never by their offset-bearing text.
///
/// The lexical fallback preserves deterministic ordering for legacy malformed
/// values while valid values with different offsets are always normalized by
/// `DateTime` before comparison.
pub fn compare_rfc3339(left: &str, right: &str) -> Ordering {
    match (
        DateTime::parse_from_rfc3339(left),
        DateTime::parse_from_rfc3339(right),
    ) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => left.cmp(right),
    }
}

pub fn rfc3339_at_or_before(value: &str, threshold: &str) -> bool {
    compare_rfc3339(value, threshold).is_le()
}

fn resolve_runtime_timezone(configured: Option<&str>) -> Result<RuntimeTimezone, String> {
    let environment = std::env::var("HONE_TIMEZONE").ok();
    let host_iana = iana_time_zone::get_timezone().ok();
    let host_offset_seconds = Local::now().offset().local_minus_utc();
    resolve_runtime_timezone_candidates(
        configured,
        environment.as_deref(),
        host_iana.as_deref(),
        host_offset_seconds,
    )
}

fn resolve_runtime_timezone_candidates(
    configured: Option<&str>,
    environment: Option<&str>,
    host_iana: Option<&str>,
    host_offset_seconds: i32,
) -> Result<RuntimeTimezone, String> {
    if let Some(name) = configured.map(str::trim).filter(|name| !name.is_empty()) {
        return RuntimeTimezone::parse_iana(name);
    }

    if let Some(name) = environment.map(str::trim).filter(|value| !value.is_empty()) {
        match RuntimeTimezone::parse_iana(name) {
            Ok(timezone) => return Ok(timezone),
            Err(error) => tracing::warn!(
                timezone = %name,
                "ignoring invalid HONE_TIMEZONE and falling back to the host timezone: {error}"
            ),
        }
    }

    if let Some(name) = host_iana
        && let Ok(timezone) = RuntimeTimezone::parse_iana(name)
    {
        return Ok(timezone);
    }

    Ok(RuntimeTimezone::fixed_offset_seconds(host_offset_seconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn iana_timezone_preserves_dst_for_rendering_and_date_keys() {
        let timezone = RuntimeTimezone::parse_iana("America/New_York").unwrap();
        let winter = Utc.with_ymd_and_hms(2026, 1, 15, 4, 30, 0).unwrap();
        let summer = Utc.with_ymd_and_hms(2026, 7, 15, 4, 30, 0).unwrap();

        let winter_local = timezone.at_utc(winter);
        let summer_local = timezone.at_utc(summer);
        assert_eq!(winter_local.date_naive().to_string(), "2026-01-14");
        assert_eq!(winter_local.hour(), 23);
        assert_eq!(winter_local.offset().local_minus_utc(), -5 * 3600);
        assert_eq!(summer_local.date_naive().to_string(), "2026-07-15");
        assert_eq!(summer_local.hour(), 0);
        assert_eq!(summer_local.offset().local_minus_utc(), -4 * 3600);
    }

    #[test]
    fn configured_timezone_has_priority_over_environment_candidate() {
        let timezone = resolve_runtime_timezone_candidates(
            Some("America/New_York"),
            Some("Europe/London"),
            Some("UTC"),
            0,
        )
        .unwrap();
        assert_eq!(timezone.name(), "America/New_York");
    }

    #[test]
    fn environment_then_host_then_utc_fallback_order_is_stable() {
        let environment =
            resolve_runtime_timezone_candidates(None, Some("Europe/London"), Some("UTC"), 0)
                .unwrap();
        assert_eq!(environment.name(), "Europe/London");

        let host =
            resolve_runtime_timezone_candidates(None, None, Some("America/New_York"), 0).unwrap();
        assert_eq!(host.name(), "America/New_York");

        let utc = resolve_runtime_timezone_candidates(None, None, None, i32::MAX).unwrap();
        assert_eq!(utc.name(), "UTC");
    }

    #[test]
    fn invalid_iana_timezone_is_rejected() {
        assert!(validate_timezone_name("Mars/Olympus").is_err());
        assert!(validate_timezone_name(" ").is_err());
    }

    #[test]
    fn rfc3339_comparison_uses_instants_across_offsets() {
        assert!(compare_rfc3339("2026-08-16T09:30:00-04:00", "2026-08-16T20:00:00+08:00").is_gt());
        assert!(rfc3339_at_or_before(
            "2026-08-16T20:00:00+08:00",
            "2026-08-16T09:30:00-04:00"
        ));
    }
}
