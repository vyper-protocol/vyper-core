pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U");

#[program]
pub mod vyper_core {
    use super::*;

    // #[state]
    // pub struct Counter {
    //     pub count: u64,
    //     pub auth_program: Pubkey,
    // }

    pub fn initialize(ctx: Context<InitializeContext>, input_data: InitializeInput) -> Result<()> {
        instructions::initialize::handler(ctx, input_data)
    }

    pub fn update_tranche_data(ctx: Context<UpdateTrancheDataContext>, input_data: UpdateTrancheDataInput) -> Result<()> {
        instructions::update_tranche_data::handler(ctx, input_data)
    }

    pub fn refresh_deposited_value(ctx: Context<RefreshDepositedValueContext>) -> Result<()> {
        instructions::refresh_reserve_fair_value::handler(ctx)
    }

    // pub fn deposit(ctx: Context<DepositContext>, input_data: DepositInput) -> Result<()> {
    //     instructions::deposit::handler(ctx, input_data)
    // }

    // pub fn redeem(ctx: Context<RedeemContext>, redeem_quantity: [u64; 2]) -> ProgramResult {
    //     instructions::redeem::handler(ctx, redeem_quantity)
    // }


}