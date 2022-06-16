use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_math::bps::{from_bps, BpsRangeValue};

declare_id!("Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA");

#[program]
pub mod redeem_logic_lending_fee {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        interest_split: u32,
        mgmt_fee: u32,
        perf_fee: u32,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        let interest_split = BpsRangeValue::new(interest_split).unwrap();
        let mgmt_fee = BpsRangeValue::new(mgmt_fee).unwrap();
        let perf_fee = BpsRangeValue::new(perf_fee).unwrap();

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.interest_split = interest_split.get();
        redeem_logic_config.mgmt_fee = mgmt_fee.get();
        redeem_logic_config.perf_fee = perf_fee.get();

        Ok(())
    }

    pub fn update(
        ctx: Context<UpdateContext>,
        interest_split: u32,
        mgmt_fee: u32,
        perf_fee: u32,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        let interest_split = BpsRangeValue::new(interest_split).unwrap();
        let mgmt_fee = BpsRangeValue::new(mgmt_fee).unwrap();
        let perf_fee = BpsRangeValue::new(perf_fee).unwrap();

        redeem_logic_config.interest_split = interest_split.get();
        redeem_logic_config.mgmt_fee = mgmt_fee.get();
        redeem_logic_config.perf_fee = perf_fee.get();

        Ok(())
    }

    pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        let result: RedeemLogicExecuteResult = execute_plugin(
            input_data.old_quantity,
            input_data.old_reserve_fair_value_bps,
            input_data.new_reserve_fair_value_bps,
            ctx.accounts.redeem_logic_config.interest_split,
            ctx.accounts.redeem_logic_config.mgmt_fee,
            ctx.accounts.redeem_logic_config.perf_fee,
        );

        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value_bps: u32,
    pub new_reserve_fair_value_bps: u32,
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
    pub interest_split: u32,
    pub mgmt_fee: u32,
    pub perf_fee: u32,
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    4+4+4+32;
}

fn execute_plugin(
    old_quantity: [u64; 2],
    old_reserve_fair_value_bps: u32,
    new_reserve_fair_value_bps: u32,
    interest_split_bps: u32,
    mgmt_fee_bps: u32,
    perf_fee_bps: u32,
) -> RedeemLogicExecuteResult {
    // ensure fees and split are between 0 and 100%
    let interest_split = BpsRangeValue::new(interest_split_bps).unwrap();
    let mgmt_fee = BpsRangeValue::new(mgmt_fee_bps).unwrap();
    let perf_fee = BpsRangeValue::new(perf_fee_bps).unwrap();

    let total_old_quantity = old_quantity.iter().sum::<u64>();

    // default
    if (old_reserve_fair_value_bps == 0) || (new_reserve_fair_value_bps == 0) {
        let senior_new_quantity = (Decimal::from(old_quantity.iter().sum::<u64>())
            * (Decimal::ONE - mgmt_fee.get_decimal().unwrap()))
        .floor()
        .to_u64()
        .unwrap();
        return RedeemLogicExecuteResult {
            new_quantity: [senior_new_quantity, 0u64],
            fee_quantity: total_old_quantity - senior_new_quantity,
        };
    }

    let old_reserve_fair_value = from_bps(old_reserve_fair_value_bps).unwrap();
    let new_reserve_fair_value = from_bps(new_reserve_fair_value_bps).unwrap();

    let old_quantity = old_quantity.map(|x| Decimal::from(x));

    let old_value_mgmt = old_quantity
        .map(|x| x * old_reserve_fair_value * (Decimal::ONE - mgmt_fee.get_decimal().unwrap()));

    let new_value_mgmt =
        old_value_mgmt.map(|x| x / old_reserve_fair_value * new_reserve_fair_value);

    let new_value_perf: Vec<Decimal> = old_value_mgmt
        .iter()
        .zip(new_value_mgmt.iter())
        .map(|(&old, &new)| {
            if new > old {
                old + (new - old) * (Decimal::ONE - perf_fee.get_decimal().unwrap())
            } else {
                new
            }
        })
        .collect();

    let senior_new_value = if new_value_perf[0] > old_value_mgmt[0] {
        old_value_mgmt[0]
            + (new_value_perf[0] - old_value_mgmt[0])
                * (Decimal::ONE - interest_split.get_decimal().unwrap())
    } else {
        old_value_mgmt[0].min(new_value_perf.iter().sum())
    };

    let senior_new_quantity = (senior_new_value / new_reserve_fair_value)
        .floor()
        .to_u64()
        .unwrap();

    let junior_new_quantity = ((new_value_perf.iter().sum::<Decimal>() - senior_new_value)
        / new_reserve_fair_value)
        .floor()
        .to_u64()
        .unwrap();

    let fee_quantity = total_old_quantity - senior_new_quantity - junior_new_quantity;

    return RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity: fee_quantity,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO check errors

    #[test]
    fn test_flat_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 10_000; // 100%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 10_000; // 100%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 100; // 1%
        let perf_fee_bps = 10_000u32; // 100%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 7_500; // 75%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 7_500; // 75%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 250; // 2.5%
        let perf_fee_bps = 700; // 7%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 6_100; // 61%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 6_100; // 61%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 649; // 6.49%
        let perf_fee_bps = 123; // 1.23%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 7_500; // 75%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 7_500; // 75%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 800; // 8%
        let perf_fee_bps = 1_600; // 16%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 12_500; // 125%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 12_500; // 125%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 749; // 7.49%
        let perf_fee_bps = 10; // 0.10%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 8_000; // 80%
        let new_reserve_bps = 6_400; // 64%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 8_000; // 80%
        let new_reserve_bps = 6_400; // 64%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 1_333; // 13.33%
        let perf_fee_bps = 812; // 8.12%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 5_900; // 59%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 5_900; // 59%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 100; // 1%
        let perf_fee_bps = 500; // 5%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 4_800; // 48%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 4_800; // 48%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 50; // 0.5%
        let perf_fee_bps = 567; // 5.67%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 8_000; // 80%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 8_000; // 80%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 999; // 9.99%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 5_000; // 50%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 5_000; // 50%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 999; // 9.99%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 2_500; // 25%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 2_500; // 25%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 132; // 1.32%
        let perf_fee_bps = 8_500; // 85%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 13_000; // 130%
        let new_reserve_bps = 0; // 0%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 13_000; // 130%
        let new_reserve_bps = 0; // 0%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 100; // 1%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 0; // 0%
        let new_reserve_bps = 0; // 0%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 0u32; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

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
        let old_reserve_bps = 0; // 0%
        let new_reserve_bps = 0; // 0%
        let interest_split = 2_000; // 20%
        let mgmt_fee_bps = 1_000; // 0%
        let perf_fee_bps = 0u32; // 0%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
            mgmt_fee_bps,
            perf_fee_bps,
        );

        assert_eq!(res.new_quantity[0], 990_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 110_000);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }
}
