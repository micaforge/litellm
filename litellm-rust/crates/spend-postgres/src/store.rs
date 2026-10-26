//! Minimal Postgres persistence owned by the spend module
//! itself (Lead's design call #1 in the crate-1 brief: the
//! general `crates/db` stub is not grown for this). The store
//! writes only the database it is connected to — the trial
//! gateway's scratch database; production is a read-only
//! reference and is never connected by this crate's tests.

use sqlx::PgPool;

use crate::{Error, SpendLogRecord};

/// A pool-backed writer/reader for `LiteLLM_SpendLogs`.
pub struct SpendStore {
    pool: PgPool,
}

impl SpendStore {
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(database_url: &str) -> Result<Self, Error> {
        let _ = database_url;
        todo!("crate-1 stage 3: connect the pool")
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Insert one record as one row.
    pub async fn insert(&self, record: &SpendLogRecord) -> Result<(), Error> {
        let _ = record;
        todo!("crate-1 stage 3: insert the record")
    }

    /// Read one record back by request id (trial comparisons).
    pub async fn fetch(&self, request_id: &str) -> Result<Option<SpendLogRecord>, Error> {
        let _ = request_id;
        todo!("crate-1 stage 3: fetch the record")
    }

    /// The `/global/spend` equivalent over the whole table.
    pub async fn global_spend(&self) -> Result<f64, Error> {
        todo!("crate-1 stage 4: aggregate the table")
    }
}
