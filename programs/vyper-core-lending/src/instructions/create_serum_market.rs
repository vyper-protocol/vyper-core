use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    dex,
    token::{ Mint, Token, TokenAccount },
};

#[derive(Accounts)]
pub struct CreateSerumMarketContext<'info> {
    /**
     * Signer account
     */
    #[account(mut)]
    pub authority: Signer<'info>,

    // Senior tranche mint
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    // Senior tranche serum vault
    #[account(mut)]
    pub senior_tranche_serum_vault: Box<Account<'info, TokenAccount>>,

    // Junior tranche mint
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    // Junior tranche serum vault
    #[account(mut)]
    pub junior_tranche_serum_vault: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub usdc_serum_vault: Box<Account<'info, TokenAccount>>,

    // * * * * * * * * * * * * * * * * *

    // serum accounts
    #[account(mut)]
    pub market: Signer<'info>,
    #[account(mut)]
    pub request_queue: Signer<'info>,
    #[account(mut)]
    pub event_queue: Signer<'info>,
    #[account(mut)]
    pub asks: Signer<'info>,
    #[account(mut)]
    pub bids: Signer<'info>,

    /// CHECK: Safe
    pub serum_dex: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateSerumMarketContext>,
    vault_signer_nonce: u8,
) -> ProgramResult {
    // * * * * * * * * * * * * * * * * * * * * * * *
    // initialize market on serum

    msg!("initialize market on serum");

    let initialize_market_ctx = dex::InitializeMarket {
        market: ctx.accounts.market.to_account_info().clone(),
        coin_mint: ctx.accounts.junior_tranche_mint.to_account_info().clone(),
        coin_vault: ctx
            .accounts
            .junior_tranche_serum_vault
            .to_account_info()
            .clone(),
        bids: ctx.accounts.bids.to_account_info().clone(),
        asks: ctx.accounts.asks.to_account_info().clone(),
        req_q: ctx.accounts.request_queue.to_account_info().clone(),
        event_q: ctx.accounts.event_queue.to_account_info().clone(),
        rent: ctx.accounts.rent.to_account_info().clone(),
        pc_mint: ctx.accounts.usdc_mint.to_account_info().clone(),
        pc_vault: ctx.accounts.usdc_serum_vault.to_account_info().clone(),
    };

    dex::initialize_market(
        CpiContext::new(
            ctx.accounts.serum_dex.to_account_info().clone(),
            initialize_market_ctx,
        ),
        100_000,
        100,
        vault_signer_nonce.into(),
        500,
    )?;

    Ok(())
}