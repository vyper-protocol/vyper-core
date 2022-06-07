use anchor_lang::prelude::*;
use crate::{state::TrancheConfig};

#[derive(Accounts)]
pub struct RefreshTrancheFairValue<'info> {
    
    pub signer: Signer<'info>,
    
    #[account(mut)]
    pub tranche_config: Account<'info, TrancheConfig>,

    /// TODO check if rate program is executable and it implements the right interface
    /// CHECK: 
    pub redeem_logic_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<RefreshTrancheFairValue>) -> Result<()> {
    msg!("refresh_deposited_value begin");

    // todo check if refresh reserve is owner restricted

    let clock = Clock::get()?;
    let tranche_data = &mut ctx.accounts.tranche_config.tranche_data;

    // TODO retrieve exchange rate from redeem_logic_program

    tranche_data.reserve_fair_value.slot_tracking.update(clock.slot);
    tranche_data.tranche_fair_value.slot_tracking.update(clock.slot);

    Ok(())
}