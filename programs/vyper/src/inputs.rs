use crate::{error::ErrorCode, state::TrancheConfig};
use anchor_lang::prelude::*;
use std::result::Result;


pub trait Input {
    fn is_valid(&self) -> Result<(), ErrorCode>;
}

// + + + + + + + + + + + +

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct CreateTrancheConfigInput {
    pub capital_split: [u32; 2],
    pub interest_split: [u32; 2],
    pub mint_count: [u64; 2],
    pub start_date: u64,
    pub end_date: u64,
    pub create_serum: bool,
    pub can_mint_more: bool,
    pub protocol_bump: u8,
}

impl Input for CreateTrancheConfigInput {
    fn is_valid(&self) -> Result<(), ErrorCode> {

        if self.start_date > self.end_date {
            return Result::Err(ErrorCode::InvalidInput);
        }

        return Result::Ok(());
    }
}

impl CreateTrancheConfigInput {
    pub fn create_tranche_config(&self, data: &mut TrancheConfig) {
        data.deposited_quantiy = 0;
        data.capital_split = self.capital_split.clone();
        data.interest_split = self.interest_split.clone();
        data.mint_count = self.mint_count.clone();
        data.start_date = self.start_date;
        data.end_date = self.end_date;
        data.create_serum = self.create_serum;
        data.can_mint_more = self.can_mint_more;
        data.protocol_bump = self.protocol_bump;
        
        match Clock::get() {
            Ok(val) => data.created_at = val.unix_timestamp as u64,
            Err(_) => data.created_at = 0,
        };
    }
}

// + + + + + + + + + + + +

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct RedeemTrancheInput {
    pub quantity: u64,
}

impl Input for RedeemTrancheInput {
    fn is_valid(&self) -> Result<(), ErrorCode> {
        if self.quantity == 0 {
            return Result::Err(ErrorCode::InvalidInput);
        }

        return Result::Ok(());
    }
}
