use anchor_lang::prelude::*;
use vyper_utils::math::{to_bps};

declare_id!("9h7eHiqpPbj5Mw5AG59bFH7XDWfHYK52XMfJPbKVas2m");

#[program]
pub mod rate_mock {

    use super::*;

    pub fn fetch_rate(_ctx: Context<CpiReturnContext>) -> Result<u64> {
        Ok(to_bps(0.33).into())
    }
}

#[derive(Accounts)]
pub struct CpiReturnContext<'info> {
    pub account: Account<'info, CpiReturnAccount>,
}

#[account]
pub struct CpiReturnAccount {
    pub value: u64,
}
