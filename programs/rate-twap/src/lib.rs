pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use instructions::*;

solana_security_txt::security_txt! {
    name: "Rate TWAP | Vyper Core",
    project_url: "https://vyperprotocol.io",
    contacts: "email:info@vyperprotocol.io,link:https://docs.vyperprotocol.io/",
    policy: "https://github.com/vyper-protocol/vyper-core/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/vyper-protocol/vyper-core/tree/main/programs/rate-twap"
}

declare_id!("8szo8C2w8rvtn1X7sux5PmxeErG5t2fxcV2Utugay4ct");

#[program]
pub mod rate_twap {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>, input_data: InitializeInput) -> Result<()> {
        instructions::initialize::handler(ctx, input_data)
    }

    pub fn refresh(ctx: Context<RefreshRateContext>) -> Result<()> {
        instructions::refresh::handler(ctx)
    }
}
