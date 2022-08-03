use anchor_lang::prelude::*;
use rust_decimal::Decimal;

use super::SlotTracking;

#[repr(C, align(8))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct TrancheFairValue {
    /// tranches [senior, junior] fair values expressed in bps
    pub value: [Decimal; 2],
    pub slot_tracking: SlotTracking,
}

impl TrancheFairValue {
    pub const LEN: usize = 16*2 + // pub value: [Decimal;2],
    SlotTracking::LEN; // pub slot_tracking: SlotTracking
}
