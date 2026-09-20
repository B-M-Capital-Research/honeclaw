use super::*;
use crate::ActorIdentity;
use crate::cloud_runtime::{CloudPgRuntime, CloudPortfolioRecord, PG_QUERY_POOL_SIZE};
use crate::config::CloudConfig;
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::task::JoinSet;

async fn fixture() -> CloudPgRuntime {
    let runtime = CloudPgRuntime::from_cloud_config(&CloudConfig::default())
        .expect("tests require the development HONE_POSTGRES_* settings")
        .with_isolated_test_connection(format!(
            "hone_memory_pool_{}",
            uuid::Uuid::new_v4().simple()
        ))
        .unwrap();
    let client = runtime.connect_new_client().await.unwrap();
    client
        .batch_execute(
            "CREATE TABLE pool_probe (id int PRIMARY KEY, value int NOT NULL);
        INSERT INTO pool_probe VALUES (1, 0);
        CREATE TABLE cloud_portfolios (actor_storage_key text PRIMARY KEY, actor jsonb NOT NULL,
            portfolio jsonb NOT NULL, updated_at timestamptz NOT NULL DEFAULT now());",
        )
        .await
        .unwrap();
    runtime
}

async fn cleanup(runtime: &CloudPgRuntime) {
    runtime.evict_cached_test_client();
    runtime.drop_isolated_memory_test_schema().await.unwrap();
}

async fn pid(client: &PgQueryClient) -> i32 {
    client
        .query_one("SELECT pg_backend_pid()", &[])
        .await
        .unwrap()
        .get(0)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pool_bounds_exclusive_leases_and_reuses_released_connection() {
    let runtime = fixture().await;
    let pool = Arc::new(PgQueryPool::new(2));
    let first = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    let second = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    let first_pid = pid(&first).await;
    assert_ne!(first_pid, pid(&second).await, "leases must be exclusive");
    assert!(
        tokio::time::timeout(
            Duration::from_millis(25),
            pool.acquire(runtime.connect_managed_client())
        )
        .await
        .is_err()
    );
    drop(first);
    let reused = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    assert_eq!(first_pid, pid(&reused).await);
    drop((reused, second, pool));
    cleanup(&runtime).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pool_concurrent_cold_start_preserves_response_ownership_and_limit() {
    let runtime = fixture().await;
    let mut tasks = JoinSet::new();
    for request_id in 0..32_i32 {
        let runtime = runtime.clone();
        tasks.spawn(async move {
            let client = runtime.connect_client().await.unwrap();
            let row = client
                .query_one(
                    "SELECT pg_backend_pid(), $1::int, pg_sleep(0.005)",
                    &[&request_id],
                )
                .await
                .unwrap();
            assert_eq!(row.get::<_, i32>(1), request_id);
            row.get::<_, i32>(0)
        });
    }
    let mut backends = BTreeSet::new();
    while let Some(result) = tasks.join_next().await {
        backends.insert(result.unwrap());
    }
    assert!(
        backends.len() <= PG_QUERY_POOL_SIZE,
        "unbounded cold connections: {backends:?}"
    );
    cleanup(&runtime).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pool_concurrent_actor_portfolios_and_test_namespaces_do_not_cross() {
    let left = fixture().await;
    let right = fixture().await;
    let mut tasks = JoinSet::new();
    for (namespace, runtime) in [("left", &left), ("right", &right)] {
        for index in 0..24 {
            let runtime = runtime.clone();
            tasks.spawn(async move {
                // Reuse user IDs across channels and scopes to exercise the
                // composite actor key as well as the database namespace.
                let actor = ActorIdentity::new(
                    if index % 2 == 0 { "web" } else { "feishu" },
                    format!("user-{}", index / 4),
                    Some(format!("scope-{}", index % 4)),
                )
                .unwrap();
                let key = actor.storage_key();
                for turn in 0..4 {
                    let value =
                        serde_json::json!({"namespace":namespace,"owner":index,"turn":turn});
                    runtime
                        .upsert_portfolio(CloudPortfolioRecord {
                            actor_storage_key: key.clone(),
                            actor: serde_json::to_value(&actor).unwrap(),
                            portfolio: value.clone(),
                        })
                        .await
                        .unwrap();
                    tokio::task::yield_now().await;
                    let stored = runtime.get_portfolio(&key).await.unwrap().unwrap();
                    assert_eq!(stored.actor_storage_key, key);
                    assert_eq!(stored.portfolio, value);
                }
            });
        }
    }
    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }
    assert_eq!(left.list_portfolios().await.unwrap().len(), 24);
    assert_eq!(right.list_portfolios().await.unwrap().len(), 24);
    cleanup(&left).await;
    cleanup(&right).await;
}

#[tokio::test]
async fn pool_discards_failed_and_cancelled_queries_without_reusing_state() {
    let runtime = fixture().await;
    let pool = Arc::new(PgQueryPool::new(1));
    let client = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    let failed_pid = pid(&client).await;
    assert!(client.query_one("SELECT 1 / 0", &[]).await.is_err());
    drop(client);
    let client = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    let cancelled_pid = pid(&client).await;
    assert_ne!(failed_pid, cancelled_pid);
    assert!(
        tokio::time::timeout(
            Duration::from_millis(25),
            client.query_one("SELECT pg_sleep(1)", &[])
        )
        .await
        .is_err()
    );
    drop(client);
    let client = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    assert_ne!(cancelled_pid, pid(&client).await);
    assert_eq!(
        client
            .query_one("SELECT 42::int", &[])
            .await
            .unwrap()
            .get::<_, i32>(0),
        42
    );
    drop((client, pool));
    cleanup(&runtime).await;
}

#[tokio::test]
async fn pool_recovers_after_idle_backend_disconnect_and_connect_failure() {
    let runtime = fixture().await;
    let pool = Arc::new(PgQueryPool::new(1));
    assert!(
        pool.acquire(async { Err(HoneError::Config("synthetic connection failure".into())) })
            .await
            .is_err()
    );
    let client = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    let old_pid = pid(&client).await;
    drop(client);
    let admin = runtime.connect_new_client().await.unwrap();
    admin
        .query_one("SELECT pg_terminate_backend($1)", &[&old_pid])
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if pool.idle.lock().unwrap()[0].client().is_closed() {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let recovered = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    assert_ne!(old_pid, pid(&recovered).await);
    drop((recovered, pool, admin));
    cleanup(&runtime).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pool_does_not_share_transactions_and_cancellation_rolls_back() {
    let runtime = fixture().await;
    let transaction_runtime = runtime.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let mut dedicated = transaction_runtime.connect_new_client().await.unwrap();
        let transaction = dedicated.transaction().await.unwrap();
        transaction
            .execute("UPDATE pool_probe SET value=99 WHERE id=1", &[])
            .await
            .unwrap();
        let backend: i32 = transaction
            .query_one("SELECT pg_backend_pid()", &[])
            .await
            .unwrap()
            .get(0);
        ready_tx.send(backend).unwrap();
        std::future::pending::<()>().await;
        transaction.commit().await.unwrap();
    });
    let transaction_pid = ready_rx.await.unwrap();
    let reader = runtime.connect_client().await.unwrap();
    assert_ne!(pid(&reader).await, transaction_pid);
    assert_eq!(
        reader
            .query_one("SELECT value FROM pool_probe WHERE id=1", &[])
            .await
            .unwrap()
            .get::<_, i32>(0),
        0
    );
    // An unrelated autocommit write must not join the pending transaction.
    reader
        .execute("INSERT INTO pool_probe VALUES (2, 7)", &[])
        .await
        .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    tokio::time::timeout(
        Duration::from_secs(2),
        reader.execute("UPDATE pool_probe SET value=value+1 WHERE id=1", &[]),
    )
    .await
    .unwrap()
    .unwrap();
    let rows = reader
        .query("SELECT id,value FROM pool_probe ORDER BY id", &[])
        .await
        .unwrap();
    assert_eq!((rows[0].get::<_, i32>(0), rows[0].get::<_, i32>(1)), (1, 1));
    assert_eq!((rows[1].get::<_, i32>(0), rows[1].get::<_, i32>(1)), (2, 7));
    drop(reader);
    cleanup(&runtime).await;
}

#[tokio::test]
async fn pool_keeps_advisory_locks_exclusive_and_pg_temp_connection_local() {
    let runtime = fixture().await;
    let lock = runtime
        .acquire_community_publish_lock("pool-test", "ownership")
        .await
        .unwrap();
    assert!(
        runtime
            .acquire_community_publish_lock("pool-test", "ownership")
            .await
            .is_err()
    );
    let ordinary = runtime.connect_client().await.unwrap();
    let lock_name = super::super::community_publish_lock_name("pool-test", "ownership").unwrap();
    assert!(
        !ordinary
            .query_one(
                "SELECT pg_try_advisory_lock(hashtextextended($1,0))",
                &[&lock_name]
            )
            .await
            .unwrap()
            .get::<_, bool>(0)
    );
    drop(ordinary);
    lock.release().await.unwrap();
    // A new dedicated connection can acquire the exact runtime lock again.
    let lock = runtime
        .acquire_community_publish_lock("pool-test", "ownership")
        .await
        .unwrap();
    lock.release().await.unwrap();
    let temp = runtime
        .with_isolated_test_connection(format!("pool_temp_{}", uuid::Uuid::new_v4().simple()))
        .unwrap();
    let setup = temp.connect_cached_client().await.unwrap();
    setup
        .batch_execute("CREATE TEMP TABLE only_here(value int); INSERT INTO only_here VALUES(19)")
        .await
        .unwrap();
    let pinned = temp.connect_client().await.unwrap();
    assert_eq!(
        pinned
            .query_one("SELECT value FROM only_here", &[])
            .await
            .unwrap()
            .get::<_, i32>(0),
        19
    );
    drop((pinned, setup));
    temp.evict_cached_test_client();
    cleanup(&runtime).await;
}

/// Non-gating, local PostgreSQL diagnostic. Measures the connection strategy,
/// not a whole HTTP endpoint; run explicitly with --ignored --nocapture.
#[tokio::test]
#[ignore = "manual PostgreSQL connection performance baseline"]
async fn pool_benchmark_eight_sequential_queries() {
    let runtime = fixture().await;
    let warm = runtime.connect_client().await.unwrap();
    warm.query_one("SELECT value FROM pool_probe WHERE id=$1", &[&1_i32])
        .await
        .unwrap();
    drop(warm);
    for pooled in [false, true] {
        let mut samples = Vec::new();
        let mut backends = BTreeSet::new();
        for _ in 0..12 {
            let start = std::time::Instant::now();
            for _ in 0..8 {
                let row = if pooled {
                    let client = runtime.connect_client().await.unwrap();
                    client
                        .query_one(
                            "SELECT pg_backend_pid(),value FROM pool_probe WHERE id=$1",
                            &[&1_i32],
                        )
                        .await
                        .unwrap()
                } else {
                    let client = runtime.connect_new_client().await.unwrap();
                    client
                        .query_one(
                            "SELECT pg_backend_pid(),value FROM pool_probe WHERE id=$1",
                            &[&1_i32],
                        )
                        .await
                        .unwrap()
                };
                backends.insert(row.get::<_, i32>(0));
                assert_eq!(row.get::<_, i32>(1), 0);
            }
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "strategy={} batches=12 queries_per_batch=8 distinct_backends={} median_ms={:.3} min_ms={:.3} max_ms={:.3}",
            if pooled { "pooled" } else { "fresh" },
            backends.len(),
            (samples[5] + samples[6]) / 2.0,
            samples[0],
            samples[11]
        );
    }
    cleanup(&runtime).await;
}

#[tokio::test]
async fn pool_panic_discards_lease_without_replaying_committed_write() {
    let runtime = fixture().await;
    let pool = Arc::new(PgQueryPool::new(1));
    let task_pool = pool.clone();
    let task_runtime = runtime.clone();
    let (send, receive) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let client = task_pool
            .acquire(task_runtime.connect_managed_client())
            .await
            .unwrap();
        client
            .execute("INSERT INTO pool_probe VALUES (2,41)", &[])
            .await
            .unwrap();
        send.send(pid(&client).await).unwrap();
        panic!("synthetic handler panic after committed write");
    });
    let abandoned_pid = receive.await.unwrap();
    assert!(task.await.unwrap_err().is_panic());
    let client = pool
        .acquire(runtime.connect_managed_client())
        .await
        .unwrap();
    assert_ne!(abandoned_pid, pid(&client).await);
    let row = client
        .query_one(
            "SELECT count(*)::bigint,sum(value)::bigint FROM pool_probe WHERE id=2",
            &[],
        )
        .await
        .unwrap();
    assert_eq!((row.get::<_, i64>(0), row.get::<_, i64>(1)), (1, 41));
    drop((client, pool));
    cleanup(&runtime).await;
}
