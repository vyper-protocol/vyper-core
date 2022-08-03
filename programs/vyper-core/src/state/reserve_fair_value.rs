use anchor_lang::prelude::*;

use super::{SlotTracking, DecimalWrapper};

#[repr(C, align(8))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct ReserveFairValue {
    /// reserve fair value expressed in bps
    pub value: [DecimalWrapper; 10],
    pub slot_tracking: SlotTracking,
}

impl ReserveFairValue {
    pub const LEN: usize = 16*10 + // pub value: [DecimalWrapper; 10],
    SlotTracking::LEN; // pub slot_tracking: SlotTracking
}
