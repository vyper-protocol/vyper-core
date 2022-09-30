use anchor_lang::prelude::*;

#[error_code]
pub enum RateTwapErrorCode {
    #[msg("generic error")]
    GenericError,

    #[msg("input error")]
    InputError,

    #[msg("empty samples")]
    EmptySamples,

    #[msg("another too recent sample")]
    AnotherTooRecentSample,
}
