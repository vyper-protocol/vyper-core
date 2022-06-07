use anchor_lang::prelude::*;

declare_id!("9h7eHiqpPbj5Mw5AG59bFH7XDWfHYK52XMfJPbKVas2m");

#[program]
pub mod rate_mock {

    use super::*;

    pub fn initialize(_ctx: Context<InitializeContext>) -> Result<()> {
        Ok(())
    }

    pub fn set_random_fair_value(ctx: Context<RefreshRateContext>) -> Result<()> {

        // random rate
        let clock = Clock::get()?;
        ctx.accounts.rate_data.fair_value = clock.unix_timestamp.checked_rem(10000).unwrap() as u64;
        ctx.accounts.rate_data.refreshed_slot = clock.slot;

        Ok(())
    }

    pub fn set_fair_value(ctx: Context<RefreshRateContext>, fair_value: u64) -> Result<()> {

        let clock = Clock::get()?;
        ctx.accounts.rate_data.fair_value = fair_value;
        ctx.accounts.rate_data.refreshed_slot = clock.slot;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    
    /// Signer account
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = 1024)]
    pub rate_data: Account<'info, RateState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefreshRateContext<'info> {
    /// Signer account
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: 
    #[account(mut)]
    pub rate_data: Account<'info, RateState>,
}

#[account]
pub struct RateState {
    pub fair_value: u64,
    pub refreshed_slot: u64,
}