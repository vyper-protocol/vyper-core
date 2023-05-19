
use crate::{
    state::{TrancheConfig},
};
use anchor_lang::{prelude::*, AccountsClose};
use anchor_spl::{token::{Mint, Token, TokenAccount, CloseAccount, self, Transfer}};

#[derive(Accounts)]
pub struct CloseContext<'info> {
    /// Signer account
    #[account()]
    pub owner: Signer<'info>,
    
    /// CHECK:   
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,

    #[account(mut, 
        has_one = reserve,
        has_one = senior_tranche_mint,
        has_one = junior_tranche_mint,
        has_one = tranche_authority,
        has_one = owner)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// CHECK:
    #[account(seeds = [tranche_config.key().as_ref(), b"authority".as_ref()], bump)]
    pub tranche_authority: AccountInfo<'info>,


    /// vyper core reserve
    #[account(mut)]
    pub reserve: Account<'info, TokenAccount>,

    #[account(mut, token::mint = tranche_config.reserve_mint)]
    pub user_reserve_token: Account<'info, TokenAccount>,

    /// Senior tranche mint
    #[account(mut)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    /// Junior tranche mint
    #[account(mut)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CloseContext>) -> Result<()> {
    
    // check that senior/junior tranche mint supply is zero
    require_eq!(ctx.accounts.senior_tranche_mint.supply, 0);
    require_eq!(ctx.accounts.junior_tranche_mint.supply, 0);

    if ctx.accounts.reserve.amount != 0 {
        token::transfer(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                authority: ctx.accounts.tranche_authority.to_account_info(),
                from: ctx.accounts.reserve.to_account_info(),
                to: ctx.accounts.user_reserve_token.to_account_info()
            },
            &[&ctx.accounts.tranche_config.authority_seeds()]
        ), ctx.accounts.reserve.amount)?;
    }

    // close reserve token account
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.reserve.to_account_info(),
            authority: ctx.accounts.tranche_authority.to_account_info(),
            destination: ctx.accounts.fee_receiver.to_account_info()
        },
        &[&ctx.accounts.tranche_config.authority_seeds()]
    ))?;
   
    // close tranche configuration
    ctx.accounts.tranche_config.close(ctx.accounts.fee_receiver.to_account_info())?;

    Ok(())
}
