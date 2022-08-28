use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

declare_id!("Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA");

#[program]
pub mod redeem_logic_lending {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeContext>,
        interest_split: f64,
        fixed_fee_per_tranche: u64,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        require!(interest_split >= 0., RedeemLogicErrors::InvalidInput);
        require!(interest_split <= 1., RedeemLogicErrors::InvalidInput);

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.interest_split = Decimal::from_f64(interest_split)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.fixed_fee_per_tranche = fixed_fee_per_tranche;

        Ok(())
    }

    pub fn update(
        ctx: Context<UpdateContext>,
        interest_split: f64,
        fixed_fee_per_tranche: u64,
    ) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        require!(interest_split >= 0., RedeemLogicErrors::InvalidInput);
        require!(interest_split <= 1., RedeemLogicErrors::InvalidInput);

        redeem_logic_config.interest_split = Decimal::from_f64(interest_split)
            .ok_or(RedeemLogicErrors::MathError)?
            .serialize();
        redeem_logic_config.fixed_fee_per_tranche = fixed_fee_per_tranche;

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
            Decimal::deserialize(input_data.new_reserve_fair_value[0]),
            Decimal::deserialize(ctx.accounts.redeem_logic_config.interest_split),
            ctx.accounts.redeem_logic_config.fixed_fee_per_tranche,
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
    pub fixed_fee_per_tranche: u64,
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    16 + // pub interest_split: [u8; 16],
    8 + // pub fixed_fee_per_tranche: u64,
    32 // pub owner: Pubkey,
    ;
}

fn execute_plugin(
    old_quantity: [u64; 2],
    old_reserve_fair_value: Decimal,
    new_reserve_fair_value: Decimal,
    interest_split: Decimal,
    fixed_fee_per_tranche: u64,
) -> Result<RedeemLogicExecuteResult> {
    // default in the past
    if old_reserve_fair_value == Decimal::ZERO {
        return Ok(RedeemLogicExecuteResult {
            new_quantity: old_quantity,
            fee_quantity: 0,
        });
    }

    let total_old_quantity = old_quantity
        .map(|u| Decimal::from(u))
        .iter()
        .sum::<Decimal>();

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

    let total_old_quantity = total_old_quantity
        .round()
        .to_u64()
        .ok_or(RedeemLogicErrors::MathError)?;

    let senior_new_quantity_with_fee = senior_new_quantity
        .round()
        .to_u64()
        .ok_or(RedeemLogicErrors::MathError)?;
    let junior_new_quantity_with_fee =
        std::cmp::max(0, total_old_quantity - senior_new_quantity_with_fee);

    // fee calculation

    let (senior_new_quantity, senior_tranche_fee) =
        if senior_new_quantity_with_fee > fixed_fee_per_tranche {
            (
                senior_new_quantity_with_fee - fixed_fee_per_tranche,
                fixed_fee_per_tranche,
            )
        } else {
            (senior_new_quantity_with_fee, 0)
        };

    let (junior_new_quantity, junior_tranche_fee) =
        if junior_new_quantity_with_fee > fixed_fee_per_tranche {
            (
                junior_new_quantity_with_fee - fixed_fee_per_tranche,
                fixed_fee_per_tranche,
            )
        } else {
            (junior_new_quantity_with_fee, 0)
        };

    return Ok(RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity: senior_tranche_fee + junior_tranche_fee,
    });
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_flat_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = Decimal::ONE; // 100%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 100_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.75); // 75%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 96_000);
        assert_eq!(res.new_quantity[1], 104_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns_rounding() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.61); // 61%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 99_672);
        assert_eq!(res.new_quantity[1], 100_328);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns_senior_imbalance() {
        let old_quantity = [100_000, 1000];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.75); // 75%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 96_000);
        assert_eq!(res.new_quantity[1], 5_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_positive_returns_junior_imbalance() {
        let old_quantity = [1000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(1.25); // 125%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 960);
        assert_eq!(res.new_quantity[1], 100_040);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.8); // 80%
        let new_reserve = dec!(0.64); // 64%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 125_000);
        assert_eq!(res.new_quantity[1], 75_000);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns_rounding() {
        let old_quantity = [100_000; 2];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.59); // 59%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 101_695);
        assert_eq!(res.new_quantity[1], 98_305);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns_senior_imbalance() {
        let old_quantity = [100_000, 1000];
        let old_reserve = dec!(0.6); // 60%
        let new_reserve = dec!(0.48); // 48%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 101_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_negative_returns_junior_imbalance() {
        let old_quantity = [1000, 100_000];
        let old_reserv = Decimal::ONE; // 100%
        let new_reserve = dec!(0.8); // 80%
        let interest_split = dec!(0.2); // 20%

        let res = execute_plugin(old_quantity, old_reserv, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 1_250);
        assert_eq!(res.new_quantity[1], 99_750);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_junior_wipeout() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.5); // 50%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_junior_wipeout_senior_partial() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = Decimal::ONE; // 100%
        let new_reserve = dec!(0.25); // 25%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_junior_wipeout_senior_wipeout() {
        let old_quantity = [100_000, 100_000];
        let old_reserve = dec!(1.3); // 130%
        let new_reserve = Decimal::ZERO; // 0%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 200_000);
        assert_eq!(res.new_quantity[1], 0);
        assert_eq!(res.fee_quantity, 0);
    }

    #[test]
    fn test_past_wipeout() {
        let old_quantity = [1_000_000, 100_000];
        let old_reserve = Decimal::ZERO; // 0%
        let new_reserve = Decimal::ZERO; // 0%
        let interest_split = dec!(0.2); // 20%

        let res =
            execute_plugin(old_quantity, old_reserve, new_reserve, interest_split, 0).unwrap();

        assert_eq!(res.new_quantity[0], 1_000_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
    }
}
