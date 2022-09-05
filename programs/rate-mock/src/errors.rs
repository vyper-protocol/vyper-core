use anchor_lang::prelude::*;

#[error_code]
pub enum RateMockErrorCode {
    #[msg("generic error")]
    GenericError,

    #[msg("math error")]
    MathError,
}
