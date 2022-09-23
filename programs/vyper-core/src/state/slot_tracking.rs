use anchor_lang::prelude::*;

use crate::errors::VyperErrorCode;

use super::LastUpdate;

/// Tracking of slot information
#[repr(C, align(8))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct SlotTracking {
    last_update: LastUpdate,

    /// threshold for defining a slot tracked value stale
    pub stale_slot_threshold: u64,
}

impl SlotTracking {
    pub fn new(slot: u64) -> Self {
        Self {
            last_update: LastUpdate::new(slot),
            stale_slot_threshold: 2,
        }
    }

    pub fn update(&mut self, slot: u64) {
        self.last_update.update_slot(slot);
    }

    pub fn slot_elapsed(&self, current_slot: u64) -> Result<u64> {
        current_slot
            .checked_sub(self.last_update.get_slot())
            .ok_or_else(|| VyperErrorCode::MathError.into())
    }

    pub fn is_stale(&self, current_slot: u64) -> Result<bool> {
        Ok(self.slot_elapsed(current_slot)? >= self.stale_slot_threshold)
    }

    pub fn get_last_update_slot(&self) -> u64 {
        self.last_update.get_slot()
    }

    pub const LEN: usize = LastUpdate::LEN + // last_update: LastUpdate,
    8; // pub stale_slot_threshold: u64,
}
