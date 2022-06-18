use anchor_lang::prelude::*;

declare_id!("FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG");

#[program]
pub mod rate_mock {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        msg!("rate-mock: initialize");

        let clock = Clock::get()?;
        let rate_data = &mut ctx.accounts.rate_data;
        rate_data.fair_value = [0; 10];
        rate_data.refreshed_slot = clock.slot;

        msg!("rate_data.fair_value: {:?}", rate_data.fair_value);
        msg!("rate_data.refreshed_slot: {}", rate_data.refreshed_slot);

        Ok(())
    }

    pub fn set_random_fair_value(ctx: Context<RefreshRateContext>) -> Result<()> {
        // random rate
        let clock = Clock::get()?;
        ctx.accounts.rate_data.fair_value[0] =
            clock.unix_timestamp.checked_rem(10000).unwrap() as u32;
        ctx.accounts.rate_data.refreshed_slot = clock.slot;

        Ok(())
    }

    pub fn set_fair_value(ctx: Context<RefreshRateContext>, fair_value: u32) -> Result<()> {
        msg!("rate-mock: set_fair_value");

        let clock = Clock::get()?;
        let rate_data = &mut ctx.accounts.rate_data;
        rate_data.fair_value[0] = fair_value;
        rate_data.refreshed_slot = clock.slot;

        msg!("rate_data.fair_value: {:?}", rate_data.fair_value);
        msg!("rate_data.refreshed_slot: {}", rate_data.refreshed_slot);

        Ok(())
    }

    pub fn refresh(ctx: Context<RefreshRateContext>) -> Result<()> {
        msg!("rate-mock: refresh");

        let clock = Clock::get()?;
        let rate_data = &mut ctx.accounts.rate_data;
        rate_data.refreshed_slot = clock.slot;

        msg!("rate_data.refreshed_slot: {}", rate_data.refreshed_slot);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    /// Signer account
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = 8+4*10+8)]
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
    pub fair_value: [u32; 10],
    pub refreshed_slot: u64,
}
