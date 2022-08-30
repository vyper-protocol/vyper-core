use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

declare_id!("Fd87TGcYmWs1Gfa7XXZycJwt9kXjRs8axMtxCWtCmowN");

#[program]
pub mod redeem_logic_farming {

    use vyper_utils::redeem_logic_common::RedeemLogicErrors;

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>, interest_split: f64) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        require!(interest_split >= 0., RedeemLogicErrors::InvalidInput);
        require!(interest_split <= 1., RedeemLogicErrors::InvalidInput);

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.interest_split = Decimal::from_f64(interest_split)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();

        Ok(())
    }

    pub fn update(ctx: Context<UpdateContext>, interest_split: f64) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        require!(interest_split >= 0., RedeemLogicErrors::InvalidInput);
        require!(interest_split <= 1., RedeemLogicErrors::InvalidInput);

        redeem_logic_config.interest_split = Decimal::from_f64(interest_split)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();

        Ok(())
    }

    pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        input_data.is_valid()?;

        let result: RedeemLogicExecuteResult = execute_plugin(
            input_data.old_quantity,
            Decimal::deserialize(input_data.old_reserve_fair_value[0]),
            Decimal::deserialize(input_data.old_reserve_fair_value[1]),
            Decimal::deserialize(input_data.new_reserve_fair_value[0]),
            Decimal::deserialize(input_data.new_reserve_fair_value[1]),
            Decimal::deserialize(ctx.accounts.redeem_logic_config.interest_split),
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

        return Result::Ok(());
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
    pub interest_split: [u8; 16],
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    16 + // pub interest_split: Decimal,
    32 // pub owner: Pubkey,
    ;
}

fn execute_plugin(
    old_quantity: [u64; 2],
    old_lp_fair_value: Decimal,
    old_ul_fair_value: Decimal,
    new_lp_fair_value: Decimal,
    new_ul_fair_value: Decimal,
    interest_split: Decimal,
) -> Result<RedeemLogicExecuteResult> {
    // split is between 0 and 100%
    require!(
        interest_split >= Decimal::ZERO,
        RedeemLogicErrors::InvalidInput
    );
    require!(
        interest_split <= Decimal::ONE,
        RedeemLogicErrors::InvalidInput
    );

    // default
    if (old_lp_fair_value == Decimal::ZERO)
        || (old_ul_fair_value == Decimal::ZERO)
        || (new_lp_fair_value == Decimal::ZERO)
        || (new_ul_fair_value == Decimal::ZERO)
    {
        let senior_new_quantity = old_quantity.iter().sum::<u64>();
        return Ok(RedeemLogicExecuteResult {
            new_quantity: [senior_new_quantity, 0],
            fee_quantity: 0,
        });
    }

    let total_old_quantity = Decimal::from(old_quantity.iter().sum::<u64>());

    // half of LP token is quote ccy
    let lp_delta = (new_ul_fair_value - old_ul_fair_value)
        * old_lp_fair_value
        * Decimal::from_f64(0.5f64).ok_or(RedeemLogicErrors::MathError)?
        / old_ul_fair_value;
    let lp_il = Decimal::from(2u64)
        * (old_ul_fair_value * new_ul_fair_value)
            .sqrt()
            .ok_or(RedeemLogicErrors::MathError)?
        - old_ul_fair_value
        - new_ul_fair_value;
    let lp_no_accrued = old_lp_fair_value + lp_delta + lp_il;

    let accrued = Decimal::ZERO.max(new_lp_fair_value - lp_no_accrued);

    let net_value = accrued * (Decimal::ONE - interest_split) + old_lp_fair_value + lp_delta;

    let senior_new_quantity =
        total_old_quantity.min(Decimal::from(old_quantity[0]) * net_value / new_lp_fair_value);
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

    return Ok(RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal_macros::dec;

    // TODO check errors

    #[test]
    fn test_flat_returns() {
        let old_quantity = [10_000; 2];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = Decimal::TWO; // 200%
        let new_ul_fair_value = Decimal::ONE; // 100%
        let interest_split = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 10_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_no_il() {
        let old_quantity = [10_000; 2];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(3); // 300%
        let new_ul_fair_value = Decimal::ONE; // 100%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 9_000);
        assert_eq!(res.new_quantity[1], 11_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_no_il_rounding() {
        let old_quantity = [10_000; 2];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(3); // 300%
        let new_ul_fair_value = Decimal::ONE; // 100%
        let interest_split = dec!(0.25); // 25%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 9_166);
        assert_eq!(res.new_quantity[1], 10_833);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_il() {
        let old_quantity = [10_000; 2];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(2.1213); // 212.13%
        let new_ul_fair_value = dec!(0.5); // 50%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 9_404);
        assert_eq!(res.new_quantity[1], 10_595);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_senior_imbalance() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(2.1213); // 212.13%
        let new_ul_fair_value = dec!(0.5); // 50%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 9_404);
        assert_eq!(res.new_quantity[1], 1_595);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_junior_imbalance() {
        let old_quantity = [1_000, 10_000];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(2.1213); // 212.13%
        let new_ul_fair_value = dec!(0.5); // 50%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 940);
        assert_eq!(res.new_quantity[1], 10_059);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_no_fees() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(1.4142); // 141.42%
        let new_ul_fair_value = dec!(0.5); // 50%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 10_606);
        assert_eq!(res.new_quantity[1], 0_393);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_fees() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(1.7678); // 176.78%
        let new_ul_fair_value = dec!(0.5); // 50%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 9_885);
        assert_eq!(res.new_quantity[1], 1_114);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_junior_wipeout() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = Decimal::TWO; // 200%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(0.2); // 20%
        let new_ul_fair_value = dec!(0.01); // 1%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 11_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_default() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = Decimal::ZERO; // 0%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = Decimal::TWO; // 200%
        let new_ul_fair_value = Decimal::ONE; // 100%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 11_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_lp_accrued_flat() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = dec!(4); // 400%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(4); // 400%
        let new_ul_fair_value = Decimal::ONE; // 100%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 1_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_lp_accrued_positive_returns() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = dec!(4); // 400%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(5.2); // 520%
        let new_ul_fair_value = Decimal::ONE; // 100%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 9_307);
        assert_eq!(res.new_quantity[1], 1_692);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_lp_accrued_negative_returns() {
        let old_quantity = [10_000, 1_000];
        let old_lp_fair_value = dec!(4); // 400%
        let old_ul_fair_value = Decimal::ONE; // 100%
        let new_lp_fair_value = dec!(3.677); // 367.70%
        let new_ul_fair_value = dec!(0.5); // 50%
        let interest_split = dec!(0.3); // 30%

        let res = execute_plugin(
            old_quantity,
            old_lp_fair_value,
            old_ul_fair_value,
            new_lp_fair_value,
            new_ul_fair_value,
            interest_split,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 9_610);
        assert_eq!(res.new_quantity[1], 1_389);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }
}
