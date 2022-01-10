mod constants;
mod error;
mod inputs;
mod state;
mod utils;

use std::cmp;
use anchor_lang::prelude::*;
use anchor_spl::{self, associated_token::AssociatedToken, token::{ self, Mint, TokenAccount, Token }};
// use port_anchor_adaptor::*;
use utils::*;
use inputs::{ Input, CreateTrancheConfigInput, RedeemTrancheInput };
use state::{ TrancheConfig };
use error::ErrorCode;

declare_id!("EgCnSZMSvrz6t8N4fB4tGesibVtNUJMMkS7t6VvyiiHV");

#[program]
pub mod vyper {
    use super::*;

    /**
     * create a new tranche configuration and deposit
     */
    pub fn create_tranche(
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
        if let Result::Err(err) = input_data.is_valid() {
            msg!("input data is not valid");
            return Err(err.into());
        }
        
        // * * * * * * * * * * * * * * * * * * * * * * * 
        // create tranche config account
        
        msg!("create tranche config");
        input_data.create_tranche_config(&mut ctx.accounts.tranche_config);
        ctx.accounts.tranche_config.authority = ctx.accounts.authority.key();
        ctx.accounts.tranche_config.senior_tranche_mint = ctx.accounts.senior_tranche_mint.key();
        ctx.accounts.tranche_config.junior_tranche_mint = ctx.accounts.junior_tranche_mint.key();
        ctx.accounts.tranche_config.tranche_config_bump = tranche_config_bump;
        ctx.accounts.tranche_config.senior_tranche_mint_bump = senior_tranche_mint_bump;
        ctx.accounts.tranche_config.junior_tranche_mint_bump = junior_tranche_mint_bump;
        
        // * * * * * * * * * * * * * * * * * * * * * * * 
        // execute the deposit

        msg!("transfer tokens to program owned account");
        
        let deposit_transfer_ctx = token::Transfer {
            from: ctx.accounts.deposit_from_account.to_account_info(),
            to: ctx.accounts.tranche_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), deposit_transfer_ctx), input_data.quantity)?;

        // * * * * * * * * * * * * * * * * * * * * * * * 
        // mint senior tranche tokens
        msg!("mint senior tranche");

        let senior_mint_to_ctx = token::MintTo {
            mint: ctx.accounts.senior_tranche_mint.to_account_info(),
            to: ctx.accounts.senior_tranche_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info()
        };
        token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), senior_mint_to_ctx), ctx.accounts.tranche_config.mint_count[0])?;

        // * * * * * * * * * * * * * * * * * * * * * * * 
        // mint junior tranche tokens

        msg!("mint junior tranche");

        let junior_mint_to_ctx = token::MintTo {
            mint: ctx.accounts.junior_tranche_mint.to_account_info(),
            to: ctx.accounts.junior_tranche_vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info()
        };
        token::mint_to(CpiContext::new(ctx.accounts.token_program.to_account_info(), junior_mint_to_ctx), ctx.accounts.tranche_config.mint_count[1])?;

        // * * * * * * * * * * * * * * * * * * * * * * * 

        Ok(())
    }

    pub fn redeem(ctx: Context<RedeemContext>, input_data: RedeemTrancheInput) -> ProgramResult  {
        msg!("redeem_tranche begin");
        
        // * * * * * * * * * * * * * * * * * * * * * * * 
        // check input
        
        msg!("check if input is valid");
        if let Result::Err(err) = input_data.is_valid() {
            msg!("input adta is not valid");
            return Err(err.into());
        }

        // check if before or after end date

        if ctx.accounts.tranche_config.end_date > ctx.accounts.clock.unix_timestamp as u64 {
        
            // check if user has same ration of senior/junior tokens than origin

            let user_ratio = ctx.accounts.senior_tranche_vault.amount as f64 / ctx.accounts.junior_tranche_vault.amount as f64;
            let origin_ratio = ctx.accounts.tranche_config.mint_count[0] as f64 / ctx.accounts.tranche_config.mint_count[1] as f64;

            if user_ratio != origin_ratio {
                return Result::Err(ErrorCode::InvalidTrancheAmount.into());
            }
        }

        // calculate capital redeem and interest to redeem

        let [capital_to_redeem, interest_to_redeem] = if ctx.accounts.tranche_vault.amount >= ctx.accounts.tranche_config.quantity {
            [ctx.accounts.tranche_config.quantity, ctx.accounts.tranche_vault.amount - ctx.accounts.tranche_config.quantity]
        } else {
            [ctx.accounts.tranche_vault.amount, 0]
        };
        msg!("+ capital_to_redeem: {}", capital_to_redeem);
        msg!("+ interest_to_redeem: {}", interest_to_redeem);

        let capital_split_f: [f64;2] = [
            from_bps(ctx.accounts.tranche_config.capital_split[0]),
            from_bps(ctx.accounts.tranche_config.capital_split[1]),
        ];
            
        let interest_split_f: [f64;2] = [
            from_bps(ctx.accounts.tranche_config.interest_split[0]),
            from_bps(ctx.accounts.tranche_config.interest_split[1]),
        ];
                
        let mut senior_total: f64 = 0.0;
        let mut junior_total: f64 = 0.0;

        if interest_to_redeem > 0 {
            // we have interest

            let senior_capital = ctx.accounts.tranche_config.quantity as f64 * capital_split_f[0];
            let junior_capital = ctx.accounts.tranche_config.quantity as f64 * capital_split_f[1];

            senior_total += senior_capital;
            junior_total += junior_capital;


            let senior_interest = interest_to_redeem as f64 * capital_split_f[0] * interest_split_f[0];
            let junior_interest = interest_to_redeem as f64 * capital_split_f[0] * interest_split_f[1] + interest_to_redeem as f64 * capital_split_f[1];
            
            if junior_interest + senior_interest != interest_to_redeem as f64 {
                msg!("error");
                return Result::Err(ErrorCode::InvalidTrancheAmount.into());
            }
            
            senior_total += senior_interest;
            junior_total += junior_interest;

        } else {

            let senior_capital = from_bps(cmp::min(
                to_bps(ctx.accounts.tranche_config.quantity as f64 * capital_split_f[0]),
                to_bps(capital_to_redeem as f64)));
            let junior_capital = capital_to_redeem - senior_capital as u64;

            senior_total += senior_capital;
            junior_total += junior_capital as f64;
        }

        let user_senior_part = senior_total * ctx.accounts.tranche_config.mint_count[0] as f64 / ctx.accounts.senior_tranche_vault.amount as f64;
        let user_junior_part = junior_total * ctx.accounts.tranche_config.mint_count[1] as f64 / ctx.accounts.junior_tranche_vault.amount as f64;

        let user_total = user_senior_part + user_junior_part;
        msg!("user_total to redeem: {}", user_total);

        let seeds = &[
            ctx.accounts.deposit_mint.to_account_info().key.as_ref(),
            ctx.accounts.senior_tranche_mint.to_account_info().key.as_ref(),
            ctx.accounts.junior_tranche_mint.to_account_info().key.as_ref(),
             &[ctx.accounts.tranche_config.tranche_config_bump]];

        token::transfer(CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.tranche_vault.to_account_info(),
                to: ctx.accounts.deposit_to_account.to_account_info(),
                authority: ctx.accounts.tranche_config.to_account_info(),
            },
            &[&seeds[..]]), user_total as u64)?;

        Ok(())
    }
}


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
        seeds = [deposit_mint.key().as_ref(), senior_tranche_mint.key().as_ref(), junior_tranche_mint.key().as_ref()],
        bump = tranche_config_bump,
        space = TrancheConfig::LEN)]
    pub tranche_config: Account<'info, TrancheConfig>,

    /**
     * mint token to deposit
     */ 
    #[account()]
    pub deposit_mint: Box<Account<'info, Mint>>,

    /**
     * deposit from
     */ 
    #[account(mut, associated_token::mint = deposit_mint, associated_token::authority = authority)]
    pub deposit_from_account: Box<Account<'info, TokenAccount>>,
    
    /**
     * deposit to
     */ 
    #[account(init, payer = authority, associated_token::mint = deposit_mint, associated_token::authority = tranche_config)]
    pub tranche_vault: Box<Account<'info, TokenAccount>>,

    // * * * * * * * * * * * * * * * * * 

    // Senior tranche mint
    #[account(
        init,
        seeds = [b"senior".as_ref(), deposit_mint.key().as_ref()],
        bump = senior_tranche_mint_bump,
        payer = authority, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(init, payer = authority, associated_token::mint = senior_tranche_mint, associated_token::authority = authority)]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(init,
        seeds = [b"junior".as_ref(), deposit_mint.key().as_ref()],
        bump = junior_tranche_mint_bump,
        payer = authority, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche token account
    #[account(init, payer = authority, associated_token::mint = junior_tranche_mint, associated_token::authority = authority)]
    pub junior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // * * * * * * * * * * * * * * * * * 
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(input_data: RedeemTrancheInput)]
pub struct RedeemContext<'info> {

    /**
     * Signer account
     */
    #[account(mut)]
    pub authority: Signer<'info>,

    /**
     * Tranche config account, where all the parameters are saved
     */
    #[account(seeds = [deposit_mint.key().as_ref(), senior_tranche_mint.key().as_ref(), junior_tranche_mint.key().as_ref()], bump = tranche_config.tranche_config_bump)]
    pub tranche_config: Account<'info, TrancheConfig>,

    /**
     * mint token to deposit
     */ 
    #[account()]
    pub deposit_mint: Box<Account<'info, Mint>>,
    
    /**
     * deposit to
     */ 
    #[account(mut, associated_token::mint = deposit_mint, associated_token::authority = tranche_config)]
    pub tranche_vault: Box<Account<'info, TokenAccount>>,

    /**
     * deposit from
     */ 
    #[account(mut, associated_token::mint = deposit_mint, associated_token::authority = authority)]
    pub deposit_to_account: Box<Account<'info, TokenAccount>>,

    // * * * * * * * * * * * * * * * * * 

    // Senior tranche mint
    #[account(
        seeds = [b"senior".as_ref(), deposit_mint.key().as_ref()],
        bump = tranche_config.senior_tranche_mint_bump,
        )]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(associated_token::mint = senior_tranche_mint, associated_token::authority = authority)]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(
        seeds = [b"junior".as_ref(), deposit_mint.key().as_ref()],
        bump = tranche_config.junior_tranche_mint_bump)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche token account
    #[account(associated_token::mint = junior_tranche_mint, associated_token::authority = authority)]
    pub junior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // * * * * * * * * * * * * * * * * * 

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}
