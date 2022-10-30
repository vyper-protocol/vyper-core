pub mod errors;

use crate::errors::RateMockErrorCode;

use anchor_lang::prelude::*;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

solana_security_txt::security_txt! {
    name: "Rate Mock | Vyper Core",
    project_url: "https://vyperprotocol.io",
    contacts: "email:info@vyperprotocol.io,link:https://docs.vyperprotocol.io/",
    policy: "https://github.com/vyper-protocol/vyper-core/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/vyper-protocol/vyper-core/tree/main/programs/rate-mock"
}

declare_id!("FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG");

#[program]
pub mod rate_mock {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        msg!("rate-mock: initialize");

        let clock = Clock::get()?;
        let rate_data = &mut ctx.accounts.rate_data;
        rate_data.fair_value = [dec!(1).serialize(); 10];
        rate_data.refreshed_slot = clock.slot;
        rate_data.authority = ctx.accounts.authority.key();

        msg!("rate_data.fair_value: {:?}", rate_data.fair_value);
        msg!("rate_data.refreshed_slot: {}", rate_data.refreshed_slot);
        msg!("rate_data.authority: {}", rate_data.authority);

        Ok(())
    }

    pub fn set_fair_value(ctx: Context<SetFairValueContext>, fair_value: f64) -> Result<()> {
        msg!("rate-mock: set_fair_value");

        let clock = Clock::get()?;
        let rate_data = &mut ctx.accounts.rate_data;
        rate_data.fair_value[0] = Decimal::from_f64(fair_value)
            .ok_or(RateMockErrorCode::MathError)?
            .serialize();
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

    /// CHECK:
    #[account()]
    pub authority: AccountInfo<'info>,

    #[account(init, payer = signer, space = RateState::LEN)]
    pub rate_data: Account<'info, RateState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetFairValueContext<'info> {
    /// Signer account
    #[account()]
    pub authority: Signer<'info>,

    /// CHECK:
    #[account(mut, has_one = authority)]
    pub rate_data: Account<'info, RateState>,
}

#[derive(Accounts)]
pub struct RefreshRateContext<'info> {
    /// Signer account
    #[account()]
    pub authority: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub rate_data: Account<'info, RateState>,
}

#[account]
pub struct RateState {
    pub fair_value: [[u8; 16]; 10],
    pub refreshed_slot: u64,
    pub authority: Pubkey,
}

impl RateState {
    pub const LEN: usize = 8 + // discriminator
    16*10 + // pub fair_value: [[u8; 16]; 10],
    8 + // pub refreshed_slot: u64,
    32 // pub authority: Pubkey,
    ;
}
