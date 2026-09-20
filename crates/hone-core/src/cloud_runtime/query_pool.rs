//! Exclusive, bounded leases for the runtime's private autocommit query paths.
//!
//! Transactions, migrations, advisory locks and session-local settings must use
//! their existing dedicated connections. No raw mutable client is exposed here.
//! Failed, cancelled or panicking operations discard their connection; they are
//! never retried, since a failed write may already have committed on the server.

use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, Error, Row, Statement, ToStatement};

use crate::{HoneError, HoneResult};

pub(super) struct PgQueryPool {
    idle: Mutex<Vec<PgConnection>>,
    permits: Arc<Semaphore>,
}

impl PgQueryPool {
    pub(super) fn new(size: usize) -> Self {
        assert!(size > 0);
        Self {
            idle: Mutex::new(Vec::with_capacity(size)),
            permits: Arc::new(Semaphore::new(size)),
        }
    }

    pub(super) async fn acquire(
        self: &Arc<Self>,
        connect: impl Future<Output = HoneResult<PgConnection>>,
    ) -> HoneResult<PgQueryClient> {
        let permit = self
            .permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| HoneError::Config("Postgres query pool 已关闭".into()))?;
        let cached = {
            let mut idle = self
                .idle
                .lock()
                .map_err(|_| HoneError::Config("Postgres query pool 锁失败".into()))?;
            loop {
                match idle.pop() {
                    Some(client) if client.client().is_closed() => continue,
                    other => break other,
                }
            }
        };
        let client = match cached {
            Some(client) => client,
            None => connect.await?,
        };
        Ok(PgQueryClient {
            connection: QueryConnection::Pooled {
                client: Some(client),
                pool: self.clone(),
                _permit: permit,
            },
            in_flight: AtomicUsize::new(0),
            failed: AtomicBool::new(false),
        })
    }
}

enum QueryConnection {
    // Sync bridges must own both their driver and capacity independently of
    // the caller runtime; even waiting on the shared semaphore can deadlock.
    Dedicated(PgConnection),
    Pooled {
        client: Option<PgConnection>,
        pool: Arc<PgQueryPool>,
        _permit: OwnedSemaphorePermit,
    },
    // Only connection-local pg_temp fixtures use this existing pinned client.
    Pinned(Arc<Client>),
}

/// Aborting the driver closes the transport instead of leaving a cancelled
/// query draining on a connection that has already released its pool permit.
pub(super) struct PgConnection {
    client: Option<Client>,
    driver: Option<tokio::task::AbortHandle>,
}

impl PgConnection {
    pub(super) fn new(client: Client, driver: tokio::task::AbortHandle) -> Self {
        Self {
            client: Some(client),
            driver: Some(driver),
        }
    }

    pub(super) fn client(&self) -> &Client {
        self.client.as_ref().expect("live connection")
    }

    // Preserve the existing ownership of dedicated/cached clients: their
    // caller, including tokio-postgres Transaction, controls graceful shutdown.
    pub(super) fn detach(mut self) -> Client {
        self.driver.take();
        self.client.take().expect("live connection")
    }
}

impl Drop for PgConnection {
    fn drop(&mut self) {
        if let Some(driver) = self.driver.take() {
            driver.abort();
        }
    }
}

pub(super) struct PgQueryClient {
    connection: QueryConnection,
    in_flight: AtomicUsize,
    failed: AtomicBool,
}

impl PgQueryClient {
    pub(super) fn dedicated(client: PgConnection) -> Self {
        Self {
            connection: QueryConnection::Dedicated(client),
            in_flight: AtomicUsize::new(0),
            failed: AtomicBool::new(false),
        }
    }

    pub(super) fn pinned(client: Arc<Client>) -> Self {
        Self {
            connection: QueryConnection::Pinned(client),
            in_flight: AtomicUsize::new(0),
            failed: AtomicBool::new(false),
        }
    }

    pub(super) fn raw(&self) -> &Client {
        match &self.connection {
            QueryConnection::Dedicated(client) => client.client(),
            QueryConnection::Pooled { client, .. } => client.as_ref().expect("live lease").client(),
            QueryConnection::Pinned(client) => client,
        }
    }

    // Cancellation intentionally leaves in_flight nonzero, so Drop will never
    // return a connection with an unfinished query to the pool.
    pub(super) async fn run<T, E>(
        &self,
        operation: impl Future<Output = Result<T, E>>,
    ) -> Result<T, E> {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        let result = operation.await;
        if result.is_err() {
            self.failed.store(true, Ordering::Relaxed);
        }
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
        result
    }

    pub(super) async fn query<T: ?Sized + ToStatement>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error> {
        self.run(self.raw().query(statement, params)).await
    }

    pub(super) async fn query_one<T: ?Sized + ToStatement>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, Error> {
        self.run(self.raw().query_one(statement, params)).await
    }

    pub(super) async fn query_opt<T: ?Sized + ToStatement>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<Row>, Error> {
        self.run(self.raw().query_opt(statement, params)).await
    }

    pub(super) async fn execute<T: ?Sized + ToStatement>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, Error> {
        self.run(self.raw().execute(statement, params)).await
    }

    pub(super) async fn prepare(&self, statement: &str) -> Result<Statement, Error> {
        self.run(self.raw().prepare(statement)).await
    }
}

impl Drop for PgQueryClient {
    fn drop(&mut self) {
        if let QueryConnection::Pooled { client, pool, .. } = &mut self.connection
            && let Some(client) = client.take()
            && !client.client().is_closed()
            && !self.failed.load(Ordering::Relaxed)
            && self.in_flight.load(Ordering::Relaxed) == 0
            && !std::thread::panicking()
            && let Ok(mut idle) = pool.idle.lock()
        {
            idle.push(client);
        }
        // The permit is released after returning/discarding the client.
    }
}

#[cfg(test)]
mod tests;
