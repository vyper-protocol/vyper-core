use anchor_lang::prelude::*;

use super::SlotTracking;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct ReserveFairValue {
    /// reserve fair value expressed in bps
    pub value: [u32; 10],
    pub slot_tracking: SlotTracking,
}

impl ReserveFairValue {
    pub const LEN: usize = 4*10 + // pub value: u32,
    SlotTracking::LEN; // pub slot_tracking: SlotTracking
}
