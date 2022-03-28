pub mod constants;
pub mod error;
pub mod inputs;
pub mod state;

use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    dex,
    token::{self, Mint, Token, TokenAccount},
};
use error::ErrorCode;
use inputs::{CreateTrancheConfigInput, Input};
use proxy_lending_interface::*;
use state::TrancheConfig;
use std::cmp;
use vyper_utils::{ 
    math::from_bps
};

declare_id!("9pnvhZfrKPKpM58b6oTxYrfgNLRDcvfbjtGRm57fwXiv");

#[program]
pub mod vyper_core_lending {
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

    pub fn deposit(
        ctx: Context<DepositContext>,
        vault_authority_bump: u8,
        quantity: [u64; 2],
        mint_count: [u64; 2],
    ) -> ProgramResult {
        msg!("deposit begin");

        // * * * * * * * * * * * * * * * * * * * * * * *

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
            ctx.accounts.protocol_program.to_account_info(),
            proxy_lending_interface::DepositProxyLendingContext {
                authority: ctx.accounts.authority.clone(),
                vault_authority: ctx.accounts.vault_authority.clone(),
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

        vyper_proxy::deposit_to_proxy(cpi_ctx, vault_authority_bump, quantity[0] + quantity[1])?;
     
        // * * * * * * * * * * * * * * * * * * * * * * *

        // increase the deposited quantity


        ctx.accounts.tranche_config.deposited_quantiy += quantity[0] + quantity[1];

        // * * * * * * * * * * * * * * * * * * * * * * *

        // mint tranche tokens to user

        if mint_count[0] > 0 {
            msg!("mint senior tranche");
    
            let senior_mint_to_ctx = token::MintTo {
                mint: ctx.accounts.senior_tranche_mint.to_account_info(),
                to: ctx.accounts.senior_tranche_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            token::mint_to(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    senior_mint_to_ctx,
                ),
                mint_count[0],
            )?;
        }

        if mint_count[1] > 0 {
            msg!("mint junior tranche");

            let junior_mint_to_ctx = token::MintTo {
                mint: ctx.accounts.junior_tranche_mint.to_account_info(),
                to: ctx.accounts.junior_tranche_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            token::mint_to(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    junior_mint_to_ctx,
                ),
                mint_count[1],
            )?;
        }

        // * * * * * * * * * * * * * * * * * * * * * * *

        Ok(())
    }

    pub fn update_interest_split(
        ctx: Context<UpdateInterestSplitContext>,
        interest_split: [u32; 2],
    ) -> ProgramResult {
        msg!("update_interest_split begin");
        msg!("+ old_interest_split[0]: {}", ctx.accounts.tranche_config.interest_split[0]);
        msg!("+ old_interest_split[1]: {}", ctx.accounts.tranche_config.interest_split[1]);
        
        ctx.accounts.tranche_config.interest_split = interest_split;

        msg!("+ new_interest_split[0]: {}", ctx.accounts.tranche_config.interest_split[0]);
        msg!("+ new_interest_split[1]: {}", ctx.accounts.tranche_config.interest_split[1]);
        Ok(())
    }

    pub fn create_serum_market(
        ctx: Context<CreateSerumMarketContext>,
        vault_signer_nonce: u8,
    ) -> ProgramResult {
        // * * * * * * * * * * * * * * * * * * * * * * *
        // initialize market on serum

        msg!("initialize market on serum");

        let initialize_market_ctx = dex::InitializeMarket {
            market: ctx.accounts.market.to_account_info().clone(),
            coin_mint: ctx.accounts.junior_tranche_mint.to_account_info().clone(),
            coin_vault: ctx
                .accounts
                .junior_tranche_serum_vault
                .to_account_info()
                .clone(),
            bids: ctx.accounts.bids.to_account_info().clone(),
            asks: ctx.accounts.asks.to_account_info().clone(),
            req_q: ctx.accounts.request_queue.to_account_info().clone(),
            event_q: ctx.accounts.event_queue.to_account_info().clone(),
            rent: ctx.accounts.rent.to_account_info().clone(),
            pc_mint: ctx.accounts.usdc_mint.to_account_info().clone(),
            pc_vault: ctx.accounts.usdc_serum_vault.to_account_info().clone(),
        };

        dex::initialize_market(
            CpiContext::new(
                ctx.accounts.serum_dex.to_account_info().clone(),
                initialize_market_ctx,
            ),
            100_000,
            100,
            vault_signer_nonce.into(),
            500,
        )?;

        Ok(())
    }

    pub fn redeem(ctx: Context<RedeemContext>, vault_authority_bump: u8, redeem_quantity: [u64; 2]) -> ProgramResult {
        msg!("redeem_tranche begin");

        if redeem_quantity[0] > ctx.accounts.senior_tranche_vault.amount
            || redeem_quantity[1] > ctx.accounts.junior_tranche_vault.amount
        {
            msg!("redeem quantity invalid");
            return Err(ErrorCode::InvalidInput.into());
        }

        // calculate capital redeem and interest to redeem

        let [capital_to_redeem, interest_to_redeem] = if ctx.accounts.protocol_vault.amount
            > ctx.accounts.tranche_config.deposited_quantiy
        {
            [
                ctx.accounts.tranche_config.deposited_quantiy,
                ctx.accounts.protocol_vault.amount - ctx.accounts.tranche_config.deposited_quantiy,
            ]
        } else {
            [ctx.accounts.protocol_vault.amount, 0]
        };
        msg!("+ capital_to_redeem: {}", capital_to_redeem);
        msg!("+ interest_to_redeem: {}", interest_to_redeem);

        let capital_split_f: [f64; 2] = [
            from_bps(ctx.accounts.tranche_config.capital_split[0]),
            from_bps(ctx.accounts.tranche_config.capital_split[1]),
        ];

        let interest_split_f: [f64; 2] = [
            from_bps(ctx.accounts.tranche_config.interest_split[0]),
            from_bps(ctx.accounts.tranche_config.interest_split[1]),
        ];

        let mut senior_total: f64 = 0.0;
        let mut junior_total: f64 = 0.0;

        if interest_to_redeem > 0 {

            let senior_capital =
                ctx.accounts.tranche_config.deposited_quantiy as f64 * capital_split_f[0];

            senior_total += senior_capital;

            let senior_interest =
                interest_to_redeem as f64 * capital_split_f[0] * (1.0 as f64 - interest_split_f[0]);

            senior_total += senior_interest;

            junior_total += ctx.accounts.tranche_config.deposited_quantiy as f64 + interest_to_redeem as f64 - senior_total;

        } else {
            let senior_capital = cmp::min(
                (ctx.accounts.tranche_config.deposited_quantiy as f64 * capital_split_f[0]) as u64,
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
            proxy_lending_interface::WithdrawProxyLendingContext {
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

        vyper_proxy::withdraw_from_proxy(cpi_ctx, vault_authority_bump, user_total as u64)?;

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
        seeds = [mint.key().as_ref(), senior_tranche_mint.key().as_ref(), junior_tranche_mint.key().as_ref()],
        bump = tranche_config_bump,
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
        seeds = [b"senior".as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = senior_tranche_mint_bump,
        payer = authority, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche mint
    #[account(init,
        seeds = [b"junior".as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = junior_tranche_mint_bump,
        payer = authority, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // * * * * * * * * * * * * * * * * *
    pub protocol_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(vault_authority_bump: u8)]
pub struct DepositContext<'info> {
    /**
     * Signer account
     */
    #[account(mut)]
    pub authority: Signer<'info>,

    /**
     * Tranche config account, where all the parameters are saved
     */
    #[account()]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /**
     * mint token to deposit
     */
    #[account()]
    pub mint: Box<Account<'info, Mint>>,

    /**
     * deposit from
     */
    #[account(
        mut,
        // associated_token::mint = mint,
        // associated_token::authority = authority
    )]
    pub deposit_source_account: Box<Account<'info, TokenAccount>>,

    /**
     * protocol vault (SOLEND RESERVE https://docs.solend.fi/developers/addresses/devnet#reserves )
     */
    #[account(mut)]
    pub protocol_vault: Box<Account<'info, TokenAccount>>,

    // proxy stuff
    // Vyper Vault authority
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
        associated_token::mint = collateral_mint,
        associated_token::authority = vault_authority
    )]
    pub collateral_token_account: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    // State account for protocol (reserve-state-account)
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    // Lending market account (https://docs.solend.fi/developers/addresses/devnet#devnet)
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    pub lending_market_authority: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *

    // Senior tranche mint
    #[account(
        seeds = [vyper_utils::constants::SENIOR.as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = tranche_config.senior_tranche_mint_bump
    )]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(
        mut,
        // associated_token::mint = senior_tranche_mint,
        // associated_token::authority = authority
    )]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(
        seeds = [vyper_utils::constants::JUNIOR.as_ref(), protocol_program.key().as_ref(), mint.key().as_ref()],
        bump = tranche_config.junior_tranche_mint_bump
    )]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche token account
    #[account(
        mut,
        // associated_token::mint = junior_tranche_mint,
        // associated_token::authority = authority
    )]
    pub junior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // solend: ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx
    #[account(
        // constraint = tranche_config.protocol_program_id == *protocol_program.key
    )]
    pub protocol_program: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct UpdateInterestSplitContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = tranche_config.authority == *authority.key)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateSerumMarketContext<'info> {
    /**
     * Signer account
     */
    #[account(mut)]
    pub authority: Signer<'info>,

    // Senior tranche mint
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche serum vault
    #[account(mut)]
    pub senior_tranche_serum_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche serum vault
    #[account(mut)]
    pub junior_tranche_serum_vault: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub usdc_serum_vault: Box<Account<'info, TokenAccount>>,

    // * * * * * * * * * * * * * * * * *

    // serum accounts
    #[account(mut)]
    pub market: Signer<'info>,
    #[account(mut)]
    pub request_queue: Signer<'info>,
    #[account(mut)]
    pub event_queue: Signer<'info>,
    #[account(mut)]
    pub asks: Signer<'info>,
    #[account(mut)]
    pub bids: Signer<'info>,

    pub serum_dex: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

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
    #[account(
        mut,
        seeds = [b"vault_authority"],
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
    #[account(mut)]
    pub refreshed_reserve_account: AccountInfo<'info>,

    // Lending market account
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    pub lending_market_authority: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub protocol_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[interface]
pub trait VyperProxy<'info> {
    fn deposit_to_proxy(
        ctx: Context<DepositProxyLendingContext<'info>>,
        vault_authority_bump: u8,
        amount: u64,
    ) -> ProgramResult;

    fn withdraw_from_proxy(
        ctx: Context<WithdrawProxyLendingContext<'info>>,
        vault_authority_bump: u8,
        collateral_amount: u64,
    ) -> ProgramResult;
}
