use anchor_lang::prelude::*;
use rust_decimal::Decimal;
use vyper_math::bps::{from_bps, ONE_BPS};

use crate::errors::VyperErrorCode;

#[account]
pub struct TrancheConfig {
    pub reserve_mint: Pubkey,
    pub reserve: Pubkey,

    pub tranche_data: TrancheData,

    /// Senior tranche mint public key
    pub senior_tranche_mint: Pubkey,

    /// Junior tranche mint public key
    pub junior_tranche_mint: Pubkey,

    /// Tranche configuration authority
    pub tranche_authority: Pubkey,

    pub authority_seed: Pubkey,

    pub authority_bump: [u8; 1],

    /// Account which is allowed to call restricted instructions
    pub owner: Pubkey,

    pub rate_program: Pubkey,
    pub rate_program_state: Pubkey,

    pub redeem_logic_program: Pubkey,
    pub redeem_logic_program_state: Pubkey,

    /// Program version when initialized: [major, minor, patch]
    pub version: [u8; 3],

    /// Creation date
    pub created_at: i64,

    /// Reserved space for future upgrades
    _reserved: [u8; 256],
}

impl TrancheConfig {
    pub fn authority_seeds(&self) -> [&[u8]; 3] {
        [
            self.authority_seed.as_ref(),
            b"authority".as_ref(),
            &self.authority_bump,
        ]
    }

    pub const LEN: usize = 8 + // discriminator
        32 + // pub reserve_mint: Pubkey,
        32 + // pub reserve: Pubkey,
        TrancheData::LEN + // pub tranche_data: TrancheData,
        32 + // pub senior_tranche_mint: Pubkey,
        32 + // pub junior_tranche_mint: Pubkey,
        32 + // pub tranche_authority: Pubkey,
        32 + // pub authority_seed: Pubkey,
        1 + // pub authority_bump: [u8; 1],
        32 + // pub owner: Pubkey,
        32 + // pub rate_program: Pubkey,
        32 + // pub rate_program_state: Pubkey,
        32 + // pub redeem_logic_program: Pubkey,
        32 + // pub redeem_logic_program_state: Pubkey,
        3 + // pub version: [u8; 3],
        8 + // pub created_at: i64;
        256; // _reserved: [u8; 256],
}

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
                value: [ONE_BPS; 10],
                slot_tracking: SlotTracking::new(slot),
            },
            tranche_fair_value: TrancheFairValue {
                value: [ONE_BPS; 2],
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

bitflags::bitflags! {
    pub struct TrancheHaltFlags: u16 {
        /// Disable deposits
        const HALT_DEPOSITS = 1 << 0;

        /// Disable refreshes
        const HALT_REFRESHES = 1 << 1;

        /// Disable redeems
        const HALT_REDEEMS = 1 << 2;

        /// Disable all operations
        const HALT_ALL = Self::HALT_DEPOSITS.bits
                       | Self::HALT_REFRESHES.bits
                       | Self::HALT_REDEEMS.bits;

    }
}

bitflags::bitflags! {
    pub struct OwnerRestrictedIxFlags: u16 {
        /// Owner restricted: Deposits
        const DEPOSITS = 1 << 0;

        /// Owner restricted: Refreshes
        const REFRESHES = 1 << 1;

        /// Owner restricted: Redeems
        const REDEEMS = 1 << 2;

        /// Disable all operations
        const ALL = Self::DEPOSITS.bits
                       | Self::REFRESHES.bits
                       | Self::REDEEMS.bits;

    }
}

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
            .checked_sub(self.last_update.slot)
            .ok_or(VyperErrorCode::MathError.into())
    }

    pub fn is_stale(&self, current_slot: u64) -> Result<bool> {
        Ok(self.slot_elapsed(current_slot)? >= self.stale_slot_threshold)
    }

    pub fn get_last_update_slot(&self) -> u64 {
        self.last_update.slot
    }

    pub const LEN: usize = LastUpdate::LEN + // last_update: LastUpdate,
    8; // pub stale_slot_threshold: u64,
}

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
    pub fn update_slot(&mut self, slot: u64) {
        self.slot = slot;
    }

    pub const LEN: usize = 8 + // slot: u64,
        8; // _padding: [u8; 8],
}
