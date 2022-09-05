use anchor_lang::prelude::*;

#[error_code]
pub enum RatePythErrorCode {
    #[msg("generic error")]
    GenericError,

    // #[msg("invalid aggregator owner")]
    // InvalidAggregatorOwner,
    #[msg("invalid aggregators number")]
    InvalidAggregatorsNumber,

    #[msg("math error")]
    MathError,
}
