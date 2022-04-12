use crate::adapters::{
    CommonAdapterTraits, DepositReserveLiquidity, RedeemReserveCollateral, RefreshReserve,
};
use anchor_lang::{prelude::*, solana_program};
use spl_token_lending::state::Reserve;
use std::io::Write;
use std::ops::Deref;
pub struct SolendAdapter;

impl<'info> CommonAdapterTraits<'info> for SolendAdapter {
    fn refresh_reserve(
        &self,
        ctx: CpiContext<'_, '_, '_, 'info, RefreshReserve<'info>>,
    ) -> ProgramResult {
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

    fn deposit_reserve_liquidity(
        &self,
        ctx: CpiContext<'_, '_, '_, 'info, DepositReserveLiquidity<'info>>,
        liquidity_amount: u64,
    ) -> ProgramResult {
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

        solana_program::program::invoke(&ix, &ToAccountInfos::to_account_infos(&ctx))?;

        Ok(())
    }

    fn redeem_reserve_collateral(
        &self,
        ctx: CpiContext<'_, '_, '_, 'info, RedeemReserveCollateral<'info>>,
        collateral_amount: u64,
    ) -> ProgramResult {
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

        solana_program::program::invoke(&ix, &ToAccountInfos::to_account_infos(&ctx))?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct SolendReserve(Reserve);

impl anchor_lang::AccountDeserialize for SolendReserve {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        SolendReserve::try_deserialize_unchecked(buf)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        <spl_token_lending::state::Reserve as solana_program::program_pack::Pack>::unpack(buf)
            .map(SolendReserve)
    }
}

impl anchor_lang::AccountSerialize for SolendReserve {
    fn try_serialize<W: Write>(&self, _writer: &mut W) -> Result<(), ProgramError> {
        // no-op
        Ok(())
    }
}

impl anchor_lang::Owner for SolendReserve {
    fn owner() -> Pubkey {
        spl_token_lending::id()
    }
}

impl Deref for SolendReserve {
    type Target = spl_token_lending::state::Reserve;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
