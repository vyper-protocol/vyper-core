// Vyper reedem logic for forward contracts settled in a third currency
// For example an ABC/XYZ forward settled in USD at the prevailing XYZ/USD or ABC/USD exchange rate
// Collateral is in the quote currency of the settlement pair, e.g. USD for ABC/XYZ settled in XYZ/USD
// The contract is fully collateralized, which amounts to the counterparties writing each other calls/puts at bankruptcy level
// Senior ([0]) is long, junior ([1]) is short
// The notional of the contract is in base asset (e.g. ABC in an ABC/XYZ contract)
// Supports both linear and inverse settlement. For example for an ABC/XYZ contract, if is_linear provide XYZ/USD else provide ABC/USD

use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod redeem_logic_settled_forward {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        strike: f64,
        notional: u64,
        is_linear: bool,
    ) -> Result<()> {
        require!(strike >= 0., RedeemLogicErrors::InvalidInput);

        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;
        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.strike = Decimal::from_f64(strike)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.notional = notional;
        redeem_logic_config.is_linear = is_linear;

        Ok(())
    }

    pub fn update(
        ctx: Context<UpdateContext>,
        strike: f64,
        notional: u64,
        is_linear: bool,
    ) -> Result<()> {
        require!(strike >= 0., RedeemLogicErrors::InvalidInput);

        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;
        redeem_logic_config.strike = Decimal::from_f64(strike)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.notional = notional;
        redeem_logic_config.is_linear = is_linear;

        Ok(())
    }

    pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        input_data.is_valid()?;
        ctx.accounts.redeem_logic_config.dump();

        let result: RedeemLogicExecuteResult = execute_plugin(
            input_data.old_quantity,
            Decimal::deserialize(input_data.new_reserve_fair_value[0]),
            Decimal::deserialize(input_data.new_reserve_fair_value[1]),
            Decimal::deserialize(ctx.accounts.redeem_logic_config.strike),
            ctx.accounts.redeem_logic_config.notional,
            ctx.accounts.redeem_logic_config.is_linear,
        )?;

        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value: [[u8; 16]; 10],
    pub new_reserve_fair_value: [[u8; 16]; 10],
}

impl RedeemLogicExecuteInput {
    fn is_valid(&self) -> Result<()> {
        for r in self.old_reserve_fair_value {
            require!(
                Decimal::deserialize(r) >= Decimal::ZERO,
                RedeemLogicErrors::InvalidInput
            );
        }

        for r in self.new_reserve_fair_value {
            require!(
                Decimal::deserialize(r) >= Decimal::ZERO,
                RedeemLogicErrors::InvalidInput
            );
        }

        Result::Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteResult {
    pub new_quantity: [u64; 2],
    pub fee_quantity: u64,
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    /// Tranche config account, where all the parameters are saved
    #[account(init, payer = payer, space = RedeemLogicConfig::LEN)]
    pub redeem_logic_config: Box<Account<'info, RedeemLogicConfig>>,

    /// CHECK: Owner of the tranche config
    #[account()]
    pub owner: AccountInfo<'info>,

    /// Signer account
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateContext<'info> {
    #[account(mut, has_one = owner)]
    pub redeem_logic_config: Account<'info, RedeemLogicConfig>,

    /// CHECK: Owner of the tranche config
    #[account()]
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteContext<'info> {
    #[account()]
    pub redeem_logic_config: Account<'info, RedeemLogicConfig>,
}

#[account]
pub struct RedeemLogicConfig {
    pub notional: u64,

    /// true if linear, false if inverse
    pub is_linear: bool,

    pub strike: [u8; 16],
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    8 + // pub notional: u64,
    1 + // pub is_linear: bool,
    16 + // pub strike: [u8; 16],
    32 // pub owner: Pubkey,
    ;

    fn dump(&self) {
        msg!("redeem logic config:");
        msg!("+ notional: {:?}", self.notional);
        msg!("+ is_linear: {:?}", self.is_linear);
        msg!("+ strike: {:?}", Decimal::deserialize(self.strike))
    }
}

fn execute_plugin(
    old_quantity: [u64; 2],
    new_ul_spot: Decimal,
    new_settle_spot: Decimal,
    strike: Decimal,
    notional: u64,
    is_linear: bool,
) -> Result<RedeemLogicExecuteResult> {
    require!(
        new_ul_spot >= Decimal::ZERO,
        RedeemLogicErrors::InvalidInput
    );
    require!(strike >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);

    if new_ul_spot == Decimal::ZERO && !is_linear && strike > Decimal::ZERO {
        return Ok(RedeemLogicExecuteResult {
            new_quantity: [0, old_quantity.iter().sum::<u64>()],
            fee_quantity: 0,
        });
    }

    let senior_old_quantity = Decimal::from(old_quantity[0]);
    // let junior_old_quantity = Decimal::from(old_quantity[1]);
    let total_old_quantity = Decimal::from(old_quantity.iter().sum::<u64>());
    let notional = Decimal::from(notional);

    let payoff = new_settle_spot * {
        if new_ul_spot == Decimal::ZERO && !is_linear && strike == Decimal::ZERO {
            notional
        } else {
            notional * (new_ul_spot - strike) / {
                if is_linear {
                    Decimal::ONE
                } else {
                    new_ul_spot
                }
            }
        }
    };

    let senior_new_quantity =
        total_old_quantity.min(Decimal::ZERO.max(senior_old_quantity + payoff));
    let junior_new_quantity = Decimal::ZERO.max(total_old_quantity - senior_new_quantity);

    let senior_new_quantity = senior_new_quantity
        .floor()
        .to_u64()
        .ok_or(RedeemLogicErrors::MathError)?;
    let junior_new_quantity = junior_new_quantity
        .floor()
        .to_u64()
        .ok_or(RedeemLogicErrors::MathError)?;
    let fee_quantity = old_quantity.iter().sum::<u64>() - senior_new_quantity - junior_new_quantity;

    Ok(RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity,
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_linear_flat_returns() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ONE_HUNDRED;
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 1;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_flat_returns() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ONE_HUNDRED;
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 1;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_spot_up() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = dec!(120);
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 1_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 120_000);
        assert_eq!(res.new_quantity[1], 80_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_spot_down() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = dec!(80);
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 1_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 80_000);
        assert_eq!(res.new_quantity[1], 120_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_spot_up() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = dec!(120);
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 1_000;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_166);
        assert_eq!(res.new_quantity[1], 99_833);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_spot_down() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = dec!(80);
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 1_000;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 99_750);
        assert_eq!(res.new_quantity[1], 100_250);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_long_bankrupt() {
        let old_quantity = [50_000, 100_000];
        let new_ul_spot = dec!(75);
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 2_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 150_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_short_bankrupt() {
        let old_quantity = [100_000, 50_000];
        let new_ul_spot = dec!(125);
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let notional = 2_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 150_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_luna_rekt_linear() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ZERO;
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE;
        let notional = 1_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 99_000);
        assert_eq!(res.new_quantity[1], 101_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_luna_rekt_inverse() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ZERO;
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ONE;
        let notional = 1_000;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_strike_linear() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ONE;
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ZERO;
        let notional = 1_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 101_000);
        assert_eq!(res.new_quantity[1], 99_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_strike_inverse() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = dec!(50);
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ZERO;
        let notional = 1_000;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 101_000);
        assert_eq!(res.new_quantity[1], 99_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_luna_rekt_zero_strike_linear() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ZERO;
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ZERO;
        let notional = 1_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_luna_rekt_zero_strike_inverse() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ZERO;
        let new_settled_spot = Decimal::ONE;
        let strike = Decimal::ZERO;
        let notional = 1_000;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 101_000);
        assert_eq!(res.new_quantity[1], 99_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_settled_linear() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ONE_HUNDRED;
        let new_settled_spot = dec!(0.5);
        let strike = dec!(50);
        let notional = 1_000;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 125_000);
        assert_eq!(res.new_quantity[1], 75_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_settled_inverse() {
        let old_quantity = [100_000; 2];
        let new_ul_spot = Decimal::ONE_HUNDRED;
        let new_settled_spot = dec!(0.5);
        let strike = dec!(50);
        let notional = 1_000;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_ul_spot,
            new_settled_spot,
            strike,
            notional,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_250);
        assert_eq!(res.new_quantity[1], 99_750);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }
}
