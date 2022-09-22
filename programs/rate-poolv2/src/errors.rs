use anchor_lang::prelude::*;

#[error_code]
pub enum RatePoolv2ErrorCode {
    #[msg("generic error")]
    GenericError,

    #[msg("math error")]
    MathError,
}
