//! Minimal Postgres persistence owned by the spend module
//! itself (Lead's design call #1 in the crate-1 brief: the
//! general `crates/db` stub is not grown for this). The store
//! writes only the database it is connected to — the trial
//! gateway's scratch database; production is a read-only
//! reference and is never connected by this crate's tests.
//!
//! The statements are fixed constants checked by the live
//! scratch-database test, not by the sqlx macros: the schema
//! they target is the Python gateway's Prisma schema, for which
//! no sqlx migration set exists in this repo to check against,
//! so the workspace's checked-query rule cannot apply here.
#![allow(clippy::disallowed_methods)]

use serde_json::Value;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;

use crate::{Error, SpendLogRecord};

// The schema's time columns are `TIMESTAMP(3)` holding UTC wall
// time (Prisma `DateTime`, no zone). The statements below convert
// at the boundary — `AT TIME ZONE 'UTC'` in both directions — so
// records carry exact instants whatever the session time zone is.
const INSERT: &str = "INSERT INTO \"LiteLLM_SpendLogs\" (request_id, call_type, api_key, spend, total_tokens, prompt_tokens, completion_tokens, \"startTime\", \"endTime\", request_duration_ms, \"completionStartTime\", model, model_id, model_group, custom_llm_provider, api_base, \"user\", metadata, cache_hit, cache_key, request_tags, team_id, organization_id, end_user, requester_ip_address, messages, response, session_id, status, mcp_namespaced_tool_name, proxy_server_request, litellm_call_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, ($8 AT TIME ZONE 'UTC'), ($9 AT TIME ZONE 'UTC'), $10, ($11 AT TIME ZONE 'UTC'), $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, ($33 AT TIME ZONE 'UTC'), ($34 AT TIME ZONE 'UTC'))";

const SELECT_ONE: &str = "SELECT request_id, call_type, api_key, spend, total_tokens, prompt_tokens, completion_tokens, \"startTime\" AT TIME ZONE 'UTC' AS \"startTime\", \"endTime\" AT TIME ZONE 'UTC' AS \"endTime\", request_duration_ms, \"completionStartTime\" AT TIME ZONE 'UTC' AS \"completionStartTime\", model, model_id, model_group, custom_llm_provider, api_base, \"user\", metadata, cache_hit, cache_key, request_tags, team_id, organization_id, end_user, requester_ip_address, messages, response, session_id, status, mcp_namespaced_tool_name, proxy_server_request, litellm_call_id FROM \"LiteLLM_SpendLogs\" WHERE request_id = $1";

const SELECT_GLOBAL_SPEND: &str = "SELECT COALESCE(SUM(spend), 0) FROM \"LiteLLM_SpendLogs\"";

/// A pool-backed writer/reader for `LiteLLM_SpendLogs`.
pub struct SpendStore {
    pool: PgPool,
}

impl SpendStore {
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(database_url: &str) -> Result<Self, Error> {
        Ok(Self::from_pool(PgPool::connect(database_url).await?))
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Insert one record as one row. `created_at`/`updated_at`
    /// are stamped with the record's end time; the Python schema
    /// maintains them at the application layer, as here.
    pub async fn insert(&self, record: &SpendLogRecord) -> Result<(), Error> {
        sqlx::query(INSERT)
            .bind(&record.request_id)
            .bind(&record.call_type)
            .bind(&record.api_key)
            .bind(record.spend)
            .bind(record.total_tokens)
            .bind(record.prompt_tokens)
            .bind(record.completion_tokens)
            .bind(record.start_time)
            .bind(record.end_time)
            .bind(record.request_duration_ms)
            .bind(record.completion_start_time)
            .bind(&record.model)
            .bind(&record.model_id)
            .bind(&record.model_group)
            .bind(&record.custom_llm_provider)
            .bind(&record.api_base)
            .bind(&record.user)
            .bind(&record.metadata)
            .bind(&record.cache_hit)
            .bind(&record.cache_key)
            .bind(&record.request_tags)
            .bind(&record.team_id)
            .bind(&record.organization_id)
            .bind(&record.end_user)
            .bind(&record.requester_ip_address)
            .bind(&record.messages)
            .bind(&record.response)
            .bind(&record.session_id)
            .bind(&record.status)
            .bind(&record.mcp_namespaced_tool_name)
            .bind(&record.proxy_server_request)
            .bind(&record.litellm_call_id)
            .bind(record.end_time)
            .bind(record.end_time)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Read one record back by request id (trial comparisons).
    pub async fn fetch(&self, request_id: &str) -> Result<Option<SpendLogRecord>, Error> {
        let row = sqlx::query(SELECT_ONE)
            .bind(request_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(|row| {
            Ok(SpendLogRecord {
                request_id: row.try_get("request_id")?,
                call_type: row.try_get("call_type")?,
                api_key: row.try_get("api_key")?,
                spend: row.try_get("spend")?,
                total_tokens: row.try_get("total_tokens")?,
                prompt_tokens: row.try_get("prompt_tokens")?,
                completion_tokens: row.try_get("completion_tokens")?,
                start_time: row.try_get::<OffsetDateTime, _>("startTime")?,
                end_time: row.try_get::<OffsetDateTime, _>("endTime")?,
                request_duration_ms: row.try_get("request_duration_ms")?,
                completion_start_time: row
                    .try_get::<Option<OffsetDateTime>, _>("completionStartTime")?,
                model: row.try_get("model")?,
                model_id: row.try_get("model_id")?,
                model_group: row.try_get("model_group")?,
                custom_llm_provider: row.try_get("custom_llm_provider")?,
                api_base: row.try_get("api_base")?,
                user: row.try_get("user")?,
                metadata: row.try_get::<Value, _>("metadata")?,
                cache_hit: row.try_get("cache_hit")?,
                cache_key: row.try_get("cache_key")?,
                request_tags: row.try_get::<Value, _>("request_tags")?,
                team_id: row.try_get("team_id")?,
                organization_id: row.try_get("organization_id")?,
                end_user: row.try_get("end_user")?,
                requester_ip_address: row.try_get("requester_ip_address")?,
                messages: row.try_get::<Value, _>("messages")?,
                response: row.try_get::<Value, _>("response")?,
                session_id: row.try_get("session_id")?,
                status: row.try_get("status")?,
                mcp_namespaced_tool_name: row.try_get("mcp_namespaced_tool_name")?,
                proxy_server_request: row.try_get::<Value, _>("proxy_server_request")?,
                litellm_call_id: row.try_get("litellm_call_id")?,
            })
        })
        .transpose()
    }

    /// The `/global/spend` equivalent over the whole table.
    pub async fn global_spend(&self) -> Result<f64, Error> {
        let row = sqlx::query(SELECT_GLOBAL_SPEND)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.try_get::<f64, _>(0)?)
    }
}
