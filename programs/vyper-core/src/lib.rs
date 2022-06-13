pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use instructions::*;
use vyper_macros::*;

use solana_security_txt::security_txt;

security_txt! {
    // Required fields
    name: "vyper core",
    project_url: "https://github.com/vyper-protocol/vyper-core",
    contacts: "email:info@vyperprotocol.io,link:https://docs.vyperprotocol.io/security,discord:uKHfg58j",
    policy: "https://github.com/vyper-protocol/blob/master/SECURITY.md"
}

declare_id!("mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U");

#[program]
pub mod vyper_core {
    use super::*;

    #[log_wrap_ix()]
    pub fn initialize(ctx: Context<InitializeContext>, input_data: InitializeInput) -> Result<()> {
        instructions::initialize::handler(ctx, input_data)
    }
    
    #[log_wrap_ix()]
    pub fn update_tranche_data(ctx: Context<UpdateTrancheDataContext>, input_data: UpdateTrancheDataInput) -> Result<()> {
        instructions::update_tranche_data::handler(ctx, input_data)
    }
    
    #[log_wrap_ix()]
    pub fn refresh_tranche_fair_value(ctx: Context<RefreshTrancheFairValue>) -> Result<()> {
        instructions::refresh_tranche_fair_value::handler(ctx)
    }
    
    #[log_wrap_ix()]
    pub fn deposit(ctx: Context<DepositContext>, input_data: DepositInput) -> Result<()> {
        instructions::deposit::handler(ctx, input_data)
    }
    
    #[log_wrap_ix()]
    pub fn redeem(ctx: Context<RedeemContext>, input_data: RedeemInput) -> Result<()> {
        instructions::redeem::handler(ctx, input_data)
    }

    #[log_wrap_ix()]
    pub fn collect_fee(ctx: Context<CollectFeeContext>) -> Result<()> {
        instructions::collect_fee::handler(ctx)
    }
}