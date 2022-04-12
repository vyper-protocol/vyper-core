use crate::state::TrancheConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct UpdateTrancheConfigContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = tranche_config.authority == *authority.key)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    pub system_program: Program<'info, System>,
}

pub fn handler_update_interest_split(
    ctx: Context<UpdateTrancheConfigContext>,
    interest_split: [u32; 2],
) -> ProgramResult {
    msg!("update_interest_split begin");

    for (i, x) in ctx
        .accounts
        .tranche_config
        .interest_split
        .iter()
        .enumerate()
    {
        msg!("+ old_interest_split[{}]: {}", i, x);
    }

    ctx.accounts.tranche_config.interest_split = interest_split;

    for (i, x) in ctx
        .accounts
        .tranche_config
        .interest_split
        .iter()
        .enumerate()
    {
        msg!("+ new_interest_split[{}]: {}", i, x);
    }

    Ok(())
}

pub fn handler_update_capital_split(
    ctx: Context<UpdateTrancheConfigContext>,
    capital_split: [u32; 2],
) -> ProgramResult {
    msg!("update_capital_split begin");

    for (i, x) in ctx.accounts.tranche_config.capital_split.iter().enumerate() {
        msg!("+ old_capital_split[{}]: {}", i, x);
    }

    ctx.accounts.tranche_config.capital_split = capital_split;

    for (i, x) in ctx.accounts.tranche_config.capital_split.iter().enumerate() {
        msg!("+ new_capital_split[{}]: {}", i, x);
    }

    Ok(())
}
