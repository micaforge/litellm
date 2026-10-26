//! The estate's own gateway deployments, as shapes for the spend
//! record. Values come from `~/workspace/litellm/config.yaml`
//! (the production Python gateway config) at brief time
//! (a07 crate-1 brief, §2): the alias is the public model name,
//! `model` is the upstream model id the spend record carries,
//! `provider` is the record's `custom_llm_provider`, and
//! `catalog_key` is the entry in the LiteLLM cost map
//! (`model_prices_and_context_window.json`) whose per-token rates
//! price the deployment. Free routes are priced at exactly zero
//! by the brief's rule, whatever the catalog says.

/// One deployment of the production gateway config.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deployment {
    /// Public model name (the spend record's `model_group`).
    pub alias: &'static str,
    /// Upstream model id (the spend record's `model`).
    pub model: &'static str,
    /// The record's `custom_llm_provider`.
    pub provider: &'static str,
    /// Cost-map key used to price this deployment, when priced.
    pub catalog_key: Option<&'static str>,
    /// The record's `api_base`, when the deployment names one.
    pub api_base: Option<&'static str>,
    /// Free routes price at exactly zero.
    pub free: bool,
}

const OPENROUTER_API_BASE: &str = "https://openrouter.ai/api/v1";

/// The eight deployments of the production config (P1+P2 set in
/// the crate-1 brief, §2).
pub const OUR_DEPLOYMENTS: [Deployment; 8] = [
    Deployment {
        alias: "or-flash",
        model: "google/gemini-2.5-flash",
        provider: "openai",
        catalog_key: Some("gemini/gemini-2.5-flash"),
        api_base: Some(OPENROUTER_API_BASE),
        free: false,
    },
    Deployment {
        alias: "or-gpt-mini",
        model: "openai/gpt-5-mini",
        provider: "openai",
        catalog_key: Some("gpt-5-mini"),
        api_base: Some(OPENROUTER_API_BASE),
        free: false,
    },
    Deployment {
        alias: "or-gpt-image-mini",
        model: "openai/gpt-5-image-mini",
        provider: "openai",
        catalog_key: None,
        api_base: Some(OPENROUTER_API_BASE),
        free: false,
    },
    Deployment {
        alias: "or-free-code",
        model: "poolside/laguna-s-2.1:free",
        provider: "openai",
        catalog_key: None,
        api_base: Some(OPENROUTER_API_BASE),
        free: true,
    },
    Deployment {
        alias: "or-free",
        model: "openrouter/free",
        provider: "openai",
        catalog_key: None,
        api_base: Some(OPENROUTER_API_BASE),
        free: true,
    },
    Deployment {
        alias: "gem-flash",
        model: "gemini-3.5-flash",
        provider: "gemini",
        catalog_key: Some("gemini/gemini-3.5-flash"),
        api_base: None,
        free: false,
    },
    Deployment {
        alias: "gem-flash-lite",
        model: "gemini-3.5-flash-lite",
        provider: "gemini",
        catalog_key: Some("gemini/gemini-3.5-flash-lite"),
        api_base: None,
        free: false,
    },
    Deployment {
        alias: "gem-gemma",
        model: "gemma-4-31b-it",
        provider: "gemini",
        catalog_key: Some("gemini/gemma-4-31b-it"),
        api_base: None,
        free: false,
    },
];

/// The deployment serving an alias in the production config.
pub fn deployment(alias: &str) -> Option<&'static Deployment> {
    OUR_DEPLOYMENTS.iter().find(|entry| entry.alias == alias)
}

/// The production fallback chains (config.yaml `litellm_settings`):
/// a request to the key alias may be served by a target deployment,
/// and its spend record lands on the serving deployment.
pub const FALLBACKS: [(&str, &[&str]); 4] = [
    ("or-free-code", &["or-free"]),
    ("or-free", &["gem-flash"]),
    ("gem-flash", &["or-free-code"]),
    ("gem-gemma", &["or-free"]),
];

/// The fallback targets configured for an alias, when any.
pub fn fallback_targets(alias: &str) -> Option<&'static [&'static str]> {
    FALLBACKS
        .iter()
        .find(|(source, _)| *source == alias)
        .map(|(_, targets)| *targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::or_flash("or-flash", "google/gemini-2.5-flash", "openai", false)]
    #[case::or_gpt_mini("or-gpt-mini", "openai/gpt-5-mini", "openai", false)]
    #[case::or_gpt_image_mini("or-gpt-image-mini", "openai/gpt-5-image-mini", "openai", false)]
    #[case::or_free_code("or-free-code", "poolside/laguna-s-2.1:free", "openai", true)]
    #[case::or_free("or-free", "openrouter/free", "openai", true)]
    #[case::gem_flash("gem-flash", "gemini-3.5-flash", "gemini", false)]
    #[case::gem_flash_lite("gem-flash-lite", "gemini-3.5-flash-lite", "gemini", false)]
    #[case::gem_gemma("gem-gemma", "gemma-4-31b-it", "gemini", false)]
    fn our_deployments_match_the_production_config(
        #[case] alias: &str,
        #[case] model: &str,
        #[case] provider: &str,
        #[case] free: bool,
    ) {
        let found = deployment(alias).expect("alias is one of our deployments");
        assert_eq!(found.model, model);
        assert_eq!(found.provider, provider);
        assert_eq!(found.free, free);
    }

    #[rstest]
    #[case::or_free_code("or-free-code", "or-free")]
    #[case::or_free("or-free", "gem-flash")]
    #[case::gem_flash("gem-flash", "or-free-code")]
    #[case::gem_gemma("gem-gemma", "or-free")]
    fn fallback_chains_match_the_production_config(#[case] alias: &str, #[case] target: &str) {
        assert_eq!(fallback_targets(alias), Some(&[target][..]));
    }

    #[rstest]
    fn unknown_alias_is_not_a_deployment() {
        assert!(deployment("not-our-model").is_none());
        assert!(fallback_targets("or-flash").is_none());
    }
}
