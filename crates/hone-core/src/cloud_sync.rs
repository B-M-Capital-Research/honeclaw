//! 同步调用方访问异步 cloud runtime 的共享桥接。
//!
//! `CloudPgRuntime` 的连接驱动必须挂在长驻 Tokio runtime 上。同步存储 API
//! 不能为每次调用临时创建 runtime，也不能在已有 Tokio worker 上直接
//! `Handle::block_on`。这里集中维护一套按进程可用 CPU 数量伸缩的长驻 runtime，
//! 并在 Tokio 上下文内用 channel 等待结果。

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::cloud_runtime::CloudPgRuntime;
use crate::{HoneError, HoneResult};

const CLOUD_SYNC_WORKER_THREADS_ENV: &str = "HONE_CLOUD_SYNC_WORKER_THREADS";

static CLOUD_SYNC_RUNTIME: std::sync::LazyLock<std::io::Result<tokio::runtime::Runtime>> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(cloud_sync_worker_threads())
            .thread_name("hone-cloud-sync")
            .enable_all()
            .build()
    });

static CLOUD_SCHEMA_READY: AtomicBool = AtomicBool::new(false);

fn cloud_sync_worker_threads() -> usize {
    let available = std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN);
    resolve_cloud_sync_worker_threads(
        available,
        std::env::var(CLOUD_SYNC_WORKER_THREADS_ENV).ok().as_deref(),
    )
}

fn resolve_cloud_sync_worker_threads(available: NonZeroUsize, configured: Option<&str>) -> usize {
    match configured.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => match value.parse::<NonZeroUsize>() {
            Ok(worker_threads) => worker_threads.get(),
            Err(error) => {
                tracing::warn!(
                    env = CLOUD_SYNC_WORKER_THREADS_ENV,
                    value,
                    "cloud sync worker thread override 无效，回退到可用 CPU 数: {error}"
                );
                available.get()
            }
        },
        None => available.get(),
    }
}

fn cloud_sync_runtime() -> HoneResult<&'static tokio::runtime::Runtime> {
    CLOUD_SYNC_RUNTIME
        .as_ref()
        .map_err(|err| HoneError::Config(format!("cloud sync runtime 构建失败: {err}")))
}

/// 在共享长驻 runtime 上执行一个 async cloud 操作，同时保持同步调用语义。
pub fn run_cloud_sync<T, F>(
    future: F,
    operation_timeout: Option<Duration>,
    operation_name: &'static str,
) -> HoneResult<T>
where
    T: Send + 'static,
    F: std::future::Future<Output = HoneResult<T>> + Send + 'static,
{
    let runtime = cloud_sync_runtime()?;
    let guarded = async move {
        match operation_timeout {
            Some(operation_timeout) => {
                match tokio::time::timeout(operation_timeout, future).await {
                    Ok(result) => result,
                    Err(_) => Err(HoneError::Storage(format!(
                        "{operation_name} timed out after {}ms",
                        operation_timeout.as_millis()
                    ))),
                }
            }
            None => future.await,
        }
    };

    if tokio::runtime::Handle::try_current().is_err() {
        return runtime.block_on(guarded);
    }

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    runtime.spawn(async move {
        let _ = tx.send(guarded.await);
    });
    let result = match operation_timeout {
        Some(operation_timeout) => rx
            .recv_timeout(operation_timeout.saturating_add(Duration::from_secs(5)))
            .map_err(|err| err.to_string()),
        None => rx.recv().map_err(|err| err.to_string()),
    };
    match result {
        Ok(result) => result,
        Err(err) => Err(HoneError::Storage(format!(
            "{operation_name} worker did not report a result: {err}"
        ))),
    }
}

/// 每个进程首次成功后不再重复执行整套 cloud schema DDL。
///
/// schema 失败时不置位，后续调用仍会重试。测试隔离连接应直接调用
/// `CloudPgRuntime::ensure_schema`，因为每个临时会话都有独立的 `pg_temp` schema。
pub fn ensure_cloud_schema_once(
    postgres: CloudPgRuntime,
    operation_timeout: Option<Duration>,
) -> HoneResult<()> {
    if CLOUD_SCHEMA_READY.load(Ordering::Acquire) {
        return Ok(());
    }
    let schema_postgres = postgres.clone();
    run_cloud_sync(
        async move { schema_postgres.ensure_schema().await },
        operation_timeout,
        "cloud schema operation",
    )?;
    CLOUD_SCHEMA_READY.store(true, Ordering::Release);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn worker_thread_override_takes_precedence_over_available_parallelism() {
        let available = NonZeroUsize::new(7).expect("non-zero available parallelism");

        assert_eq!(resolve_cloud_sync_worker_threads(available, Some("11")), 11);
        assert_eq!(resolve_cloud_sync_worker_threads(available, None), 7);
        assert_eq!(resolve_cloud_sync_worker_threads(available, Some("0")), 7);
        assert_eq!(
            resolve_cloud_sync_worker_threads(available, Some("invalid")),
            7
        );
    }

    #[test]
    fn timeout_bounds_a_stuck_operation() {
        let started = Instant::now();
        let error = run_cloud_sync(
            async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok::<(), HoneError>(())
            },
            Some(Duration::from_millis(20)),
            "test cloud operation",
        )
        .expect_err("operation should time out");
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(error.to_string().contains("test cloud operation timed out"));
    }

    #[tokio::test]
    async fn bridge_works_inside_a_tokio_context_and_reuses_runtime() {
        let values = tokio::task::spawn_blocking(|| {
            (0..32)
                .map(|value| {
                    run_cloud_sync(
                        async move { Ok::<u32, HoneError>(value) },
                        Some(Duration::from_millis(500)),
                        "test cloud operation",
                    )
                })
                .collect::<HoneResult<Vec<_>>>()
        })
        .await
        .expect("join")
        .expect("bridge calls");
        assert_eq!(values.len(), 32);
        assert_eq!(values[31], 31);
    }

    #[tokio::test]
    async fn bridge_without_timeout_works_directly_inside_a_tokio_context() {
        let value = run_cloud_sync(
            async { Ok::<u32, HoneError>(7) },
            None,
            "unbounded test cloud operation",
        )
        .expect("bridge should return from inside the tokio context");

        assert_eq!(value, 7);
    }
}
