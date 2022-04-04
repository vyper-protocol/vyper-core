use vyper_utils::math::{
    from_bps,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{ Mint, Token, TokenAccount },
};
use std::cmp;
use crate::{
    state::{
        TrancheConfig
    },
error::ErrorCode,
    interface_context::{
        WithdrawProxyLendingContext
    }
};

#[derive(Accounts)]
#[instruction(vault_authority_bump: u8)]
pub struct RedeemContext<'info> {
    /**
     * Signer account
     */
    #[account(mut)]
    pub authority: Signer<'info>,

    /**
     * Tranche config account, where all the parameters are saved
     */
    #[account(
        seeds = [mint.key().as_ref(), senior_tranche_mint.key().as_ref(), junior_tranche_mint.key().as_ref()],
        bump = tranche_config.tranche_config_bump,
        constraint = tranche_config.authority == *authority.key)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /**
     * mint token to deposit
     */
    #[account()]
    pub mint: Box<Account<'info, Mint>>,

    /**
     * deposit to
     */
    #[account(mut)]
    pub protocol_vault: Box<Account<'info, TokenAccount>>,

    /**
     * deposit from
     */
    #[account(
        mut, 
        // associated_token::mint = mint,
        // associated_token::authority = authority
    )]
    pub deposit_dest_account: Box<Account<'info, TokenAccount>>,

    // * * * * * * * * * * * * * * * * *

    // Senior tranche mint
    #[account(
        mut,
        seeds = [vyper_utils::constants::SENIOR.as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = tranche_config.senior_tranche_mint_bump)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(mut)]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(
        mut,
        seeds = [vyper_utils::constants::JUNIOR.as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = tranche_config.junior_tranche_mint_bump)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche token account
    #[account(mut)]
    pub junior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // proxy stuff
    // Vyper Vault authority
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [b"vault_authority".as_ref(), tranche_config.key().as_ref()],
        bump = vault_authority_bump,
   )]
    pub vault_authority: AccountInfo<'info>,

    // Token account for receiving collateral token (as proof of deposit)
    // TODO: init_if_needed
    #[account(
        mut, 
        // associated_token::mint = collateral_mint, 
        // associated_token::authority = vault_authority
    )]
    pub collateral_token_account: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    // Protocol reserve account
    /// CHECK: Safe
    #[account(mut)]
    pub refreshed_reserve_account: AccountInfo<'info>,

    // Lending market account
    /// CHECK: Safe
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    /// CHECK: Safe
    pub lending_market_authority: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    /// CHECK: Safe
    pub protocol_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[interface]
pub trait WithdrawVyperProxyLending<'info, T: Accounts<'info>> {
    fn withdraw_from_proxy(ctx: Context<T>, vault_authority_bump: u8, collateral_amount: u64) -> ProgramResult;
}


pub fn handler(ctx: Context<RedeemContext>, vault_authority_bump: u8, redeem_quantity: [u64; 2]) -> ProgramResult {
    msg!("redeem_tranche begin");

        if redeem_quantity[0] > ctx.accounts.senior_tranche_vault.amount
            || redeem_quantity[1] > ctx.accounts.junior_tranche_vault.amount
        {
            msg!("redeem quantity invalid");
            return Err(ErrorCode::InvalidInput.into());
        }

        // calculate capital redeem and interest to redeem

        let [capital_to_redeem, interest_to_redeem] = if ctx.accounts.protocol_vault.amount > ctx.accounts.tranche_config.deposited_quantiy[0]
        {
            [
                ctx.accounts.tranche_config.get_total_deposited_quantity(),
                ctx.accounts.protocol_vault.amount - ctx.accounts.tranche_config.get_total_deposited_quantity(),
            ]
        } else {
            [
                ctx.accounts.protocol_vault.amount,
                0
            ]
        };
        msg!("+ capital_to_redeem: {}", capital_to_redeem);
        msg!("+ interest_to_redeem: {}", interest_to_redeem);

        let capital_split_f: [f64; 2] = ctx.accounts.tranche_config.capital_split.map(|x| from_bps(x));
        let interest_split_f: [f64; 2] = ctx.accounts.tranche_config.interest_split.map(|x| from_bps(x));

        let mut senior_total: f64 = 0.0;
        let mut junior_total: f64 = 0.0;

        if interest_to_redeem > 0 {

            let senior_capital =
                ctx.accounts.tranche_config.get_total_deposited_quantity() as f64 * capital_split_f[0];

            senior_total += senior_capital;

            let senior_interest =
                interest_to_redeem as f64 * capital_split_f[0] * (1.0 as f64 - interest_split_f[0]);

            senior_total += senior_interest;

            junior_total += ctx.accounts.tranche_config.get_total_deposited_quantity() as f64 + interest_to_redeem as f64 - senior_total;

        } else {
            let senior_capital = cmp::min(
                (ctx.accounts.tranche_config.get_total_deposited_quantity() as f64 * capital_split_f[0]) as u64,
                capital_to_redeem,
            );
            let junior_capital = capital_to_redeem - senior_capital as u64;

            senior_total += senior_capital as f64;
            junior_total += junior_capital as f64;
        }

        let user_senior_part = if ctx.accounts.senior_tranche_vault.amount > 0 {
            senior_total * redeem_quantity[0] as f64 / ctx.accounts.senior_tranche_mint.supply as f64
        } else {
            0 as f64
        };

        let user_junior_part = if ctx.accounts.junior_tranche_vault.amount > 0 {
            junior_total * redeem_quantity[1] as f64 / ctx.accounts.junior_tranche_mint.supply as f64
        } else {
            0 as f64
        };

        let user_total = user_senior_part + user_junior_part;
        msg!("user_total to redeem: {}", user_total);

        let tranche_config_key = ctx.accounts.tranche_config.key();
        let seeds = &[
            b"vault_authority".as_ref(),
            tranche_config_key.as_ref(),
            &[vault_authority_bump]
        ];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.protocol_program.to_account_info(),
            WithdrawProxyLendingContext {
                authority: ctx.accounts.authority.clone(),
                vault_authority: ctx.accounts.vault_authority.clone(),
                protocol_program: ctx.accounts.protocol_program.clone(),

                withdraw_to: ctx.accounts.deposit_dest_account.clone(),
                withdraw_from_protocol_reserve: ctx.accounts.protocol_vault.clone(),
                withdraw_mint: ctx.accounts.mint.clone(),

                collateral_from: ctx.accounts.collateral_token_account.clone(),
                collateral_mint: ctx.accounts.collateral_mint.clone(),

                refreshed_reserve_account: ctx.accounts.refreshed_reserve_account.clone(),

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

        withdraw_vyper_proxy_lending::withdraw_from_proxy(cpi_ctx, vault_authority_bump, user_total as u64)?;

        // * * * * * * * * * * * * * * * * * * * * * * *
        // burn senior tranche tokens

        // TODO: burn on vault
        // msg!("burn senior tranche tokens: {}", ctx.accounts.senior_tranche_vault.amount);
        // spl_token_burn(TokenBurnParams { 
        //     mint: ctx.accounts.senior_tranche_mint.to_account_info(),
        //     to: ctx.accounts.senior_tranche_vault.to_account_info(),
        //     amount: redeem_quantity[0],
        //     authority: ctx.accounts.authority.to_account_info(),
        //     authority_signer_seeds: &[],
        //     token_program: ctx.accounts.token_program.to_account_info()
        // })?;

        // * * * * * * * * * * * * * * * * * * * * * * *
        // burn junior tranche tokens

        // TODO: burn on vault
        // msg!("burn junior tranche tokens: {}", ctx.accounts.junior_tranche_vault.amount);
        // spl_token_burn(TokenBurnParams { 
        //     mint: ctx.accounts.junior_tranche_mint.to_account_info(),
        //     to: ctx.accounts.junior_tranche_vault.to_account_info(),
        //     amount: redeem_quantity[1],
        //     authority: ctx.accounts.authority.to_account_info(),
        //     authority_signer_seeds: &[],
        //     token_program: ctx.accounts.token_program.to_account_info()
        // })?;

        Ok(())
}