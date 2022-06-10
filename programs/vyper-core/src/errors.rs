use anchor_lang::prelude::*;

#[error_code]
pub enum VyperErrorCode {
    #[msg("generic error")]
    GenericError,

    #[msg("invalid input")]
    InvalidInput,

    #[msg("failed to perform some math operation safely")]
    MathError,
    
    #[msg("Bits passed in do not result in valid halt flags")]
    InvalidTranchHaltFlags,
    
    #[msg("Current operation is not available because is halted")]
    HaltError,

    #[msg("Bits passed in do not result in valid owner restricted instruction flags")]
    InvalidOwnerRestrictedIxFlags,
    
    #[msg("Current operation is available only for tranche config owner")]
    OwnerRestrictedIx,

    #[msg("Fair value is stale, refresh it")]
    StaleFairValue,
    
    #[msg("The redeem logic plugin didn't return anything, maybe we forgot to set solana_program::program::set_return_data()?")]
    RedeemLogicNoReturn,
    
    #[msg("cross-program invocation error calling a vyper plugin")]
    PluginCpiError
}
