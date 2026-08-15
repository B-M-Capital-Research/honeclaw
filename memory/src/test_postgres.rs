use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, Weak};

use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::cloud_sync::run_cloud_sync;
use hone_core::config::CloudConfig;
use hone_core::{HoneError, HoneResult};

pub(crate) struct TestPostgresLease {
    namespace: String,
    postgres: CloudPgRuntime,
}

impl Drop for TestPostgresLease {
    fn drop(&mut self) {
        let postgres = self.postgres.clone();
        let _ = run_cloud_sync(
            async move { postgres.drop_isolated_memory_test_schema().await },
            None,
            "memory PostgreSQL test schema cleanup",
        );
        self.postgres.evict_cached_test_client();
        if let Ok(mut leases) = TEST_POSTGRES_LEASES.lock() {
            leases.remove(&self.namespace);
        }
    }
}

static TEST_POSTGRES_LEASES: LazyLock<Mutex<HashMap<String, Weak<TestPostgresLease>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn isolated_postgres(
    namespace_hint: impl AsRef<Path>,
) -> HoneResult<(CloudPgRuntime, Arc<TestPostgresLease>)> {
    let cloud = CloudConfig::default();
    let postgres = CloudPgRuntime::from_cloud_config(&cloud).ok_or_else(|| {
        HoneError::Config(
            "PostgreSQL 测试需要 HONE_POSTGRES_* 或 HONE_POSTGRES_DATABASE_URL".to_string(),
        )
    })?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    namespace_hint.as_ref().hash(&mut hasher);
    let namespace = format!("hone_memory_{:016x}", hasher.finish());
    let postgres = postgres.with_isolated_test_connection(namespace.clone())?;
    let lease = acquire_lease(&namespace, postgres.clone())?;
    let schema_postgres = postgres.clone();
    run_cloud_sync(
        async move { schema_postgres.ensure_schema().await },
        None,
        "memory PostgreSQL test schema",
    )?;
    Ok((postgres, lease))
}

fn acquire_lease(namespace: &str, postgres: CloudPgRuntime) -> HoneResult<Arc<TestPostgresLease>> {
    let mut leases = TEST_POSTGRES_LEASES
        .lock()
        .map_err(|error| HoneError::Config(format!("PostgreSQL 测试 lease 锁失败: {error}")))?;
    if let Some(existing) = leases.get(namespace).and_then(Weak::upgrade) {
        return Ok(existing);
    }
    let lease = Arc::new(TestPostgresLease {
        namespace: namespace.to_string(),
        postgres,
    });
    leases.insert(namespace.to_string(), Arc::downgrade(&lease));
    Ok(lease)
}
