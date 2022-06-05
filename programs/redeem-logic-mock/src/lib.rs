use anchor_lang::prelude::*;

declare_id!("2iLDdNkkU4PWWAr2d5rf2kJTYbfcHWLH7k8cLqHRus4C");

#[program]
pub mod redeem_logic_mock {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
