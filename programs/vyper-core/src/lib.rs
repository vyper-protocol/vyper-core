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
    name: "Vyper Core",
    project_url: "https://vyperprotocol.io",
    contacts: "email:info@vyperprotocol.io,link:https://docs.vyperprotocol.io/",
    policy: "https://github.com/vyper-protocol/vyper-core/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/vyper-protocol/vyper-core/tree/main/programs/vyper-core"
}

declare_id!("vyPErCcGJKQQBeeQ59gXcWrDyU4vBrq8qQfacwmsAsp");

#[program]
pub mod vyper_core {
    use super::*;

    #[log_wrap_ix()]
    pub fn initialize(ctx: Context<InitializeContext>, input_data: InitializeInput) -> Result<()> {
        instructions::initialize::handler(ctx, input_data)
    }

    #[log_wrap_ix()]
    pub fn update_tranche_data(
        ctx: Context<UpdateTrancheDataContext>,
        input_data: UpdateTrancheDataInput,
    ) -> Result<()> {
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
