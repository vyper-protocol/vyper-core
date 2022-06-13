use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, MintTo, Mint};
use boolinator::Boolinator;
use crate::{utils::{ Input }, state::{TrancheConfig, TrancheHaltFlags, OwnerRestrictedIxFlags}, errors::VyperErrorCode};

#[derive(Accounts)]
pub struct DepositContext<'info> {
    
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(mut, 
        // TODO check tranches mint
        has_one = junior_tranche_mint,
        has_one = senior_tranche_mint,
        has_one = reserve,
        has_one = tranche_authority)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// CHECK: 
    #[account(seeds = [tranche_config.key().as_ref(), b"authority".as_ref()], bump)]
    pub tranche_authority: AccountInfo<'info>,

    /// mint token A to deposit
    #[account(mut)]
    pub reserve: Box<Account<'info, TokenAccount>>,

    /// mint token A to deposit
    #[account(mut, token::mint = tranche_config.reserve_mint)]
    pub user_reserve_token: Box<Account<'info, TokenAccount>>,

    /// Senior tranche mint
    #[account(mut)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    /// Junior tranche mint
    #[account(mut)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    #[account(mut, token::mint = tranche_config.senior_tranche_mint)]
    pub senior_tranche_dest: Box<Account<'info, TokenAccount>>,

    #[account(mut, token::mint = tranche_config.junior_tranche_mint)]
    pub junior_tranche_dest: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> DepositContext<'info> {
    fn are_valid(&self) -> Result<()> {

        let clock = Clock::get()?;
        let tranche_data = &self.tranche_config.tranche_data;

        // check that deposits are not halted
        (!tranche_data.get_halt_flags().contains(TrancheHaltFlags::HALT_DEPOSITS)).ok_or(VyperErrorCode::HaltError)?;
    
        // check that tranche fair values are not halted
        (!tranche_data.tranche_fair_value.slot_tracking.is_stale(clock.slot)?).ok_or(VyperErrorCode::StaleFairValue)?;
    
        // check if the current ix is restricted to owner
        if tranche_data.get_owner_restricted_ixs().contains(OwnerRestrictedIxFlags::DEPOSITS) {
            require_keys_eq!(self.tranche_config.owner, self.signer.key(), VyperErrorCode::OwnerRestrictedIx)
        }

        Result::Ok(())
    }

    /// CpiContext for transferring reserve tokens from user to vault
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_reserve_token.to_account_info(),
                to: self.reserve.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

    /// CpiContext for minting senior tranches
    fn senior_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.senior_tranche_mint.to_account_info(),
                to: self.senior_tranche_dest.to_account_info(),
                authority: self.tranche_authority.clone(),
            },
        )
    }

    /// CpiContext for minting senior tranches
    fn junior_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.junior_tranche_mint.to_account_info(),
                to: self.junior_tranche_dest.to_account_info(),
                authority: self.tranche_authority.clone(),
            },
        )
    }
    
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct DepositInput {
    reserve_quantity: [u64; 2],
}

impl Input for DepositInput {
    fn is_valid(&self) -> Result<()> {

        if self.reserve_quantity.iter().sum::<u64>() == 0 {
            msg!("quantity must me greater than zero");
            return err!(VyperErrorCode::InvalidInput);
        }

        return Result::Ok(());
    }
}

pub fn handler(
    ctx: Context<DepositContext>,
    input_data: DepositInput,
) -> Result<()> {

    // check if accounts are valid
    msg!("check if accounts are valid");
    ctx.accounts.are_valid()?;

    // check if input is valid
    msg!("check if input is valid");
    input_data.is_valid()?;

    // increase deposited_quantity
    let tranche_data = &mut ctx.accounts.tranche_config.tranche_data;
    for i in 0..input_data.reserve_quantity.len() {
        tranche_data.deposited_quantity[i] = tranche_data.deposited_quantity[i].checked_add(input_data.reserve_quantity[i]).ok_or_else(|| VyperErrorCode::MathError)?;
    }

    // transfer token from source account to tranche config token account
    token::transfer(ctx.accounts.transfer_context(), input_data.reserve_quantity.iter().sum::<u64>())?;

    // mint tranches

    let mut mint_count: [u64; 2] = [0; 2];
    for i in 0..mint_count.len() {
        mint_count[i] = input_data.reserve_quantity[i].checked_div(ctx.accounts.tranche_config.tranche_data.tranche_fair_value.value[i].into()).ok_or_else(|| VyperErrorCode::MathError)?;
    }

    if mint_count[0] > 0 {
        msg!("mint {} senior tranches", mint_count[0]);
        token::mint_to(ctx.accounts.senior_mint_to_context().with_signer(&[&ctx.accounts.tranche_config.authority_seeds()]),mint_count[0])?;
    }

    if mint_count[1] > 0 {
        msg!("mint {} junior tranches", mint_count[1]);
        token::mint_to(ctx.accounts.junior_mint_to_context().with_signer(&[&ctx.accounts.tranche_config.authority_seeds()]),mint_count[1])?;
    }

    Ok(())
}