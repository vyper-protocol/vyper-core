pub mod decimal_wrapper;

use crate::decimal_wrapper::DecimalWrapper;

use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

declare_id!("3mq416it8YJsd5DKNuWeoCCAH8GYJfpuefHSNkSP6LyS");

#[program]
pub mod redeem_logic_lending_fee {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        interest_split: f64,
        mgmt_fee: f64,
        perf_fee: f64,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        require!(interest_split >= 0., RedeemLogicErrors::InvalidInput);
        require!(interest_split <= 1., RedeemLogicErrors::InvalidInput);

        require!(mgmt_fee >= 0., RedeemLogicErrors::InvalidInput);
        require!(mgmt_fee <= 1., RedeemLogicErrors::InvalidInput);

        require!(perf_fee >= 0., RedeemLogicErrors::InvalidInput);
        require!(perf_fee <= 1., RedeemLogicErrors::InvalidInput);

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config
            .interest_split
            .set(Decimal::from_f64(interest_split).ok_or(RedeemLogicErrors::MathError)?);
        redeem_logic_config
            .mgmt_fee
            .set(Decimal::from_f64(mgmt_fee).ok_or(RedeemLogicErrors::MathError)?);
        redeem_logic_config
            .perf_fee
            .set(Decimal::from_f64(perf_fee).ok_or(RedeemLogicErrors::MathError)?);

        Ok(())
    }

    pub fn update(
        ctx: Context<UpdateContext>,
        interest_split: f64,
        mgmt_fee: f64,
        perf_fee: f64,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        require!(interest_split >= 0., RedeemLogicErrors::InvalidInput);
        require!(interest_split <= 1., RedeemLogicErrors::InvalidInput);

        require!(mgmt_fee >= 0., RedeemLogicErrors::InvalidInput);
        require!(mgmt_fee <= 1., RedeemLogicErrors::InvalidInput);

        require!(perf_fee >= 0., RedeemLogicErrors::InvalidInput);
        require!(perf_fee <= 1., RedeemLogicErrors::InvalidInput);

        redeem_logic_config
            .interest_split
            .set(Decimal::from_f64(interest_split).ok_or(RedeemLogicErrors::MathError)?);
        redeem_logic_config
            .mgmt_fee
            .set(Decimal::from_f64(mgmt_fee).ok_or(RedeemLogicErrors::MathError)?);
        redeem_logic_config
            .perf_fee
            .set(Decimal::from_f64(perf_fee).ok_or(RedeemLogicErrors::MathError)?);

        Ok(())
    }

    pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        input_data.is_valid()?;
        let result: RedeemLogicExecuteResult = execute_plugin(
            input_data.old_quantity,
            input_data.old_reserve_fair_value[0].get(),
            input_data.new_reserve_fair_value[0].get(),
            ctx.accounts.redeem_logic_config.interest_split.get(),
            ctx.accounts.redeem_logic_config.mgmt_fee.get(),
            ctx.accounts.redeem_logic_config.perf_fee.get(),
        )?;

        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value: [DecimalWrapper; 10],
    pub new_reserve_fair_value: [DecimalWrapper; 10],
}

impl RedeemLogicExecuteInput {
    fn is_valid(&self) -> Result<()> {
        for r in self.old_reserve_fair_value {
            require!(r.get() >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
        }

        for r in self.new_reserve_fair_value {
            require!(r.get() >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
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
    pub interest_split: DecimalWrapper,
    pub mgmt_fee: DecimalWrapper,
    pub perf_fee: DecimalWrapper,
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    16 + // pub interest_split: DecimalWrapper,
    16 + // pub mgmt_fee: DecimalWrapper,
    16 + // pub perf_fee: DecimalWrapper,
    32 // pub owner: Pubkey,
    ;
}

fn execute_plugin(
    old_quantity: [u64; 2],
    old_reserve_fair_value: Decimal,
    new_reserve_fair_value: Decimal,
    interest_split: Decimal,
    mgmt_fee: Decimal,
    perf_fee: Decimal,
) -> Result<RedeemLogicExecuteResult> {
    // ensure fees and split are between 0 and 100%
    require!(
        interest_split >= Decimal::ZERO,
        RedeemLogicErrors::InvalidInput
    );
    require!(
        interest_split <= Decimal::ONE,
        RedeemLogicErrors::InvalidInput
    );

    require!(mgmt_fee >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
    require!(mgmt_fee <= Decimal::ONE, RedeemLogicErrors::InvalidInput);

    require!(perf_fee >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
    require!(perf_fee <= Decimal::ONE, RedeemLogicErrors::InvalidInput);

    let total_old_quantity = old_quantity.iter().sum::<u64>();

    // default
    if (old_reserve_fair_value == Decimal::ZERO) || (new_reserve_fair_value == Decimal::ZERO) {
        let senior_new_quantity = (Decimal::from(old_quantity.iter().sum::<u64>())
            * (Decimal::ONE - mgmt_fee))
            .floor()
            .to_u64()
            .ok_or(RedeemLogicErrors::MathError)?;
        return Ok(RedeemLogicExecuteResult {
            new_quantity: [senior_new_quantity, 0u64],
            fee_quantity: total_old_quantity - senior_new_quantity,
        });
    }

    let old_quantity = old_quantity.map(|x| Decimal::from(x));

    let old_value_mgmt =
        old_quantity.map(|x| x * old_reserve_fair_value * (Decimal::ONE - mgmt_fee));

    let new_value_mgmt =
        old_value_mgmt.map(|x| x / old_reserve_fair_value * new_reserve_fair_value);

    let new_value_perf: Vec<Decimal> = old_value_mgmt
        .iter()
        .zip(new_value_mgmt.iter())
        .map(|(&old, &new)| {
            if new > old {
                old + (new - old) * (Decimal::ONE - perf_fee)
            } else {
                new
            }
        })
        .collect();

    let senior_new_value = if new_value_perf[0] > old_value_mgmt[0] {
        old_value_mgmt[0]
            + (new_value_perf[0] - old_value_mgmt[0]) * (Decimal::ONE - interest_split)
    } else {
        old_value_mgmt[0].min(new_value_perf.iter().sum())
    };

    let senior_new_quantity = (senior_new_value / new_reserve_fair_value)
        .floor()
        .to_u64()
        .ok_or(RedeemLogicErrors::MathError)?;

    let junior_new_quantity = ((new_value_perf.iter().sum::<Decimal>() - senior_new_value)
        / new_reserve_fair_value)
        .floor()
        .to_u64()
        .ok_or(RedeemLogicErrors::MathError)?;

    let fee_quantity = total_old_quantity - senior_new_quantity - junior_new_quantity;

    return Ok(RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity: fee_quantity,
    });
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    // TODO check errors

    #[test]
    fn test_flat_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = Decimal::ONE; // 100%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_flat_returns_fee() {
        let old_quantity = [100_000; 2];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = Decimal::ONE; // 100%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.01); // 1%
        let perf_fee = Decimal::ONE; // 100%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 99_000);
        assert_eq!(res.new_quantity[1], 99_000);
        assert_eq!(res.fee_quantity, 2_000);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.75); // 75%
        let interest_split = dec!(0.02); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 96_000);
        assert_eq!(res.new_quantity[1], 104_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_fee() {
        let old_quantity = [100_000; 2];
        let old_reserve_bps = dec!(0.6); // 60%
        let new_reserve_bps = dec!(0.75); // 75%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee_bps = dec!(0.025); // 2.5%
        let perf_fee_bps = dec!(0.07); // 7%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 92_508);
        assert_eq!(res.new_quantity[1], 99_762);
        assert_eq!(res.fee_quantity, 7_730);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_rounding() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.61); // 61%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 99_672);
        assert_eq!(res.new_quantity[1], 100_327);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_rounding_fee() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.61); // 61%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.0649); // 6.49%
        let perf_fee = dec!(0.0123); // 1.23%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 93_188);
        assert_eq!(res.new_quantity[1], 93_793);
        assert_eq!(res.fee_quantity, 13_019);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_senior_imbalance() {
        let old_quantity = [100_000, 1_000];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.75); // 75%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 96_000);
        assert_eq!(res.new_quantity[1], 5_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_senior_imbalance_fee() {
        let old_quantity = [100_000, 1_000];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.75); // 75%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.08); // 8%
        let perf_fee = dec!(0.16); // 16%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 85_964);
        assert_eq!(res.new_quantity[1], 3_981);
        assert_eq!(res.fee_quantity, 11_055);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_junior_imbalance() {
        let old_quantity = [1000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(1.25); // 125%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 960);
        assert_eq!(res.new_quantity[1], 100_040);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_positive_returns_junior_imbalance_fee() {
        let old_quantity = [1000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(1.25); // 125%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.0749); // 7.49%
        let perf_fee = dec!(0.1); // 0.10%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 887);
        assert_eq!(res.new_quantity[1], 92_528);
        assert_eq!(res.fee_quantity, 7_585);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.8); // 80%
        let new_reserve = dec!(0.64); // 64%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 125_000);
        assert_eq!(res.new_quantity[1], 75_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_fee() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.8); // 80%
        let new_reserve = dec!(0.64); // 64%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.1333); // 13.33%
        let perf_fee = dec!(0.812); // 8.12%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 108_337);
        assert_eq!(res.new_quantity[1], 65_002);
        assert_eq!(res.fee_quantity, 26_661);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_rounding() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.59); // 59%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 101_694);
        assert_eq!(res.new_quantity[1], 98_305);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_rounding_fee() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.59); // 59%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.01); // 1%
        let perf_fee = dec!(0.05); // 5%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_677);
        assert_eq!(res.new_quantity[1], 97_322);
        assert_eq!(res.fee_quantity, 2_001);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_senior_imbalance() {
        let old_quantity = [100_000, 1000];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.48); // 48%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 101_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_senior_imbalance_fee() {
        let old_quantity = [100_000, 1000];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.48); // 48%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.05); // 0.5%
        let perf_fee = dec!(0.0567); // 5.67%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 100_495);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 505);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_junior_imbalance() {
        let old_quantity = [1000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.8); // 80%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 1_250);
        assert_eq!(res.new_quantity[1], 99_750);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_negative_returns_junior_imbalance_fee() {
        let old_quantity = [1000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.8); // 80%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.0999); // 9.99%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 1_125);
        assert_eq!(res.new_quantity[1], 89_784);
        assert_eq!(res.fee_quantity, 10_091);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_junior_wipeout() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.5); // 50%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_junior_wipeout_fee() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.5); // 50%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.0999); // 9.99%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 180_020);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 19_980);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_junior_wipeout_senior_partial() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.25); // 25%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_junior_wipeout_senior_partial_fee() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.25); // 25%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.0132); // 1.32%
        let perf_fee = dec!(0.85); // 85%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 197_360);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 2_640);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_junior_wipeout_senior_wipeout() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = dec!(1.3); // 130%
        let new_reserve = Decimal::ZERO; // 0%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_junior_wipeout_senior_wipeout_fee() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = dec!(1.3); // 130%
        let new_reserve = Decimal::ZERO; // 0%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.01); // 1%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 198_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 2_000);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_past_wipeout() {
        let old_quantity = [1_000_000, 100_000];
        let old_reserve = Decimal::ZERO; // 0%
        let new_reserve = Decimal::ZERO; // 0%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = Decimal::ZERO; // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 1_100_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_past_wipeout_fee() {
        let old_quantity = [1_000_000, 100_000];
        let old_reserve = Decimal::ZERO; // 0%
        let new_reserve = Decimal::ZERO; // 0%
        let interest_split = dec!(0.2); // 20%
        let mgmt_fee = dec!(0.1); // 0%
        let perf_fee = Decimal::ZERO; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve,
            new_reserve,
            interest_split,
            mgmt_fee,
            perf_fee,
        )
        .unwrap();

        assert_eq!(res.new_quantity[0], 990_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 110_000);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }
}
