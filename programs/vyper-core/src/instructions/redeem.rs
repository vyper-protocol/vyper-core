use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint, Burn};
use boolinator::Boolinator;
use crate::{utils::{ Input }, state::{TrancheConfig, TrancheHaltFlags, OwnerRestrictedIxFlags}, errors::VyperErrorCode};

#[derive(Accounts)]
pub struct RedeemContext<'info> {
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(mut, 
        // constraint = !vault.value.last_update.is_stale(clock.slot)? @ ErrorCode::VaultIsNotRefreshed,
        has_one = reserve,
        has_one = tranche_authority)]
    pub tranche_config: Account<'info, TrancheConfig>,

    /// CHECK: 
    #[account(seeds = [tranche_config.key().as_ref(), b"authority".as_ref()], bump)]
    pub tranche_authority: AccountInfo<'info>,

    /// mint token A to deposit
    #[account(mut)]
    pub reserve: Account<'info, TokenAccount>,

    /// mint token A to deposit
    #[account(mut, token::mint = tranche_config.reserve_mint)]
    pub user_reserve_token: Account<'info, TokenAccount>,

    /// Senior tranche mint
    #[account(mut)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    /// Junior tranche mint
    #[account(mut)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    #[account(mut, token::mint = tranche_config.senior_tranche_mint)]
    pub senior_tranche_source: Account<'info, TokenAccount>,

    #[account(mut, token::mint = tranche_config.junior_tranche_mint)]
    pub junior_tranche_source: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> RedeemContext<'info> {

    fn are_valid(&self) -> Result<()> {

        let clock = Clock::get()?;
        let tranche_data = &self.tranche_config.tranche_data;

        // check that deposits are not halted
        (!tranche_data.get_halt_flags().contains(TrancheHaltFlags::HALT_REDEEMS)).ok_or(VyperErrorCode::HaltError)?;
    
        // check that tranche fair values are not halted
        (!tranche_data.tranche_fair_value.slot_tracking.is_stale(clock.slot)?).ok_or(VyperErrorCode::StaleFairValue)?;
    
        // check if the current ix is restricted to owner
        if tranche_data.get_owner_restricted_ixs().contains(OwnerRestrictedIxFlags::REDEEMS) {
            require_keys_eq!(self.tranche_config.owner, self.signer.key(), VyperErrorCode::OwnerRestrictedIx)
        }

        Result::Ok(())
    }

    /// CpiContext for transferring reserve tokens from user to vault
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.reserve.to_account_info(),
                to: self.user_reserve_token.to_account_info(),
                authority: self.tranche_authority.to_account_info(),
            },
        )
    }

    /// CpiContext for burning senior tranches
    fn senior_burn_to_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.senior_tranche_mint.to_account_info(),
                from: self.senior_tranche_source.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

    /// CpiContext for burning senior tranches
    fn junior_burn_to_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.junior_tranche_mint.to_account_info(),
                from: self.junior_tranche_source.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }
    
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct RedeemInput {
    tranche_quantity: [u64; 2],
}

impl Input for RedeemInput {
    fn is_valid(&self) -> Result<()> {

        if self.tranche_quantity.iter().sum::<u64>() == 0 {
            msg!("quantity must me greater than zero");
            return err!(VyperErrorCode::InvalidInput);
        }

        return Result::Ok(());
    }
}

pub fn handler(
    ctx: Context<RedeemContext>,
    input_data: RedeemInput,
) -> Result<()> {
    msg!("redeem begin");

    // check if accounts are valid
    msg!("check if accounts are valid");
    ctx.accounts.are_valid()?;

    // check input
    msg!("check if input is valid");
    input_data.is_valid()?;

    // decrease deposited_quantity
    let tranche_data = &mut ctx.accounts.tranche_config.tranche_data;
    for i in 0..input_data.tranche_quantity.len() {
        tranche_data.deposited_quantity[i] = tranche_data.deposited_quantity[i].checked_sub(input_data.tranche_quantity[i]).ok_or_else(|| VyperErrorCode::MathError)?;
    }

    // transfer token from tranche config token account to source account
    token::transfer(ctx.accounts.transfer_context().with_signer(&[&ctx.accounts.tranche_config.authority_seeds()]), input_data.tranche_quantity.iter().sum::<u64>())?;     

    // burn tranches
    let mut burn_mint_count: [u64; 2] = [0; 2];
    for i in 0..burn_mint_count.len() {
        burn_mint_count[i] = input_data.tranche_quantity[i].checked_div(ctx.accounts.tranche_config.tranche_data.tranche_fair_value.value[i]).ok_or_else(|| VyperErrorCode::MathError)?;
    }

    if burn_mint_count[0] > 0 {
        msg!("burn {} senior tranches", burn_mint_count[0]);
        token::burn(ctx.accounts.senior_burn_to_context(), burn_mint_count[0])?;
    }

    if burn_mint_count[1] > 0 {
        msg!("burn {} junior tranches", burn_mint_count[1]);
        token::burn(ctx.accounts.junior_burn_to_context(),burn_mint_count[1])?;
    }

    Ok(())
}