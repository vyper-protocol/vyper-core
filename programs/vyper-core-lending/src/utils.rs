use anchor_lang::prelude::*;
use {
    crate::*,
    anchor_lang::prelude::{AccountInfo, ProgramResult},
};

/// TokenTransfer
pub struct TokenTransferParams<'a: 'b, 'b> {
    pub from: AccountInfo<'a>,
    pub to: AccountInfo<'a>,
    pub amount: u64,
    pub authority: AccountInfo<'a>,
    pub authority_signer_seeds: &'b [&'b [u8]],
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        from,
        to,
        amount,
        authority,
        authority_signer_seeds,
        token_program,
    } = params;

    let transfer_ctx = token::Transfer {
        from: from,
        to: to,
        authority: authority,
    };
    token::transfer(
        CpiContext::new_with_signer(token_program, transfer_ctx, &[authority_signer_seeds]),
        amount,
    )?;

    Ok(())
}

/// TokenMint
pub struct TokenMintParams<'a: 'b, 'b> {
    pub mint: AccountInfo<'a>,
    pub to: AccountInfo<'a>,
    pub amount: u64,
    pub authority: AccountInfo<'a>,
    pub authority_signer_seeds: &'b [&'b [u8]],
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_mint(params: TokenMintParams<'_, '_>) -> ProgramResult {
    let TokenMintParams {
        mint,
        to,
        amount,
        authority,
        authority_signer_seeds,
        token_program,
    } = params;

    let mint_to_ctx = token::MintTo {
        mint: mint,
        to: to,
        authority: authority,
    };
    token::mint_to(
        CpiContext::new_with_signer(token_program, mint_to_ctx, &[authority_signer_seeds]),
        amount,
    )?;

    Ok(())
}

/// TokenBurn
pub struct TokenBurnParams<'a: 'b, 'b> {
    pub mint: AccountInfo<'a>,
    pub to: AccountInfo<'a>,
    pub amount: u64,
    pub authority: AccountInfo<'a>,
    pub authority_signer_seeds: &'b [&'b [u8]],
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_burn(params: TokenBurnParams<'_, '_>) -> ProgramResult {
    let TokenBurnParams {
        mint,
        to,
        amount,
        authority,
        authority_signer_seeds,
        token_program,
    } = params;

    let burn_ctx = token::Burn {
        mint: mint,
        to: to,
        authority: authority,
    };
    token::burn(
        CpiContext::new_with_signer(token_program, burn_ctx, &[authority_signer_seeds]),
        amount,
    )?;

    Ok(())
}

pub fn from_bps(input: u32) -> f64 {
    input as f64 / 10000.0
}

pub fn to_bps(input: f64) -> u32 {
    (input * 10000.0) as u32
}
