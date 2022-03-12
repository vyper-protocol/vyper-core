use anchor_lang::prelude::*;
use proxy_interface::*;
use solana_program::program::invoke;
use spl_token_lending::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod proxy_solend {
    use super::*;
    pub struct ProxySolend;

    impl<'info> VyperProxy<'info> for ProxySolend {
        fn deposit_to_proxy(
            ctx: Context<DepositProxyContext>,
            vault_authority_bump: u8,
            amount: u64,
        ) -> ProgramResult {
            let ins = spl_token_lending::instruction::deposit_reserve_liquidity(
                ctx.accounts.protocol_program.key(),
                amount,
                ctx.accounts.deposit_from.key(),
                ctx.accounts.collateral_token_account.key(),
                ctx.accounts.protocol_state.key(),
                ctx.accounts.deposit_to_protocol_reserve.key(),
                ctx.accounts.collateral_mint.key(),
                ctx.accounts.lending_market_account.key(),
                ctx.accounts.lending_market_authority.key(),
            );

            invoke(&ins, &ctx.accounts.to_account_infos())?;
            Ok(())
        }

        fn withdraw_from_proxy(
            ctx: Context<WithdrawProxyContext>,
            vault_authority_bump: u8,
            collateral_amount: u64,
        ) -> ProgramResult {
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
            Ok(())
        }
    }
}
