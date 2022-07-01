use anchor_lang::prelude::*;

#[error_code]
pub enum RateErrors {
    #[msg("generic error")]
    GenericError,

    #[msg("invalid input")]
    InvalidInput,

    #[msg("failed to perform some math operation safely")]
    MathError,
}
