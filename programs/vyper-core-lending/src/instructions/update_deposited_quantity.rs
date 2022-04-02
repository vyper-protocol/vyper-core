use vyper_utils::math::{
    from_bps,
    get_quantites_with_capital_split
};
use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    token::{ TokenAccount },
};
use crate::{
    state::{
        TrancheConfig
    },
};

#[derive(Accounts)]
pub struct UpdateDepositedQuantityContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = tranche_config.authority == *authority.key)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    #[account(mut)]
    pub protocol_vault: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<UpdateDepositedQuantityContext>,
) -> ProgramResult {

    // crystalize interested (sum all accrued interests in deposited_quantity)
    let quantities = get_quantites_with_capital_split(ctx.accounts.protocol_vault.amount, ctx.accounts.tranche_config.capital_split.map(|x| from_bps(x)));

    for i in 0..quantities.len() {
        ctx.accounts.tranche_config.deposited_quantiy[i] = quantities[i];
    }

    Ok(())
}