pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use instructions::*;

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
