use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token, TokenAccount}};
use crate::{state::{TrancheConfig, TrancheData}};

#[derive(Accounts)]
#[instruction(_input_data: InitializeInput)]
pub struct InitializeContext<'info> {
    
    /// Signer account
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Owner of the tranche config
    #[account()]
    pub owner: AccountInfo<'info>,

    /// Tranche config account, where all the parameters are saved
    #[account(init, payer = payer, space = TrancheConfig::LEN)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// CHECK: 
    #[account(seeds = [tranche_config.key().as_ref(), b"authority".as_ref()], bump)]
    pub tranche_authority: AccountInfo<'info>,

    /// TODO check if rate program is executable and it implements the right interface
    /// CHECK: 
    #[account()]
    pub rate_program: AccountInfo<'info>,

    /// TODO check if rate program is executable and it implements the right interface
    /// CHECK: 
    #[account()]
    pub rate_program_state: AccountInfo<'info>,
    
    /// TODO check if rate program is executable and it implements the right interface
    /// CHECK: 
    #[account()]
    pub redeem_logic_program: AccountInfo<'info>,

    /// TODO check if rate program is executable and it implements the right interface
    /// CHECK: 
    #[account()]
    pub redeem_logic_program_state: AccountInfo<'info>,

    /// LP mint token to deposit
    #[account()]
    pub reserve_mint: Box<Account<'info, Mint>>,

    /// Token account for vault reserve tokens
    #[account(init, payer = payer, seeds = [tranche_config.key().as_ref(), reserve_mint.key().as_ref()], bump, token::authority = tranche_authority, token::mint = reserve_mint)]
    pub reserve: Box<Account<'info, TokenAccount>>,

    /// Senior tranche mint
    #[account(init, payer = payer, mint::decimals = _input_data.tranche_mint_decimals, mint::authority = tranche_authority)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    /// Junior tranche mint
    #[account(init, payer = payer, mint::decimals = _input_data.tranche_mint_decimals, mint::authority = tranche_authority)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug)]
pub struct InitializeInput {
    pub tranche_mint_decimals: u8
}

pub fn handler(
    ctx: Context<InitializeContext>,
    _input_data: InitializeInput,
) -> Result<()> {

    let clock = Clock::get()?;

    // create tranche config account

    msg!("create tranche config");
    let tranche_config = &mut ctx.accounts.tranche_config;

    tranche_config.version = get_version_arr();
    tranche_config.owner = ctx.accounts.owner.key();
    tranche_config.tranche_data = TrancheData::new(clock.slot);
    tranche_config.tranche_authority = ctx.accounts.tranche_authority.key();
    tranche_config.authority_seed = tranche_config.key();
    tranche_config.authority_bump = [*ctx.bumps.get("tranche_authority").unwrap()];
    tranche_config.reserve_mint = ctx.accounts.reserve_mint.key();
    tranche_config.reserve = ctx.accounts.reserve.key();
    tranche_config.rate_program = ctx.accounts.rate_program.key();
    tranche_config.rate_program_state = ctx.accounts.rate_program_state.key();
    tranche_config.redeem_logic_program = ctx.accounts.redeem_logic_program.key();
    tranche_config.redeem_logic_program_state = ctx.accounts.redeem_logic_program_state.key();
    tranche_config.senior_tranche_mint = ctx.accounts.senior_tranche_mint.key();
    tranche_config.junior_tranche_mint = ctx.accounts.junior_tranche_mint.key();
    tranche_config.created_at = clock.unix_timestamp;

    Ok(())
}

fn get_version_arr() -> [u8; 3] {
    [
        env!("CARGO_PKG_VERSION_MAJOR")
            .parse::<u8>()
            .expect("failed to parse major version"),
        env!("CARGO_PKG_VERSION_MINOR")
            .parse::<u8>()
            .expect("failed to parse minor version"),
        env!("CARGO_PKG_VERSION_PATCH")
            .parse::<u8>()
            .expect("failed to parse patch version"),
    ]
}
