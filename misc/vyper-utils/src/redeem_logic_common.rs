use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_tranche_fair_value: [u64; 2],
    pub old_reserve_fair_value: u64,
    pub new_reserve_fair_value: u64
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteResult {
    pub new_tranche_fairvalue: [u64;2]
}