pub mod error;
pub mod inputs;
pub mod interface_context;
pub mod state;
pub mod instructions;

use anchor_lang::prelude::*;
use inputs::{CreateTrancheConfigInput};
use instructions::*;

declare_id!("5UZpLufUpmnSXor6hgsGyPRMaGS3DsTUYaBZVLX1Jmzc");

#[program]
pub mod vyper_core_lending {

use super::*;

    pub fn create_tranche(
        ctx: Context<CreateTranchesContext>,
        input_data: CreateTrancheConfigInput,
        tranche_config_bump: u8,
        senior_tranche_mint_bump: u8,
        junior_tranche_mint_bump: u8,
    ) -> ProgramResult {
        instructions::create_tranche::handler(ctx, input_data, tranche_config_bump, senior_tranche_mint_bump, junior_tranche_mint_bump)
    }

    pub fn deposit(
        ctx: Context<DepositContext>,
        vault_authority_bump: u8,
        quantity: u64,
        mint_count: [u64; 2],
    ) -> ProgramResult {
        instructions::deposit::handler(ctx, vault_authority_bump, quantity, mint_count)
    }

    pub fn update_interest_split(
        ctx: Context<UpdateTrancheConfigContext>,
        interest_split: [u32; 2],
    ) -> ProgramResult {
        instructions::update_tranche_config::handler_update_interest_split(ctx, interest_split)
    }

    pub fn update_capital_split(
        ctx: Context<UpdateTrancheConfigContext>,
        capital_split: [u32; 2],
    ) -> ProgramResult {
        instructions::update_tranche_config::handler_update_capital_split(ctx, capital_split)
    }

    pub fn update_deposited_quantity(
        ctx: Context<UpdateDepositedQuantityContext>,
    ) -> ProgramResult {
        instructions::update_deposited_quantity::handler(ctx)
    }

    pub fn create_serum_market(
        ctx: Context<CreateSerumMarketContext>,
        vault_signer_nonce: u8,
    ) -> ProgramResult {
        instructions::create_serum_market::handler(ctx, vault_signer_nonce)
    }

    pub fn redeem(ctx: Context<RedeemContext>, vault_authority_bump: u8, redeem_quantity: [u64; 2]) -> ProgramResult {
        instructions::redeem::handler(ctx, vault_authority_bump, redeem_quantity)
    }
}

