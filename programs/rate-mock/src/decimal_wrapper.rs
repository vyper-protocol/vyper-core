use anchor_lang::prelude::*;
use rust_decimal::Decimal;

#[repr(C, align(8))]
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug, Default)]
pub struct DecimalWrapper {
    bytes: [u8; 16],
}

impl DecimalWrapper {
    pub fn new(val: Decimal) -> Self {
        Self {
            bytes: val.serialize(),
        }
    }

    pub fn get(&self) -> Decimal {
        Decimal::deserialize(self.bytes)
    }

    pub fn set(&mut self, val: Decimal) {
        self.bytes = val.serialize();
    }
}
