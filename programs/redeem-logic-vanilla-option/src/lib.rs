// Vyper Redeem Logic: Vanilla Option Contract
// Example: SOL/USD call option
// The notional of the contract is in base asset (e.g. SOL in a SOL/USD contract)
// Supports both linear and inverse settlement. For example for a SOL/USD contract:
// - if is_linear provide USD as collateral
// - else provide SOL
// Senior [0] is long the option, junior [1] is short
// Senior amount is the option premium paid in any case, junior amount is the collateral paid only if it expires ITM
// Learn more here: https://vyperprotocol.notion.site/Contract-Payoff-Vanilla-Option-47b362270a164d7b96732d139e4d7ee2

use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "Redeem Logic Vanilla Option | Vyper Core",
    project_url: "https://vyperprotocol.io",
    contacts: "email:info@vyperprotocol.io,link:https://docs.vyperprotocol.io/",
    policy: "https://github.com/vyper-protocol/vyper-core/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/vyper-protocol/vyper-core/tree/main/programs/redeem-logic-vanilla-option"
}

declare_id!("8fSeRtFseNrjdf8quE2YELhuzLkHV7WEGRPA9Jz8xEVe");

#[program]
pub mod redeem_logic_vanilla_option {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        strike: f64,
        notional: u64,
        is_call: bool,
        is_linear: bool,
    ) -> Result<()> {
        require!(strike >= 0., RedeemLogicErrors::InvalidInput);

        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;
        redeem_logic_config.strike = Decimal::from_f64(strike)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.notional = notional;
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
            Decimal::deserialize(input_data.new_reserve_fair_value[0]),
            Decimal::deserialize(ctx.accounts.redeem_logic_config.strike),
            ctx.accounts.redeem_logic_config.notional,
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

    /// Signer account
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteContext<'info> {
    #[account()]
    pub redeem_logic_config: Account<'info, RedeemLogicConfig>,
}

#[account]
pub struct RedeemLogicConfig {
    pub strike: [u8; 16],
    pub notional: u64,

    /// true if call, false if put
    pub is_call: bool,

    /// true if linear, false if inverse
    pub is_linear: bool,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    16 +  // pub strike: [u8; 16],
    8 + // pub notional: u64,
    1 + // pub is_call: bool,
    1 // pub is_linear: bool,
    ;

    fn dump(&self) {
        msg!("redeem logic config:");
        msg!("+ strike: {:?}", Decimal::deserialize(self.strike));
        msg!("+ notional: {:?}", self.notional);
        msg!("+ is_call: {:?}", self.is_call);
        msg!("+ is_linear: {:?}", self.is_linear);
    }
}

fn execute_plugin(
    old_quantity: [u64; 2],
    new_spot: Decimal,
    strike: Decimal,
    notional: u64,
    is_call: bool,
    is_linear: bool,
) -> Result<RedeemLogicExecuteResult> {
    require!(strike >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);

    let notional = Decimal::from(notional);

    let payoff = notional * {
        if new_spot == Decimal::ZERO && !is_linear {
            if !is_call || strike > Decimal::ZERO {
                Decimal::ZERO
            } else {
                Decimal::ONE
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

    let junior_old_quantity = Decimal::from(old_quantity[1]);
    let senior_new_quantity = junior_old_quantity.min(payoff);

    let senior_new_quantity = senior_new_quantity
        .floor()
        .to_u64()
        .ok_or(RedeemLogicErrors::MathError)?;
    let junior_new_quantity = old_quantity.iter().sum::<u64>() - senior_new_quantity;

    Ok(RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity: 0,
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_linear_call_otm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(5);
        let strike = dec!(7);
        let notional = 5_000;
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_call_itm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(5);
        let strike = dec!(3);
        let notional = 5_000;
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_call_itm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(5);
        let strike = dec!(3);
        let notional = 5_000;
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 2_000);
        assert_eq!(res.new_quantity[1], 108_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_call_deep_itm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(500);
        let strike = dec!(3);
        let notional = 5_000;
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 10_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_put_otm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(5);
        let strike = dec!(3);
        let notional = 5_000;
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_put_itm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(3);
        let strike = dec!(5);
        let notional = 5_000;
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_inverse_put_itm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(3);
        let strike = dec!(5);
        let notional = 5_000;
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 3_333);
        assert_eq!(res.new_quantity[1], 106_667);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_linear_put_deep_itm() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(3);
        let strike = dec!(500);
        let notional = 5_000;
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 10_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_linear_call_zero_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let notional = 5_000;
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_linear_call_pos_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(1);
        let notional = 5_000;
        let is_call = true;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_linear_put_zero_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let notional = 5_000;
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_linear_put_pos_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(1);
        let notional = 5_000;
        let is_call = false;
        let is_linear = true;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 5_000);
        assert_eq!(res.new_quantity[1], 105_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_inverse_call_zero_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let notional = 5_000;
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 5_000);
        assert_eq!(res.new_quantity[1], 105_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_inverse_call_pos_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(1);
        let notional = 5_000;
        let is_call = true;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_inverse_put_zero_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(0);
        let notional = 5_000;
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_zero_sett_inverse_put_pos_strike() {
        let old_quantity = [10_000, 100_000];
        let new_spot_value = dec!(0);
        let strike = dec!(1);
        let notional = 5_000;
        let is_call = false;
        let is_linear = false;

        let res = execute_plugin(
            old_quantity,
            new_spot_value,
            strike,
            notional,
            is_call,
            is_linear,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 110_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }
}
