//! Shared fixtures for the crate's inline test modules: a small
//! catalog carrying the cost-map rates for our priced deployments
//! (values copied from `model_prices_and_context_window.json` at
//! brief time) so tests never read the repo data file.

use litellm_model_catalog::{Catalog, Provenance};

use crate::CatalogPricer;

pub const FIXTURE_CATALOG_JSON: &str = r#"{
    "gemini/gemini-2.5-flash": {
        "litellm_provider": "gemini",
        "input_cost_per_token": 3e-7,
        "output_cost_per_token": 2.5e-6
    },
    "gpt-5-mini": {
        "litellm_provider": "openai",
        "input_cost_per_token": 2.5e-7,
        "output_cost_per_token": 2e-6
    },
    "gemini/gemini-3.5-flash": {
        "litellm_provider": "gemini",
        "input_cost_per_token": 1.5e-6,
        "output_cost_per_token": 9e-6
    },
    "gemini/gemini-3.5-flash-lite": {
        "litellm_provider": "gemini",
        "input_cost_per_token": 3e-7,
        "output_cost_per_token": 2.5e-6
    },
    "gemini/gemma-4-31b-it": {
        "litellm_provider": "gemini",
        "input_cost_per_token": 0.0,
        "output_cost_per_token": 0.0
    }
}"#;

pub fn fixture_catalog() -> Catalog {
    Catalog::parse(FIXTURE_CATALOG_JSON.as_bytes(), Provenance::default())
        .expect("fixture catalog parses")
}

pub fn fixture_pricer() -> CatalogPricer {
    CatalogPricer::new(fixture_catalog())
}
