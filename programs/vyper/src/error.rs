use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("generic error")]
    GenericError,
    
    #[msg("invalid input")]
    InvalidInput,
    
    #[msg("invalid tranche amount")]
    InvalidTrancheAmount
}