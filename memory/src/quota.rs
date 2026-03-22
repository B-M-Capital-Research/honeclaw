use hone_core::{ActorIdentity, HoneError, HoneResult, beijing_now_rfc3339};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationQuotaReservation {
    actor: ActorIdentity,
    quota_date: String,
}

impl ConversationQuotaReservation {
    pub fn actor(&self) -> &ActorIdentity {
        &self.actor
    }

    pub fn quota_date(&self) -> &str {
        &self.quota_date
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationQuotaSnapshot {
    pub quota_date: String,
    pub success_count: u32,
    pub in_flight: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationQuotaReserveResult {
    Reserved(ConversationQuotaReservation),
    Bypassed,
    Rejected(ConversationQuotaSnapshot),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConversationQuotaFile {
    quota_date: String,
    success_count: u32,
    in_flight: u32,
    updated_at: String,
}

pub struct ConversationQuotaStorage {
    root_dir: PathBuf,
}

static QUOTA_LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = OnceLock::new();

fn get_quota_lock(key: &str) -> Arc<Mutex<()>> {
    let map = QUOTA_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().unwrap();
    if let Some(lock) = guard.get(key) {
        lock.clone()
    } else {
        let lock = Arc::new(Mutex::new(()));
        guard.insert(key.to_string(), lock.clone());
        lock
    }
}

impl ConversationQuotaStorage {
    pub fn new(path: impl AsRef<Path>) -> HoneResult<Self> {
        let root_dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&root_dir)
            .map_err(|e| HoneError::Config(format!("创建 quota 目录失败: {e}")))?;
        Ok(Self { root_dir })
    }

    pub fn try_reserve_daily_conversation(
        &self,
        actor: &ActorIdentity,
        daily_limit: u32,
        bypass: bool,
    ) -> HoneResult<ConversationQuotaReserveResult> {
        let quota_date = quota_date_today();
        self.try_reserve_daily_conversation_for_date(actor, &quota_date, daily_limit, bypass)
    }

    pub fn commit_daily_conversation(
        &self,
        reservation: &ConversationQuotaReservation,
    ) -> HoneResult<()> {
        self.finish_reservation(reservation, true)
    }

    pub fn release_daily_conversation(
        &self,
        reservation: &ConversationQuotaReservation,
    ) -> HoneResult<()> {
        self.finish_reservation(reservation, false)
    }

    fn try_reserve_daily_conversation_for_date(
        &self,
        actor: &ActorIdentity,
        quota_date: &str,
        daily_limit: u32,
        bypass: bool,
    ) -> HoneResult<ConversationQuotaReserveResult> {
        if bypass {
            return Ok(ConversationQuotaReserveResult::Bypassed);
        }

        let lock = get_quota_lock(&quota_lock_key(actor, quota_date));
        let _guard = lock.lock().map_err(lock_err)?;
        let mut current = self.read_quota_file(actor, quota_date)?.unwrap_or_default();
        if current.quota_date.is_empty() {
            current.quota_date = quota_date.to_string();
        }

        if current.success_count + current.in_flight >= daily_limit {
            return Ok(ConversationQuotaReserveResult::Rejected(
                ConversationQuotaSnapshot {
                    quota_date: quota_date.to_string(),
                    success_count: current.success_count,
                    in_flight: current.in_flight,
                    limit: daily_limit,
                },
            ));
        }

        current.in_flight += 1;
        current.updated_at = beijing_now_rfc3339();
        self.write_quota_file(actor, &current)?;

        Ok(ConversationQuotaReserveResult::Reserved(
            ConversationQuotaReservation {
                actor: actor.clone(),
                quota_date: quota_date.to_string(),
            },
        ))
    }

    fn finish_reservation(
        &self,
        reservation: &ConversationQuotaReservation,
        committed: bool,
    ) -> HoneResult<()> {
        let lock = get_quota_lock(&quota_lock_key(&reservation.actor, &reservation.quota_date));
        let _guard = lock.lock().map_err(lock_err)?;
        let Some(mut current) =
            self.read_quota_file(&reservation.actor, &reservation.quota_date)?
        else {
            return Ok(());
        };

        current.in_flight = current.in_flight.saturating_sub(1);
        if committed {
            current.success_count = current.success_count.saturating_add(1);
        }
        current.updated_at = beijing_now_rfc3339();
        self.write_quota_file(&reservation.actor, &current)?;
        Ok(())
    }

    pub fn snapshot_for_date(
        &self,
        actor: &ActorIdentity,
        quota_date: &str,
    ) -> HoneResult<Option<ConversationQuotaSnapshot>> {
        Ok(self
            .read_quota_file(actor, quota_date)?
            .map(|file| ConversationQuotaSnapshot {
                quota_date: file.quota_date,
                success_count: file.success_count,
                in_flight: file.in_flight,
                limit: 0,
            }))
    }

    fn actor_dir(&self, actor: &ActorIdentity) -> PathBuf {
        self.root_dir.join(actor.storage_key())
    }

    fn quota_file_path(&self, actor: &ActorIdentity, quota_date: &str) -> PathBuf {
        self.actor_dir(actor).join(format!("{quota_date}.json"))
    }

    fn read_quota_file(
        &self,
        actor: &ActorIdentity,
        quota_date: &str,
    ) -> HoneResult<Option<ConversationQuotaFile>> {
        let path = self.quota_file_path(actor, quota_date);
        if !path.exists() {
            return Ok(None);
        }
        let content =
            std::fs::read_to_string(&path).map_err(|e| HoneError::Storage(e.to_string()))?;
        let parsed = serde_json::from_str(&content)
            .map_err(|e| HoneError::Serialization(format!("解析 quota 文件失败: {e}")))?;
        Ok(Some(parsed))
    }

    fn write_quota_file(
        &self,
        actor: &ActorIdentity,
        quota: &ConversationQuotaFile,
    ) -> HoneResult<()> {
        let dir = self.actor_dir(actor);
        std::fs::create_dir_all(&dir).map_err(|e| HoneError::Storage(e.to_string()))?;
        let path = self.quota_file_path(actor, &quota.quota_date);
        let content = serde_json::to_string_pretty(quota)
            .map_err(|e| HoneError::Serialization(e.to_string()))?;
        std::fs::write(path, content).map_err(|e| HoneError::Storage(e.to_string()))?;
        Ok(())
    }
}

fn quota_date_today() -> String {
    hone_core::beijing_now().format("%F").to_string()
}

fn quota_lock_key(actor: &ActorIdentity, quota_date: &str) -> String {
    format!("{}::{quota_date}", actor.storage_key())
}

fn lock_err<E>(_: E) -> HoneError {
    HoneError::Storage("quota storage lock poisoned".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()))
    }

    fn actor(channel: &str, user_id: &str, channel_scope: Option<&str>) -> ActorIdentity {
        ActorIdentity::new(channel, user_id, channel_scope).expect("actor")
    }

    #[test]
    fn quota_is_isolated_by_actor() {
        let root = make_temp_dir("hone_quota_actor");
        let storage = ConversationQuotaStorage::new(&root).expect("storage");
        let date = "2026-03-17";
        let left = actor("discord", "alice", None);
        let right = actor("discord", "bob", None);

        let left_res = match storage
            .try_reserve_daily_conversation_for_date(&left, date, 20, false)
            .expect("reserve left")
        {
            ConversationQuotaReserveResult::Reserved(reservation) => reservation,
            other => panic!("unexpected reserve result: {other:?}"),
        };
        storage
            .commit_daily_conversation(&left_res)
            .expect("commit left");

        let left_snapshot = storage
            .snapshot_for_date(&left, date)
            .expect("left snapshot")
            .expect("left row");
        let right_snapshot = storage
            .snapshot_for_date(&right, date)
            .expect("right snapshot");
        assert_eq!(left_snapshot.success_count, 1);
        assert!(right_snapshot.is_none());
    }

    #[test]
    fn quota_resets_on_new_beijing_day() {
        let root = make_temp_dir("hone_quota_day_reset");
        let storage = ConversationQuotaStorage::new(&root).expect("storage");
        let actor = actor("telegram", "alice", None);
        let first_day = "2026-03-17";
        let second_day = "2026-03-18";

        let reservation = match storage
            .try_reserve_daily_conversation_for_date(&actor, first_day, 20, false)
            .expect("reserve")
        {
            ConversationQuotaReserveResult::Reserved(reservation) => reservation,
            other => panic!("unexpected reserve result: {other:?}"),
        };
        storage
            .commit_daily_conversation(&reservation)
            .expect("commit");

        let next_day = storage
            .try_reserve_daily_conversation_for_date(&actor, second_day, 20, false)
            .expect("reserve second day");
        assert!(matches!(
            next_day,
            ConversationQuotaReserveResult::Reserved(_)
        ));
    }

    #[test]
    fn reservation_commit_and_release_update_counts() {
        let root = make_temp_dir("hone_quota_reserve");
        let storage = ConversationQuotaStorage::new(&root).expect("storage");
        let actor = actor("discord", "alice", Some("g:1:c:2"));
        let date = "2026-03-17";

        let committed = match storage
            .try_reserve_daily_conversation_for_date(&actor, date, 20, false)
            .expect("reserve committed")
        {
            ConversationQuotaReserveResult::Reserved(reservation) => reservation,
            other => panic!("unexpected reserve result: {other:?}"),
        };
        storage
            .commit_daily_conversation(&committed)
            .expect("commit");

        let released = match storage
            .try_reserve_daily_conversation_for_date(&actor, date, 20, false)
            .expect("reserve released")
        {
            ConversationQuotaReserveResult::Reserved(reservation) => reservation,
            other => panic!("unexpected reserve result: {other:?}"),
        };
        storage
            .release_daily_conversation(&released)
            .expect("release");

        let snapshot = storage
            .snapshot_for_date(&actor, date)
            .expect("snapshot")
            .expect("row");
        assert_eq!(snapshot.success_count, 1);
        assert_eq!(snapshot.in_flight, 0);
    }

    #[test]
    fn concurrent_reservations_cap_at_daily_limit() {
        let root = make_temp_dir("hone_quota_concurrent");
        let storage = Arc::new(ConversationQuotaStorage::new(&root).expect("storage"));
        let actor = actor("discord", "alice", None);
        let date = "2026-03-17";

        let handles = (0..30)
            .map(|_| {
                let storage = storage.clone();
                let actor = actor.clone();
                std::thread::spawn(move || {
                    match storage
                        .try_reserve_daily_conversation_for_date(&actor, date, 20, false)
                        .expect("reserve")
                    {
                        ConversationQuotaReserveResult::Reserved(reservation) => {
                            storage
                                .commit_daily_conversation(&reservation)
                                .expect("commit");
                            true
                        }
                        ConversationQuotaReserveResult::Rejected(_) => false,
                        ConversationQuotaReserveResult::Bypassed => unreachable!("no bypass"),
                    }
                })
            })
            .collect::<Vec<_>>();

        let committed = handles
            .into_iter()
            .map(|handle| handle.join().expect("join"))
            .filter(|success| *success)
            .count();

        let snapshot = storage
            .snapshot_for_date(&actor, date)
            .expect("snapshot")
            .expect("row");
        assert_eq!(committed, 20);
        assert_eq!(snapshot.success_count, 20);
        assert_eq!(snapshot.in_flight, 0);
    }

    #[test]
    fn bypass_skips_quota_tracking() {
        let root = make_temp_dir("hone_quota_bypass");
        let storage = ConversationQuotaStorage::new(&root).expect("storage");
        let actor = actor("discord", "admin", None);
        let date = "2026-03-17";

        let result = storage
            .try_reserve_daily_conversation_for_date(&actor, date, 20, true)
            .expect("reserve");
        assert!(matches!(result, ConversationQuotaReserveResult::Bypassed));
        assert!(
            storage
                .snapshot_for_date(&actor, date)
                .expect("snapshot")
                .is_none()
        );
    }
}
