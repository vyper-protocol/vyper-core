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
    
    /// Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Tranche config account, where all the parameters are saved
    #[account(mut)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// mint token to deposit
    #[account()]
    pub reserve_token: Box<Account<'info, Mint>>,

    /// deposit from
    #[account(mut)]
    pub source_liquidity: Box<Account<'info, TokenAccount>>,

    /// protocol vault
    #[account(mut)]
    pub deposit_to_protocol_reserve: Box<Account<'info, TokenAccount>>,

    // Token account for receiving collateral token (as proof of deposit)
    #[account(mut)]
    pub collateral_token_account: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    /// CHECK: Safe
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    /// CHECK: Safe
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    /// CHECK: Safe
    pub lending_market_authority: AccountInfo<'info>,
    
    /// CHECK: Safe
    pub pyth_reserve_liquidity_oracle: AccountInfo<'info>,

    /// CHECK: Safe
    pub switchboard_reserve_liquidity_oracle: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *

    // Senior tranche mint
    #[account(mut, seeds = [vyper_utils::constants::SENIOR.as_ref(), proxy_protocol_program.key().as_ref(), reserve_token.key().as_ref()], bump = tranche_config.senior_tranche_mint_bump)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(mut)]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(mut, seeds = [vyper_utils::constants::JUNIOR.as_ref(), proxy_protocol_program.key().as_ref(), reserve_token.key().as_ref()], bump = tranche_config.junior_tranche_mint_bump)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche token account
    #[account(mut)]
    pub junior_tranche_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Safe
    #[account(constraint = tranche_config.proxy_protocol_program_id == *proxy_protocol_program.key)]
    pub proxy_protocol_program: AccountInfo<'info>,
    
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
    quantity: u64,
    mint_count: [u64; 2],
) -> ProgramResult {
    msg!("deposit begin");

    msg!("+ quantity: {}", quantity);
    for i in 0..mint_count.len() {
        msg!("+ mint_count[{}]: {}", i, mint_count[i]);
    }

    // deposit on final protocol

    msg!("deposit tokens to protocol");

    let cpi_ctx = CpiContext::new(
        ctx.accounts.proxy_protocol_program.to_account_info(),
        DepositProxyLendingContext {
            authority: ctx.accounts.authority.clone(),
            protocol_program: ctx.accounts.protocol_program.clone(),

            source_liquidity: ctx.accounts.source_liquidity.clone(),
            deposit_to_protocol_reserve: ctx.accounts.deposit_to_protocol_reserve.clone(),
            reserve_token: ctx.accounts.reserve_token.clone(),

            collateral_token_account: ctx.accounts.collateral_token_account.clone(),
            collateral_mint: ctx.accounts.collateral_mint.clone(),

            protocol_state: ctx.accounts.protocol_state.clone(),

            lending_market_account: ctx.accounts.lending_market_account.clone(),
            lending_market_authority: ctx.accounts.lending_market_authority.clone(),
            pyth_reserve_liquidity_oracle: ctx.accounts.pyth_reserve_liquidity_oracle.clone(),
            switchboard_reserve_liquidity_oracle: ctx.accounts.switchboard_reserve_liquidity_oracle.clone(),

            system_program: ctx.accounts.system_program.clone(),
            token_program: ctx.accounts.token_program.clone(),
            associated_token_program: ctx.accounts.associated_token_program.clone(),
            rent: ctx.accounts.rent.clone(),
            clock: ctx.accounts.clock.clone(),
        },
    );
    deposit_vyper_proxy_lending::deposit_to_proxy(cpi_ctx, quantity)?;
 
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

    let mint_key = ctx.accounts.reserve_token.clone().key();
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

    Ok(())
}

#[interface]
pub trait DepositVyperProxyLending<'info, T: Accounts<'info>> {
    fn deposit_to_proxy(ctx: Context<T>, amount: u64) -> ProgramResult;
}