use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{ Mint, Token },
};
use crate::{
    inputs::{
        CreateTrancheConfigInput,
        Input
    },
    state::{
        TrancheConfig
    }
};

#[derive(Accounts)]
#[instruction(input_data: CreateTrancheConfigInput, tranche_config_bump: u8, senior_tranche_mint_bump: u8, junior_tranche_mint_bump: u8)]
pub struct CreateTranchesContext<'info> {
    /**
     * Signer account
     */
    #[account(mut)]
    pub authority: Signer<'info>,

    /**
     * Tranche config account, where all the parameters are saved
     */
    #[account(
        init,
        payer = authority,
        seeds = [mint.key().as_ref(), senior_tranche_mint.key().as_ref(), junior_tranche_mint.key().as_ref()],
        // bump = tranche_config_bump,
        bump,
        space = TrancheConfig::LEN)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /**
     * mint token to deposit
     */
    #[account()]
    pub mint: Box<Account<'info, Mint>>,

    // * * * * * * * * * * * * * * * * *

    // Senior tranche mint
    #[account(
        init,
        seeds = [vyper_utils::constants::SENIOR.as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        // bump = senior_tranche_mint_bump,
        bump,
        payer = authority,
        mint::decimals = 0,
        mint::authority = tranche_config,
        mint::freeze_authority = tranche_config
    )]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche mint
    #[account(init,
        seeds = [vyper_utils::constants::JUNIOR.as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        // bump = junior_tranche_mint_bump,
        bump,
        payer = authority,
        mint::decimals = 0,
        mint::authority = tranche_config,
        mint::freeze_authority = tranche_config
    )]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // * * * * * * * * * * * * * * * * *
    /// CHECK: Safe
    pub protocol_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(
    ctx: Context<CreateTranchesContext>,
    input_data: CreateTrancheConfigInput,
    tranche_config_bump: u8,
    senior_tranche_mint_bump: u8,
    junior_tranche_mint_bump: u8,
) -> ProgramResult {
    msg!("create_tranche begin");

    // * * * * * * * * * * * * * * * * * * * * * * *
    // check input

    msg!("check if input is valid");
    input_data.is_valid()?;

    // * * * * * * * * * * * * * * * * * * * * * * *
    // create tranche config account

    msg!("create tranche config");
    input_data.create_tranche_config(&mut ctx.accounts.tranche_config);
    ctx.accounts.tranche_config.authority = ctx.accounts.authority.key();
    ctx.accounts.tranche_config.protocol_program_id = ctx.accounts.protocol_program.key();
    ctx.accounts.tranche_config.senior_tranche_mint = ctx.accounts.senior_tranche_mint.key();
    ctx.accounts.tranche_config.junior_tranche_mint = ctx.accounts.junior_tranche_mint.key();
    ctx.accounts.tranche_config.tranche_config_bump = tranche_config_bump;
    ctx.accounts.tranche_config.senior_tranche_mint_bump = senior_tranche_mint_bump;
    ctx.accounts.tranche_config.junior_tranche_mint_bump = junior_tranche_mint_bump;

    // * * * * * * * * * * * * * * * * * * * * * * *

    Ok(())
}