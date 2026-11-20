//! Private, actor-owned snapshots. One statement commits CAS, history and retention.
use hone_core::cloud_runtime::CloudPgRuntime;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(super) const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS cloud_trade_workspaces (
 actor_storage_key TEXT PRIMARY KEY,
 state JSONB NOT NULL,
 revision BIGINT NOT NULL CHECK (revision > 0),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_trade_revisions (
 actor_storage_key TEXT NOT NULL,
 id TEXT NOT NULL,
 revision BIGINT NOT NULL CHECK (revision > 0),
 state JSONB NOT NULL,
 label TEXT NOT NULL,
 request_id TEXT NOT NULL,
 request_fingerprint TEXT NOT NULL,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 PRIMARY KEY (actor_storage_key, id),
 UNIQUE (actor_storage_key, revision),
 UNIQUE (actor_storage_key, request_id)
);
"#;

#[derive(Debug, Serialize, PartialEq)]
pub(super) struct Snapshot {
    pub state: Value,
    pub revision: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct HistoryEntry {
    pub id: String,
    pub date: String,
    pub label: String,
    pub revision: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct HistorySnapshot {
    #[serde(flatten)]
    pub entry: HistoryEntry,
    pub state: Value,
}

#[derive(Debug)]
pub(super) enum StoreError {
    Conflict,
    Missing,
    Database(String),
}

impl From<tokio_postgres::Error> for StoreError {
    fn from(error: tokio_postgres::Error) -> Self {
        Self::Database(error.to_string())
    }
}

pub(super) struct Store(pub CloudPgRuntime);

impl Store {
    pub async fn initialize(&self) -> Result<(), StoreError> {
        let db = self.client().await?;
        let exists: bool = db.query_one(
            "SELECT to_regclass('cloud_trade_workspaces') IS NOT NULL AND to_regclass('cloud_trade_revisions') IS NOT NULL",
            &[],
        ).await?.get(0);
        // Pre-migrated production roles only need SELECT/INSERT/UPDATE/DELETE.
        // Do not run DDL or request broader schema permissions when present.
        if exists {
            return Ok(());
        }
        // A single DO statement is its own transaction. Never issue BEGIN on
        // CloudPgRuntime's shared client. Serialize first-use DDL across workers.
        db.batch_execute(&format!(
            "DO $trade_schema$ BEGIN PERFORM pg_advisory_xact_lock(742103821); {SCHEMA} END $trade_schema$;"
        ))
        .await?;
        Ok(())
    }

    async fn client(&self) -> Result<std::sync::Arc<tokio_postgres::Client>, StoreError> {
        self.0
            .connect_cached_client()
            .await
            .map_err(|e| StoreError::Database(e.to_string()))
    }

    pub async fn load(&self, actor: &str) -> Result<Snapshot, StoreError> {
        let db = self.client().await?;
        let row = db
            .query_opt(
                "SELECT state, revision FROM cloud_trade_workspaces WHERE actor_storage_key=$1",
                &[&actor],
            )
            .await?;
        Ok(row.map_or(
            Snapshot {
                state: Value::Null,
                revision: 0,
            },
            |r| Snapshot {
                state: r.get(0),
                revision: r.get(1),
            },
        ))
    }

    pub async fn history(&self, actor: &str) -> Result<Vec<HistoryEntry>, StoreError> {
        let db = self.client().await?;
        Ok(db
            .query(
                "SELECT id, created_at, label, revision FROM cloud_trade_revisions WHERE actor_storage_key=$1 AND revision < (SELECT revision FROM cloud_trade_workspaces WHERE actor_storage_key=$1) ORDER BY revision DESC LIMIT 8",
                &[&actor],
            )
            .await?
            .iter()
            .map(history_entry)
            .collect())
    }

    pub async fn historical(&self, actor: &str, id: &str) -> Result<HistorySnapshot, StoreError> {
        let db = self.client().await?;
        let row = db
            .query_opt(
                "SELECT id, created_at, label, revision, state FROM cloud_trade_revisions WHERE actor_storage_key=$1 AND id=$2",
                &[&actor, &id],
            )
            .await?
            .ok_or(StoreError::Missing)?;
        Ok(HistorySnapshot {
            entry: history_entry(&row),
            state: row.get("state"),
        })
    }

    /// The authenticated actor is supplied by the route, never from JSON.
    /// An idempotent retry succeeds only while its saved revision is current;
    /// otherwise it conflicts, so the caller cannot adopt an obsolete snapshot.
    pub async fn save(
        &self,
        actor: &str,
        state: &Value,
        expected: i64,
        label: &str,
        request_id: &str,
        restore_id: Option<&str>,
    ) -> Result<Snapshot, StoreError> {
        if expected < 0 || expected >= 9_007_199_254_740_990 {
            return Err(StoreError::Conflict);
        }
        let db = self.client().await?;
        let id = uuid::Uuid::new_v4().to_string();
        let fingerprint = request_fingerprint(state, expected, label, restore_id);
        // The existence gate disallows creation with a nonzero expected
        // revision. ON CONFLICT rechecks revision after taking the row lock,
        // including two concurrent initial saves. Restore also predicates the
        // source by owner and id inside the same statement as the write.
        let row = db.query_opt(
            r#"
WITH changed AS (
 INSERT INTO cloud_trade_workspaces(actor_storage_key, state, revision)
 SELECT $1, $2, $3::bigint + 1
 WHERE ($3::bigint = 0 OR EXISTS (
   SELECT 1 FROM cloud_trade_workspaces WHERE actor_storage_key=$1 AND revision=$3
 )) AND NOT EXISTS (
   SELECT 1 FROM cloud_trade_revisions WHERE actor_storage_key=$1 AND request_id=$5
 ) AND ($8::text IS NULL OR EXISTS (
   SELECT 1 FROM cloud_trade_revisions WHERE actor_storage_key=$1 AND id=$8 AND state=$2
 ))
 ON CONFLICT (actor_storage_key) DO UPDATE SET
   state=EXCLUDED.state, revision=cloud_trade_workspaces.revision+1, updated_at=now()
 WHERE cloud_trade_workspaces.revision=$3
 RETURNING state, revision
), saved AS (
 INSERT INTO cloud_trade_revisions(actor_storage_key, id, state, revision, label, request_id, request_fingerprint)
 SELECT $1, $4, state, revision, $6, $5, $7 FROM changed
 RETURNING state, revision
), trimmed AS (
 DELETE FROM cloud_trade_revisions
 WHERE actor_storage_key=$1 AND revision <= (SELECT revision-9 FROM saved)
)
SELECT state, revision FROM saved
"#,
            &[&actor, state, &expected, &id, &request_id, &label, &fingerprint, &restore_id],
        ).await?;
        if let Some(row) = row {
            return Ok(Snapshot {
                state: row.get(0),
                revision: row.get(1),
            });
        }
        // A separate statement observes a concurrent retry's completed commit.
        // The request key is scoped by actor and never reveals another user's
        // state, even if a caller guesses their UUID.
        self.committed_retry(actor, request_id, &fingerprint)
            .await?
            .ok_or(StoreError::Conflict)
    }

    async fn committed_retry(
        &self,
        actor: &str,
        request_id: &str,
        fingerprint: &str,
    ) -> Result<Option<Snapshot>, StoreError> {
        let db = self.client().await?;
        let retry = db.query_opt(
            "SELECT w.state, w.revision FROM cloud_trade_workspaces w JOIN cloud_trade_revisions r ON r.actor_storage_key=w.actor_storage_key AND r.revision=w.revision WHERE w.actor_storage_key=$1 AND r.request_id=$2 AND r.request_fingerprint=$3",
            &[&actor, &request_id, &fingerprint],
        ).await?;
        Ok(retry.map(|row| Snapshot {
            state: row.get(0),
            revision: row.get(1),
        }))
    }

    pub async fn restore(
        &self,
        actor: &str,
        id: &str,
        expected: i64,
        request_id: &str,
    ) -> Result<Snapshot, StoreError> {
        // A restore of the oldest retained point can prune its own source.
        // Check the committed request first, so retrying an acknowledged-lost
        // restore does not depend on that source still existing.
        let fingerprint = request_fingerprint(&Value::Null, expected, "", Some(id));
        if let Some(snapshot) = self
            .committed_retry(actor, request_id, &fingerprint)
            .await?
        {
            return Ok(snapshot);
        }
        let snapshot = self.historical(actor, id).await?;
        let label = format!("恢复版本 {}", snapshot.entry.revision);
        self.save(
            actor,
            &snapshot.state,
            expected,
            &label,
            request_id,
            Some(id),
        )
        .await
    }
}

fn request_fingerprint(
    state: &Value,
    expected: i64,
    label: &str,
    restore_id: Option<&str>,
) -> String {
    let payload = match restore_id {
        Some(id) => serde_json::json!({"restore":id,"revision":expected}),
        None => serde_json::json!({"state":state,"revision":expected,"label":label}),
    };
    format!("{:x}", Sha256::digest(payload.to_string().as_bytes()))
}

fn history_entry(row: &tokio_postgres::Row) -> HistoryEntry {
    HistoryEntry {
        id: row.get("id"),
        date: row
            .get::<_, chrono::DateTime<chrono::Utc>>("created_at")
            .to_rfc3339(),
        label: row.get("label"),
        revision: row.get("revision"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::{ActorIdentity, config::CloudConfig};
    use serde_json::json;

    #[test]
    fn release_migration_matches_the_runtime_schema() {
        let migration = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/sql/trade-manager-v1.sql"
        ));
        assert!(migration.contains(SCHEMA.trim()));
        assert!(migration.contains("BEGIN;"));
        assert!(migration.contains("pg_advisory_xact_lock(742103821)"));
        assert!(migration.contains("COMMIT;"));
    }

    struct Fixture {
        first: Store,
        second: Store,
        schema: String,
    }

    impl Fixture {
        async fn new() -> Self {
            let base = CloudPgRuntime::from_cloud_config(&CloudConfig::default())
                .expect("trade manager tests require disposable HONE_POSTGRES_*");
            let unique = uuid::Uuid::new_v4().simple().to_string();
            let first = Store(
                base.with_isolated_test_connection(format!("trade_a_{unique}"))
                    .unwrap(),
            );
            let second = Store(
                base.with_isolated_test_connection(format!("trade_b_{unique}"))
                    .unwrap(),
            );
            let schema = format!("trade_test_{unique}");
            let a = first.client().await.unwrap();
            let b = second.client().await.unwrap();
            let a_pid: i32 = a
                .query_one("SELECT pg_backend_pid()", &[])
                .await
                .unwrap()
                .get(0);
            let b_pid: i32 = b
                .query_one("SELECT pg_backend_pid()", &[])
                .await
                .unwrap()
                .get(0);
            assert_ne!(
                a_pid, b_pid,
                "concurrency must use two real PostgreSQL connections"
            );
            // Identifier consists only of a fixed prefix and generated UUID.
            a.batch_execute(&format!(
                "CREATE SCHEMA {schema}; SET search_path TO {schema};"
            ))
            .await
            .unwrap();
            b.batch_execute(&format!("SET search_path TO {schema};"))
                .await
                .unwrap();
            let (one, two) = tokio::join!(first.initialize(), second.initialize());
            one.unwrap();
            two.unwrap();
            Self {
                first,
                second,
                schema,
            }
        }
        async fn cleanup(self) {
            self.first
                .client()
                .await
                .unwrap()
                .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.schema))
                .await
                .unwrap();
            self.first.0.evict_cached_test_client();
            self.second.0.evict_cached_test_client();
        }
    }

    fn actor(id: &str) -> String {
        ActorIdentity::new("web", id, None::<String>)
            .unwrap()
            .storage_key()
    }
    fn request() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    #[tokio::test]
    async fn initial_and_subsequent_cas_have_exactly_one_winner_across_connections() {
        let f = Fixture::new().await;
        let a = actor("alice");
        assert_eq!(
            f.first.load(&a).await.unwrap(),
            Snapshot {
                state: Value::Null,
                revision: 0
            }
        );
        assert!(matches!(
            f.first
                .save(&a, &json!({"v":1}), 2, "bad initial", &request(), None)
                .await,
            Err(StoreError::Conflict)
        ));
        let (one, two) = tokio::join!(
            async {
                f.first
                    .save(&a, &json!({"v":1}), 0, "one", &request(), None)
                    .await
            },
            async {
                f.second
                    .save(&a, &json!({"v":2}), 0, "two", &request(), None)
                    .await
            }
        );
        assert_eq!(usize::from(one.is_ok()) + usize::from(two.is_ok()), 1);
        assert!(f.first.history(&a).await.unwrap().is_empty());
        let (one, two) = tokio::join!(
            async {
                f.first
                    .save(&a, &json!({"v":3}), 1, "three", &request(), None)
                    .await
            },
            async {
                f.second
                    .save(&a, &json!({"v":4}), 1, "four", &request(), None)
                    .await
            }
        );
        assert_eq!(usize::from(one.is_ok()) + usize::from(two.is_ok()), 1);
        assert_eq!(f.first.load(&a).await.unwrap().revision, 2);
        assert_eq!(f.second.history(&a).await.unwrap().len(), 1);
        f.cleanup().await;
    }

    #[tokio::test]
    async fn snapshots_and_restore_ids_are_actor_scoped_and_durable_between_handles() {
        let f = Fixture::new().await;
        let a = actor("alice");
        let b = actor("bob");
        f.first
            .save(
                &a,
                &json!({"private":"alice-v1"}),
                0,
                "a1",
                &request(),
                None,
            )
            .await
            .unwrap();
        f.first
            .save(
                &a,
                &json!({"private":"alice-v2"}),
                1,
                "a2",
                &request(),
                None,
            )
            .await
            .unwrap();
        f.second
            .save(&b, &json!({"private":"bob"}), 0, "b1", &request(), None)
            .await
            .unwrap();
        assert_eq!(
            f.second.load(&a).await.unwrap().state,
            json!({"private":"alice-v2"})
        );
        assert_eq!(
            f.first.load(&b).await.unwrap().state,
            json!({"private":"bob"})
        );
        let id = f.first.history(&a).await.unwrap().remove(0).id;
        assert!(f.second.history(&b).await.unwrap().is_empty());
        assert!(matches!(
            f.second.historical(&b, &id).await,
            Err(StoreError::Missing)
        ));
        assert!(matches!(
            f.second.restore(&b, &id, 1, &request()).await,
            Err(StoreError::Missing)
        ));
        assert!(matches!(
            f.second.restore(&a, &id, 1, &request()).await,
            Err(StoreError::Conflict)
        ));
        let restored = f.second.restore(&a, &id, 2, &request()).await.unwrap();
        assert_eq!(
            restored,
            Snapshot {
                state: json!({"private":"alice-v1"}),
                revision: 3
            }
        );
        assert_eq!(f.first.load(&b).await.unwrap().revision, 1);
        assert_eq!(f.first.history(&a).await.unwrap().len(), 2);
        f.cleanup().await;
    }

    #[tokio::test]
    async fn request_retries_do_not_double_save_or_restore_even_when_source_is_pruned() {
        let f = Fixture::new().await;
        let a = actor("alice");
        let key = request();
        let state = json!({"v":1});
        let (one, two) = tokio::join!(
            f.first.save(&a, &state, 0, "same", &key, None),
            f.second.save(&a, &state, 0, "same", &key, None)
        );
        assert_eq!(one.unwrap(), two.unwrap());
        assert_eq!(f.first.load(&a).await.unwrap().revision, 1);
        assert!(matches!(
            f.first
                .save(&a, &json!({"v":2}), 0, "same", &key, None)
                .await,
            Err(StoreError::Conflict)
        ));
        for revision in 1..9 {
            f.first
                .save(
                    &a,
                    &json!({"v":revision+1}),
                    revision,
                    "next",
                    &request(),
                    None,
                )
                .await
                .unwrap();
        }
        let history = f.second.history(&a).await.unwrap();
        assert_eq!(history.len(), 8);
        assert_eq!(history.last().unwrap().revision, 1);
        assert!(matches!(
            f.first.save(&a, &state, 0, "same", &key, None).await,
            Err(StoreError::Conflict)
        ));
        let oldest = &history.last().unwrap().id;
        let restore_key = request();
        let restored = f.first.restore(&a, oldest, 9, &restore_key).await.unwrap();
        assert_eq!(restored.revision, 10);
        assert_eq!(restored.state, state);
        assert!(matches!(
            f.second.historical(&a, oldest).await,
            Err(StoreError::Missing)
        ));
        assert_eq!(
            f.second.restore(&a, oldest, 9, &restore_key).await.unwrap(),
            restored
        );
        assert_eq!(f.first.history(&a).await.unwrap().len(), 8);
        let count: i64 = f
            .first
            .client()
            .await
            .unwrap()
            .query_one(
                "SELECT count(*) FROM cloud_trade_revisions WHERE actor_storage_key=$1",
                &[&a],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(
            count, 9,
            "current snapshot plus eight preceding restore points"
        );
        f.cleanup().await;
    }
}
