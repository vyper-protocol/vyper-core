use super::SlotTracking;
use anchor_lang::prelude::*;

#[repr(C, align(8))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct TrancheFairValue {
    /// tranches [senior, junior] fair values expressed in bps
    pub value: [[u8; 16]; 2],
    pub slot_tracking: SlotTracking,
}

impl TrancheFairValue {
    pub const LEN: usize = 16*2 + // pub value: [[u8; 16];2],
    SlotTracking::LEN; // pub slot_tracking: SlotTracking
}
