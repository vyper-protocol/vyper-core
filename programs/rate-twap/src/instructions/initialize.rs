use crate::state::{RateState, SamplingData};
use anchor_lang::prelude::*;
use rust_decimal::Decimal;

#[derive(Accounts)]
#[instruction(input_data: InitializeInput)]
pub struct InitializeContext<'info> {
    /// rate state data account
    #[account(init, payer = payer, space = RateState::len(input_data.sampling_size.try_into().unwrap()))]
    pub rate_state: Account<'info, RateState>,

    /// CHECK: source of rate values
    pub rate_state_source: AccountInfo<'info>,

    /// payer account
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug)]
pub struct InitializeInput {
    /// delta between two samples slot
    /// this is useful to avoid sampling burst
    pub min_slot_delta: u64,

    /// numbero of samples to keep in storage, on those samples will be computed the avg
    pub sampling_size: u32,
}

pub fn handler(ctx: Context<InitializeContext>, input_data: InitializeInput) -> Result<()> {
    let rate_state = &mut ctx.accounts.rate_state;

    rate_state.rate_state_source = ctx.accounts.rate_state_source.key();
    rate_state.sampling_data =
        SamplingData::new(input_data.min_slot_delta, input_data.sampling_size)?;

    let account_data = ctx.accounts.rate_state_source.try_borrow_data()?;
    let mut account_data_slice: &[u8] = &account_data;
    let rate_state_source = CommonRateState::try_deserialize_unchecked(&mut account_data_slice)?;

    rate_state.sampling_data.try_add(
        rate_state_source
            .fair_value
            .map(|c| Decimal::deserialize(c)),
        rate_state_source.refreshed_slot,
    )?;

    rate_state.compute_twap()?;

    msg!("sampling_data: {:?}", rate_state.sampling_data);

    Ok(())
}

#[account]
pub struct CommonRateState {
    pub fair_value: [[u8; 16]; 10],
    pub refreshed_slot: u64,
}
