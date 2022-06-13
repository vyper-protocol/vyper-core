use anchor_lang::prelude::*;

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
    1024; // TODO TBD
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct TrancheData {
    /// Current deposited quantities, for senior and junior cUSDC
    pub deposited_quantity: [u64; 2],

    /// pe cUSDC / USDC
    pub reserve_fair_value: ReserveFairValue,

    /// pe [ sTranche / cUSDC ; jTranche / cUSDC ]
    pub tranche_fair_value: TrancheFairValue,

    /// halt flags
    halt_flags: u16,

    /// flags for owner-only instructions
    owner_restricted_ix: u16,
}

impl TrancheData {
    pub fn new(slot: u64) -> Self {
        Self {
            deposited_quantity: [0; 2],
            reserve_fair_value: ReserveFairValue { value: 1, slot_tracking: SlotTracking::new(slot) },
            tranche_fair_value: TrancheFairValue { value: [1;2], slot_tracking: SlotTracking::new(slot) },
            halt_flags: 0,
            owner_restricted_ix: 0
        }
    }

    pub fn get_halt_flags(&self) -> TrancheHaltFlags {
        TrancheHaltFlags::from_bits(self.halt_flags)
            .unwrap_or_else(|| panic!("{:?} does not resolve to valid TrancheHaltFlags", self.halt_flags))
    }

    pub fn set_halt_flags(&mut self, bits: u16) -> Result<()> {
        TrancheHaltFlags::from_bits(bits).ok_or_else::<VyperErrorCode, _>(|| VyperErrorCode::InvalidTranchHaltFlags.into())?;
        self.halt_flags = bits;
        Ok(())
    }

    pub fn get_owner_restricted_ixs(&self) -> OwnerRestrictedIxFlags {
        OwnerRestrictedIxFlags::from_bits(self.owner_restricted_ix)
            .unwrap_or_else(|| panic!("{:?} does not resolve to valid OwnerRestrictedInstructions", self.owner_restricted_ix))
    }

    pub fn set_owner_restricted_instructions(&mut self, bits: u16) -> Result<()> {
        OwnerRestrictedIxFlags::from_bits(bits).ok_or_else::<VyperErrorCode, _>(|| VyperErrorCode::InvalidOwnerRestrictedIxFlags.into())?;
        self.owner_restricted_ix = bits;
        Ok(())
    }
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
    pub value: u32,
    pub slot_tracking: SlotTracking
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct TrancheFairValue {
    /// tranches [senior, junior] fair values expressed in bps
    pub value: [u32;2],
    pub slot_tracking: SlotTracking
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
            stale_slot_threshold: 2
        }
    }

    pub fn update(&mut self, slot: u64) {
        self.last_update.update_slot(slot);
    }

    pub fn slot_elapsed(&self, current_slot: u64) -> Result<u64> {
        current_slot.checked_sub(self.last_update.slot).ok_or_else(|| VyperErrorCode::MathError.into())
    }

    pub fn is_stale(&self, current_slot: u64) -> Result<bool> {
        msg!("current_slot: {}", current_slot);
        msg!("self.get_last_update_slot(): {}", self.get_last_update_slot());
        msg!("self.slot_elapsed: {}", self.slot_elapsed(current_slot)?);
        msg!("self.stale_slot_threshold: {}", self.stale_slot_threshold);

        Ok(self.slot_elapsed(current_slot)? >= self.stale_slot_threshold)
    }

    pub fn get_last_update_slot(&self) -> u64 {
        self.last_update.slot
    }
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
        slot.checked_sub(self.slot).ok_or_else(|| VyperErrorCode::MathError.into())
    }

    /// Set last update slot
    pub fn update_slot(&mut self, slot: u64) {
        self.slot = slot;
    }
}