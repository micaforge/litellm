use litellm_cost::PricingError;

/// Failures the spend-postgres crate can return. One top-level error
/// per the workspace convention; callers distinguish pricing,
/// record-shape, and database failures.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The cost crate rejected the pricing inputs for a model.
    #[error("pricing failed for model {model}: {kind:?}")]
    Pricing { model: String, kind: PricingError },

    /// A token count does not fit the spend log's integer columns.
    #[error("token count in {field} overflows the spend log integer range")]
    TokenOverflow { field: &'static str },

    /// The Postgres round trip failed.
    #[error("spend log database error: {0}")]
    Database(#[from] sqlx::Error),
}
