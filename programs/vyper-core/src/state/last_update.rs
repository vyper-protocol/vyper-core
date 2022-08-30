use anchor_lang::prelude::*;

use crate::errors::VyperErrorCode;

// TODO check jet protocol assert_size macro: https://github.com/jet-lab/program-libraries/blob/main/proc-macros/src/lib.rs
// #[assert_size(aligns, 16)]
#[repr(C, align(8))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct LastUpdate {
    slot: u64,
    _padding: [u8; 8],
}

impl LastUpdate {
    /// Create new last update
    pub fn new(slot: u64) -> Self {
        Self {
            slot,
            _padding: [0_u8; 8],
        }
    }

    /// Return slots elapsed since given slot
    pub fn slots_elapsed(&self, slot: u64) -> Result<u64> {
        slot.checked_sub(self.slot)
            .ok_or(VyperErrorCode::MathError.into())
    }

    /// Set last update slot
    pub fn get_slot(&self) -> u64 {
        self.slot
    }

    /// Set last update slot
    pub fn update_slot(&mut self, slot: u64) {
        self.slot = slot;
    }

    pub const LEN: usize = 8 + // slot: u64,
        8; // _padding: [u8; 8],
}
