use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, Weak};

use hone_core::cloud_runtime::CloudPgRuntime;
use hone_core::config::CloudConfig;
use hone_core::{HoneError, HoneResult};

pub(crate) struct TestPostgresLease {
    namespace: String,
    postgres: CloudPgRuntime,
}

impl TestPostgresLease {
    pub(crate) fn postgres(&self) -> CloudPgRuntime {
        self.postgres.clone()
    }
}

impl Drop for TestPostgresLease {
    fn drop(&mut self) {
        // **先拿锁，再清理。** 清理会 drop schema 并关掉缓存连接;如果这些动作发生在
        // 锁外,并发的 `acquire_lease` 恰好在 `Weak::upgrade` 返回 `None` 之后、
        // 本次清理之前建起新 lease,新 lease 就会拿到一条马上被关掉的连接、
        // 用一个马上被 drop 的 schema —— 表现为 `Postgres 连接失败: connection closed`。
        // 2026-08-16 在全 workspace 并行跑时实测到过一次(单独重跑 4/4 通过,
        // 属低频竞态,不是逻辑错误)。持锁清理让获取方必须等清理结束再重建。
        let leases = TEST_POSTGRES_LEASES.lock();
        let postgres = self.postgres.clone();
        // `Drop` 不能 async；这里是仅测试使用的最小同步边界。独立线程上的临时
        // current-thread runtime 只执行 schema 回收，且持 lease 锁等待它完成，
        // 以保住上面说明的“清理完成前不得复用同 namespace”约束。
        let _ = std::thread::Builder::new()
            .name("hone-memory-test-pg-cleanup".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| error.to_string())?;
                runtime
                    .block_on(postgres.drop_isolated_memory_test_schema())
                    .map_err(|error| error.to_string())
            })
            .and_then(|thread| {
                thread
                    .join()
                    .map_err(|_| std::io::Error::other("memory PostgreSQL cleanup panicked"))?
                    .map_err(std::io::Error::other)
            });
        self.postgres.evict_cached_test_client();
        if let Ok(mut leases) = leases {
            leases.remove(&self.namespace);
        }
    }
}

static TEST_POSTGRES_LEASES: LazyLock<Mutex<HashMap<String, Weak<TestPostgresLease>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) async fn isolated_postgres(
    namespace_hint: impl AsRef<Path>,
) -> HoneResult<(CloudPgRuntime, Arc<TestPostgresLease>)> {
    let cloud = CloudConfig::default();
    let base = CloudPgRuntime::from_cloud_config(&cloud).ok_or_else(|| {
        HoneError::Config(
            "PostgreSQL 测试需要 HONE_POSTGRES_* 或 HONE_POSTGRES_DATABASE_URL".to_string(),
        )
    })?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    namespace_hint.as_ref().hash(&mut hasher);
    let namespace = format!("hone_memory_{:016x}", hasher.finish());

    // 隔离连接必须在**持锁之后**才建立:在锁外建连会重新打开上面 Drop 注释里那个窗口。
    let lease = acquire_lease(&namespace, &base)?;
    let postgres = lease.postgres();
    let schema_postgres = postgres.clone();
    schema_postgres.ensure_schema().await?;
    Ok((postgres, lease))
}

fn acquire_lease(namespace: &str, base: &CloudPgRuntime) -> HoneResult<Arc<TestPostgresLease>> {
    let mut leases: MutexGuard<'_, HashMap<String, Weak<TestPostgresLease>>> = TEST_POSTGRES_LEASES
        .lock()
        .map_err(|error| HoneError::Config(format!("PostgreSQL 测试 lease 锁失败: {error}")))?;
    if let Some(existing) = leases.get(namespace).and_then(Weak::upgrade) {
        return Ok(existing);
    }
    let postgres = base.with_isolated_test_connection(namespace.to_string())?;
    let lease = Arc::new(TestPostgresLease {
        namespace: namespace.to_string(),
        postgres,
    });
    leases.insert(namespace.to_string(), Arc::downgrade(&lease));
    Ok(lease)
}
