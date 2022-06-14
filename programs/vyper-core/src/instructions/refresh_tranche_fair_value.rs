use anchor_lang::{prelude::*, solana_program::{self, hash::hashv, instruction::Instruction}};
use anchor_spl::token::Mint;
use boolinator::Boolinator;
use vyper_utils::redeem_logic_common::{RedeemLogicExecuteResult, RedeemLogicExecuteInput};
use crate::{state::{TrancheConfig, TrancheHaltFlags, OwnerRestrictedIxFlags}, errors::VyperErrorCode};

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
    pub tranche_config: Account<'info, TrancheConfig>,

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
        (!tranche_data.get_halt_flags().contains(TrancheHaltFlags::HALT_REFRESHES)).ok_or(VyperErrorCode::HaltError)?;
    
        // check if the current ix is restricted to owner
        if tranche_data.get_owner_restricted_ixs().contains(OwnerRestrictedIxFlags::REFRESHES) {
            require_keys_eq!(self.tranche_config.owner, self.signer.key(), VyperErrorCode::OwnerRestrictedIx)
        }

        Result::Ok(())
    }
}

pub fn handler(ctx: Context<RefreshTrancheFairValue>) -> Result<()> {

    // check if accounts are valid
    msg!("check if accounts are valid");
    ctx.accounts.are_valid()?;

    // retrieve exchange rate from rate_program
    msg!("deserializing rate state account");
    let account_data =  ctx.accounts.rate_program_state.try_borrow_data()?;
    let mut account_data_slice: &[u8] = &account_data;
    let rate_state = RateState::try_deserialize_unchecked(&mut account_data_slice)?;
    
    // TODO check rate_State not stale

    // get old and new reserve fair value
    let tranche_data = &mut ctx.accounts.tranche_config.tranche_data;
    let old_reserve_fair_value = tranche_data.reserve_fair_value.value;
    let new_reserve_fair_value = rate_state.fair_value;
    msg!("+ old_reserve_fair_value: {}", old_reserve_fair_value);
    msg!("+ new_reserve_fair_value: {}", new_reserve_fair_value);
    
    // call execute redeem logic plugin
    msg!("execute redeem logic CPI");
    let cpi_res = cpi_plugin(
        ctx.accounts.redeem_logic_program.key,
        ctx.accounts.redeem_logic_program_state.to_account_info(),
        RedeemLogicExecuteInput {
            old_reserve_fair_value_bps: old_reserve_fair_value,
            new_reserve_fair_value_bps: new_reserve_fair_value,
            old_quantity: tranche_data.deposited_quantity
        });
    let plugin_result = cpi_res.unwrap();

    msg!("cpi return result: {:?}", plugin_result);

    msg!("updating fee_to_collect_quantity");
    tranche_data.fee_to_collect_quantity = tranche_data.fee_to_collect_quantity.checked_add(plugin_result.fee_quantity).unwrap();

    msg!("updating deposited quantity");
    tranche_data.deposited_quantity = plugin_result.new_quantity;

    msg!("updating tranche fair value");
    if ctx.accounts.senior_tranche_mint.supply > 0 {
        tranche_data.tranche_fair_value.value[0] =
            tranche_data.deposited_quantity[0].checked_div(ctx.accounts.senior_tranche_mint.supply).unwrap().try_into().ok().unwrap()
    }
    if ctx.accounts.junior_tranche_mint.supply > 0 {
        tranche_data.tranche_fair_value.value[1] =
            tranche_data.deposited_quantity[1].checked_div(ctx.accounts.junior_tranche_mint.supply).unwrap().try_into().ok().unwrap()
    }
    tranche_data.tranche_fair_value.slot_tracking.update(rate_state.refreshed_slot);
    
    msg!("updating reserve fair value");
    tranche_data.reserve_fair_value.value = rate_state.fair_value;
    tranche_data.reserve_fair_value.slot_tracking.update(rate_state.refreshed_slot);

    Ok(())
}

#[account]
pub struct RateState {
    pub fair_value: u32,
    pub refreshed_slot: u64,
}

pub fn cpi_plugin(plugin_program: &Pubkey, plugin_state: AccountInfo, input_data: RedeemLogicExecuteInput) -> Result<RedeemLogicExecuteResult> {
    let mut data = hashv(&[b"global:execute"]).to_bytes()[..8].to_vec();
    data.append(&mut input_data.try_to_vec().unwrap());

    let account_metas = vec![
        AccountMeta::new_readonly(*plugin_state.key, false),
    ];

    let ix = Instruction::new_with_bytes(
        *plugin_program,
        &data,
        account_metas,
    );
    let account_infos = [
        plugin_state
    ];
    solana_program::program::invoke(&ix, &account_infos)?;

    let (program_key, serialized_result)= solana_program::program::get_return_data().unwrap();
    require_keys_eq!(program_key, *plugin_program);

    let mut serialized_result_slice: &[u8] = &serialized_result;
    RedeemLogicExecuteResult::deserialize(&mut serialized_result_slice).map_err(|_| VyperErrorCode::PluginCpiError.into())
}