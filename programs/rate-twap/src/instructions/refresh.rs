use anchor_lang::prelude::*;
use rust_decimal::Decimal;

use crate::state::rate_state::RateState;

use super::initialize::CommonRateState;

#[derive(Accounts)]
pub struct RefreshRateContext<'info> {
    /// Signer account
    #[account()]
    pub signer: Signer<'info>,

    #[account(mut, has_one = rate_state_source)]
    pub rate_state: Account<'info, RateState>,

    /// CHECK: source of rate values
    pub rate_state_source: AccountInfo<'info>,
}

pub fn handler(ctx: Context<RefreshRateContext>) -> Result<()> {
    let rate_state = &mut ctx.accounts.rate_state;

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
    msg!(
        "rate_state fair_value: {:?}",
        rate_state.fair_value.map(|c| Decimal::deserialize(c))
    );
    msg!(
        "rate_state refreshed_slot: {:#?}",
        rate_state.refreshed_slot
    );

    Ok(())
}
