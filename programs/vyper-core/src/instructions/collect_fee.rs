use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Transfer, self, Token};
use crate::{state::TrancheConfig};

#[derive(Accounts)]
pub struct CollectFeeContext<'info> {
    
    /// Tranche config account, where all the parameters are saved
    #[account(mut,
        has_one = owner,
        has_one = tranche_authority,
        has_one = reserve)]
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

    pub token_program: Program<'info, Token>,
}

impl<'info> CollectFeeContext<'info> {
    /// CpiContext for transferring reserve tokens from user to vault
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve.to_account_info(),
                to: self.dest_reserve.to_account_info(),
                authority: self.tranche_authority.to_account_info(),
            },
        )
    }
}

pub fn handler(
    ctx: Context<CollectFeeContext>,
) -> Result<()> {
    
    // TODO calculate tranching rounding and collect in fee the exceding part

    let fee_to_collect_quantity = ctx.accounts.tranche_config.tranche_data.fee_to_collect_quantity;
    msg!("collecting fee: {}", fee_to_collect_quantity);
    token::transfer(ctx.accounts
        .transfer_context()
        .with_signer(&[&ctx.accounts.tranche_config.authority_seeds()]), 
        fee_to_collect_quantity)?;

    ctx.accounts.tranche_config.tranche_data.fee_to_collect_quantity = 0;

    Ok(())
}