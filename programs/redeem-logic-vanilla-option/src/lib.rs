use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_math::bps::from_bps;

declare_id!("Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA");

// showcasing Vyper reedem logic for vanilla options
// supports both linear and inverse settlement (=self quanto) e.g. SOL/USDC option settled in USDC vs SOL
// the option is fully collateralized, which amounts to the option writer buying back an option of the same type at bankruptcy level (i.e. call/put spread)
// senior ([0]) is option buyer, junior ([1]) is option writer
// the notional of the trade is junior_qty if inverse and junior_qty / initial_price if linear
// the premium is senior_qty and paid at expiry

#[program]
pub mod redeem_logic_lending {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        strike: u32,
        is_call: bool,
        is_linear: bool,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.strike = strike;
        redeem_logic_config.is_call = is_call;
        redeem_logic_config.is_linear = is_linear;

        Ok(())
    }

    pub fn update(
        ctx: Context<UpdateContext>,
        strike: u32,
        is_call: bool,
        is_linear: bool,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.strike = strike;
        redeem_logic_config.is_call = is_call;
        redeem_logic_config.is_linear = is_linear;

        Ok(())
    }

    pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        let result: RedeemLogicExecuteResult = execute_plugin(
            input_data.old_quantity,
            input_data.old_reserve_fair_value_bps[0],
            input_data.new_reserve_fair_value_bps[0],
            ctx.accounts.redeem_logic_config.strike,
            ctx.accounts.redeem_logic_config.is_call,
            ctx.accounts.redeem_logic_config.is_linear,
        );

        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value_bps: [u32; 10],
    pub new_reserve_fair_value_bps: [u32; 10],
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
    pub is_call: bool,   // true if call, false if put
    pub is_linear: bool, // true if linear, false if inverse
    pub strike: u32,
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    1 + 1 + 4 + 32;
}

fn execute_plugin(
    old_quantity: [u64; 2],
    old_spot_value_bps: u32,
    new_spot_value_bps: u32,
    strike_bps: u32,
    is_call: bool,
    is_linear: bool,
) -> RedeemLogicExecuteResult {
    // TODO: CHECK OVERFLOW

    if old_spot_value_bps == 0 {
        return RedeemLogicExecuteResult {
            new_quantity: old_quantity,
            fee_quantity: 0,
        };
    }

    // let senior_old_quantity = Decimal::from(old_quantity[0]);
    let junior_old_quantity = Decimal::from(old_quantity[1]);
    let total_old_quantity = Decimal::from(old_quantity.iter().sum::<u64>());

    let old_spot = from_bps(old_spot_value_bps).unwrap();
    let new_spot = from_bps(new_spot_value_bps).unwrap();
    let strike = from_bps(strike_bps).unwrap();

    let notional = if is_linear {
        junior_old_quantity / old_spot
    } else {
        junior_old_quantity
    };

    let payoff = {
        if !is_linear & (new_spot == Decimal::ZERO) {
            if strike == Decimal::ZERO {
                if is_call {
                    Decimal::ONE
                } else {
                    Decimal::ZERO
                }
            } else {
                if is_call {
                    Decimal::ZERO
                } else {
                    Decimal::ONE
                }
            }
        } else {
            Decimal::ZERO.max({
                if is_call {
                    new_spot - strike
                } else {
                    strike - new_spot
                }
            }) / {
                if is_linear {
                    Decimal::ONE
                } else {
                    new_spot
                }
            }
        }
    };

    let senior_new_quantity = junior_old_quantity.min(notional * payoff);
    let junior_new_quantity = Decimal::ZERO.max(total_old_quantity - senior_new_quantity);

    let senior_new_quantity = senior_new_quantity.floor().to_u64().unwrap();
    let junior_new_quantity = junior_new_quantity.floor().to_u64().unwrap();
    let fee_quantity = old_quantity.iter().sum::<u64>() - senior_new_quantity - junior_new_quantity;

    return RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_call_otm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 5_000; // 50%
        let strike_bps = 7_000; // 70%
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_linear_call_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 5_000; // 50%
        let strike_bps = 5_000; // 50%
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_call_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 10_000; // 100%
        let strike_bps = 5_000; // 50%
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_put_otm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 6_000; // 50%
        let strike_bps = 5_000; // 50%
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_put_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 5_000; // 50%
        let strike_bps = 5_000; // 50%
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_put_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 3_000; // 30%
        let strike_bps = 5_000; // 50%
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 40_000);
        assert_eq!(res.new_quantity[1], 160_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_call_otm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 5_000; // 50%
        let strike_bps = 7_000; // 70%
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_call_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 5_000; // 50%
        let strike_bps = 5_000; // 50%
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_call_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 10_000; // 100%
        let strike_bps = 5_000; // 50%
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 50_000);
        assert_eq!(res.new_quantity[1], 150_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_put_otm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 5_000; // 50%
        let strike_bps = 3_000; // 30%
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_put_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 5_000; // 50%
        let strike_bps = 5_000; // 50%
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_put_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 2_500; // 25%
        let strike_bps = 5_000; // 50%
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_old_spot() {
        let old_quantity = [100_000, 200_000];
        let old_spot_value_bps = 0_000; // 50%
        let new_spot_value_bps = 2_500; // 25%
        let strike_bps = 5_000; // 50%
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_linear_call() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 500; // 5%
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_inverse_call() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 500; // 5%
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_linear_put() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 500; // 5%
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 190_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_inverse_put() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 500; // 5%
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_zero_strike_linear_call() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 0; // 0%
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_zero_strike_inverse_call() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 0; // 0%
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_zero_strike_linear_put() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 0; // 0%
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_new_spot_zero_strike_inverse_put() {
        let old_quantity = [100_000; 2];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 0; // 0%
        let strike_bps = 0; // 0%
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_rounding() {
        let old_quantity = [13_456, 52_215];
        let old_spot_value_bps = 5_000; // 50%
        let new_spot_value_bps = 6_698; // 66.98%
        let strike_bps = 5_119; // 51.19%
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value_bps,
            new_spot_value_bps,
            strike_bps,
            is_call,
            is_linear,
        );

        assert_eq!(res.new_quantity[0], 12_309);
        assert_eq!(res.new_quantity[1], 53_361);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }
}
