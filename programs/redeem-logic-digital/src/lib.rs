// Vyper Redeem Logic: Digital Option Contract
// Example: BTC/USD digital paying 1 USD if BTC/USD >= 20000 at expiry
// Supports both digi-calls and digi-puts
// Senior [0] is long the option, junior [1] is short
// Senior amount is the option premium paid in any case, junior amount is the digi-size paid only if it expires ITM
// For convention, call expires ITM if >=, while put expires ITM if <
// Learn more here: https://vyperprotocol.notion.site/Contract-Payoff-Digital-39cab877a28a4a6fa349d9b816bd15a4

use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "Redeem Logic Digital | Vyper Core",
    project_url: "https://vyperprotocol.io",
    contacts: "email:info@vyperprotocol.io,link:https://docs.vyperprotocol.io/",
    policy: "https://github.com/vyper-protocol/vyper-core/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/vyper-protocol/vyper-core/tree/main/programs/redeem-logic-digital"
}

declare_id!("5Dq9PjUJUG5dM9DzYFqKA4YZYeKJfGaM5Gy7NjpY3p5r");

#[program]
pub mod redeem_logic_digital {
    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>, strike: f64, is_call: bool) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;
        redeem_logic_config.strike = Decimal::from_f64(strike)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.is_call = is_call;

        Ok(())
    }

    pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        // input_data.is_valid()?;
        ctx.accounts.redeem_logic_config.dump();

        let result: RedeemLogicExecuteResult = execute_plugin(
            input_data.old_quantity,
            Decimal::deserialize(input_data.new_reserve_fair_value[0]),
            Decimal::deserialize(ctx.accounts.redeem_logic_config.strike),
            ctx.accounts.redeem_logic_config.is_call,
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

// impl RedeemLogicExecuteInput {
//     fn is_valid(&self) -> Result<()> {
//         Result::Ok(())
//     }
// }

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
    /// true if call, false if put
    pub is_call: bool,

    pub strike: [u8; 16],
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    1 + // pub is_call: bool,
    16  // pub strike: [u8; 16],
    ;

    fn dump(&self) {
        msg!("redeem logic config:");
        msg!("+ is_call: {:?}", self.is_call);
        msg!("+ strike: {:?}", Decimal::deserialize(self.strike))
    }
}

fn execute_plugin(
    old_quantity: [u64; 2],
    new_spot: Decimal,
    strike: Decimal,
    is_call: bool,
) -> Result<RedeemLogicExecuteResult> {
    let senior_new_quantity = {
        if (is_call && new_spot >= strike) || (!is_call && new_spot < strike) {
            old_quantity[1]
        } else {
            0
        }
    };

    let junior_new_quantity = old_quantity.iter().sum::<u64>() - senior_new_quantity;

    Ok(RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_itm() {
        let old_quantity = [100_000; 2];
        let new_spot = Decimal::ONE_HUNDRED;
        let strike = Decimal::ONE_HUNDRED;
        let is_call = true;

        let res = execute_plugin(old_quantity, new_spot, strike, is_call).unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_call_otm() {
        let old_quantity = [100_000; 2];
        let new_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let is_call = true;

        let res = execute_plugin(old_quantity, new_spot, strike, is_call).unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_put_itm() {
        let old_quantity = [100_000; 2];
        let new_spot = Decimal::ONE;
        let strike = Decimal::ONE_HUNDRED;
        let is_call = false;

        let res = execute_plugin(old_quantity, new_spot, strike, is_call).unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }

    #[test]
    fn test_put_otm() {
        let old_quantity = [100_000; 2];
        let new_spot = Decimal::ONE_HUNDRED;
        let strike = Decimal::ONE_HUNDRED;
        let is_call = false;

        let res = execute_plugin(old_quantity, new_spot, strike, is_call).unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        );
    }
}
