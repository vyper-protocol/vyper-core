use anchor_lang::prelude::*;

// tranches enum
#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum TrancheID {
    Senior,
    Junior,
}
