use anchor_lang::prelude::*;
use anchor_spl::{
    token::{
        Token,
        Mint,
        TokenAccount
    },
    associated_token::AssociatedToken
};

#[interface]
pub trait RefreshReserveVyperProxyLending<'info, T: Accounts<'info>> {
    fn refresh_reserve(ctx: Context<T>) -> ProgramResult;
}

#[derive(Accounts)]
pub struct RefreshReserveProxyLendingContext<'info> {
    // Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Safe
    #[account(executable)]
    pub lending_program: AccountInfo<'info>,

    /// CHECK:
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    /// CHECK: Safe
    pub pyth_reserve_liquidity_oracle: AccountInfo<'info>,

    /// CHECK: Safe
    pub switchboard_reserve_liquidity_oracle: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct DepositProxyLendingContext<'info> {
    
    // Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    // Protocol Program
    /// CHECK: Safe
    #[account(executable)]
    pub lending_program: AccountInfo<'info>,

    // Token account that is depositing the amount
    #[account(mut)]
    pub source_liquidity: Box<Account<'info, TokenAccount>>,

    // Token account that holds the reserves of the protocol
    #[account(mut)]
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,

    // Token mint for depositing token
    #[account()]
    pub reserve_token: Box<Account<'info, Mint>>,

    // Token account for receiving collateral token (as proof of deposit)
    // TODO: init_if_needed
    #[account(mut)]
    pub destination_collateral_account: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    // State account for protocol
    /// CHECK: Safe
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    // Lending market account
    /// CHECK: Safe
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    /// CHECK: Safe
    pub lending_market_authority: AccountInfo<'info>,

    /// CHECK: Safe
    pub pyth_reserve_liquidity_oracle: AccountInfo<'info>,

    /// CHECK: Safe
    pub switchboard_reserve_liquidity_oracle: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct WithdrawProxyLendingContext<'info> {

    // Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    // Protocol Program
    /// CHECK: Safe
    #[account(executable)]
    pub lending_program: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK:
    pub source_collateral: AccountInfo<'info>,
    
    
    #[account(mut)]
    /// CHECK:
    pub destination_liquidity: AccountInfo<'info>,

    // Token account that holds the reserves of the protocol
    /// CHECK:
    #[account(mut)]
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,

    // Token mint for depositing token
    /// CHECK:
    #[account()]
    pub reserve_token: Box<Account<'info, Mint>>,

    // Token account for receiving collateral token (as proof of deposit)
    /// CHECK:
    #[account(mut)]
    pub source_collateral_account: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    /// CHECK:
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    // State account for protocol
    /// CHECK: Safe
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    // Lending market account
    /// CHECK: Safe
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    /// CHECK: Safe
    pub lending_market_authority: AccountInfo<'info>,

    /// CHECK: Safe
    pub pyth_reserve_liquidity_oracle: AccountInfo<'info>,

    /// CHECK: Safe
    pub switchboard_reserve_liquidity_oracle: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

