use crate::{adapters::common::*, state::TrancheConfig};
use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};
use vyper_utils::math::{from_bps, get_quantites_with_capital_split};

#[derive(Accounts)]
pub struct DepositContext<'info> {
    /// Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Tranche config account, where all the parameters are saved
    /// TODO add constraints
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
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,

    // Token account for receiving collateral token (as proof of deposit)
    #[account(mut)]
    pub destination_collateral_account: Box<Account<'info, TokenAccount>>,

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
    #[account(mut, seeds = [vyper_utils::constants::SENIOR.as_ref(), lending_program.key().as_ref(), reserve_token.key().as_ref()], bump = tranche_config.senior_tranche_mint_bump)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(mut)]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(mut, seeds = [vyper_utils::constants::JUNIOR.as_ref(), lending_program.key().as_ref(), reserve_token.key().as_ref()], bump = tranche_config.junior_tranche_mint_bump)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche token account
    #[account(mut)]
    pub junior_tranche_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Safe
    #[account()]
    pub lending_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> DepositContext<'info> {
    fn to_refresh_reserve_context(&self) -> CpiContext<'_, '_, '_, 'info, RefreshReserve<'info>> {
        CpiContext::new(
            self.lending_program.to_account_info(),
            RefreshReserve {
                lending_program: self.lending_program.clone(),
                reserve: self.protocol_state.clone(),
                pyth_reserve_liquidity_oracle: self.pyth_reserve_liquidity_oracle.clone(),
                switchboard_reserve_liquidity_oracle: self
                    .switchboard_reserve_liquidity_oracle
                    .clone(),
                clock: self.clock.to_account_info(),
            },
        )
    }

    fn to_deposit_reserve_liquidity_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, DepositReserveLiquidity<'info>> {
        CpiContext::new(
            self.lending_program.to_account_info(),
            DepositReserveLiquidity {
                lending_program: self.lending_program.clone(),
                source_liquidity: self.source_liquidity.to_account_info(),
                destination_collateral_account: self
                    .destination_collateral_account
                    .to_account_info(),
                reserve: self.protocol_state.clone(),
                reserve_collateral_mint: self.collateral_mint.to_account_info(),
                reserve_liquidity_supply: self.reserve_liquidity_supply.to_account_info(),
                lending_market: self.lending_market_account.clone(),
                lending_market_authority: self.lending_market_authority.clone(),
                transfer_authority: self.authority.to_account_info(),
                clock: self.clock.to_account_info(),
                token_program_id: self.token_program.to_account_info(),
            },
        )
    }
}

pub fn handler(ctx: Context<DepositContext>, quantity: u64, mint_count: [u64; 2]) -> ProgramResult {
    msg!("deposit begin");

    msg!("+ quantity: {}", quantity);
    for i in 0..mint_count.len() {
        msg!("+ mint_count[{}]: {}", i, mint_count[i]);
    }

    // deposit on final protocol

    msg!("deposit tokens to protocol");

    crate::adapters::common::adpater_factory(LendingMarketID::Solend)
        .unwrap()
        .refresh_reserve(ctx.accounts.to_refresh_reserve_context())?;
    crate::adapters::common::adpater_factory(LendingMarketID::Solend)
        .unwrap()
        .deposit_reserve_liquidity(
            ctx.accounts.to_deposit_reserve_liquidity_context(),
            quantity,
        )?;
    // deposit_vyper_proxy_lending::refresh_reserve_for_deposit(ctx.accounts.to_deposit_proxy_lending_context())?;
    // deposit_vyper_proxy_lending::deposit_reserve_liquidity(ctx.accounts.to_deposit_proxy_lending_context(), quantity)?;

    // * * * * * * * * * * * * * * * * * * * * * * *
    // increase the deposited quantity

    let split_quantities = get_quantites_with_capital_split(
        quantity,
        ctx.accounts
            .tranche_config
            .capital_split
            .map(|x| from_bps(x)),
    );
    for i in 0..split_quantities.len() {
        msg!("split_quantities[{}] = {}", i, split_quantities[i]);
        msg!(
            "old deposited quantity [{}] = {}",
            i,
            ctx.accounts.tranche_config.deposited_quantiy[i]
        );

        ctx.accounts.tranche_config.deposited_quantiy[i] += split_quantities[i];

        msg!(
            "new deposited quantity [{}] = {}",
            i,
            ctx.accounts.tranche_config.deposited_quantiy[i]
        );
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
        &[ctx.accounts.tranche_config.tranche_config_bump],
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
                tranche_config_signer,
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
                tranche_config_signer,
            ),
            mint_count[1],
        )?;
    }

    Ok(())
}
