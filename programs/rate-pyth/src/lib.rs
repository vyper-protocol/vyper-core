pub mod errors;

use crate::errors::RatePythErrorCode;

use anchor_lang::prelude::*;
use pyth_sdk_solana::{load_price_feed_from_account_info, Price, PriceFeed};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

declare_id!("3mxtC2cGVhHucUg4p58MVzVqUKLyiy1zWqRkRQdgUBPT");

#[program]
pub mod rate_pyth {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        let aggregators = ctx.remaining_accounts;

        // check the correct number of provided aggregators
        if aggregators.len() == 0 || aggregators.len() > 10 {
            return Err(error!(RatePythErrorCode::InvalidAggregatorsNumber));
        }

        // TODO check feed owner ?

        // build the rate data state
        let rate_data = &mut ctx.accounts.rate_data;
        rate_data.fair_value = [Decimal::ZERO.serialize(); 10];
        rate_data.pyth_oracles = [None; 10];
        for (i, aggr) in aggregators.iter().enumerate() {
            rate_data.pyth_oracles[i] = Some(aggr.key());
        }

        set_data_from_oracles(rate_data, aggregators)?;

        Ok(())
    }

    pub fn refresh(ctx: Context<RefreshRateContext>) -> Result<()> {
        let aggregators = ctx.remaining_accounts;
        let rate_data = &mut ctx.accounts.rate_data;

        // check that the number of oracles provided is eq to the one saved during init
        require_eq!(
            aggregators.len(),
            rate_data
                .pyth_oracles
                .iter()
                .filter(|&n| n.is_some())
                .count(),
            RatePythErrorCode::InvalidAggregatorsNumber
        );

        set_data_from_oracles(rate_data, aggregators)?;

        let clock = Clock::get()?;
        rate_data.refreshed_slot = clock.slot;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    /// Signer account
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = RateState::LEN)]
    pub rate_data: Account<'info, RateState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RefreshRateContext<'info> {
    /// Signer account
    #[account()]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub rate_data: Account<'info, RateState>,
}

#[account]
pub struct RateState {
    pub fair_value: [[u8; 16]; 10],
    pub refreshed_slot: u64,
    pub pyth_oracles: [Option<Pubkey>; 10],
}

impl RateState {
    pub const LEN: usize = 8 + // discriminator
    16*10 +     // pub fair_value: [[u8; 16]; 10],
    8 +         // pub refreshed_slot: u64,
    10*(1+32)   // pub pyth_oracles: [Option<Pubkey>; 10],
    ;
}

fn set_data_from_oracles(
    rate_data: &mut Account<RateState>,
    oracles: &[AccountInfo],
) -> Result<()> {
    for (i, oracle) in oracles.iter().enumerate() {
        if let Some(serialized_oracle) = rate_data.pyth_oracles[i] {
            // check that the provided aggregator is the correct one
            require_keys_eq!(serialized_oracle, oracle.key());

            // load and deserialize oracle
            let price_feed: PriceFeed = load_price_feed_from_account_info(&oracle).unwrap();
            let current_price: Price = price_feed.get_current_price().unwrap();

            #[cfg(feature = "debug")]
            {
                msg!("+ current pyth feed: {:?}", price_feed);
                msg!("+ current pyth price: {:?}", current_price);
            }

            let current_price_mantissa = Decimal::from(current_price.price);
            let current_price_expo = Decimal::from(current_price.expo);

            let fair_value = current_price_mantissa * dec!(10).powd(current_price_expo);
            msg!("saving fair value {:?}", fair_value);

            rate_data.fair_value[i] = fair_value.serialize();
        } else {
            return Err(error!(RatePythErrorCode::InvalidAggregatorsNumber));
        }
    }

    // save current slot
    rate_data.refreshed_slot = Clock::get()?.slot;

    msg!("rate_data.fair_value {:?}", rate_data.fair_value);
    msg!("rate_data.refreshed_slot: {}", rate_data.refreshed_slot);

    Ok(())
}
