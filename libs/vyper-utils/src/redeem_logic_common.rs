use anchor_lang::prelude::*;

use crate::decimal_wrapper::DecimalWrapper;

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value: [DecimalWrapper; 10],
    pub new_reserve_fair_value: [DecimalWrapper; 10],
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteResult {
    pub new_quantity: [u64; 2],
    pub fee_quantity: u64,
}

#[error_code]
pub enum RedeemLogicErrors {
    #[msg("generic error")]
    GenericError,

    #[msg("invalid input")]
    InvalidInput,

    #[msg("failed to perform some math operation safely")]
    MathError,
}
