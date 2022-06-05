use anchor_lang::prelude::*;
use crate::{state::TrancheConfig};

#[derive(Accounts)]
pub struct RefreshDepositedValueContext<'info> {
    
    pub signer: Signer<'info>,
    
    #[account(mut, has_one = rate_program, has_one = rate_program_state)]
    pub tranche_config: Account<'info, TrancheConfig>,

    /// TODO check if rate program is executable and it implements the right interface
    /// CHECK: 
    pub rate_program: AccountInfo<'info>,
    
    /// CHECK: 
    pub rate_program_state: AccountInfo<'info>,
}

pub fn handler(_ctx: Context<RefreshDepositedValueContext>) -> Result<()> {
    msg!("refresh_deposited_value begin");

    // todo check if refresh reserve is owner restricted

    // todo check if in the same transaction we have the refresh istruction for rate_program

    // todo retrieve exchange rate from rate_program

    Ok(())
}