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


    pub fn execute(_ctx: Context<ExecuteContext>, _input_data: RedeemLogicExecuteInput) -> Result<()> {
        
        msg!("execute now");


        let result: RedeemLogicExecuteResult = RedeemLogicExecuteResult { new_tranche_fairvalue: [1993, 1993] };
        anchor_lang::solana_program::program::set_return_data(&result.try_to_vec()?);

        Ok(())
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
}

#[account]
pub struct RedeemLogicConfig {
    pub interest_split: [u64; 2],
    pub owner: Pubkey,
}

impl RedeemLogicConfig {
    pub const LEN: usize = 8 + // discriminator
    8+8+32;
}

/// We can't move this struct in a library because anchor can't recognize it during IDL generation
/// https://github.com/project-serum/anchor/issues/1058
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteInput {
    pub old_tranche_fair_value: [u64; 2],
    pub old_reserve_fair_value: u64,
    pub new_reserve_fair_value: u64
}

/// We can't move this struct in a library because anchor can't recognize it during IDL generation
/// https://github.com/project-serum/anchor/issues/1058
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct RedeemLogicExecuteResult {
    pub new_tranche_fairvalue: [u64;2]
}