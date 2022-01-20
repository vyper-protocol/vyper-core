use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod mock_protocol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, vault_bump: u8) -> ProgramResult {
        Ok(())
    }

    pub fn simulate_interest(ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }

    pub fn simulate_hack(ctx: Context<Initialize>) -> ProgramResult {
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
