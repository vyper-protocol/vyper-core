use anchor_lang::prelude::*;
use vyper_core_lending::{
    instructions::deposit::DepositVyperProxyLending,
    instructions::redeem::WithdrawVyperProxyLending,
    interface_context::{
        DepositProxyLendingContext,
        WithdrawProxyLendingContext
    }
};
use solana_program::program::invoke;
use spl_token_lending::*;


declare_id!("9R88Mc2NBfhaxozbdwSHajAT94UUwe2ExrALq3FZK11L");

#[program]
pub mod proxy_lending_solend {
    use super::*;

    #[state]
    pub struct ProxyLendingSolend;

    impl<'info> DepositVyperProxyLending<'info, DepositProxyLendingContext<'info>> for ProxyLendingSolend {
        fn deposit_to_proxy(
            ctx: Context<DepositProxyLendingContext>,
            amount: u64,
        ) -> ProgramResult {
            msg!("deposit_to_proxy begin");
            
            msg!("refresh the serve");
            let refresh_context = CpiContext::new(
                ctx.accounts.protocol_program.clone(),
                RefreshReserve {
                    lending_program: ctx.accounts.protocol_program.clone(),
                    reserve: ctx.accounts.protocol_state.to_account_info(),
                    pyth_reserve_liquidity_oracle: ctx.accounts.pyth_reserve_liquidity_oracle.clone(),
                    switchboard_reserve_liquidity_oracle: ctx.accounts.switchboard_reserve_liquidity_oracle.clone(),
                    clock: ctx.accounts.clock.to_account_info(),
                },
            );
            refresh_reserve(refresh_context)?;

            msg!("deposit");
            let deposit_context = CpiContext::new(
                ctx.accounts.protocol_program.clone(),
                DepositReserveLiquidity {
                    lending_program: ctx.accounts.protocol_program.clone(),
                    source_liquidity: ctx.accounts.source_liquidity.to_account_info(),
                    destination_collateral_account: ctx.accounts.collateral_token_account.to_account_info(),
                    reserve: ctx.accounts.protocol_state.to_account_info(),
                    reserve_collateral_mint: ctx.accounts.collateral_mint.to_account_info(),
                    reserve_liquidity_supply: ctx.accounts.deposit_to_protocol_reserve.to_account_info(),
                    lending_market: ctx.accounts.lending_market_account.clone(),
                    lending_market_authority: ctx.accounts.lending_market_authority.clone(),
                    transfer_authority: ctx.accounts.authority.to_account_info(),
                    clock: ctx.accounts.clock.to_account_info(),
                    token_program_id: ctx.accounts.token_program.to_account_info(),
                },
            );
            match amount {
                0 => Ok(()),
                _ => deposit_reserve_liquidity(
                    deposit_context,
                    amount,
                ),
            }?;

            msg!("deposit_to_proxy end");

            Ok(())
        }
    }

    impl<'info> WithdrawVyperProxyLending<'info, WithdrawProxyLendingContext<'info>> for ProxyLendingSolend {
        fn withdraw_from_proxy(
            ctx: Context<WithdrawProxyLendingContext>,
            vault_authority_bump: u8,
            collateral_amount: u64,
        ) -> ProgramResult {
            msg!("withdraw_from_proxy begin");
            
            msg!("WITHDRAW FROM SOLEND");
            let ins = spl_token_lending::instruction::redeem_reserve_collateral(
                ctx.accounts.protocol_program.key(),
                collateral_amount,
                ctx.accounts.collateral_from.key(),
                ctx.accounts.withdraw_to.key(),
                ctx.accounts.refreshed_reserve_account.key(),
                ctx.accounts.collateral_mint.key(),
                ctx.accounts.withdraw_from_protocol_reserve.key(),
                ctx.accounts.lending_market_account.key(),
                ctx.accounts.lending_market_authority.key(),
            );

            invoke(&ins, &ctx.accounts.to_account_infos())?;

            msg!("withdraw_from_proxy end");
            
            Ok(())
        }
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

pub fn refresh_reserve<'info>(
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

pub fn deposit_reserve_liquidity<'info>(
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

    solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;

    Ok(())
}

pub fn redeem_reserve_collateral<'info>(
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

    solana_program::program::invoke_signed(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )?;

    Ok(())
}