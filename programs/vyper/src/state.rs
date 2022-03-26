
use anchor_lang::prelude::*;

#[account]
pub struct TrancheConfig {
    pub authority: Pubkey,
    pub protocol_program_id: Pubkey,
    pub deposited_quantiy: u64,
    pub capital_split: [u32; 2],
    pub interest_split: [u32; 2],
    pub junior_tranche_mint: Pubkey,
    pub senior_tranche_mint: Pubkey,

    pub created_at: u64,
    pub create_serum: bool,

    pub tranche_config_bump: u8,
    pub senior_tranche_mint_bump: u8,
    pub junior_tranche_mint_bump: u8,
    pub protocol_bump: u8
}

impl TrancheConfig {
    pub const LEN: usize = 8 + // discriminator
    32 + // authority: Pubkey,
    32 + // protocol_id: Pubkey
    8 + // deposited_quantiy: u64
    2 * 4 + // interest_split: [u32;2],
    2 * 4 + // capital_split: [u32;2],
    32 + // junior_tranche_mint: Pubkey,
    32 + // senior_tranche_mint: Pubkey,
    8 + // created_at: u64
    1 + // create_serum: bool,
    1 + // trancheConfigBump: u8,
    1 + // seniorTrancheMintBump: u8,
    1 + // juniorTrancheMintBump: u8
    1; // protocol_bump: u8
}
