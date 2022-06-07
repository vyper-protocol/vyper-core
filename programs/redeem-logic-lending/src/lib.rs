use anchor_lang::prelude::*;

declare_id!("Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA");

#[program]
pub mod redeem_logic_lending {

    use super::*;

    pub fn initialize(ctx: Context<InitializeContext>, interest_split: [u64; 2]) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.interest_split = interest_split;

        Ok(())
    }

    pub fn update(ctx: Context<UpdateContext>, interest_split: [u64; 2]) -> Result<()> {
        let redeem_logic_config = &mut ctx.accounts.redeem_logic_config;

        redeem_logic_config.owner = ctx.accounts.owner.key();
        redeem_logic_config.interest_split = interest_split;

        Ok(())
    }

    pub fn execute(_ctx: Context<ExecuteContext>, _old_tranche_fair_value: [u64; 2], _old_reserve_fair_value: u64, _new_reserve_fair_value: u64) -> Result<[u64; 2]> {
        msg!("execute now");

        Ok([1993, 1993])
    }
}

#[derive(Accounts)]
pub struct InitializeContext<'info> {
    
    /// Tranche config account, where all the parameters are saved
    /// TODO size TBD
    #[account(init, payer = payer, space = 8 + 16 + 32)]
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

    /// CHECK: Owner of the tranche config
    #[account()]
    pub signer: Signer<'info>,
}

#[account]
pub struct RedeemLogicConfig {
    pub interest_split: [u64; 2],
    pub owner: Pubkey,
}
