use anchor_lang::prelude::*;

use super::TrancheData;

#[repr(C, align(8))]
#[account]
pub struct TrancheConfig {
    pub reserve_mint: Pubkey,
    pub reserve: Pubkey,

    pub tranche_data: TrancheData,

    /// Senior tranche mint public key
    pub senior_tranche_mint: Pubkey,

    /// Junior tranche mint public key
    pub junior_tranche_mint: Pubkey,

    /// Tranche configuration authority
    pub tranche_authority: Pubkey,

    pub authority_seed: Pubkey,

    pub authority_bump: [u8; 1],

    /// Account which is allowed to call restricted instructions
    pub owner: Pubkey,

    pub rate_program: Pubkey,
    pub rate_program_state: Pubkey,

    pub redeem_logic_program: Pubkey,
    pub redeem_logic_program_state: Pubkey,

    /// Program version when initialized: [major, minor, patch]
    pub version: [u8; 3],

    /// Creation date
    pub created_at: i64,

    /// Reserved space for future upgrades
    _reserved: [u8; 256],
}

impl TrancheConfig {
    pub fn authority_seeds(&self) -> [&[u8]; 3] {
        [
            self.authority_seed.as_ref(),
            b"authority".as_ref(),
            &self.authority_bump,
        ]
    }

    pub const LEN: usize = 8 + // discriminator
        32 + // pub reserve_mint: Pubkey,
        32 + // pub reserve: Pubkey,
        TrancheData::LEN + // pub tranche_data: TrancheData,
        32 + // pub senior_tranche_mint: Pubkey,
        32 + // pub junior_tranche_mint: Pubkey,
        32 + // pub tranche_authority: Pubkey,
        32 + // pub authority_seed: Pubkey,
        1 + // pub authority_bump: [u8; 1],
        32 + // pub owner: Pubkey,
        32 + // pub rate_program: Pubkey,
        32 + // pub rate_program_state: Pubkey,
        32 + // pub redeem_logic_program: Pubkey,
        32 + // pub redeem_logic_program_state: Pubkey,
        3 + // pub version: [u8; 3],
        8 + // pub created_at: i64;
        256; // _reserved: [u8; 256],
}
