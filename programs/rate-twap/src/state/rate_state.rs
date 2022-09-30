use anchor_lang::prelude::*;

use super::SamplingData;

#[account]
pub struct RateState {
    pub fair_value: [[u8; 16]; 10],
    pub refreshed_slot: u64,
    pub rate_state_source: Pubkey,
    pub sampling_data: SamplingData,
}

impl RateState {
    pub fn compute_twap(&mut self) -> Result<()> {
        let (twap_value, twap_refreshed_slot) = self.sampling_data.twap()?;

        self.fair_value = twap_value.map(|c| c.serialize());
        self.refreshed_slot = twap_refreshed_slot;

        Ok(())
    }

    pub fn len(sampling_size: usize) -> usize {
        8 + // discriminator
            10*16 + // pub fair_value: [[u8; 16]; 10],
            8 + // pub refreshed_slot: u64,
            32 + // pub rate_state_source: Pubkey,
            SamplingData::len(sampling_size) // pub sampling_data: SamplingData
    }
}
