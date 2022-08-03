use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use vyper_utils::redeem_logic_common::RedeemLogicErrors;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// payoff for a Forward Impermanent Loss Agreement
// for payoff derivation see: https://heremitas.notion.site/FILA-a-new-hedge-for-impermanent-loss-91531343bda7420c8082fc9a98202074
// senior ([0]) is IL receiver (i.e. buying IL protection) and pays the premium senior_qty at expiry
// junior ([1]) is IL payer (i.e. providing IL protection), receives the premium (senior_qty) at expiry and any residual collateral
// collateral should be the quote asset, e.g. USDC for SOL/USDC (i.e. no quanto)
// fully collateralized payoff, which amounts to the IL protection being active only in a range around the strike, see link above for more details
// notional is in base asset (e.g. SOL for SOL/USDC) and the equivalent USDC (determined by the strike)

#[program]
pub mod redeem_logic_fila {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>, strike: f64, notional: f64) -> Result<()> {
        require!(strike >= 0., RedeemLogicErrors::InvalidInput);
        require!(notional >= 0., RedeemLogicErrors::InvalidInput);

        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.strike =
            Decimal::from_f64(strike).ok_or(RedeemLogicErrors::MathError)?;
        redeem_logic_config.notional =
            Decimal::from_f64(notional).ok_or(RedeemLogicErrors::MathError)?;

        Ok(())
    }

    pub fn update(ctx: Context<UpdateContext>, strike: f64, notional: f64) -> Result<()> {
        require!(strike >= 0., RedeemLogicErrors::InvalidInput);
        require!(notional >= 0., RedeemLogicErrors::InvalidInput);

        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.strike =
            Decimal::from_f64(strike).ok_or(RedeemLogicErrors::MathError)?;
        redeem_logic_config.notional =
            Decimal::from_f64(notional).ok_or(RedeemLogicErrors::MathError)?;

        Ok(())
    }

    pub fn execute(
        ctx: Context<ExecuteContext>,
        input_data: RedeemLogicExecuteInput,
    ) -> Result<()> {
        input_data.is_valid()?;
        let result: RedeemLogicExecuteResult = execute_plugin(
            input_data.old_quantity,
            // input_data.old_reserve_fair_value[0],
            input_data.new_reserve_fair_value[0],
            ctx.accounts.redeem_logic_config.strike,
            ctx.accounts.redeem_logic_config.notional,
        )?;

        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_quantity: [u64; 2],
    pub old_reserve_fair_value: [Decimal; 10],
    pub new_reserve_fair_value: [Decimal; 10],
}

impl RedeemLogicExecuteInput {
    fn is_valid(&self) -> Result<()> {
        for r in self.old_reserve_fair_value {
            require!(r >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
        }

        for r in self.new_reserve_fair_value {
            require!(r >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
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
    pub strike: Decimal,
    pub notional: Decimal,
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    16 + // pub strike: Decimal,
    16 + // pub notional: Decimal,
    32 // pub owner: Pubkey,
    ;
}

fn execute_plugin(
    old_quantity: [u64; 2],
    // old_spot: Decimal,
    new_spot: Decimal,
    strike: Decimal,
    notional: Decimal,
) -> Result<RedeemLogicExecuteResult> {
    // TODO check overflow

    require!(new_spot >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
    require!(strike >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);
    require!(notional >= Decimal::ZERO, RedeemLogicErrors::InvalidInput);

    let junior_old_quantity = Decimal::from(old_quantity[1]);
    let total_old_quantity = Decimal::from(old_quantity.iter().sum::<u64>());

    let payoff = strike + new_spot - Decimal::TWO * (new_spot * strike).sqrt().unwrap();

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

    return Ok(RedeemLogicExecuteResult {
        new_quantity: [senior_new_quantity, junior_new_quantity],
        fee_quantity,
    });
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_flat_returns() {
        let old_quantity = [100_000; 2];
        let new_spot_value = dec!(1);
        let strike = dec!(1);
        let notional = dec!(1);

        let res = execute_plugin(old_quantity, new_spot_value, strike, notional).unwrap();

        assert_eq!(res.new_quantity[0], 0);
        assert_eq!(res.new_quantity[1], 200_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_il_down() {
        let old_quantity = [100_000; 2];
        let new_spot_value = dec!(50);
        let strike = dec!(100);
        let notional = dec!(1_000);

        let res = execute_plugin(old_quantity, new_spot_value, strike, notional).unwrap();

        assert_eq!(res.new_quantity[0], 8_578);
        assert_eq!(res.new_quantity[1], 191_421);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_il_up() {
        let old_quantity = [100_000; 2];
        let new_spot_value = dec!(150);
        let strike = dec!(100);
        let notional = dec!(1_000);

        let res = execute_plugin(old_quantity, new_spot_value, strike, notional).unwrap();

        assert_eq!(res.new_quantity[0], 5_051);
        assert_eq!(res.new_quantity[1], 194_948);
        assert_eq!(res.fee_quantity, 1);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_range_bottom() {
        let old_quantity = [100_000, 10_000];
        let new_spot_value = dec!(46.75);
        let strike = dec!(100);
        let notional = dec!(1_000);

        let res = execute_plugin(old_quantity, new_spot_value, strike, notional).unwrap();

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_range_top() {
        let old_quantity = [100_000, 10_000];
        let new_spot_value = dec!(173.25);
        let strike = dec!(100);
        let notional = dec!(1_000);

        let res = execute_plugin(old_quantity, new_spot_value, strike, notional).unwrap();

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_zero_new() {
        let old_quantity = [100_000, 10_000];
        let new_spot_value = Decimal::ZERO;
        let strike = dec!(100);
        let notional = dec!(1_000);

        let res = execute_plugin(old_quantity, new_spot_value, strike, notional).unwrap();

        assert_eq!(res.new_quantity[0], 10_000);
        assert_eq!(res.new_quantity[1], 100_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }

    #[test]
    fn test_zero_strike() {
        let old_quantity = [100_000; 2];
        let new_spot_value = dec!(50);
        let strike = Decimal::ZERO;
        let notional = dec!(1_000);

        let res = execute_plugin(old_quantity, new_spot_value, strike, notional).unwrap();

        assert_eq!(res.new_quantity[0], 50_000);
        assert_eq!(res.new_quantity[1], 150_000);
        assert_eq!(res.fee_quantity, 0);
        assert_eq!(
            old_quantity.iter().sum::<u64>(),
            res.new_quantity.iter().sum::<u64>() + res.fee_quantity
        )
    }
}
