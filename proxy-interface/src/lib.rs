use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::*;
use vyper::ID as vyper_id;

#[interface]
pub trait VyperProxy<'info> {
    fn deposit_to_proxy(
        ctx: Context<DepositProxyContext<'info>>,
        vault_authority_bump: u8,
        amount: u64,
    ) -> Result<()>;

    fn withdraw_from_proxy(
        ctx: Context<WithdrawProxyContext<'info>>,
        vault_authority_bump: u8,
        collateral_amount: u64,
    ) -> Result<()>;
}

#[derive(Accounts)]
#[instruction(vault_authority_bump: u8)]
pub struct DepositProxyContext<'info> {
    // Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    // Vyper Vault authority
    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority_bump,
        seeds::program = vyper_id,
    )]
    pub vault_authority: Signer<'info>,

    // Protocol Program
    #[account(executable)]
    pub protocol_program: AccountInfo<'info>,

    // Token account that is depositing the amount
    #[account(mut, associated_token::mint = deposit_mint, associated_token::authority = vault_authority)]
    pub deposit_from: Box<Account<'info, TokenAccount>>,

    // Token account that holds the reserves of the protocol
    #[account(mut, associated_token::mint = deposit_mint, associated_token::authority = lending_market_authority)]
    pub deposit_to_protocol_reserve: Box<Account<'info, TokenAccount>>,

    // Token mint for depositing token
    #[account(mut)]
    pub deposit_mint: Box<Account<'info, Mint>>,

    // Token account for receiving collateral token (as proof of deposit)
    #[account(init_if_needed, associated_token::mint = collateral_mint, associated_token::authority = vault_authority, payer = authority)]
    pub collateral_token_account: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    // State account for protocol
    #[account(mut)]
    pub protocol_state: AccountInfo<'info>,

    // Lending market account
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    pub lending_market_authority: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction(vault_authority_bump: u8)]
pub struct WithdrawProxyContext<'info> {
    // Signer account
    #[account(mut)]
    pub authority: Signer<'info>,

    // Vyper Vault authority
    #[account(
        seeds = [b"vault_authority"],
        bump = vault_authority_bump,
        seeds::program = vyper_id,
    )]
    pub vault_authority: Signer<'info>,

    // Protocol Program
    #[account(executable)]
    pub protocol_program: AccountInfo<'info>,

    // Token account that is withdrawing the amount
    #[account(mut, associated_token::mint = withdraw_mint, associated_token::authority = vault_authority)]
    pub withdraw_to: Box<Account<'info, TokenAccount>>,

    // Token account that holds the reserves of the protocol
    #[account(mut, associated_token::mint = withdraw_mint, associated_token::authority = lending_market_authority)]
    pub withdraw_from_protocol_reserve: Box<Account<'info, TokenAccount>>,

    // Token mint for withdrawing token
    #[account(mut)]
    pub withdraw_mint: Box<Account<'info, Mint>>,

    // Token account for sending collateral token (as proof of deposit)
    #[account(mut, associated_token::mint = collateral_mint, associated_token::authority = vault_authority)]
    pub collateral_from: Box<Account<'info, TokenAccount>>,

    // SPL token mint for collateral token
    #[account(mut)]
    pub collateral_mint: Box<Account<'info, Mint>>,

    // Refreshed reserve account
    pub refreshed_reserve_account: AccountInfo<'info>,

    // Lending market account
    pub lending_market_account: AccountInfo<'info>,

    // Lending market authority (PDA)
    pub lending_market_authority: AccountInfo<'info>,

    // * * * * * * * * * * * * * * * * *
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}
