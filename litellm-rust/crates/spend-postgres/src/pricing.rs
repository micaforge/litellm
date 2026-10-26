//! Spend computation for one served request: the cost map
//! (via `litellm-model-catalog`) priced through the `litellm-cost`
//! calculator, under the crate-1 brief's rules — free routes price
//! at exactly zero, and a deployment with no per-token pricing in
//! the cost map records zero, as the Python gateway does for
//! unpriced models.

use litellm_model_catalog::Catalog;

use crate::{Deployment, Error};

/// Prices deployments against one parsed cost map.
pub struct CatalogPricer {
    catalog: Catalog,
}

impl CatalogPricer {
    pub fn new(catalog: Catalog) -> Self {
        Self { catalog }
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// USD spend for a request the deployment served, given the
    /// response's usage counts (the record's token source of truth,
    /// per the brief §3).
    pub fn spend(
        &self,
        deployment: &Deployment,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) -> Result<f64, Error> {
        let _ = (deployment, prompt_tokens, completion_tokens);
        todo!("crate-1 stage 2: price via litellm-cost + litellm-model-catalog")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{deployment, test_support::fixture_pricer};
    use rstest::rstest;

    fn assert_spend(alias: &str, prompt: u64, completion: u64, expected: f64) {
        let pricer = fixture_pricer();
        let spend = pricer
            .spend(deployment(alias).unwrap(), prompt, completion)
            .unwrap();
        assert!(
            (spend - expected).abs() < 1e-12,
            "{alias}: spend {spend} != expected {expected}"
        );
    }

    #[rstest]
    #[case::or_flash("or-flash", 1000, 500, 0.00155)]
    #[case::or_gpt_mini("or-gpt-mini", 1000, 500, 0.00125)]
    #[case::gem_flash("gem-flash", 2000, 1000, 0.012)]
    #[case::gem_flash_lite("gem-flash-lite", 1000, 500, 0.00155)]
    fn priced_deployments_use_the_cost_map(
        #[case] alias: &str,
        #[case] prompt: u64,
        #[case] completion: u64,
        #[case] expected: f64,
    ) {
        assert_spend(alias, prompt, completion, expected);
    }

    #[rstest]
    #[case::or_free_code("or-free-code")]
    #[case::or_free("or-free")]
    fn free_routes_price_at_exactly_zero(#[case] alias: &str) {
        assert_spend(alias, 1_000_000, 1_000_000, 0.0);
    }

    #[rstest]
    fn zero_rated_catalog_entry_prices_at_zero() {
        // gem-gemma is priced in the cost map at 0.0 / 0.0; that is
        // a distinct path from the free-route rule.
        assert_spend("gem-gemma", 10_000, 10_000, 0.0);
    }

    #[rstest]
    fn unpriced_deployment_records_zero() {
        // or-gpt-image-mini has no per-token entry in the cost map.
        assert_spend("or-gpt-image-mini", 1000, 500, 0.0);
    }
}
