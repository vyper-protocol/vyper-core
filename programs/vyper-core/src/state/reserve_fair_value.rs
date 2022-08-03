use anchor_lang::prelude::*;
use rust_decimal::Decimal;

use super::SlotTracking;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct ReserveFairValue {
    /// reserve fair value expressed in bps
    pub value: [Decimal; 10],
    pub slot_tracking: SlotTracking,
}

impl ReserveFairValue {
    pub const LEN: usize = 16*10 + // pub value: [Decimal; 10],
    SlotTracking::LEN; // pub slot_tracking: SlotTracking
}
