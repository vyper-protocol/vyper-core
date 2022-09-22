pub mod errors;
mod state;

use crate::errors::RatePoolv2ErrorCode;
use crate::state::SupplyWrapper;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use rust_decimal::Decimal;

declare_id!("5Vm2YZK3SeGbXbtQpKVByP9EvYy78ahnjFXKkf9B3yzW");

#[program]
pub mod rate_poolv2 {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        let rate_data = &mut ctx.accounts.rate_data;

        rate_data.lp_mint = ctx.accounts.lp_mint.key();
        rate_data.base_mint = ctx.accounts.base_mint.key();
        rate_data.quote_mint = ctx.accounts.quote_mint.key();
        rate_data.base_token_account = ctx.accounts.base_token_account.key();
        rate_data.quote_token_account = ctx.accounts.quote_token_account.key();

        // calculate prices

        let prices: [Decimal; 2] = get_prices(
            SupplyWrapper {
                supply: ctx.accounts.base_token_account.amount,
                decimals: ctx.accounts.base_mint.decimals,
            },
            SupplyWrapper {
                supply: ctx.accounts.quote_token_account.amount,
                decimals: ctx.accounts.quote_mint.decimals,
            },
            SupplyWrapper {
                supply: ctx.accounts.lp_mint.supply,
                decimals: ctx.accounts.lp_mint.decimals,
            },
        )?;

        for (i, dec) in prices.iter().enumerate() {
            rate_data.fair_value[i] = dec.serialize();
        }

        // set refreshed slot

        rate_data.refreshed_slot = Clock::get()?.slot;

        Ok(())
    }

    pub fn refresh(ctx: Context<RefreshRateContext>) -> Result<()> {
        let rate_data = &mut ctx.accounts.rate_data;

        // calculate prices

        let prices: [Decimal; 2] = get_prices(
            SupplyWrapper {
                supply: ctx.accounts.base_token_account.amount,
                decimals: ctx.accounts.base_mint.decimals,
            },
            SupplyWrapper {
                supply: ctx.accounts.quote_token_account.amount,
                decimals: ctx.accounts.quote_mint.decimals,
            },
            SupplyWrapper {
                supply: ctx.accounts.lp_mint.supply,
                decimals: ctx.accounts.lp_mint.decimals,
            },
        )?;

        for (i, dec) in prices.iter().enumerate() {
            rate_data.fair_value[i] = dec.serialize();
        }

        // set refreshed slot

        rate_data.refreshed_slot = Clock::get()?.slot;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    /// Signer account
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = RateState::LEN)]
    pub rate_data: Box<Account<'info, RateState>>,

    /// CHECK: the pool id
    #[account()]
    pub pool: AccountInfo<'info>,

    /// Mint of the lp tokens
    #[account(mint::authority = pool)]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// CHECK: Base mint, for a SOL/USDC pool this is SOL
    #[account()]
    pub base_mint: Box<Account<'info, Mint>>,

    /// CHECK: Quote mint, for a SOL/USDC pool this is USDC
    #[account()]
    pub quote_mint: Box<Account<'info, Mint>>,

    /// Base token account, for a SOL/USDC pool this is the SOL token account
    #[account(token::mint = base_mint, token::authority = pool)]
    pub base_token_account: Box<Account<'info, TokenAccount>>,

    /// Quote token account, for a SOL/USDC pool this is the USDC token account
    #[account(token::mint = quote_mint, token::authority = pool)]
    pub quote_token_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefreshRateContext<'info> {
    #[account(mut,
        has_one = base_mint,
        has_one = quote_mint,
        has_one = lp_mint,
        has_one = base_token_account,
        has_one = quote_token_account)]
    pub rate_data: Account<'info, RateState>,

    /// CHECK: Mint of the lp tokens
    #[account()]
    pub lp_mint: Account<'info, Mint>,

    /// CHECK: Base mint, for a SOL/USDC pool this is SOL
    #[account()]
    pub base_mint: Account<'info, Mint>,

    /// CHECK: Quote mint, for a SOL/USDC pool this is USDC
    #[account()]
    pub quote_mint: Account<'info, Mint>,

    /// Base token account, for a SOL/USDC pool this is the SOL token account
    #[account(token::mint = base_mint)]
    pub base_token_account: Account<'info, TokenAccount>,

    /// Quote token account, for a SOL/USDC pool this is the USDC token account
    #[account(token::mint = quote_mint)]
    pub quote_token_account: Account<'info, TokenAccount>,
}

#[account]
pub struct RateState {
    pub fair_value: [[u8; 16]; 10],
    pub refreshed_slot: u64,

    /// Mint of the lp tokens
    pub lp_mint: Pubkey,

    /// Base mint, for a SOL/USDC pool this is SOL
    pub base_mint: Pubkey,

    /// Quote mint, for a SOL/USDC pool this is USDC
    pub quote_mint: Pubkey,

    /// Base token account, for a SOL/USDC pool this is the SOL token account
    pub base_token_account: Pubkey,

    /// Quote token account, for a SOL/USDC pool this is the USDC token account
    pub quote_token_account: Pubkey,
}

impl RateState {
    pub const LEN: usize = 8 + // discriminator
    16*10 +     // pub fair_value: [[u8; 16]; 10],
    8 +         // pub refreshed_slot: u64,
    32 +        // pub lp_mint: Pubkey,
    32 +        // pub base_mint: Pubkey,
    32 +        // pub quote_mint: Pubkey,
    32 +        // pub base_token_account: Pubkey,
    32          // pub quote_token_account: Pubkey,
    ;
}

fn get_prices(
    base_supply: SupplyWrapper,
    quote_supply: SupplyWrapper,
    lp_supply: SupplyWrapper,
) -> Result<[Decimal; 2]> {
    msg!("base supply: {}", base_supply);
    msg!("quote supply: {}", quote_supply);
    msg!("lp supply: {}", lp_supply);

    // lp_price = quote_supply * 2 / lp_supply
    // base_price = quote_supply / base_supply

    let lp_price = quote_supply
        .to_dec()?
        .checked_mul(Decimal::TWO)
        .ok_or(RatePoolv2ErrorCode::MathError)?
        .checked_div(lp_supply.to_dec()?)
        .ok_or(RatePoolv2ErrorCode::MathError)?;

    let base_price = quote_supply
        .to_dec()?
        .checked_div(base_supply.to_dec()?)
        .ok_or(RatePoolv2ErrorCode::MathError)?;

    msg!("lp price: {}", lp_price);
    msg!("base price: {}", base_price);

    Ok([lp_price, base_price])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test() {
        let sol_supply = 100u64;
        let usdc_supply = 10u64;
        let lp_supply = 1000u64;

        let res = get_prices(
            SupplyWrapper {
                supply: sol_supply,
                decimals: 0,
            },
            SupplyWrapper {
                supply: usdc_supply,
                decimals: 0,
            },
            SupplyWrapper {
                supply: lp_supply,
                decimals: 0,
            },
        )
        .unwrap();

        assert_eq!(res[0], dec!(0.02));
        assert_eq!(res[1], dec!(0.1));
    }
}
