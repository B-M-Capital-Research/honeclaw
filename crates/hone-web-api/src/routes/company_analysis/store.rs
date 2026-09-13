//! Actor-owned, attempt-fenced checkpoints for the migrated research workflow.
use hone_core::cloud_runtime::CloudPgRuntime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS company_analysis_tasks (
 task_id TEXT PRIMARY KEY, actor_user_id TEXT NOT NULL, company TEXT NOT NULL,
 model TEXT NOT NULL, status TEXT NOT NULL, progress SMALLINT NOT NULL DEFAULT 0,
 info TEXT NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), lease_until TIMESTAMPTZ NOT NULL,
 lease_token TEXT NOT NULL, checkpoint JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE UNIQUE INDEX IF NOT EXISTS company_analysis_one_running_actor
 ON company_analysis_tasks(actor_user_id) WHERE status = 'running';
CREATE INDEX IF NOT EXISTS company_analysis_actor_created
 ON company_analysis_tasks(actor_user_id, created_at DESC);
";

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Checkpoint {
    pub outputs: BTreeMap<String, String>,
    pub usage: BTreeMap<String, Value>,
    pub pdf_key: Option<String>,
    pub pdf_sha256: Option<String>,
    pub pdf_bytes: Option<usize>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Serialize)]
pub(super) struct Task {
    pub task_id: String,
    pub company: String,
    pub model: String,
    pub status: String,
    pub progress: i16,
    pub info: String,
    pub created_at: String,
    pub updated_at: String,
    pub pdf_ready: bool,
    pub warnings: Vec<String>,
    pub completed_stages: Vec<String>,
    #[serde(skip)]
    pub actor_user_id: String,
    #[serde(skip)]
    pub lease_token: String,
    #[serde(skip)]
    pub checkpoint: Checkpoint,
}

fn task_from_row(row: &tokio_postgres::Row) -> Result<Task, String> {
    let checkpoint: Checkpoint =
        serde_json::from_value(row.get("checkpoint")).map_err(|e| e.to_string())?;
    let stored_status: String = row.get("status");
    let expired: bool = row.get("expired");
    let status = if stored_status == "running" && expired {
        "interrupted".to_owned()
    } else {
        stored_status
    };
    Ok(Task {
        task_id: row.get("task_id"),
        company: row.get("company"),
        model: row.get("model"),
        pdf_ready: status == "completed" && checkpoint.pdf_key.is_some(),
        status,
        progress: row.get("progress"),
        info: row.get("info"),
        created_at: row
            .get::<_, chrono::DateTime<chrono::Utc>>("created_at")
            .to_rfc3339(),
        updated_at: row
            .get::<_, chrono::DateTime<chrono::Utc>>("updated_at")
            .to_rfc3339(),
        actor_user_id: row.get("actor_user_id"),
        lease_token: row.get("lease_token"),
        completed_stages: checkpoint
            .outputs
            .keys()
            .filter(|key| !key.starts_with('_'))
            .cloned()
            .collect(),
        warnings: checkpoint.warnings.clone(),
        checkpoint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn first_use_schema_creation_is_atomic_and_repeatable() {
        let pg = CloudPgRuntime::from_cloud_config(&hone_core::config::CloudConfig::default())
            .unwrap()
            .with_isolated_test_connection(format!("company_bootstrap_{}", uuid::Uuid::new_v4()))
            .unwrap();
        let db = pg.connect_cached_client().await.unwrap();
        db.batch_execute(
            "CREATE TEMP TABLE company_bootstrap_anchor (id int); SET search_path TO pg_temp;",
        )
        .await
        .unwrap();
        let store = Store(pg);
        store.initialize().await.unwrap();
        store.initialize().await.unwrap();
        assert_eq!(
            store
                .create("bootstrap-admin", "TEM", "test-model")
                .await
                .unwrap()
                .progress,
            0
        );
    }

    #[tokio::test]
    async fn checkpoints_are_actor_owned_attempt_fenced_and_require_a_persisted_pdf() {
        let pg = CloudPgRuntime::from_cloud_config(&hone_core::config::CloudConfig::default())
            .expect("PostgreSQL tests require HONE_POSTGRES_*")
            .with_isolated_test_connection(format!("company_analysis_{}", uuid::Uuid::new_v4()))
            .unwrap();
        let db = pg.connect_cached_client().await.unwrap();
        db.batch_execute(&SCHEMA.replacen("CREATE TABLE IF NOT EXISTS", "CREATE TEMP TABLE", 1))
            .await
            .unwrap();
        let store = Store(pg);
        let task = store.create("admin-a", "TEM", "model-a").await.unwrap();
        assert!(store.get("admin-b", &task.task_id).await.unwrap().is_none());
        assert!(
            store
                .resume("admin-b", &task.task_id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.create("admin-a", "OTHER", "model-a").await.is_err());
        let mut checkpoint = Checkpoint::default();
        checkpoint
            .outputs
            .insert("公司基本介绍".into(), "已完成的正文\n\n> 保留引用".into());
        store
            .update(&task, 25, "基本介绍完成", "running", &checkpoint)
            .await
            .unwrap();
        assert!(
            store
                .update(&task, 100, "完成", "completed", &checkpoint)
                .await
                .is_err()
        );
        db.execute("UPDATE company_analysis_tasks SET lease_until=now()-interval '1 second' WHERE task_id=$1",&[&task.task_id]).await.unwrap();
        assert_eq!(
            store
                .get("admin-a", &task.task_id)
                .await
                .unwrap()
                .unwrap()
                .status,
            "interrupted"
        );
        let resumed = store
            .resume("admin-a", &task.task_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resumed.checkpoint.outputs, checkpoint.outputs);
        assert_ne!(resumed.lease_token, task.lease_token);
        assert!(store.heartbeat(&task).await.is_err());
        assert!(
            store
                .update(&task, 80, "过期进程", "running", &Checkpoint::default())
                .await
                .is_err()
        );
        store
            .update(&resumed, 10, "重新连接", "running", &checkpoint)
            .await
            .unwrap();
        assert_eq!(
            store
                .get("admin-a", &task.task_id)
                .await
                .unwrap()
                .unwrap()
                .progress,
            25
        );
        checkpoint.pdf_key = Some("actor-owned/report.pdf".into());
        checkpoint.pdf_sha256 = Some("test-hash".into());
        checkpoint.pdf_bytes = Some(2000);
        store
            .update(&resumed, 100, "完成", "completed", &checkpoint)
            .await
            .unwrap();
        let complete = store.get("admin-a", &task.task_id).await.unwrap().unwrap();
        assert!(complete.pdf_ready);
        assert_eq!(complete.status, "completed");
        assert!(
            store
                .resume("admin-a", &task.task_id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.heartbeat(&resumed).await.is_err());
        assert!(store.list("admin-b").await.unwrap().is_empty());
    }
}

#[derive(Clone)]
pub(super) struct Store(pub CloudPgRuntime);

impl Store {
    pub async fn initialize(&self) -> Result<(), String> {
        let db = self
            .0
            .connect_cached_client()
            .await
            .map_err(|e| e.to_string())?;
        // The runtime owns its schema/search_path. No client-supplied identifiers.
        let exists: bool = db
            .query_one(
                "SELECT to_regclass('company_analysis_tasks') IS NOT NULL",
                &[],
            )
            .await
            .map_err(|e| e.to_string())?
            .get(0);
        if !exists {
            // Serialize first-use DDL across processes; IF NOT EXISTS alone can
            // still race while PostgreSQL creates the underlying relation type.
            // One DO statement has one transaction, including cancellation;
            // it cannot leave a shared cached client inside an open BEGIN.
            db.batch_execute(&format!("DO $company_schema$ BEGIN PERFORM pg_advisory_xact_lock(742103820); {SCHEMA} END $company_schema$;"))
                .await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn create(&self, actor: &str, company: &str, model: &str) -> Result<Task, String> {
        let db = self
            .0
            .connect_cached_client()
            .await
            .map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();
        let lease = uuid::Uuid::new_v4().to_string();
        let row = db
            .query_one(
                "INSERT INTO company_analysis_tasks
             (task_id,actor_user_id,company,model,status,info,lease_until,lease_token)
             VALUES ($1,$2,$3,$4,'running','任务启动中',now()+interval '2 minutes',$5)
             RETURNING *, false AS expired",
                &[&id, &actor, &company, &model, &lease],
            )
            .await
            .map_err(|e| {
                if e.code() == Some(&tokio_postgres::error::SqlState::UNIQUE_VIOLATION) {
                    "已有公司分析任务，请查看或继续该任务".to_owned()
                } else {
                    e.to_string()
                }
            })?;
        task_from_row(&row)
    }

    pub async fn get(&self, actor: &str, id: &str) -> Result<Option<Task>, String> {
        let db = self
            .0
            .connect_cached_client()
            .await
            .map_err(|e| e.to_string())?;
        db.query_opt(
            "SELECT *, lease_until < now() AS expired FROM company_analysis_tasks
             WHERE actor_user_id=$1 AND task_id=$2",
            &[&actor, &id],
        )
        .await
        .map_err(|e| e.to_string())?
        .as_ref()
        .map(task_from_row)
        .transpose()
    }

    pub async fn list(&self, actor: &str) -> Result<Vec<Task>, String> {
        let db = self
            .0
            .connect_cached_client()
            .await
            .map_err(|e| e.to_string())?;
        db.query(
            "SELECT *, lease_until < now() AS expired FROM company_analysis_tasks
             WHERE actor_user_id=$1 ORDER BY created_at DESC LIMIT 20",
            &[&actor],
        )
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .map(task_from_row)
        .collect()
    }

    pub async fn resume(&self, actor: &str, id: &str) -> Result<Option<Task>, String> {
        let db = self
            .0
            .connect_cached_client()
            .await
            .map_err(|e| e.to_string())?;
        let lease = uuid::Uuid::new_v4().to_string();
        db.query_opt(
            "UPDATE company_analysis_tasks SET status='running', lease_token=$3,
             lease_until=now()+interval '2 minutes', updated_at=now(), info='正在继续已保存的任务'
             WHERE actor_user_id=$1 AND task_id=$2 AND
             (status='failed' OR (status='running' AND lease_until < now()))
             RETURNING *, false AS expired",
            &[&actor, &id, &lease],
        )
        .await
        .map_err(|e| e.to_string())?
        .as_ref()
        .map(task_from_row)
        .transpose()
    }

    pub async fn heartbeat(&self, task: &Task) -> Result<(), String> {
        let db = self
            .0
            .connect_cached_client()
            .await
            .map_err(|e| e.to_string())?;
        let n = db
            .execute(
                "UPDATE company_analysis_tasks SET lease_until=now()+interval '2 minutes'
             WHERE task_id=$1 AND actor_user_id=$2 AND lease_token=$3 AND status='running'",
                &[&task.task_id, &task.actor_user_id, &task.lease_token],
            )
            .await
            .map_err(|e| e.to_string())?;
        if n == 1 {
            Ok(())
        } else {
            Err("任务执行权已失效".to_owned())
        }
    }

    /// Called by actual stage completion, never by a wall-clock progress guess.
    pub async fn update(
        &self,
        task: &Task,
        progress: i16,
        info: &str,
        status: &str,
        checkpoint: &Checkpoint,
    ) -> Result<(), String> {
        if !(0..=100).contains(&progress) || !matches!(status, "running" | "failed" | "completed") {
            return Err("invalid task transition".to_owned());
        }
        if status == "completed"
            && (progress != 100
                || checkpoint.pdf_key.is_none()
                || checkpoint.pdf_sha256.is_none()
                || checkpoint.pdf_bytes.is_none())
        {
            return Err("completed task requires a persisted PDF".to_owned());
        }
        let db = self
            .0
            .connect_cached_client()
            .await
            .map_err(|e| e.to_string())?;
        let value = serde_json::to_value(checkpoint).map_err(|e| e.to_string())?;
        let n = db
            .execute(
                "UPDATE company_analysis_tasks SET progress=GREATEST(progress,$4),info=$5,
             status=$6,checkpoint=$7,updated_at=now(),lease_until=now()+interval '2 minutes'
             WHERE task_id=$1 AND actor_user_id=$2 AND lease_token=$3 AND status='running'",
                &[
                    &task.task_id,
                    &task.actor_user_id,
                    &task.lease_token,
                    &progress,
                    &info,
                    &status,
                    &value,
                ],
            )
            .await
            .map_err(|e| e.to_string())?;
        if n == 1 {
            Ok(())
        } else {
            Err("任务执行权已失效".to_owned())
        }
    }
}
