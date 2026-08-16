//! Shared storage plumbing for the research sections.
//!
//! Every daily research product persists JSON snapshots under the same data
//! root and refreshes on a Local wall-clock schedule. This module owns the
//! three pieces they previously each re-implemented: deriving the data root,
//! atomic JSON writes, and the next runtime-local `HH:MM` arithmetic.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::state::AppState;

/// Root directory for durable research snapshots, shared with the rest of the
/// file-backed state.
pub(crate) fn data_root(state: &AppState) -> PathBuf {
    state.core.config.storage.data_root()
}

/// Serialize `value` as pretty JSON and atomically replace `path`
/// (temp file + rename), creating parent directories as needed.
pub(crate) async fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("path has no parent"))?;
    tokio::fs::create_dir_all(parent).await?;
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    let bytes = serde_json::to_vec_pretty(value)?;
    tokio::fs::write(&temp, bytes).await?;
    tokio::fs::rename(temp, path).await?;
    Ok(())
}

/// Next occurrence of `hour:minute` Local time strictly in the future of
/// `now` (rolls to tomorrow once today's slot has passed).
pub(crate) fn next_local_refresh(now: DateTime<Utc>, hour: u32, minute: u32) -> DateTime<Utc> {
    next_local_refresh_in(&hone_core::runtime_timezone(), now, hour, minute)
}

fn next_local_refresh_in(
    timezone: &hone_core::RuntimeTimezone,
    now: DateTime<Utc>,
    hour: u32,
    minute: u32,
) -> DateTime<Utc> {
    let local = timezone.at_utc(now);
    let today_naive = local
        .date_naive()
        .and_hms_opt(hour, minute, 0)
        .expect("valid refresh wall-clock time");
    let today = timezone
        .from_local_datetime(&today_naive)
        .earliest()
        .or_else(|| {
            (1..=180).find_map(|minutes| {
                timezone
                    .from_local_datetime(&(today_naive + chrono::Duration::minutes(minutes)))
                    .earliest()
            })
        })
        .expect("runtime timezone has a valid instant near the refresh slot");
    (if local < today {
        today
    } else {
        today + chrono::Duration::days(1)
    })
    .with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    fn timezone() -> hone_core::RuntimeTimezone {
        hone_core::RuntimeTimezone::parse_iana("America/New_York").unwrap()
    }

    #[test]
    fn next_refresh_stays_today_before_the_slot() {
        let timezone = timezone();
        // 2026-01-15 23:00 UTC == 18:00 New York; 20:00 is still ahead today.
        let now = Utc.with_ymd_and_hms(2026, 1, 15, 23, 0, 0).unwrap();
        let next = timezone.at_utc(next_local_refresh_in(&timezone, now, 20, 0));
        assert_eq!((next.hour(), next.minute()), (20, 0));
        assert_eq!(next.date_naive().to_string(), "2026-01-15");
    }

    #[test]
    fn next_refresh_rolls_to_tomorrow_after_the_slot() {
        let timezone = timezone();
        // 2026-01-16 02:00 UTC == 21:00 New York; 20:00 already passed.
        let now = Utc.with_ymd_and_hms(2026, 1, 16, 2, 0, 0).unwrap();
        let next = timezone.at_utc(next_local_refresh_in(&timezone, now, 20, 0));
        assert_eq!(next.date_naive().to_string(), "2026-01-16");
        // Minute-level slots are honoured as-is.
        let next = timezone.at_utc(next_local_refresh_in(&timezone, now, 19, 55));
        assert_eq!((next.hour(), next.minute()), (19, 55));
        assert_eq!(next.date_naive().to_string(), "2026-01-16");
    }

    #[test]
    fn next_refresh_runs_on_weekends_too() {
        let timezone = timezone();
        // 2026-01-18 is a Sunday in New York.
        let now = Utc.with_ymd_and_hms(2026, 1, 18, 16, 0, 0).unwrap();
        let next = timezone.at_utc(next_local_refresh_in(&timezone, now, 19, 30));
        assert_eq!(next.date_naive().to_string(), "2026-01-18");
        assert_eq!((next.hour(), next.minute()), (19, 30));
    }

    #[tokio::test]
    async fn write_json_atomic_creates_parents_and_replaces() {
        let dir = std::env::temp_dir().join(format!(
            "hone-research-store-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let path = dir.join("nested").join("latest.json");
        write_json_atomic(&path, &serde_json::json!({ "v": 1 }))
            .await
            .expect("first write");
        write_json_atomic(&path, &serde_json::json!({ "v": 2 }))
            .await
            .expect("replace write");
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("read back")).expect("json");
        assert_eq!(value["v"], 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
