use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

solana_security_txt::security_txt! {
    name: "Redeem Logic Vanilla Option | Vyper Core",
    project_url: "https://vyperprotocol.io",
    contacts: "email:info@vyperprotocol.io,link:https://docs.vyperprotocol.io/",
    policy: "https://github.com/vyper-protocol/vyper-core/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/vyper-protocol/vyper-core/tree/main/programs/redeem-logic-vanilla-option"
}

declare_id!("8fSeRtFseNrjdf8quE2YELhuzLkHV7WEGRPA9Jz8xEVe");

// showcasing Vyper reedem logic for vanilla options
// supports both linear and inverse settlement (=self quanto) e.g. SOL/USDC option settled in USDC vs SOL
// the option is fully collateralized, which amounts to the option writer buying back an option of the same type at bankruptcy level (i.e. call/put spread)
// senior ([0]) is option buyer, junior ([1]) is option writer
// the notional of the trade is junior_qty if inverse and junior_qty / initial_price if linear
// the premium is senior_qty and paid at expiry

#[program]
pub mod redeem_logic_vanilla_option {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        strike: f64,
        is_call: bool,
        is_linear: bool,
    ) -> Result<()> {
        require!(strike >= 0., RedeemLogicErrors::InvalidInput);

        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;
        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.strike = Decimal::from_f64(strike)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.is_call = is_call;
        redeem_logic_config.is_linear = is_linear;

        Ok(())
    }

    pub fn update(
        ctx: Context<UpdateContext>,
        strike: f64,
        is_call: bool,
        is_linear: bool,
    ) -> Result<()> {
        require!(strike >= 0., RedeemLogicErrors::InvalidInput);

        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;
        redeem_logic_config.strike = Decimal::from_f64(strike)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.is_call = is_call;
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
            Decimal::deserialize(input_data.old_reserve_fair_value[0]),
            Decimal::deserialize(input_data.new_reserve_fair_value[0]),
            Decimal::deserialize(ctx.accounts.redeem_logic_config.strike),
            ctx.accounts.redeem_logic_config.is_call,
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
    /// true if call, false if put
    pub is_call: bool,

    /// true if linear, false if inverse
    pub is_linear: bool,

    pub strike: [u8; 16],
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    1 + // pub is_call: bool,
    1 + // pub is_linear: bool,
    16 + // pub strike: [u8; 16],
    32 // pub owner: Pubkey,
    ;

    fn dump(&self) {
        msg!("redeem logic config:");
        msg!("+ is_call: {:?}", self.is_call);
        msg!("+ is_linear: {:?}", self.is_linear);
        msg!("+ strike: {:?}", Decimal::deserialize(self.strike))
    }
}

#[allow(clippy::collapsible_else_if)]
fn execute_plugin(
    old_quantity: [u64; 2],
    old_spot: Decimal,
    new_spot: Decimal,
    strike: Decimal,
    is_call: bool,
    is_linear: bool,
) -> Result<RedeemLogicExecuteResult> {
    require!(old_spot >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
    require!(new_spot >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
    require!(strike >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);

    if old_spot == Decimal::ZERO {
        return Ok(RedeemLogicExecuteResult {
            new_quantity: old_quantity,
            fee_quantity: 0,
        });
    }

    // let senior_old_quantity = Decimal::from(old_quantity[0]);
    let junior_old_quantity = Decimal::from(old_quantity[1]);
    let total_old_quantity = Decimal::from(old_quantity.iter().sum::<u64>());

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
    fn test_linear_call_otm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(5);
        let strike = dec!(7);
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_linear_call_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(5);
        let strike = dec!(5);
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_linear_call_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(10);
        let strike = dec!(5);
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_linear_put_otm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(6);
        let strike = dec!(5);
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_linear_put_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(5);
        let strike = dec!(5);
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_linear_put_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(3);
        let strike = dec!(5);
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
            is_linear,
        )
        .unwrap();

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
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(5);
        let strike = dec!(7);
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_inverse_call_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(5);
        let strike = dec!(5);
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_inverse_call_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(10);
        let strike = dec!(5);
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
            is_linear,
        )
        .unwrap();

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
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(5);
        let strike = dec!(3);
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_inverse_put_atm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(5);
        let strike = dec!(5);
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_inverse_put_itm() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(2.5);
        let strike = dec!(5);
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_zero_old_spot() {
        let old_quantity = [100_000, 200_000];
        let old_spot_value = dec!(0);
        let new_spot_value = dec!(2.5);
        let strike = dec!(5);
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
            is_linear,
        )
        .unwrap();

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
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0.5);
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_zero_new_spot_inverse_call() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0.5);
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_zero_new_spot_linear_put() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0.5);
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
            is_linear,
        )
        .unwrap();

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
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0.5);
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_zero_new_spot_zero_strike_linear_call() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_zero_new_spot_zero_strike_inverse_call() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_zero_new_spot_zero_strike_linear_put() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_zero_new_spot_zero_strike_inverse_put() {
        let old_quantity = [100_000; 2];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
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
    fn test_rounding() {
        let old_quantity = [13_456, 52_215];
        let old_spot_value = dec!(5);
        let new_spot_value = dec!(6.698);
        let strike = dec!(5.119);
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            old_spot_value,
            new_spot_value,
            strike,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 12_309);
        assert_eq!(res.new_quantity[1], 53_361);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }
}
