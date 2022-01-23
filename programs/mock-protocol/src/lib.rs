use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod mock_protocol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, vault_bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn simulate_interest(ctx: Context<SimulateInterest>, quantity: u64) -> ProgramResult {
        
        let cc = token::Transfer {
            from: ctx.accounts.source_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.source_account_authority.to_account_info(),
        };
        token::transfer(CpiContext::new(ctx.accounts.token_program.to_account_info(), cc), quantity)?;

        Ok(())
    }

    pub fn simulate_hack(ctx: Context<SimulateHack>, quantity: u64) -> ProgramResult {

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
pub struct SimulateInterest<'info> {
    #[account()]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub source_account: Account<'info, TokenAccount>,
    
    #[account()]
    pub source_account_authority: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SimulateHack<'info> {
    #[account()]
    pub mint: Account<'info, Mint>,
    
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub dest_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}