use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount};

declare_id!("2heoT1tfJb5ayc8VA3WGrRgW9fiDtnjLuzRXpiYv1KXJ");

#[program]
pub mod mock_protocol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, vault_bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, quantity: u64, vault_bump: u8) -> ProgramResult {
        let cc = token::Transfer {
            from: ctx.accounts.src_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), cc), quantity)?;

        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>, quantity: u64, vault_bump: u8) -> ProgramResult {

        let cc = token::Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.dest_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        token::transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), cc), quantity)?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(vault_bump: u8)]
pub struct Initialize<'info> {

    #[account()]
    pub mint: Account<'info, Mint>,
    
    #[account(
        init,
        seeds = [b"my-token-seed".as_ref(), mint.key().as_ref()],
        bump = vault_bump,
        payer = authority,
        token::mint = mint,
        token::authority = authority,
    )]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(quantity: u64, vault_bump: u8)]
pub struct Deposit<'info> {
    #[account()]
    pub mint: Account<'info, Mint>,
    
    #[account(mut, seeds = [b"my-token-seed".as_ref(), mint.key().as_ref()], bump = vault_bump)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub src_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(quantity: u64, vault_bump: u8)]
pub struct Redeem<'info> {
    #[account()]
    pub mint: Account<'info, Mint>,
    
    #[account(mut, seeds = [b"my-token-seed".as_ref(), mint.key().as_ref()], bump = vault_bump)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub dest_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}