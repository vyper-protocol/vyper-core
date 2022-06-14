use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_math::bps::from_bps;

declare_id!("Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA");

#[program]
pub mod redeem_logic_lending {


    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>, interest_split: u32) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.interest_split = interest_split;

        Ok(())
    }

    pub fn update(ctx: Context<UpdateContext>, interest_split: u32) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.interest_split = interest_split;

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
        );

        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value_bps: u32,
    pub new_reserve_fair_value_bps: u32
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteResult {
    pub new_quantity: [u64;2],
    pub fee_quantity: u64
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
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    4+32;
}

fn execute_plugin(
    old_quantity: [u64; 2],
    old_reserve_fair_value_bps: u32,
    new_reserve_fair_value_bps: u32,
    interest_split_bps: u32,
) -> RedeemLogicExecuteResult {

    // default in the past
    if old_reserve_fair_value_bps == 0 {
        return RedeemLogicExecuteResult {
            new_quantity: old_quantity,
            fee_quantity: 0,
        };
    }

    let total_old_quantity = old_quantity
        .map(|u| Decimal::from(u))
        .iter()
        .sum::<Decimal>();
    let old_reserve_fair_value = from_bps(old_reserve_fair_value_bps).unwrap();
    let interest_split = from_bps(interest_split_bps).unwrap();
    let new_reserve_fair_value = from_bps(new_reserve_fair_value_bps).unwrap();

    // positive return, share proceeds
    let senior_new_quantity = if new_reserve_fair_value > old_reserve_fair_value {
        Decimal::from(old_quantity[0]) * old_reserve_fair_value / new_reserve_fair_value
            * (Decimal::ONE
                + (new_reserve_fair_value / old_reserve_fair_value - Decimal::ONE)
                    * (Decimal::ONE - interest_split))
    } else {
        // total loss
        if new_reserve_fair_value == Decimal::ZERO {
            total_old_quantity
        // partial loss
        } else {
            total_old_quantity.min(
                Decimal::from(old_quantity[0]) * old_reserve_fair_value / new_reserve_fair_value,
            )
        }
    };

    let total_old_quantity = total_old_quantity.round().to_u64().unwrap();

    let senior_new_quantity = senior_new_quantity.round().to_u64().unwrap();

    // max(0, ..) should be superfluos
    let junior_new_quantity = std::cmp::max(0, total_old_quantity - senior_new_quantity);

    // true by construction
    // assert senior + junior == old_tranche_quantity.senior + old_tranche_quantity.junior

    return RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 10_000; // 100%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 7_500; // 75%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 96_000);
        assert_eq!(res.new_quantity[1], 104_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns_rounding() {
        let old_quantity = [100_000; 2];
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 6_100; // 61%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 99_672);
        assert_eq!(res.new_quantity[1], 100_328);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns_senior_imbalance() {
        let old_quantity = [100_000, 1000];
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 7_500; // 75%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 96_000);
        assert_eq!(res.new_quantity[1], 5_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns_junior_imbalance() {
        let old_quantity = [1000, 100_000];
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 12_500; // 125%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 960);
        assert_eq!(res.new_quantity[1], 100_040);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve_bps = 8_000; // 80%
        let new_reserve_bps = 6_400; // 64%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 125_000);
        assert_eq!(res.new_quantity[1], 75_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns_rounding() {
        let old_quantity = [100_000; 2];
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 5_900; // 59%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 101_695);
        assert_eq!(res.new_quantity[1], 98_305);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns_senior_imbalance() {
        let old_quantity = [100_000, 1000];
        let old_reserve_bps = 6_000; // 60%
        let new_reserve_bps = 4_800; // 48%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 101_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns_junior_imbalance() {
        let old_quantity = [1000, 100_000];
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 8_000; // 80%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 1_250);
        assert_eq!(res.new_quantity[1], 99_750);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_junior_wipeout() {
        let old_quantity = [100_000, 100_000];
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 5_000; // 50%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_junior_wipeout_senior_partial() {
        let old_quantity = [100_000, 100_000];
        let old_reserve_bps = 10_000; // 100%
        let new_reserve_bps = 2_500; // 25%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_junior_wipeout_senior_wipeout() {
        let old_quantity = [100_000, 100_000];
        let old_reserve_bps = 13_000; // 130%
        let new_reserve_bps = 0; // 0%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_past_wipeout() {
        let old_quantity = [1_000_000, 100_000];
        let old_reserve_bps = 0; // 0%
        let new_reserve_bps = 0; // 0%
        let interest_split = 2_000; // 20%

        let res = execute_plugin(
            old_quantity,
            old_reserve_bps,
            new_reserve_bps,
            interest_split,
        );

        assert_eq!(res.new_quantity[0], 1_000_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
    }
}
