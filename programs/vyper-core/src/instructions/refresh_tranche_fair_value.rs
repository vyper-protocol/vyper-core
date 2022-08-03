use crate::{
    errors::VyperErrorCode,
    state::{OwnerRestrictedIxFlags, TrancheConfig, TrancheHaltFlags},
};
use anchor_lang::{
    prelude::*,
    solana_program::{self, hash::hashv, instruction::Instruction},
};
use anchor_spl::token::Mint;
use boolinator::Boolinator;
use rust_decimal::Decimal;
use vyper_utils::redeem_logic_common::{RedeemLogicExecuteInput, RedeemLogicExecuteResult};

#[derive(Accounts)]
pub struct RefreshTrancheFairValue<'info> {
    pub signer: Signer<'info>,

    #[account(mut,
        has_one = rate_program_state,
        has_one = redeem_logic_program,
        has_one = redeem_logic_program_state,
        has_one = senior_tranche_mint,
        has_one = junior_tranche_mint,
    )]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// Senior tranche mint
    #[account(mut)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    /// Junior tranche mint
    #[account(mut)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    /// CHECK:
    pub rate_program_state: AccountInfo<'info>,
    /// CHECK:
    pub redeem_logic_program: AccountInfo<'info>,
    /// CHECK:
    pub redeem_logic_program_state: AccountInfo<'info>,
}

impl<'info> RefreshTrancheFairValue<'info> {
    fn are_valid(&self) -> Result<()> {
        let tranche_data = &self.tranche_config.tranche_data;

        // check that deposits are not halted
        (!tranche_data
            .get_halt_flags()?
            .contains(TrancheHaltFlags::HALT_REFRESHES))
        .ok_or(VyperErrorCode::HaltError)?;

        // check if the current ix is restricted to owner
        if tranche_data
            .get_owner_restricted_ixs()?
            .contains(OwnerRestrictedIxFlags::REFRESHES)
        {
            require_keys_eq!(
                self.tranche_config.owner,
                self.signer.key(),
                VyperErrorCode::OwnerRestrictedIx
            )
        }

        Result::Ok(())
    }
}

pub fn handler(ctx: Context<RefreshTrancheFairValue>) -> Result<()> {
    let clock = Clock::get()?;

    // check if accounts are valid
    msg!("check if accounts are valid");
    ctx.accounts.are_valid()?;

    let tranche_data = &mut ctx.accounts.tranche_config.tranche_data;

    // retrieve exchange rate from rate_program
    msg!("deserializing rate state account");
    let account_data = ctx.accounts.rate_program_state.try_borrow_data()?;
    let mut account_data_slice: &[u8] = &account_data;
    let rate_state = RateState::try_deserialize_unchecked(&mut account_data_slice)?;

    // check if rate state is stale
    let elapsed_slot = clock
        .slot
        .checked_sub(rate_state.refreshed_slot)
        .ok_or(VyperErrorCode::MathError)?;
    if elapsed_slot
        >= tranche_data
            .reserve_fair_value
            .slot_tracking
            .stale_slot_threshold
    {
        return err!(VyperErrorCode::StaleFairValue);
    }

    // get old and new reserve fair value
    let old_reserve_fair_value = tranche_data.reserve_fair_value.value;
    let new_reserve_fair_value = rate_state.fair_value;
    msg!("+ old_reserve_fair_value: {:?}", old_reserve_fair_value);
    msg!("+ new_reserve_fair_value: {:?}", new_reserve_fair_value);
    msg!(
        "+ tranche_data.deposited_quantity: {:?}",
        tranche_data.deposited_quantity
    );

    // call execute redeem logic plugin
    msg!("execute redeem logic CPI");
    let cpi_res = cpi_plugin(
        ctx.accounts.redeem_logic_program.key,
        ctx.accounts.redeem_logic_program_state.to_account_info(),
        RedeemLogicExecuteInput {
            old_reserve_fair_value: old_reserve_fair_value,
            new_reserve_fair_value: new_reserve_fair_value,
            old_quantity: tranche_data.deposited_quantity,
        },
    );
    let plugin_result = cpi_res?;
    msg!("cpi return result: {:?}", plugin_result);

    msg!("updating fee_to_collect_quantity...");
    tranche_data.fee_to_collect_quantity = tranche_data
        .fee_to_collect_quantity
        .checked_add(plugin_result.fee_quantity)
        .ok_or(VyperErrorCode::MathError)?;

    msg!("updating deposited quantity...");
    tranche_data.deposited_quantity = plugin_result.new_quantity;

    msg!("updating tranche fair value...");
    if ctx.accounts.senior_tranche_mint.supply > 0 {
        let dep_qty = Decimal::from(tranche_data.deposited_quantity[0]);
        let supply = Decimal::from(ctx.accounts.senior_tranche_mint.supply);
        let fair_value = dep_qty / supply;
        #[cfg(feature = "debug")]
        {
            msg!("senior dep qty: {:?}", dep_qty);
            msg!("senior supply: {:?}", supply);
            msg!("senior fair value: {:?}", fair_value);
        }
        tranche_data.tranche_fair_value.value[0] = fair_value;
    }
    if ctx.accounts.junior_tranche_mint.supply > 0 {
        let dep_qty = Decimal::from(tranche_data.deposited_quantity[1]);
        let supply = Decimal::from(ctx.accounts.junior_tranche_mint.supply);
        let fair_value = dep_qty / supply;
        #[cfg(feature = "debug")]
        {
            msg!("junior dep qty: {:?}", dep_qty);
            msg!("junior supply: {:?}", supply);
            msg!("junior fair value: {:?}", fair_value);
        }
        tranche_data.tranche_fair_value.value[1] = fair_value;
    }
    msg!(
        "tranche fair value: {:?}",
        tranche_data.tranche_fair_value.value
    );
    tranche_data
        .tranche_fair_value
        .slot_tracking
        .update(rate_state.refreshed_slot);

    msg!("updating reserve fair value...");
    tranche_data.reserve_fair_value.value = rate_state.fair_value;
    tranche_data
        .reserve_fair_value
        .slot_tracking
        .update(rate_state.refreshed_slot);

    Ok(())
}

#[account]
pub struct RateState {
    pub fair_value: [Decimal; 10],
    pub refreshed_slot: u64,
}

pub fn cpi_plugin(
    plugin_program: &Pubkey,
    plugin_state: AccountInfo,
    input_data: RedeemLogicExecuteInput,
) -> Result<RedeemLogicExecuteResult> {
    let mut data = hashv(&[b"global:execute"]).to_bytes()[..8].to_vec();
    data.append(&mut input_data.try_to_vec()?);

    let account_metas = vec![AccountMeta::new_readonly(*plugin_state.key, false)];

    let ix = Instruction::new_with_bytes(*plugin_program, &data, account_metas);
    let account_infos = [plugin_state];
    solana_program::program::invoke(&ix, &account_infos)?;

    let (program_key, serialized_result) =
        solana_program::program::get_return_data().ok_or(VyperErrorCode::MathError)?;
    require_keys_eq!(program_key, *plugin_program);

    let mut serialized_result_slice: &[u8] = &serialized_result;
    RedeemLogicExecuteResult::deserialize(&mut serialized_result_slice)
        .map_err(|_| VyperErrorCode::PluginCpiError.into())
}
