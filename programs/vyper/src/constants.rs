use anchor_lang::prelude::*;

// tranches enum
#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum TrancheID {
    Senior,
    Junior,
}

// pub const MAX_TRANCHES: usize = 5;
pub const SENIOR: &[u8] = b"senior";
pub const JUNIOR: &[u8] = b"junior";