use anchor_lang::prelude::*;
use rust_decimal::Decimal;
use std::fmt;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct SampleRecord {
    value: [[u8; 16]; 10],
    slot: u64,
}

impl SampleRecord {

    pub const LEN: usize = 
        10 * 16 + // value: [[u8; 16]; 10],
        8 // slot: u64,
        ;

    pub fn new(value: [Decimal; 10], slot: u64) -> SampleRecord {
        SampleRecord {
            value: value.map(|f| f.serialize()),
            slot,
        }
    }

    pub fn get_value(&self) -> [Decimal; 10] {
        self.value.map(|f| Decimal::deserialize(f))
    }

    pub fn get_slot(&self) -> u64 {
        self.slot
    }
}

impl fmt::Debug for SampleRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SampleRecord")
         .field("value", &self.get_value())
         .field("slot", &self.get_slot())
         .finish()
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_getters() {
        let v = SampleRecord::new([dec!(10); 10], 10);

        assert_eq!(v.get_value(), [dec!(10); 10]);
        assert_eq!(v.get_slot(), 10);
    }
}
