use hone_core::cloud_runtime::{CloudLlmAuditFilter, CloudPgRuntime};
use hone_core::cloud_sync::run_cloud_sync;
use hone_core::{HoneError, HoneResult, LlmAuditRecord, LlmAuditSink};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Deserialize, Default)]
pub struct AuditQueryFilter {
    pub actor_channel: Option<String>,
    pub actor_user_id: Option<String>,
    pub actor_scope: Option<String>,
    pub session_id: Option<String>,
    pub success: Option<bool>,
    pub source: Option<String>,
    pub provider: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct AuditRecordSummary {
    pub id: String,
    pub created_at: String,
    pub session_id: String,
    pub actor_channel: Option<String>,
    pub actor_user_id: Option<String>,
    pub actor_scope: Option<String>,
    pub source: String,
    pub operation: String,
    pub provider: String,
    pub model: Option<String>,
    pub success: bool,
    pub latency_ms: Option<u128>,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

pub struct LlmAuditStorage {
    postgres: CloudPgRuntime,
    retention_days: u32,
    write_count: AtomicU64,
    _test_postgres_lease: Option<Arc<crate::test_postgres::TestPostgresLease>>,
}

impl LlmAuditStorage {
    /// PostgreSQL-backed test constructor. The path is only an isolation namespace.
    #[doc(hidden)]
    pub fn new(path: impl AsRef<std::path::Path>, retention_days: u32) -> HoneResult<Self> {
        let (postgres, lease) = crate::test_postgres::isolated_postgres(path)?;
        let mut storage = Self::new_cloud(postgres, retention_days)?;
        storage._test_postgres_lease = Some(lease);
        storage.prune_expired()?;
        Ok(storage)
    }

    pub fn new_cloud(postgres: CloudPgRuntime, retention_days: u32) -> HoneResult<Self> {
        let schema_postgres = postgres.clone();
        run_cloud_llm_audit(async move { schema_postgres.ensure_schema().await })?;
        Ok(Self {
            postgres,
            retention_days: retention_days.max(1),
            write_count: AtomicU64::new(0),
            _test_postgres_lease: None,
        })
    }

    pub fn prune_expired(&self) -> HoneResult<()> {
        let cutoff = (hone_core::beijing_now()
            - chrono::Duration::days(self.retention_days as i64))
        .to_rfc3339();
        let postgres = self.postgres.clone();
        run_cloud_llm_audit(async move {
            postgres.prune_llm_audit_records(&cutoff).await?;
            Ok(())
        })
    }

    #[cfg(test)]
    pub fn count_records(&self) -> HoneResult<i64> {
        let postgres = self.postgres.clone();
        run_cloud_llm_audit(async move { postgres.count_llm_audit_records().await })
    }

    fn maybe_prune_after_write(&self) -> HoneResult<()> {
        let count = self.write_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count.is_multiple_of(100) {
            self.prune_expired()?;
        }
        Ok(())
    }

    pub fn list_audit_records(
        &self,
        filter: &AuditQueryFilter,
    ) -> HoneResult<(Vec<AuditRecordSummary>, i64)> {
        let postgres = self.postgres.clone();
        let cloud_filter = CloudLlmAuditFilter {
            actor_channel: filter.actor_channel.clone(),
            actor_user_id: filter.actor_user_id.clone(),
            actor_scope: filter.actor_scope.clone(),
            session_id: filter.session_id.clone(),
            success: filter.success,
            source: filter.source.clone(),
            provider: filter.provider.clone(),
            date_from: filter.date_from.clone(),
            date_to: filter.date_to.clone(),
            page: filter.page,
            page_size: filter.page_size,
        };
        let (records, total) =
            run_cloud_llm_audit(
                async move { postgres.list_llm_audit_records(cloud_filter).await },
            )?;
        let summaries = records
            .into_iter()
            .filter_map(|value| {
                serde_json::from_value::<LlmAuditRecord>(value)
                    .ok()
                    .map(audit_summary_from_record)
            })
            .collect();
        Ok((summaries, total))
    }

    pub fn get_audit_record(&self, id: &str) -> HoneResult<Option<LlmAuditRecord>> {
        let postgres = self.postgres.clone();
        let id = id.to_string();
        run_cloud_llm_audit(async move { postgres.get_llm_audit_record(&id).await })?
            .map(serde_json::from_value::<LlmAuditRecord>)
            .transpose()
            .map_err(|error| HoneError::Serialization(error.to_string()))
    }
}

impl LlmAuditSink for LlmAuditStorage {
    fn record(&self, record: LlmAuditRecord) -> HoneResult<()> {
        let postgres = self.postgres.clone();
        run_cloud_llm_audit(async move { postgres.upsert_llm_audit_record(record).await })?;
        self.maybe_prune_after_write()
    }
}

fn audit_summary_from_record(record: LlmAuditRecord) -> AuditRecordSummary {
    AuditRecordSummary {
        id: record.id,
        created_at: record.created_at,
        session_id: record.session_id,
        actor_channel: record.actor.as_ref().map(|actor| actor.channel.clone()),
        actor_user_id: record.actor.as_ref().map(|actor| actor.user_id.clone()),
        actor_scope: record
            .actor
            .as_ref()
            .and_then(|actor| actor.channel_scope.clone()),
        source: record.source,
        operation: record.operation,
        provider: record.provider,
        model: record.model,
        success: record.success,
        latency_ms: record.latency_ms,
        prompt_tokens: record.prompt_tokens,
        completion_tokens: record.completion_tokens,
        total_tokens: record.total_tokens,
    }
}

fn run_cloud_llm_audit<T, F>(future: F) -> HoneResult<T>
where
    T: Send + 'static,
    F: std::future::Future<Output = HoneResult<T>> + Send + 'static,
{
    run_cloud_sync(future, None, "cloud llm audit operation")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::{ActorIdentity, LlmAuditRecord};
    use serde_json::{Value, json};

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn record_and_prune_expired_rows() {
        let root = std::env::temp_dir().join(format!("hone_llm_audit_{}", uuid::Uuid::new_v4()));
        let db_path = root.join("audit.sqlite3");
        let storage = LlmAuditStorage::new(&db_path, 30).expect("storage");

        let mut fresh = LlmAuditRecord::new(
            "Actor_feishu__direct__alice",
            Some(ActorIdentity::new("feishu", "alice", None::<String>).expect("actor")),
            "agent.codex_acp",
            "chat_with_tools",
            "openrouter",
            Some("moonshotai/kimi-k2.5".to_string()),
            json!({"messages":[{"role":"user","content":"hi"}]}),
        );
        fresh.success = true;
        fresh.response = Some(json!({"content":"hello"}));
        storage.record(fresh).expect("record fresh");

        let stale = LlmAuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: (hone_core::beijing_now() - chrono::Duration::days(31)).to_rfc3339(),
            session_id: "old".to_string(),
            actor: None,
            source: "agent.codex_acp".to_string(),
            operation: "chat".to_string(),
            provider: "openrouter".to_string(),
            model: Some("test".to_string()),
            success: true,
            latency_ms: Some(12),
            request: json!({"messages":[]}),
            response: Some(json!({"content":"stale"})),
            error: None,
            metadata: Value::Null,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        };
        storage.record(stale).expect("record stale");

        assert_eq!(storage.count_records().expect("count"), 2);
        storage.prune_expired().expect("prune");
        assert_eq!(storage.count_records().expect("count after"), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn query_audit_records_with_filters() {
        let root = std::env::temp_dir().join(format!("hone_llm_audit_{}", uuid::Uuid::new_v4()));
        let db_path = root.join("audit.sqlite3");
        let storage = LlmAuditStorage::new(&db_path, 30).expect("storage");

        let mut chat_audit_record = LlmAuditRecord::new(
            "sess1",
            Some(ActorIdentity::new("wx", "bob", None::<String>).expect("actor")),
            "agent",
            "chat",
            "openai",
            Some("gpt-4".to_string()),
            json!({"q": 1}),
        );
        chat_audit_record.success = true;
        storage.record(chat_audit_record.clone()).unwrap();

        let mut search_audit_record = LlmAuditRecord::new(
            "sess2",
            Some(ActorIdentity::new("feishu", "alice", None::<String>).expect("actor")),
            "tool",
            "search",
            "bing",
            None,
            json!({"q": 2}),
        );
        search_audit_record.success = false;
        search_audit_record.latency_ms = Some(150);
        storage.record(search_audit_record.clone()).unwrap();

        // 1. 无条件过滤
        let all_records_result = storage
            .list_audit_records(&AuditQueryFilter::default())
            .unwrap();
        assert_eq!(all_records_result.1, 2);

        // 2. Test filtering by actor_channel
        let filtered_records = storage
            .list_audit_records(&AuditQueryFilter {
                actor_channel: Some("feishu".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(filtered_records.1, 1);
        assert_eq!(
            filtered_records.0[0].actor_channel.as_deref(),
            Some("feishu")
        );
        assert_eq!(filtered_records.0[0].latency_ms, Some(150));
        assert_eq!(filtered_records.0[0].prompt_tokens, None);

        // 3. Test success boolean filtering
        let successful_records_result = storage
            .list_audit_records(&AuditQueryFilter {
                success: Some(true),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(successful_records_result.1, 1);
        assert_eq!(successful_records_result.0[0].session_id, "sess1");

        // 4. Test pagination
        let first_page_result = storage
            .list_audit_records(&AuditQueryFilter {
                page: Some(1),
                page_size: Some(1),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(first_page_result.1, 2); // Total count is 2
        assert_eq!(first_page_result.0.len(), 1);

        // 5. Test detail query
        let detail = storage
            .get_audit_record(&chat_audit_record.id)
            .unwrap()
            .unwrap();
        assert_eq!(detail.request, json!({"q": 1}));
        assert_eq!(detail.session_id, "sess1");

        let missing = storage.get_audit_record("not-exist").unwrap();
        assert!(missing.is_none());

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "requires HONE_POSTGRES_* and a running local PostgreSQL"]
    fn postgres_schema_persists_token_counts() {
        let root = std::env::temp_dir().join(format!("hone_llm_audit_{}", uuid::Uuid::new_v4()));
        let storage = LlmAuditStorage::new(&root, 30).expect("postgres storage");

        let mut record = LlmAuditRecord::new(
            "sess-legacy",
            Some(ActorIdentity::new("discord", "alice", None::<String>).expect("actor")),
            "agent.gemini_cli",
            "chat",
            "gemini_cli",
            Some("gemini-2.5-pro".to_string()),
            json!({"messages":[{"role":"user","content":"hi"}]}),
        );
        record.success = true;
        record.response = Some(json!({"content":"hello"}));
        record.prompt_tokens = Some(11);
        record.completion_tokens = Some(7);
        record.total_tokens = Some(18);
        storage
            .record(record.clone())
            .expect("record with token counts");

        let detail = storage
            .get_audit_record(&record.id)
            .expect("detail query")
            .expect("detail exists");
        assert_eq!(detail.prompt_tokens, Some(11));
        assert_eq!(detail.completion_tokens, Some(7));
        assert_eq!(detail.total_tokens, Some(18));

        let (records, total) = storage
            .list_audit_records(&AuditQueryFilter::default())
            .expect("list query");
        assert_eq!(total, 1);
        assert_eq!(records[0].prompt_tokens, Some(11));
        assert_eq!(records[0].completion_tokens, Some(7));
        assert_eq!(records[0].total_tokens, Some(18));

        let _ = std::fs::remove_dir_all(root);
    }
}
