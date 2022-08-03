pub mod errors;

pub mod decimal_wrapper;

use crate::decimal_wrapper::DecimalWrapper;
use crate::errors::RateSwitchboardErrorCode;

use anchor_lang::prelude::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use switchboard_v2::{AggregatorAccountData, SWITCHBOARD_V2_DEVNET, SWITCHBOARD_V2_MAINNET};
declare_id!("2hGXiH1oEQwjCXRx8bNdHTi49ScZp7Mj2bxcjxtULKe1");

#[program]
pub mod rate_switchboard {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
        let aggregators = ctx.remaining_accounts;

        // check the correct number of provided aggregators
        if aggregators.len() <= 0 || aggregators.len() > 10 {
            return Err(error!(RateSwitchboardErrorCode::InvalidAggregatorsNumber));
        }

        // check that all the switchboard aggregators have the correct owner
        for aggr in ctx.remaining_accounts {
            let owner = *aggr.owner;
            if owner != SWITCHBOARD_V2_DEVNET && owner != SWITCHBOARD_V2_MAINNET {
                return Err(error!(RateSwitchboardErrorCode::InvalidAggregatorOwner));
            }
        }

        // build the rate data state
        let rate_data = &mut ctx.accounts.rate_data;
        rate_data.fair_value = [DecimalWrapper::new(Decimal::ZERO); 10];
        rate_data.switchboard_aggregators = [None; 10];
        for (i, aggr) in aggregators.iter().enumerate() {
            rate_data.switchboard_aggregators[i] = Some(aggr.key());
        }

        set_data_from_aggregators(rate_data, aggregators)?;

        Ok(())
    }

    pub fn refresh(ctx: Context<RefreshRateContext>) -> Result<()> {
        let aggregators = ctx.remaining_accounts;
        let rate_data = &mut ctx.accounts.rate_data;

        // check that the number of aggregators provided is eq to the one saved during init
        require_eq!(
            aggregators.len(),
            rate_data
                .switchboard_aggregators
                .iter()
                .filter(|&n| n.is_some())
                .count(),
            RateSwitchboardErrorCode::InvalidAggregatorsNumber
        );

        set_data_from_aggregators(rate_data, aggregators)?;

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
    pub fair_value: [DecimalWrapper; 10],
    pub refreshed_slot: u64,
    pub switchboard_aggregators: [Option<Pubkey>; 10],
}

impl RateState {
    pub const LEN: usize = 8 + // discriminator
    16*10 +     // pub fair_value: [DecimalWrapper; 10],
    8 +         // pub refreshed_slot: u64,
    10*(1+32)   // pub switchboard_aggregators: [Option<Pubkey>; 10],
    ;
}

fn set_data_from_aggregators(
    rate_data: &mut Account<RateState>,
    aggregators: &[AccountInfo],
) -> Result<()> {
    // read the aggregators data
    let mut oldest_slot: Option<u64> = None;
    for (i, aggr) in aggregators.iter().enumerate() {
        if let Some(serialized_aggregator) = rate_data.switchboard_aggregators[i] {
            // check that the provided aggregator is the correct one
            require_keys_eq!(serialized_aggregator, aggr.key());

            // load and deserialize feed
            let feed = AggregatorAccountData::new(aggr)?;
            let latest_confirmed_round = feed.latest_confirmed_round;
            let val: f64 = latest_confirmed_round.result.try_into()?;

            if oldest_slot.is_none() || oldest_slot > Some(latest_confirmed_round.round_open_slot) {
                oldest_slot = Some(latest_confirmed_round.round_open_slot);
            }

            match std::str::from_utf8(&feed.name) {
                Ok(feed_name) => msg!("switchboard aggregator {}", feed_name),
                _ => msg!("switchboard aggregator"),
            };
            msg!("+ val: {}", val,);
            let latest_confirmed_round_slot = latest_confirmed_round.round_open_slot;
            msg!("+ confirmed_round_slot: {}", latest_confirmed_round_slot);

            rate_data.fair_value[i]
                .set(Decimal::from_f64(val).ok_or(RateSwitchboardErrorCode::MathError)?);
        } else {
            return Err(error!(RateSwitchboardErrorCode::InvalidAggregatorsNumber));
        }
    }

    rate_data.refreshed_slot = oldest_slot.ok_or(RateSwitchboardErrorCode::GenericError)?;

    msg!("rate_data.fair_value {:?}", rate_data.fair_value);
    msg!("rate_data.refreshed_slot: {}", rate_data.refreshed_slot);

    Ok(())
}
