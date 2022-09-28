use anchor_lang::prelude::*;
use rust_decimal::Decimal;
use std::fmt;

use crate::errors::RateTwapErrorCode;
use crate::state::sample_record::SampleRecord;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct SamplingData {
    min_slot_delta: u64,
    max_samples_size: u32,
    samples: Vec<SampleRecord>,
}

impl fmt::Debug for SamplingData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SamplingData")
         .field("min_slot_delta", &self.min_slot_delta)
         .field("max_samples_size", &self.max_samples_size)
         .field("samples len", &self.samples.len())
         .field("samples", &self.samples)
         .finish()
    }
}


impl SamplingData {
    pub fn new(min_slot_delta: u64, samples_size: u32) -> Result<SamplingData> {
        require!(samples_size > 0, RateTwapErrorCode::InputError);
        Ok(SamplingData {
            min_slot_delta,
            max_samples_size: samples_size,
            samples: Vec::new(),
        })
    }

    /// will try to add a new sample
    /// this can fail if there's already another sample with a close slot as the new one
    /// the threshold used for checking is the self.min_slot_delta
    pub fn try_add(&mut self, value: [Decimal; 10], slot: u64) -> Result<()> {
        if !self.samples.is_empty() {
            // check if there's a too recent samples
            if (slot - self.samples[self.get_most_recent_sample_idx()].get_slot())
                < self.min_slot_delta
            {
                return err!(RateTwapErrorCode::AnotherTooRecentSample);
            }

            // check if max length is reached
            if self.samples.len() == self.max_samples_size.try_into().unwrap() {
                // remove oldest sample
                self.samples.remove(self.get_oldest_sample_idx());
            }
        }

        self.samples.push(SampleRecord::new(value, slot));
        Ok(())
    }

    /// Get the twap
    /// Average of all the values and the most recent slot
    pub fn twap(&self) -> Result<([Decimal; 10], u64)> {
        let avg = self.avg()?;
        let most_recent_slot = self.samples[self.get_most_recent_sample_idx()].get_slot();

        Ok((avg, most_recent_slot))
    }

    /// Get the average of all the samples value
    pub fn avg(&self) -> Result<[Decimal; 10]> {
        if self.samples.is_empty() {
            return err!(RateTwapErrorCode::EmptySamples);
        }

        let mut agg = [Decimal::ZERO; 10];
        for sample_record in &self.samples {
            for (idx, val) in sample_record.get_value().iter().enumerate() {
                agg[idx] = agg[idx]
                    .checked_add(val.clone())
                    .ok_or_else(|| RateTwapErrorCode::GenericError)?;
            }
        }

        Ok(agg.map(|c| c / Decimal::from(self.samples.len())))
    }

    /// Get the idx of the oldest sample
    fn get_oldest_sample_idx(&self) -> usize {
        let mut oldest_idx = 0;

        for (i, v) in self.samples.iter().enumerate() {
            if v.get_slot() < self.samples[oldest_idx].get_slot() {
                oldest_idx = i;
            }
        }

        oldest_idx
    }

    /// Get the idx of the most recent sample
    fn get_most_recent_sample_idx(&self) -> usize {
        let mut most_recent_idx = 0;

        for (i, v) in self.samples.iter().enumerate() {
            if v.get_slot() > self.samples[most_recent_idx].get_slot() {
                most_recent_idx = i;
            }
        }

        most_recent_idx
    }

    pub fn len(sampling_size: usize) -> usize {
        return 
            8 + // min_slot_delta: u64,
            4 + // max_samples_size: u32,
            4 + (SampleRecord::LEN * sampling_size)  // samples: Vec<SampleRecord>,
            ;
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_try_add() {
        let mut sampling = SamplingData::new(0, 4).unwrap();

        sampling.try_add([Decimal::ZERO; 10], 0).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 1).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 2).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 3).unwrap();
        assert_eq!(sampling.twap().unwrap(), ([Decimal::ZERO; 10], 3));
        
        sampling.try_add([Decimal::ZERO; 10], 4).unwrap();
        assert_eq!(sampling.twap().unwrap(), ([Decimal::ZERO; 10], 4));
    }

    #[test]
    fn test_avg() {
        let mut sampling = SamplingData::new(0, 4).unwrap();

        sampling.try_add([Decimal::ZERO; 10], 0).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 1).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 2).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 3).unwrap();
        assert_eq!(sampling.avg().unwrap(), [Decimal::ZERO; 10]);

        sampling.try_add([Decimal::ONE; 10], 4).unwrap();
        assert_eq!(sampling.avg().unwrap(), [dec!(0.25); 10]);
    }

    #[test]
    fn test_error_on_recent_slot() {
        let mut sampling = SamplingData::new(2, 4).unwrap();

        sampling.try_add([Decimal::ZERO; 10], 0).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 2).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 4).unwrap();
        assert!(sampling.try_add([Decimal::ZERO; 10], 5).is_err());
    }

    #[test]
    fn test_get_most_recent_sample_idx() {
        let mut sampling = SamplingData::new(2, 4).unwrap();
        
        sampling.try_add([Decimal::ZERO; 10], 0).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 2).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 4).unwrap();
        assert_eq!(sampling.get_most_recent_sample_idx(), 2);
    }
    
    #[test]
    fn test_get_oldest_sample_idx() {
        let mut sampling = SamplingData::new(2, 4).unwrap();
        
        sampling.try_add([Decimal::ZERO; 10], 0).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 2).unwrap();
        sampling.try_add([Decimal::ZERO; 10], 4).unwrap();
        assert_eq!(sampling.get_oldest_sample_idx(), 0);
    }
}
