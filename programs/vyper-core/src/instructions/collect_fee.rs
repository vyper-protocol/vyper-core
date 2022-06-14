use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::{state::TrancheConfig};

#[derive(Accounts)]
pub struct CollectFeeContext<'info> {
    
    /// Tranche config account, where all the parameters are saved
    #[account(mut, has_one = owner, has_one = tranche_authority)]
    pub tranche_config: Account<'info, TrancheConfig>,

    /// CHECK: 
    #[account(seeds = [tranche_config.key().as_ref(), b"authority".as_ref()], bump)]
    pub tranche_authority: AccountInfo<'info>,

    /// tranche reserve vault
    #[account(mut)]
    pub reserve: Account<'info, TokenAccount>,

    /// tranche reserve vault
    #[account(mut)]
    pub dest_reserve: Account<'info, TokenAccount>,

    pub owner: Signer<'info>,
}

pub fn handler(
    ctx: Context<CollectFeeContext>,
) -> Result<()> {

    // let tranche_data = &mut ctx.accounts.tranche_config.tranche_data;

    // if tranche_data.fee_to_collect_quantity > 0 {

    // } else {
    //     msg!("no fee to collect");
    // }

    Ok(())
}