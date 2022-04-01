use anchor_lang::prelude::*;
use vyper_core_lending::{
    instructions::deposit::DepositVyperProxyLending,
    instructions::redeem::WithdrawVyperProxyLending,
    interface_context::{
        DepositProxyLendingContext,
        WithdrawProxyLendingContext
    }
};
// use solana_program::program::invoke;
// use spl_token_lending::*;

declare_id!("9R88Mc2NBfhaxozbdwSHajAT94UUwe2ExrALq3FZK11L");

#[program]
pub mod proxy_lending_solend {
    use super::*;

    #[state]
    pub struct ProxyLendingSolend;

    impl<'info> DepositVyperProxyLending<'info, DepositProxyLendingContext<'info>> for ProxyLendingSolend {
        fn deposit_to_proxy(
            _ctx: Context<DepositProxyLendingContext>,
            _vault_authority_bump: u8,
            _amount: u64,
        ) -> ProgramResult {
            msg!("deposit_to_proxy begin");
            
            msg!("DEPOSIT TO SOLEND");
            // let ins = spl_token_lending::instruction::deposit_reserve_liquidity(
            //     ctx.accounts.protocol_program.key(),
            //     amount,
            //     ctx.accounts.deposit_from.key(),
            //     ctx.accounts.collateral_token_account.key(),
            //     ctx.accounts.protocol_state.key(),
            //     ctx.accounts.deposit_to_protocol_reserve.key(),
            //     ctx.accounts.collateral_mint.key(),
            //     ctx.accounts.lending_market_account.key(),
            //     ctx.accounts.lending_market_authority.key(),
            // );

            // invoke(&ins, &ctx.accounts.to_account_infos())?;

            msg!("deposit_to_proxy end");

            Ok(())
        }
    }

    impl<'info> WithdrawVyperProxyLending<'info, WithdrawProxyLendingContext<'info>> for ProxyLendingSolend {
        fn withdraw_from_proxy(
            _ctx: Context<WithdrawProxyLendingContext>,
            _vault_authority_bump: u8,
            _collateral_amount: u64,
        ) -> ProgramResult {
            msg!("withdraw_from_proxy begin");
            
            msg!("WOTHDRAW FROM SOLEND");
            // let ins = spl_token_lending::instruction::redeem_reserve_collateral(
            //     ctx.accounts.protocol_program.key(),
            //     collateral_amount,
            //     ctx.accounts.collateral_from.key(),
            //     ctx.accounts.withdraw_to.key(),
            //     ctx.accounts.refreshed_reserve_account.key(),
            //     ctx.accounts.collateral_mint.key(),
            //     ctx.accounts.withdraw_from_protocol_reserve.key(),
            //     ctx.accounts.lending_market_account.key(),
            //     ctx.accounts.lending_market_authority.key(),
            // );

            // invoke(&ins, &ctx.accounts.to_account_infos())?;

            msg!("withdraw_from_proxy end");
            
            Ok(())
        }
    }
}