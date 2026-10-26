//! End-to-end persistence check against a scratch Postgres.
//! Gated on `LITELLM_SPEND_POSTGRES_TEST_URL` (a database the
//! test may create the spend table in and write to); returns
//! early when unset, so `cargo test` stays offline by default.
//! Never point this at the production gateway database.
//!
//! Runtime-checked DDL, for the same reason as the store: the
//! schema is Prisma-managed, outside sqlx's migration world.
#![allow(clippy::disallowed_methods)]

use litellm_model_catalog::{Catalog, Provenance};
use litellm_spend_postgres::{CatalogPricer, SpendLogDraft, SpendStore, deployment};
use time::{Duration, OffsetDateTime};

/// The gateway schema's spend table (Prisma `LiteLLM_SpendLogs`
/// in `litellm/proxy/schema.prisma`), mirrored for the scratch DB.
const DDL: &str = r#"
CREATE TABLE "LiteLLM_SpendLogs" (
    request_id TEXT PRIMARY KEY,
    call_type TEXT NOT NULL,
    api_key TEXT NOT NULL DEFAULT '',
    spend DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_tokens INT NOT NULL DEFAULT 0,
    prompt_tokens INT NOT NULL DEFAULT 0,
    completion_tokens INT NOT NULL DEFAULT 0,
    "startTime" TIMESTAMP(3) NOT NULL,
    "endTime" TIMESTAMP(3) NOT NULL,
    request_duration_ms INT,
    "completionStartTime" TIMESTAMP(3),
    model TEXT NOT NULL DEFAULT '',
    model_id TEXT,
    model_group TEXT,
    custom_llm_provider TEXT,
    api_base TEXT,
    "user" TEXT,
    metadata JSONB,
    cache_hit TEXT,
    cache_key TEXT,
    request_tags JSONB,
    team_id TEXT,
    organization_id TEXT,
    end_user TEXT,
    requester_ip_address TEXT,
    messages JSONB,
    response JSONB,
    session_id TEXT,
    status TEXT,
    mcp_namespaced_tool_name TEXT,
    proxy_server_request JSONB,
    litellm_call_id TEXT,
    created_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP
)"#;

const CATALOG_JSON: &str = r#"{
    "gemini/gemini-3.5-flash": {
        "litellm_provider": "gemini",
        "input_cost_per_token": 1.5e-6,
        "output_cost_per_token": 9e-6
    }
}"#;

#[tokio::test]
async fn records_round_trip_and_the_aggregate_sums_them() {
    let Ok(url) = std::env::var("LITELLM_SPEND_POSTGRES_TEST_URL") else {
        return;
    };
    let store = SpendStore::connect(&url).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS \"LiteLLM_SpendLogs\"")
        .execute(store.pool())
        .await
        .unwrap();
    sqlx::query(DDL).execute(store.pool()).await.unwrap();

    let catalog = Catalog::parse(CATALOG_JSON.as_bytes(), Provenance::default()).unwrap();
    let pricer = CatalogPricer::new(catalog);
    let start = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();

    let mut paid = SpendLogDraft::new(
        "req-scratch-paid",
        *deployment("gem-flash").unwrap(),
        1000,
        500,
        start,
        start + Duration::milliseconds(1500),
    );
    paid.raw_api_key = Some("sk-fixture-master-key".to_string());
    paid.status = Some("success".to_string());
    let paid = paid.build(&pricer).unwrap();

    let free = SpendLogDraft::new(
        "req-scratch-free",
        *deployment("or-free").unwrap(),
        1000,
        500,
        start,
        start + Duration::milliseconds(800),
    )
    .build(&pricer)
    .unwrap();

    store.insert(&paid).await.unwrap();
    store.insert(&free).await.unwrap();

    let fetched = store.fetch("req-scratch-paid").await.unwrap().unwrap();
    assert_eq!(fetched, paid);
    assert_eq!(fetched.api_key.len(), 64);
    assert_eq!(fetched.model_group.as_deref(), Some("gem-flash"));

    let fetched_free = store.fetch("req-scratch-free").await.unwrap().unwrap();
    assert_eq!(fetched_free.spend, 0.0);

    let total = store.global_spend().await.unwrap();
    assert!((total - 0.006).abs() < 1e-12, "aggregate {total}");
}
