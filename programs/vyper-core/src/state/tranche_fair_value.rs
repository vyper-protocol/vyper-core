use anchor_lang::prelude::*;
use rust_decimal::Decimal;
use vyper_math::bps::from_bps;

use super::SlotTracking;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct TrancheFairValue {
    /// tranches [senior, junior] fair values expressed in bps
    pub value: [u32; 2],
    pub slot_tracking: SlotTracking,
}

impl TrancheFairValue {
    pub fn get_decimals(&self) -> [Decimal; 2] {
        self.value.map(|c| from_bps(c).unwrap())
    }

    pub const LEN: usize = 2*4 + // pub value: [u32;2],
    SlotTracking::LEN; // pub slot_tracking: SlotTracking
}
