use crate::adapters::solend::*;
use crate::state::TrancheConfig;
use anchor_lang::prelude::*;
use anchor_spl::{self, token::TokenAccount};
use spl_token_lending::state::CollateralExchangeRate;
use std::cmp;
use vyper_utils::math::from_bps;
use vyper_utils::math::to_bps;

#[derive(Accounts)]
pub struct UpdateDepositedQuantityContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = tranche_config.authority == *authority.key)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    #[account(mut)]
    pub protocol_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Safe
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    #[account(mut)]
    pub source_collateral_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
}

impl<'info> UpdateDepositedQuantityContext<'info> {
    fn get_collateral_exchange_rate(&self) -> Result<CollateralExchangeRate, ProgramError> {
        // let reserve: spl_token_lending::state::Reserve = ctx.accounts.protocol_state.deserialize_data().map_err(|_| ProgramError::BorshIoError)?;
        // let reserve: SolendReserve = SolendReserve::try_deserialize(ctx.accounts.protocol_state.data)
        let protocol_state_clone = self.protocol_state.clone();
        let mut reserve_data: &[u8] = &protocol_state_clone.try_borrow_mut_data()?;
        let reserve: SolendReserve = SolendReserve::try_deserialize(&mut reserve_data)?;
        reserve.collateral_exchange_rate()
    }
}

pub fn handler(ctx: Context<UpdateDepositedQuantityContext>) -> ProgramResult {
    let collateral_exchange_rate = ctx.accounts.get_collateral_exchange_rate()?;
    let reserve_token_in_reserve = collateral_exchange_rate
        .collateral_to_liquidity(ctx.accounts.source_collateral_account.amount)?;

    msg!("reserve_token_in_reserve: {}", reserve_token_in_reserve);

    // update as in the redeem method capital split

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

    let [senior_total, junior_total] = if interest_to_redeem > 0 {
        let s_tot = ctx.accounts.tranche_config.get_total_deposited_quantity() as f64
            * capital_split_f[0]
            + interest_to_redeem as f64 * capital_split_f[0] * (1.0 as f64 - interest_split_f[0]);
        [
            s_tot as u64,
            (ctx.accounts.tranche_config.get_total_deposited_quantity() as f64
                + interest_to_redeem as f64
                - s_tot) as u64,
        ]
    } else {
        let s_tot = cmp::min(
            (ctx.accounts.tranche_config.get_total_deposited_quantity() as f64 * capital_split_f[0])
                as u64,
            capital_to_redeem,
        ) as u64;
        [s_tot, capital_to_redeem - s_tot]
    };

    ctx.accounts.tranche_config.deposited_quantiy[0] = senior_total;
    ctx.accounts.tranche_config.deposited_quantiy[1] = junior_total;

    ctx.accounts.tranche_config.capital_split[0] =
        to_bps(senior_total as f64 / (senior_total as f64 + junior_total as f64));
    ctx.accounts.tranche_config.capital_split[1] =
        10_000 - ctx.accounts.tranche_config.capital_split[0];

    Ok(())
}
