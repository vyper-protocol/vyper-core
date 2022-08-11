use anchor_lang::prelude::*;
use rust_decimal_macros::dec;

use crate::errors::VyperErrorCode;

use super::{
    OwnerRestrictedIxFlags, ReserveFairValue, SlotTracking, TrancheFairValue, TrancheHaltFlags,
};

#[repr(C, align(8))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug)]
pub struct TrancheData {
    /// Current deposited quantities, for senior and junior cUSDC
    pub deposited_quantity: [u64; 2],

    ///
    pub fee_to_collect_quantity: u64,

    /// pe cUSDC / USDC
    pub reserve_fair_value: ReserveFairValue,

    /// pe [ sTranche / cUSDC ; jTranche / cUSDC ]
    pub tranche_fair_value: TrancheFairValue,

    /// halt flags
    halt_flags: u16,

    /// flags for owner-only instructions
    owner_restricted_ix: u16,

    /// Reserved space for future upgrades
    _padding: [u8; 256],
}

impl TrancheData {
    pub fn new(slot: u64) -> Self {
        Self {
            deposited_quantity: [0; 2],
            reserve_fair_value: ReserveFairValue {
                value: [dec!(1).serialize(); 10],
                slot_tracking: SlotTracking::new(slot),
            },
            tranche_fair_value: TrancheFairValue {
                value: [dec!(1).serialize(); 2],
                slot_tracking: SlotTracking::new(slot),
            },
            halt_flags: 0,
            owner_restricted_ix: 0,
            fee_to_collect_quantity: 0,
            _padding: [0u8; 256],
        }
    }

    pub fn get_halt_flags(&self) -> Result<TrancheHaltFlags> {
        TrancheHaltFlags::from_bits(self.halt_flags)
            .ok_or(VyperErrorCode::InvalidTrancheHaltFlags.into())
    }

    pub fn set_halt_flags(&mut self, bits: u16) -> Result<()> {
        TrancheHaltFlags::from_bits(bits).ok_or(VyperErrorCode::InvalidTrancheHaltFlags)?;
        self.halt_flags = bits;
        Ok(())
    }

    pub fn get_owner_restricted_ixs(&self) -> Result<OwnerRestrictedIxFlags> {
        OwnerRestrictedIxFlags::from_bits(self.owner_restricted_ix)
            .ok_or(VyperErrorCode::InvalidOwnerRestrictedIxFlags.into())
    }

    pub fn set_owner_restricted_instructions(&mut self, bits: u16) -> Result<()> {
        OwnerRestrictedIxFlags::from_bits(bits)
            .ok_or(VyperErrorCode::InvalidOwnerRestrictedIxFlags)?;
        self.owner_restricted_ix = bits;
        Ok(())
    }

    pub const LEN: usize = 2*8 + // pub deposited_quantity: [u64; 2],
    8 + // pub fee_to_collect_quantity: u64,
    ReserveFairValue::LEN + // pub reserve_fair_value: ReserveFairValue,
    TrancheFairValue::LEN + // pub tranche_fair_value: TrancheFairValue,
    2 + // halt_flags: u16,
    2 + // owner_restricted_ix: u16,
    256; // _padding: [u8; 256],,
}
