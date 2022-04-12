use crate::adapters::solend::SolendAdapter;
use anchor_lang::prelude::*;

pub trait CommonAdapterTraits<'info> {
    fn refresh_reserve(
        &self,
        ctx: CpiContext<'_, '_, '_, 'info, RefreshReserve<'info>>,
    ) -> ProgramResult;
    fn deposit_reserve_liquidity(
        &self,
        ctx: CpiContext<'_, '_, '_, 'info, DepositReserveLiquidity<'info>>,
        liquidity_amount: u64,
    ) -> ProgramResult;
    fn redeem_reserve_collateral(
        &self,
        ctx: CpiContext<'_, '_, '_, 'info, RedeemReserveCollateral<'info>>,
        collateral_amount: u64,
    ) -> ProgramResult;
}

pub enum LendingMarketID {
    Solend,
    // Jet,
}

pub fn adpater_factory<'info>(id: LendingMarketID) -> Option<Box<dyn CommonAdapterTraits<'info>>> {
    match id {
        LendingMarketID::Solend => Some(Box::new(SolendAdapter)),
        // _ => None
    }
}

#[derive(Accounts)]
pub struct DepositReserveLiquidity<'info> {
    // Lending program
    /// CHECK:
    pub lending_program: AccountInfo<'info>,
    // Token account for asset to deposit into reserve
    /// CHECK:
    pub source_liquidity: AccountInfo<'info>,
    // Token account for reserve collateral token
    /// CHECK:
    pub destination_collateral_account: AccountInfo<'info>,
    // Reserve state account
    /// CHECK:
    pub reserve: AccountInfo<'info>,
    // Token mint for reserve collateral token
    /// CHECK:
    pub reserve_collateral_mint: AccountInfo<'info>,
    // Reserve liquidity supply SPL token account
    /// CHECK:
    pub reserve_liquidity_supply: AccountInfo<'info>,
    // Lending market account
    /// CHECK:
    pub lending_market: AccountInfo<'info>,
    // Lending market authority (PDA)
    /// CHECK:
    pub lending_market_authority: AccountInfo<'info>,
    // Transfer authority for accounts 1 and 2
    /// CHECK:
    pub transfer_authority: AccountInfo<'info>,
    // Clock
    /// CHECK:
    pub clock: AccountInfo<'info>,
    // Token program ID
    /// CHECK:
    pub token_program_id: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RedeemReserveCollateral<'info> {
    // Lending program
    /// CHECK:
    pub lending_program: AccountInfo<'info>,
    // Source token account for reserve collateral token
    /// CHECK:
    pub source_collateral: AccountInfo<'info>,
    // Destination liquidity token account
    /// CHECK:
    pub destination_liquidity: AccountInfo<'info>,
    // Refreshed reserve account
    /// CHECK:
    pub reserve: AccountInfo<'info>,
    // Reserve collateral mint account
    /// CHECK:
    pub reserve_collateral_mint: AccountInfo<'info>,
    // Reserve liquidity supply SPL Token account.
    /// CHECK:
    pub reserve_liquidity_supply: AccountInfo<'info>,
    // Lending market account
    /// CHECK:
    pub lending_market: AccountInfo<'info>,
    // Lending market authority - PDA
    /// CHECK:
    pub lending_market_authority: AccountInfo<'info>,
    // User transfer authority
    /// CHECK:
    pub transfer_authority: AccountInfo<'info>,
    // Clock
    /// CHECK:
    pub clock: AccountInfo<'info>,
    // Token program ID
    /// CHECK:
    pub token_program_id: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RefreshReserve<'info> {
    // Lending program
    /// CHECK:
    pub lending_program: AccountInfo<'info>,
    // Reserve account
    /// CHECK:
    pub reserve: AccountInfo<'info>,
    // Pyth reserve liquidity oracle
    // Must be the pyth price account specified in InitReserve
    /// CHECK:
    pub pyth_reserve_liquidity_oracle: AccountInfo<'info>,
    // Switchboard Reserve liquidity oracle account
    // Must be the switchboard price account specified in InitReserve
    /// CHECK:
    pub switchboard_reserve_liquidity_oracle: AccountInfo<'info>,
    // Clock
    /// CHECK:
    pub clock: AccountInfo<'info>,
}
