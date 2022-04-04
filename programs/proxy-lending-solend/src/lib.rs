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
            vault_authority_bump: u8,
            amount: u64,
        ) -> ProgramResult {
            msg!("deposit_to_proxy begin");
            // for i in 0..ctx.accounts.to_account_infos().len() {
            //     msg!("+ ctx.accounts.to_account_infos()[{}]: {}", i, ctx.accounts.to_account_infos()[i].key());
            // }
            for c in ctx.accounts.to_account_infos() {
                msg!("+ ctx.accounts.to_account_infos(): {}", c.key());
            }

            // msg!("protocol_program: {}", ctx.accounts.protocol_program.key());
            // msg!("deposit_from: {}", ctx.accounts.deposit_from.key());
            // msg!("collateral_token_account: {}", ctx.accounts.collateral_token_account.key());
            // msg!("protocol_state: {}", ctx.accounts.protocol_state.key());
            // msg!("deposit_to_protocol_reserve: {}", ctx.accounts.deposit_to_protocol_reserve.key());
            // msg!("collateral_mint: {}", ctx.accounts.collateral_mint.key());
            // msg!("lending_market_account: {}", ctx.accounts.lending_market_account.key());
            // msg!("lending_market_authority: {}", ctx.accounts.lending_market_authority.key());

            // msg!("DEPOSIT TO SOLEND");

            // let ix = spl_token_lending::instruction::deposit_reserve_liquidity(
            //     ctx.accounts.protocol_program.key(), // *ctx.accounts.lending_program.key,
            //     amount, // liquidity_amount,
            //     ctx.accounts.deposit_from.key(), // *ctx.accounts.source_liquidity.key,
            //     ctx.accounts.collateral_token_account.key(), // *ctx.accounts.destination_collateral_account.key,
            //     ctx.accounts.protocol_state.key(), // *ctx.accounts.reserve.key,
            //     ctx.accounts.deposit_to_protocol_reserve.key(), // *ctx.accounts.reserve_liquidity_supply.key,
            //     ctx.accounts.collateral_mint.key(), // *ctx.accounts.reserve_collateral_mint.key,
            //     ctx.accounts.lending_market_account.key(), // *ctx.accounts.lending_market.key,
            //     ctx.accounts.lending_market_authority.key(), // *ctx.accounts.transfer_authority.key,
            // );
        
            // solana_program::program::invoke(&ix, &ctx.accounts.to_account_infos())?;

            let new_context = CpiContext::new(
                ctx.accounts.protocol_program.clone(),
                DepositReserveLiquidity {
                    lending_program: ctx.accounts.protocol_program.clone(),
                    source_liquidity: ctx.accounts.deposit_from.to_account_info(),
                    destination_collateral_account: ctx.accounts.collateral_token_account.to_account_info(),
                    reserve: ctx.accounts.protocol_state.to_account_info(),
                    reserve_collateral_mint: ctx.accounts.collateral_mint.to_account_info(),
                    reserve_liquidity_supply: ctx.accounts.collateral_token_account.to_account_info(),
                    lending_market: ctx.accounts.lending_market_account.clone(),
                    lending_market_authority: ctx.accounts.lending_market_authority.clone(),
                    transfer_authority: ctx.accounts.vault_authority.clone(),
                    clock: ctx.accounts.clock.to_account_info(),
                    token_program_id: ctx.accounts.token_program.to_account_info(),
                },
            );

            // let tranche_config_key = ctx.accounts.tranche_config.key();
            // let seeds = &[
            //     b"vault_authority".as_ref(),
            //     tranche_config_key.as_ref(),
            //     &[vault_authority_bump]
            // ];
            // let signer = &[&seeds[..]];

            match amount {
                0 => Ok(()),
                _ => deposit_reserve_liquidity(
                    new_context, //.with_signer(signer),
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
    /// CHECK: Safe
    pub lending_program: AccountInfo<'info>,
    // Token account for asset to deposit into reserve
    /// CHECK: Safe
    pub source_liquidity: AccountInfo<'info>,
    // Token account for reserve collateral token
    /// CHECK: Safe
    pub destination_collateral_account: AccountInfo<'info>,
    // Reserve state account
    /// CHECK: Safe
    pub reserve: AccountInfo<'info>,
    // Token mint for reserve collateral token
    /// CHECK: Safe
    pub reserve_collateral_mint: AccountInfo<'info>,
    // Reserve liquidity supply SPL token account
    /// CHECK: Safe
    pub reserve_liquidity_supply: AccountInfo<'info>,
    // Lending market account
    /// CHECK: Safe
    pub lending_market: AccountInfo<'info>,
    // Lending market authority (PDA)
    /// CHECK: Safe
    pub lending_market_authority: AccountInfo<'info>,
    // Transfer authority for accounts 1 and 2
    /// CHECK: Safe
    pub transfer_authority: AccountInfo<'info>,
    // Clock
    /// CHECK: Safe
    pub clock: AccountInfo<'info>,
    // Token program ID
    /// CHECK: Safe
    pub token_program_id: AccountInfo<'info>,
}

pub fn deposit_reserve_liquidity<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, DepositReserveLiquidity<'info>>,
    liquidity_amount: u64,
) -> ProgramResult {

    msg!("deposit_reserve_liquidity begin");
    for c in ctx.accounts.to_account_infos() {
        msg!("+ ctx.accounts.to_account_infos(): {}", c.key());
    }

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

    msg!("to account infos");
    for c in ToAccountInfos::to_account_infos(&ctx) {
        msg!("+ to account infos: {}", c.key());
    }

    solana_program::program::invoke(
        &ix,
        &ToAccountInfos::to_account_infos(&ctx),
        // ctx.signer_seeds,
    )?;

    Ok(())
}