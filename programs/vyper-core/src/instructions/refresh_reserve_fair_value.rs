use anchor_lang::prelude::*;
use boolinator::Boolinator;
use crate::{state::{TrancheConfig, TrancheHaltFlags, OwnerRestrictedIxFlags}, errors::VyperErrorCode};

#[derive(Accounts)]
pub struct RefreshReserveFairValue<'info> {
    
    pub signer: Signer<'info>,
    
    #[account(mut, has_one = rate_program_state)]
    pub tranche_config: Account<'info, TrancheConfig>,

    /// CHECK: 
    pub rate_program_state: AccountInfo<'info>
}

impl<'info> RefreshReserveFairValue<'info> {
    fn are_valid(&self) -> Result<()> {

        let tranche_data = &self.tranche_config.tranche_data;

        // check that deposits are not halted
        (!tranche_data.get_halt_flags().contains(TrancheHaltFlags::HALT_REFRESHES)).ok_or(VyperErrorCode::HaltError)?;
    
        // check if the current ix is restricted to owner
        if tranche_data.get_owner_restricted_ixs().contains(OwnerRestrictedIxFlags::REFRESHES) {
            require_keys_eq!(self.tranche_config.owner, self.signer.key(), VyperErrorCode::OwnerRestrictedIx)
        }

        Result::Ok(())
    }
}

pub fn handler(ctx: Context<RefreshReserveFairValue>) -> Result<()> {
    msg!("refresh_deposited_value begin");

    // check if accounts are valid
    msg!("check if accounts are valid");
    ctx.accounts.are_valid()?;

    // retrieve exchange rate from rate_program
    let account_data =  ctx.accounts.rate_program_state.try_borrow_data()?;
    let mut account_data_slice: &[u8] = &account_data;
    let rate_state = RateState::try_deserialize_unchecked(&mut account_data_slice)?;

    let tranche_data = &mut ctx.accounts.tranche_config.tranche_data;
    tranche_data.reserve_fair_value.value = rate_state.fair_value;
    tranche_data.reserve_fair_value.slot_tracking.update(rate_state.refreshed_slot);

    Ok(())
}

#[account]
pub struct RateState {
    pub fair_value: u64,
    pub refreshed_slot: u64,
}