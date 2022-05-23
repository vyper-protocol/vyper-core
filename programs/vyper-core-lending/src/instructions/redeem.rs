use crate::{
    adapters::{common::*, solend::*},
    error::ErrorCode,
    state::TrancheConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use spl_token_lending::state::CollateralExchangeRate;
use std::cmp;
use vyper_utils::math::from_bps;
use vyper_utils::token::spl_token_burn;
use vyper_utils::token::TokenBurnParams;

#[derive(Accounts)]
pub struct RedeemContext<'info> {
    /// Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Tranche config account, where all the parameters are saved
    #[account(mut, constraint = tranche_config.authority == authority.key())]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// mint token to deposit
    #[account()]
    pub reserve_token: Box<Account<'info, Mint>>,

    /// deposit from
    #[account(mut)]
    pub destination_liquidity: Box<Account<'info, TokenAccount>>,

    /// protocol vault
    #[account(mut)]
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,

    // Token account for receiving collateral token (as proof of deposit)
    #[account(mut)]
    pub source_collateral_account: Box<Account<'info, TokenAccount>>,

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
    #[account(mut, seeds = [tranche_config.id.as_ref(), vyper_utils::constants::SENIOR.as_ref(), lending_program.key().as_ref(), reserve_token.key().as_ref()], bump = tranche_config.senior_tranche_mint_bump)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche token account
    #[account(mut)]
    pub senior_tranche_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    #[account(mut, seeds = [tranche_config.id.as_ref(), vyper_utils::constants::JUNIOR.as_ref(), lending_program.key().as_ref(), reserve_token.key().as_ref()], bump = tranche_config.junior_tranche_mint_bump)]
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

impl<'info> RedeemContext<'info> {
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

    fn to_withdraw_proxy_lending_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, RedeemReserveCollateral<'info>> {
        CpiContext::new(
            self.lending_program.to_account_info(),
            RedeemReserveCollateral {
                lending_program: self.lending_program.clone(),
                source_collateral: self.source_collateral_account.to_account_info(),
                destination_liquidity: self.destination_liquidity.to_account_info(),
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

    fn get_collateral_exchange_rate(&self) -> Result<CollateralExchangeRate, ProgramError> {
        // let reserve: spl_token_lending::state::Reserve = ctx.accounts.protocol_state.deserialize_data().map_err(|_| ProgramError::BorshIoError)?;
        // let reserve: SolendReserve = SolendReserve::try_deserialize(ctx.accounts.protocol_state.data)
        let protocol_state_clone = self.protocol_state.clone();
        let mut reserve_data: &[u8] = &protocol_state_clone.try_borrow_mut_data()?;
        let reserve: SolendReserve = SolendReserve::try_deserialize(&mut reserve_data)?;
        reserve.collateral_exchange_rate()
    }
}

pub fn handler(ctx: Context<RedeemContext>, redeem_quantity: [u64; 2]) -> ProgramResult {
    msg!("redeem_tranche begin");

    if redeem_quantity[0] > ctx.accounts.senior_tranche_vault.amount
        || redeem_quantity[1] > ctx.accounts.junior_tranche_vault.amount
    {
        msg!("redeem quantity invalid");
        return Err(ErrorCode::InvalidInput.into());
    }

    msg!("CPI: refesh reserve for redeem");
    crate::adapters::common::adpater_factory(LendingMarketID::Solend)
        .unwrap()
        .refresh_reserve(ctx.accounts.to_refresh_reserve_context())?;

    let collateral_exchange_rate = ctx.accounts.get_collateral_exchange_rate()?;
    let reserve_token_in_reserve = collateral_exchange_rate
        .collateral_to_liquidity(ctx.accounts.source_collateral_account.amount)?;

    msg!("reserve_token_in_reserve: {}", reserve_token_in_reserve);

    // calculate capital redeem and interest to redeem

    let [capital_to_redeem, interest_to_redeem] = if reserve_token_in_reserve
        > ctx.accounts.tranche_config.get_total_deposited_quantity()
    {
        [
            ctx.accounts.tranche_config.get_total_deposited_quantity(),
            reserve_token_in_reserve - ctx.accounts.tranche_config.get_total_deposited_quantity(),
        ]
    } else {
        [reserve_token_in_reserve, 0]
    };
    msg!("+ capital_to_redeem: {}", capital_to_redeem);
    msg!("+ interest_to_redeem: {}", interest_to_redeem);

    let capital_split_f: [f64; 2] = ctx
        .accounts
        .tranche_config
        .capital_split
        .map(|x| from_bps(x));
    let interest_split_f: [f64; 2] = ctx
        .accounts
        .tranche_config
        .interest_split
        .map(|x| from_bps(x));

    let mut senior_total: f64 = 0.0;
    let mut junior_total: f64 = 0.0;

    if interest_to_redeem > 0 {
        let senior_capital =
            ctx.accounts.tranche_config.get_total_deposited_quantity() as f64 * capital_split_f[0];

        senior_total += senior_capital;

        let senior_interest =
            interest_to_redeem as f64 * capital_split_f[0] * (1.0 as f64 - interest_split_f[0]);

        senior_total += senior_interest;

        junior_total += ctx.accounts.tranche_config.get_total_deposited_quantity() as f64
            + interest_to_redeem as f64
            - senior_total;
    } else {
        let senior_capital = cmp::min(
            (ctx.accounts.tranche_config.get_total_deposited_quantity() as f64 * capital_split_f[0])
                as u64,
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

    let user_total_to_redeem = (user_senior_part + user_junior_part) as u64;
    msg!("user_total_to_redeem: {}", user_total_to_redeem);

    let collateral_to_redeem =
        collateral_exchange_rate.liquidity_to_collateral(user_total_to_redeem)?;
    msg!("collateral_to_redeem: {}", collateral_to_redeem);

    msg!("CPI: redeem reserve collateral");
    crate::adapters::common::adpater_factory(LendingMarketID::Solend)
        .unwrap()
        .redeem_reserve_collateral(
            ctx.accounts.to_withdraw_proxy_lending_context(),
            collateral_to_redeem,
        )?;

    // * * * * * * * * * * * * * * * * * * * * * * *
    // burn senior tranche tokens

    msg!("burn senior tranche tokens: {}", redeem_quantity[0]);
    spl_token_burn(TokenBurnParams {
        mint: ctx.accounts.senior_tranche_mint.to_account_info(),
        to: ctx.accounts.senior_tranche_vault.to_account_info(),
        amount: redeem_quantity[0],
        authority: ctx.accounts.authority.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    })?;

    // * * * * * * * * * * * * * * * * * * * * * * *
    // burn junior tranche tokens

    msg!("burn junior tranche tokens: {}", redeem_quantity[1]);
    spl_token_burn(TokenBurnParams {
        mint: ctx.accounts.junior_tranche_mint.to_account_info(),
        to: ctx.accounts.junior_tranche_vault.to_account_info(),
        amount: redeem_quantity[1],
        authority: ctx.accounts.authority.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
    })?;

    Ok(())
}
