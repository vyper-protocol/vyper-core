use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use rust_decimal::Decimal;
use std::ops::RangeInclusive;

use crate::errors::MathErrorCode;

pub struct BpsRangeValue {
    value: u32,
    range: RangeInclusive<u32>,
}

impl BpsRangeValue {
    pub fn new(value: u32) -> Result<BpsRangeValue, MathErrorCode> {
        BpsRangeValue::new_with_range(value, 0..=10_000)
    }

    pub fn new_with_range(value: u32, range: RangeInclusive<u32>) -> Result<BpsRangeValue, MathErrorCode> {
        return if range.contains(&value) {
            Ok(BpsRangeValue {
                value: value,
                range: range
            })
        }
        else {
            Err(MathErrorCode::OutOfRange)
        };
    }

    pub fn set(&mut self, value: u32) {
        if !self.range.contains(&value) {
            panic!("range overflow")
        }
        self.value = value;
    }

    pub fn get_decimal(&self) -> Option<Decimal> {
        from_bps(self.value)
    }

    pub fn get_f64(&self) -> Option<f64> {
        match self.value.to_f64() {
            Some(val) => Some(val / 10_000.0),
            None => None
        }
    }
}

/// Convert an input representing a bps value into an Option<Decimal>
pub fn from_bps(input: u32) -> Option<Decimal> {
    Decimal::from_u32(input)?.checked_div(dec!(10000.0))
}

/// Convert a decimal input into a bps value
pub fn to_bps(input: Decimal) -> Option<u32> {
    input.checked_mul(dec!(10000.0))?.to_u32()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::bps::BpsRangeValue;

    #[test]
    fn test_from_bps() {
        assert_eq!(from_bps(10_000), Some(dec!(1)));
        assert_eq!(from_bps(5_000), Some(dec!(0.5)));
        assert_eq!(from_bps(0), Some(dec!(0)));
    }

    #[test]
    fn test_to_bps() {
        assert_eq!(to_bps(dec!(1)), Some(10_000));
        assert_eq!(to_bps(dec!(0.5)), Some(5_000));
        assert_eq!(to_bps(dec!(0)), Some(0));
    }

    #[test]
    fn test_check_bps_range_value() {
        let r = BpsRangeValue::new(5_000).unwrap();
        assert_eq!(r.get_decimal(), Some(dec!(0.5)));
    }

    #[test]
    fn test_check_bps_range_value_range_err() {
        let r = BpsRangeValue::new_with_range(25_000, 0..=20_000);
        assert!(r.is_err())
    }
}