//! Spend-record parity for the Python gateway's Postgres spend
//! log (a07 Forge Crew, crate 1): the `LiteLLM_SpendLogs` record,
//! its cost-map pricing, and a minimal Postgres store owned by
//! this module. Routing and auth are later crates; this crate
//! only records what a serving deployment answered.

mod deployments;
mod error;
mod pricing;
mod record;
mod store;

#[cfg(test)]
mod test_support;

pub use deployments::{Deployment, FALLBACKS, OUR_DEPLOYMENTS, deployment, fallback_targets};
pub use error::Error;
pub use pricing::CatalogPricer;
pub use record::{SpendLogDraft, SpendLogRecord, global_spend, hash_api_key};
pub use store::SpendStore;
