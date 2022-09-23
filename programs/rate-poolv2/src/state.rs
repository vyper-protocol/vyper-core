use anchor_lang::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::fmt;

use crate::errors::RatePoolv2ErrorCode;

pub struct SupplyWrapper {
    pub supply: u64,
    pub decimals: u8,
}

impl SupplyWrapper {
    pub fn to_dec(&self) -> Result<Decimal> {
        // not so readable cause checked methods,
        // we're simply doing:
        // supply / 10**decimals

        Decimal::from(self.supply)
            .checked_div(
                dec!(10)
                    .checked_powu(self.decimals.into())
                    .ok_or(RatePoolv2ErrorCode::MathError)?,
            )
            .ok_or_else(|| RatePoolv2ErrorCode::MathError.into())
    }
}

impl fmt::Display for SupplyWrapper {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{} (decimals: {})", self.supply, self.decimals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test() {
        let sw = SupplyWrapper {
            supply: 1_500_000,
            decimals: 6,
        };
        assert_eq!(sw.to_dec().unwrap(), dec!(1.5));
    }
}
