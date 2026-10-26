//! The spend record itself: one row per served request in the
//! gateway's `LiteLLM_SpendLogs` table (crate-1 brief §1). The
//! record carries the caller's key only as its SHA-256 hash —
//! exactly the value the Python gateway stores in `api_key` —
//! and jsonb payloads are carried as values, never re-shaped.

use serde_json::Value;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::{CatalogPricer, Deployment, Error};

/// The hash the Python gateway stores for a caller key
/// (`hashlib.sha256(token).hexdigest()` in LiteLLM's token utils).
pub fn hash_api_key(raw_key: &str) -> String {
    format!("{:x}", Sha256::digest(raw_key.as_bytes()))
}

/// One `LiteLLM_SpendLogs` row, in the table's column shape.
#[derive(Clone, Debug, PartialEq)]
pub struct SpendLogRecord {
    pub request_id: String,
    pub call_type: String,
    /// The caller key's hash. Never the raw key.
    pub api_key: String,
    pub spend: f64,
    pub total_tokens: i32,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub request_duration_ms: Option<i32>,
    pub completion_start_time: Option<OffsetDateTime>,
    pub model: String,
    pub model_id: Option<String>,
    pub model_group: Option<String>,
    pub custom_llm_provider: Option<String>,
    pub api_base: Option<String>,
    pub user: Option<String>,
    pub metadata: Value,
    pub cache_hit: Option<String>,
    pub cache_key: Option<String>,
    pub request_tags: Value,
    pub team_id: Option<String>,
    pub organization_id: Option<String>,
    pub end_user: Option<String>,
    pub requester_ip_address: Option<String>,
    pub messages: Value,
    pub response: Value,
    pub session_id: Option<String>,
    pub status: Option<String>,
    pub mcp_namespaced_tool_name: Option<String>,
    pub proxy_server_request: Value,
    pub litellm_call_id: Option<String>,
}

/// Everything a served request contributes to its spend record.
/// The `serving` deployment is the one that answered — for a
/// request that fell back, that is the fallback target, and the
/// record lands on it (brief §2). The raw caller key is held only
/// transiently here; `build` stores its hash.
pub struct SpendLogDraft {
    pub request_id: String,
    pub call_type: String,
    pub serving: Deployment,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub completion_start_time: Option<OffsetDateTime>,
    pub raw_api_key: Option<String>,
    pub model_id: Option<String>,
    pub user: Option<String>,
    pub metadata: Value,
    pub cache_hit: Option<String>,
    pub cache_key: Option<String>,
    pub request_tags: Value,
    pub team_id: Option<String>,
    pub organization_id: Option<String>,
    pub end_user: Option<String>,
    pub requester_ip_address: Option<String>,
    pub messages: Value,
    pub response: Value,
    pub session_id: Option<String>,
    pub status: Option<String>,
    pub mcp_namespaced_tool_name: Option<String>,
    pub proxy_server_request: Value,
    pub litellm_call_id: Option<String>,
}

impl SpendLogDraft {
    pub fn new(
        request_id: impl Into<String>,
        serving: Deployment,
        prompt_tokens: u64,
        completion_tokens: u64,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            call_type: "completion".to_string(),
            serving,
            prompt_tokens,
            completion_tokens,
            start_time,
            end_time,
            completion_start_time: None,
            raw_api_key: None,
            model_id: None,
            user: None,
            metadata: Value::Object(Default::default()),
            cache_hit: None,
            cache_key: None,
            request_tags: Value::Array(Vec::new()),
            team_id: None,
            organization_id: None,
            end_user: None,
            requester_ip_address: None,
            messages: Value::Object(Default::default()),
            response: Value::Object(Default::default()),
            session_id: None,
            status: None,
            mcp_namespaced_tool_name: None,
            proxy_server_request: Value::Object(Default::default()),
            litellm_call_id: None,
        }
    }

    /// Stamp the draft into a record: price it, total the tokens,
    /// hash the caller key, and land the record on the serving
    /// deployment.
    pub fn build(self, pricer: &CatalogPricer) -> Result<SpendLogRecord, Error> {
        let spend = pricer.spend(&self.serving, self.prompt_tokens, self.completion_tokens)?;
        let prompt_tokens =
            i32::try_from(self.prompt_tokens).map_err(|_| Error::TokenOverflow {
                field: "prompt_tokens",
            })?;
        let completion_tokens =
            i32::try_from(self.completion_tokens).map_err(|_| Error::TokenOverflow {
                field: "completion_tokens",
            })?;
        let total_tokens =
            prompt_tokens
                .checked_add(completion_tokens)
                .ok_or(Error::TokenOverflow {
                    field: "total_tokens",
                })?;
        let duration_ms = (self.end_time - self.start_time).whole_milliseconds();
        Ok(SpendLogRecord {
            request_id: self.request_id,
            call_type: self.call_type,
            api_key: self
                .raw_api_key
                .as_deref()
                .map(hash_api_key)
                .unwrap_or_default(),
            spend,
            total_tokens,
            prompt_tokens,
            completion_tokens,
            start_time: self.start_time,
            end_time: self.end_time,
            request_duration_ms: Some(i32::try_from(duration_ms).unwrap_or(i32::MAX)),
            completion_start_time: self.completion_start_time,
            model: self.serving.model.to_string(),
            model_id: self.model_id,
            model_group: Some(self.serving.alias.to_string()),
            custom_llm_provider: Some(self.serving.provider.to_string()),
            api_base: self.serving.api_base.map(str::to_string),
            user: self.user,
            metadata: self.metadata,
            cache_hit: self.cache_hit,
            cache_key: self.cache_key,
            request_tags: self.request_tags,
            team_id: self.team_id,
            organization_id: self.organization_id,
            end_user: self.end_user,
            requester_ip_address: self.requester_ip_address,
            messages: self.messages,
            response: self.response,
            session_id: self.session_id,
            status: self.status,
            mcp_namespaced_tool_name: self.mcp_namespaced_tool_name,
            proxy_server_request: self.proxy_server_request,
            litellm_call_id: self.litellm_call_id,
        })
    }
}

/// The `/global/spend` aggregate over a set of records: the plain
/// sum the Python endpoint returns for the table (brief §1).
pub fn global_spend(records: &[SpendLogRecord]) -> f64 {
    records.iter().map(|record| record.spend).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{deployment, test_support::fixture_pricer};
    use rstest::rstest;
    use serde_json::json;
    use time::Duration;

    fn at(seconds: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(seconds).unwrap()
    }

    fn draft(alias: &str, prompt: u64, completion: u64) -> SpendLogDraft {
        SpendLogDraft::new(
            "req-test",
            *deployment(alias).unwrap(),
            prompt,
            completion,
            at(1_700_000_000),
            at(1_700_000_000) + Duration::milliseconds(1500),
        )
    }

    #[rstest]
    #[case::or_flash("or-flash", "google/gemini-2.5-flash", "openai", 0.00155)]
    #[case::or_gpt_mini("or-gpt-mini", "openai/gpt-5-mini", "openai", 0.00125)]
    #[case::or_gpt_image_mini("or-gpt-image-mini", "openai/gpt-5-image-mini", "openai", 0.0)]
    #[case::or_free_code("or-free-code", "poolside/laguna-s-2.1:free", "openai", 0.0)]
    #[case::or_free("or-free", "openrouter/free", "openai", 0.0)]
    #[case::gem_flash("gem-flash", "gemini-3.5-flash", "gemini", 0.006)]
    #[case::gem_flash_lite("gem-flash-lite", "gemini-3.5-flash-lite", "gemini", 0.00155)]
    #[case::gem_gemma("gem-gemma", "gemma-4-31b-it", "gemini", 0.0)]
    fn record_lands_on_the_serving_deployment(
        #[case] alias: &str,
        #[case] model: &str,
        #[case] provider: &str,
        #[case] spend: f64,
    ) {
        let record = draft(alias, 1000, 500).build(&fixture_pricer()).unwrap();
        assert_eq!(record.model, model);
        assert_eq!(record.model_group.as_deref(), Some(alias));
        assert_eq!(record.custom_llm_provider.as_deref(), Some(provider));
        assert_eq!(record.prompt_tokens, 1000);
        assert_eq!(record.completion_tokens, 500);
        assert_eq!(record.total_tokens, 1500);
        assert!((record.spend - spend).abs() < 1e-12);
    }

    #[rstest]
    fn api_key_is_stamped_as_its_hash_only() {
        // A fixture key, not a credential: the record must carry
        // its SHA-256 hex and nothing else of it.
        let raw = "sk-fixture-master-key";
        let mut draft = draft("gem-flash", 10, 5);
        draft.raw_api_key = Some(raw.to_string());
        let record = draft.build(&fixture_pricer()).unwrap();
        assert_eq!(record.api_key, hash_api_key(raw));
        assert_eq!(record.api_key.len(), 64);
        assert_ne!(record.api_key, raw);
    }

    #[rstest]
    fn missing_api_key_records_the_python_default() {
        let record = draft("gem-flash", 10, 5).build(&fixture_pricer()).unwrap();
        assert_eq!(record.api_key, "");
    }

    #[rstest]
    fn fallback_record_lands_on_the_serving_deployment() {
        // Requested or-free-code; the chain in the production
        // config serves it from or-free. The record names the
        // serving deployment, not the requested alias.
        let mut draft = draft("or-free", 100, 50);
        draft.request_id = "req-fallback".to_string();
        let record = draft.build(&fixture_pricer()).unwrap();
        assert_eq!(record.model_group.as_deref(), Some("or-free"));
        assert_eq!(record.model, "openrouter/free");
        assert_eq!(record.spend, 0.0);
    }

    #[rstest]
    fn timestamps_and_duration_are_stamped() {
        let mut draft = draft("gem-flash", 10, 5);
        draft.completion_start_time = Some(at(1_700_000_000) + Duration::milliseconds(900));
        draft.status = Some("success".to_string());
        draft.metadata = json!({"source": "crate-1-test"});
        let record = draft.build(&fixture_pricer()).unwrap();
        assert_eq!(record.start_time, at(1_700_000_000));
        assert_eq!(record.request_duration_ms, Some(1500));
        assert_eq!(
            record.completion_start_time,
            Some(at(1_700_000_000) + Duration::milliseconds(900))
        );
        assert_eq!(record.status.as_deref(), Some("success"));
        assert_eq!(record.metadata, json!({"source": "crate-1-test"}));
    }

    #[rstest]
    fn global_spend_sums_the_records() {
        let pricer = fixture_pricer();
        let records = vec![
            draft("gem-flash", 1000, 500).build(&pricer).unwrap(),
            draft("or-free", 1000, 500).build(&pricer).unwrap(),
        ];
        assert!((global_spend(&records) - 0.006).abs() < 1e-12);
        assert_eq!(global_spend(&[]), 0.0);
    }
}
