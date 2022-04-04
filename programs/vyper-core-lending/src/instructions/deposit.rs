use vyper_utils::math::{
    from_bps,
    get_quantites_with_capital_split
};
use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{ self, Mint, Token, TokenAccount },
};
use crate::{
    state::{
        TrancheConfig
    },
    interface_context::{
        DepositProxyLendingContext
    }
};

#[derive(Accounts)]
#[instruction(vault_authority_bump: u8)]
pub struct DepositContext<'info> {
    /**
     * Signer account
     */
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Tranche config account, where all the parameters are saved
    #[account(mut)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// mint token to deposit
    #[account()]
    pub mint: Box<Account<'info, Mint>>,

    /// deposit from
    #[account(
        mut,
        // associated_token::mint = mint,
        // associated_token::authority = authority
    )]
    pub deposit_source_account: Box<Account<'info, TokenAccount>>,

    /// protocol vault (SOLEND RESERVE https://docs.solend.fi/developers/addresses/devnet#reserves )
    #[account(mut)]
    pub protocol_vault: Box<Account<'info, TokenAccount>>,

    // proxy stuff
    // Vyper Vault authority
    /// CHECK: Safe
    #[account(mut, seeds = [b"vault_authority".as_ref(), tranche_config.key().as_ref()], bump = vault_authority_bump)]
    pub vault_authority: AccountInfo<'info>,

    // Token account for receiving collateral token (as proof of deposit)
    // TODO: init_if_needed
    #[account(mut, /* associated_token::mint = collateral_mint, associated_token::authority = vault_authority */)]
    pub collateral_token_account: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    // State account for protocol (reserve-state-account)
    /// CHECK: Safe
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    // Lending market account (https://docs.solend.fi/developers/addresses/devnet#devnet)
    /// CHECK: Safe
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    /// CHECK: Safe
    pub lending_market_authority: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *

    // Senior tranche mint
    #[account(
        mut,
        seeds = [vyper_utils::constants::SENIOR.as_ref(), proxy_protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = tranche_config.senior_tranche_mint_bump
    )]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(mut, /* associated_token::mint = senior_tranche_mint, associated_token::authority = authority */)]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(
        mut,
        seeds = [vyper_utils::constants::JUNIOR.as_ref(), proxy_protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = tranche_config.junior_tranche_mint_bump
    )]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche token account
    #[account(mut, /* associated_token::mint = junior_tranche_mint, associated_token::authority = authority */)]
    pub junior_tranche_vault: Box<Account<'info, TokenAccount>>,

    
    /// CHECK: Safe
    #[account(constraint = tranche_config.proxy_protocol_program_id == *proxy_protocol_program.key)]
    pub proxy_protocol_program: AccountInfo<'info>,
    
    // pe solend: ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx
    /// CHECK: Safe
    #[account()]
    pub protocol_program: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}


pub fn handler(
    ctx: Context<DepositContext>,
    vault_authority_bump: u8,
    quantity: u64,
    mint_count: [u64; 2],
) -> ProgramResult {
    msg!("deposit begin");
    for c in ctx.accounts.to_account_infos() {
        msg!("+ ctx.accounts.to_account_infos(): {}", c.key());
    }


    msg!("+ quantity: {}", quantity);
    for i in 0..mint_count.len() {
        msg!("+ mint_count[{}]: {}", i, mint_count[i]);
    }

    // deposit on final protocol

    msg!("deposit tokens to protocol");

    let tranche_config_key = ctx.accounts.tranche_config.key();
    let seeds = &[
        b"vault_authority".as_ref(),
        tranche_config_key.as_ref(),
        &[vault_authority_bump]
    ];
    let signer = &[&seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.proxy_protocol_program.to_account_info(),
        DepositProxyLendingContext {
            authority: ctx.accounts.authority.clone(),
            vault_authority: ctx.accounts.vault_authority.clone(),
            tranche_config: ctx.accounts.tranche_config.to_account_info(),
            protocol_program: ctx.accounts.protocol_program.clone(),

            deposit_from: ctx.accounts.deposit_source_account.clone(),
            deposit_to_protocol_reserve: ctx.accounts.protocol_vault.clone(),
            deposit_mint: ctx.accounts.mint.clone(),

            collateral_token_account: ctx.accounts.collateral_token_account.clone(),
            collateral_mint: ctx.accounts.collateral_mint.clone(),

            protocol_state: ctx.accounts.protocol_state.clone(),

            lending_market_account: ctx.accounts.lending_market_account.clone(),
            lending_market_authority: ctx.accounts.lending_market_authority.clone(),

            system_program: ctx.accounts.system_program.clone(),
            token_program: ctx.accounts.token_program.clone(),
            associated_token_program: ctx.accounts.associated_token_program.clone(),
            rent: ctx.accounts.rent.clone(),
            clock: ctx.accounts.clock.clone(),
        },
        signer
    );

    msg!("authority: {}", ctx.accounts.authority.clone().key());
    msg!("vault_authority: {}", ctx.accounts.vault_authority.clone().key());
    msg!("tranche_config: {}", ctx.accounts.tranche_config.to_account_info().key());
    msg!("protocol_program: {}", ctx.accounts.protocol_program.clone().key());
    msg!("deposit_from: {}", ctx.accounts.deposit_source_account.clone().key());
    msg!("deposit_to_protocol_reserve: {}", ctx.accounts.protocol_vault.clone().key());
    msg!("deposit_mint: {}", ctx.accounts.mint.clone().key());
    msg!("collateral_token_account: {}", ctx.accounts.collateral_token_account.clone().key());
    msg!("collateral_mint: {}", ctx.accounts.collateral_mint.clone().key());
    msg!("protocol_state: {}", ctx.accounts.protocol_state.clone().key());
    msg!("lending_market_account: {}", ctx.accounts.lending_market_account.clone().key());
    msg!("lending_market_authority: {}", ctx.accounts.lending_market_authority.clone().key());
    msg!("system_program: {}", ctx.accounts.system_program.clone().key());
    msg!("token_program: {}", ctx.accounts.token_program.clone().key());
    msg!("associated_token_program: {}", ctx.accounts.associated_token_program.clone().key());
    msg!("rent: {}", ctx.accounts.rent.clone().key());
    msg!("clock: {}", ctx.accounts.clock.clone().key());

    deposit_vyper_proxy_lending::deposit_to_proxy(cpi_ctx, vault_authority_bump, quantity)?;
 
    // * * * * * * * * * * * * * * * * * * * * * * *
    // increase the deposited quantity

    let split_quantities = get_quantites_with_capital_split(quantity, ctx.accounts.tranche_config.capital_split.map(|x| from_bps(x)));
    for i in 0..split_quantities.len() {
        msg!("split_quantities[{}] = {}", i, split_quantities[i]);
        msg!("old deposited quantity [{}] = {}", i, ctx.accounts.tranche_config.deposited_quantiy[i]);
        
        ctx.accounts.tranche_config.deposited_quantiy[i] += split_quantities[i];

        msg!("new deposited quantity [{}] = {}", i, ctx.accounts.tranche_config.deposited_quantiy[i]);
    }

    // * * * * * * * * * * * * * * * * * * * * * * *
    // mint tranche tokens to user

    let mint_key = ctx.accounts.mint.clone().key();
    let senior_tranche_mint_key = ctx.accounts.senior_tranche_mint.clone().key();
    let junior_tranche_mint_key = ctx.accounts.junior_tranche_mint.clone().key();
    let tranche_config_seeds = &[
        mint_key.as_ref(),
        senior_tranche_mint_key.as_ref(),
        junior_tranche_mint_key.as_ref(),
        &[ctx.accounts.tranche_config.tranche_config_bump]
    ];
    let tranche_config_signer = &[&tranche_config_seeds[..]];

    if mint_count[0] > 0 {
        msg!("mint senior tranche");
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.senior_tranche_mint.to_account_info(),
                    to: ctx.accounts.senior_tranche_vault.to_account_info(),
                    authority: ctx.accounts.tranche_config.to_account_info(),
                },
                tranche_config_signer
            ),
            mint_count[0],
        )?;
    }

    if mint_count[1] > 0 {
        msg!("mint junior tranche");
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.junior_tranche_mint.to_account_info(),
                    to: ctx.accounts.junior_tranche_vault.to_account_info(),
                    authority: ctx.accounts.tranche_config.to_account_info(),
                },
                tranche_config_signer
            ),
            mint_count[1],
        )?;
    }

    // * * * * * * * * * * * * * * * * * * * * * * *

    Ok(())
}

#[interface]
pub trait DepositVyperProxyLending<'info, T: Accounts<'info>> {
    fn deposit_to_proxy(ctx: Context<T>, vault_authority_bump: u8, amount: u64) -> ProgramResult;
}