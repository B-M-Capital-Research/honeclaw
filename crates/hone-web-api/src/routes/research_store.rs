//! Shared storage plumbing for the research sections.
//!
//! Every daily research product persists JSON snapshots under the same data
//! root and refreshes on a Beijing wall-clock schedule. This module owns the
//! three pieces they previously each re-implemented: deriving the data root,
//! atomic JSON writes, and the "next HH:MM Asia/Shanghai" arithmetic.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
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

/// Next occurrence of `hour:minute` Beijing time strictly in the future of
/// `now` (rolls to tomorrow once today's slot has passed).
pub(crate) fn next_beijing_refresh(now: DateTime<Utc>, hour: u32, minute: u32) -> DateTime<Utc> {
    let local = now.with_timezone(&Shanghai);
    let today = Shanghai
        .with_ymd_and_hms(local.year(), local.month(), local.day(), hour, minute, 0)
        .single()
        .expect("Shanghai local time is unambiguous");
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
    use chrono::Timelike;

    #[test]
    fn next_refresh_stays_today_before_the_slot() {
        // 2026-08-08 11:00 UTC == 19:00 Beijing; 20:00 is still ahead today.
        let now = Utc.with_ymd_and_hms(2026, 8, 8, 11, 0, 0).unwrap();
        let next = next_beijing_refresh(now, 20, 0).with_timezone(&Shanghai);
        assert_eq!((next.hour(), next.minute()), (20, 0));
        assert_eq!(next.date_naive().to_string(), "2026-08-08");
    }

    #[test]
    fn next_refresh_rolls_to_tomorrow_after_the_slot() {
        // 2026-08-08 13:00 UTC == 21:00 Beijing; 20:00 already passed.
        let now = Utc.with_ymd_and_hms(2026, 8, 8, 13, 0, 0).unwrap();
        let next = next_beijing_refresh(now, 20, 0).with_timezone(&Shanghai);
        assert_eq!(next.date_naive().to_string(), "2026-08-09");
        // Minute-level slots are honoured as-is.
        let next = next_beijing_refresh(now, 19, 55).with_timezone(&Shanghai);
        assert_eq!((next.hour(), next.minute()), (19, 55));
        assert_eq!(next.date_naive().to_string(), "2026-08-09");
    }

    #[test]
    fn next_refresh_runs_on_weekends_too() {
        // 2026-08-09 is a Sunday.
        let now = Utc.with_ymd_and_hms(2026, 8, 9, 1, 0, 0).unwrap();
        let next = next_beijing_refresh(now, 19, 30).with_timezone(&Shanghai);
        assert_eq!(next.date_naive().to_string(), "2026-08-09");
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
