use anchor_lang::prelude::*;

#[account]
pub struct TrancheConfig {
    /// Tranche configuration authority
    ///
    pub authority: Pubkey,

    /// Protocol program_id
    ///
    pub protocol_program_id: Pubkey,

    /// Current deposited quantities, for senior and junior
    ///
    pub deposited_quantity: [u64; 2],

    /// Senior and junior capital split values in BPS
    /// pe [3000, 7000]
    /// attention, array sum must be 10000
    ///
    pub capital_split: [u32; 2],

    /// Senior and junior interest split values in BPS
    /// pe [8500, 10000]
    /// attention, last value (most junior) must be 10k
    ///
    pub interest_split: [u32; 2],

    /// Senior and junior tranche mint public keys
    ///
    pub senior_tranche_mint: Pubkey,
    pub junior_tranche_mint: Pubkey,

    /// Creation date
    ///
    pub created_at: u64,

    /// Create serum market
    ///
    pub create_serum: bool,

    pub tranche_config_bump: u8,
    pub senior_tranche_mint_bump: u8,
    pub junior_tranche_mint_bump: u8,
}

impl TrancheConfig {
    pub fn get_total_deposited_quantity(&self) -> u64 {
        self.deposited_quantity.iter().sum()
    }

    pub const LEN: usize = 8 + // discriminator
    32 + // authority: Pubkey,
    32 + // protocol_id: Pubkey
    8 + 8 + // deposited_quantity: [u64;2]
    2 * 4 + // interest_split: [u32;2],
    2 * 4 + // capital_split: [u32;2],
    32 + // junior_tranche_mint: Pubkey,
    32 + // senior_tranche_mint: Pubkey,
    8 + // created_at: u64
    1 + // create_serum: bool,
    1 + // trancheConfigBump: u8,
    1 + // seniorTrancheMintBump: u8,
    1; // juniorTrancheMintBump: u8
}
