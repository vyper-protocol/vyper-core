use anchor_lang::{ prelude::*, solana_program };
use crate::adapters::{ CommonAdapterTraits, RefreshReserve, DepositReserveLiquidity, RedeemReserveCollateral };

pub struct SolendAdapter;

impl<'info> CommonAdapterTraits<'info> for SolendAdapter {

    fn refresh_reserve(&self, ctx: CpiContext<'_, '_, '_, 'info, RefreshReserve<'info>>) -> ProgramResult {
        let ix = spl_token_lending::instruction::refresh_reserve(
            *ctx.accounts.lending_program.key,
            *ctx.accounts.reserve.key,
            *ctx.accounts.pyth_reserve_liquidity_oracle.key,
            *ctx.accounts.switchboard_reserve_liquidity_oracle.key,
        );
    
        solana_program::program::invoke_signed(
            &ix,
            &ToAccountInfos::to_account_infos(&ctx),
            ctx.signer_seeds,
        )?;

        Ok(())
    }

    fn deposit_reserve_liquidity(&self, ctx: CpiContext<'_, '_, '_, 'info, DepositReserveLiquidity<'info>>, liquidity_amount: u64) -> ProgramResult {
        let ix = spl_token_lending::instruction::deposit_reserve_liquidity(
            *ctx.accounts.lending_program.key,
            liquidity_amount,
            *ctx.accounts.source_liquidity.key,
            *ctx.accounts.destination_collateral_account.key,
            *ctx.accounts.reserve.key,
            *ctx.accounts.reserve_liquidity_supply.key,
            *ctx.accounts.reserve_collateral_mint.key,
            *ctx.accounts.lending_market.key,
            *ctx.accounts.transfer_authority.key,
        );
    
        solana_program::program::invoke(
            &ix,
            &ToAccountInfos::to_account_infos(&ctx)
        )?;

        Ok(())
    }
    
    fn redeem_reserve_collateral(&self, ctx: CpiContext<'_, '_, '_, 'info, RedeemReserveCollateral<'info>>, collateral_amount: u64) -> ProgramResult {
        let ix = spl_token_lending::instruction::redeem_reserve_collateral(
            *ctx.accounts.lending_program.key,
            collateral_amount,
            *ctx.accounts.source_collateral.key,
            *ctx.accounts.destination_liquidity.key,
            *ctx.accounts.reserve.key,
            *ctx.accounts.reserve_collateral_mint.key,
            *ctx.accounts.reserve_liquidity_supply.key,
            *ctx.accounts.lending_market.key,
            *ctx.accounts.transfer_authority.key,
        );
    
        solana_program::program::invoke(
            &ix,
            &ToAccountInfos::to_account_infos(&ctx),
        )?;
    
        Ok(())
    }
}