# initialize

During the initialization we ned to define all the parameters that we're going to use in a vyper tranche configuration and the accounts involved. Vyper during this instruction will crate also the two mints for the senior side tokens and the junior side tokens, for both of them the mint authority is the program itself.

## Input data

The inputs provided by the caller for the initialization are define with the following struct:

```rust
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug)]
pub struct InitializeInput {
    pub tranche_mint_decimals: u8,
    pub halt_flags: u16,
    pub owner_restricted_ixs: u16,
}
```

Where:

- `tranche_mint_decimals`: the decimals places used for both senior and junior tranches
- `halt_flags`: bitmask with the halted operations, once initialized only the owner of the tranche configuration will be capable of changing it
- `owner_restricted_ixs`: bitmask with the ixs restricted to the owner, for these instructions the owner will need to be the tx signer

At the time of writing the two bitmasks have the following values:

```rust
bitflags::bitflags! {
    pub struct TrancheHaltFlags: u16 {
        /// Disable deposits
        const HALT_DEPOSITS = 1 << 0;

        /// Disable refreshes
        const HALT_REFRESHES = 1 << 1;

        /// Disable redeems
        const HALT_REDEEMS = 1 << 2;

        /// Disable all operations
        const HALT_ALL = Self::HALT_DEPOSITS.bits
                       | Self::HALT_REFRESHES.bits
                       | Self::HALT_REDEEMS.bits;

    }
}

bitflags::bitflags! {
    pub struct OwnerRestrictedIxFlags: u16 {
        /// Owner restricted: Deposits
        const DEPOSITS = 1 << 0;

        /// Owner restricted: Refreshes
        const REFRESHES = 1 << 1;

        /// Owner restricted: Redeems
        const REDEEMS = 1 << 2;

        /// Disable all operations
        const ALL = Self::DEPOSITS.bits
                       | Self::REFRESHES.bits
                       | Self::REDEEMS.bits;

    }
}
```

## Accounts

```rust
#[derive(Accounts)]
#[instruction(input_data: InitializeInput)]
pub struct InitializeContext<'info> {
    /// Signer account
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Owner of the tranche config
    #[account()]
    pub owner: AccountInfo<'info>,

    /// Tranche config account, where all the parameters are saved
    #[account(init, payer = payer, space = TrancheConfig::LEN)]
    pub tranche_config: Box<Account<'info, TrancheConfig>>,

    /// CHECK:
    #[account(seeds = [tranche_config.key().as_ref(), b"authority".as_ref()], bump)]
    pub tranche_authority: AccountInfo<'info>,

    /// CHECK:
    #[account()]
    pub rate_program: AccountInfo<'info>,

    /// CHECK:
    #[account()]
    pub rate_program_state: AccountInfo<'info>,

    /// CHECK:
    #[account()]
    pub redeem_logic_program: AccountInfo<'info>,

    /// CHECK:
    #[account()]
    pub redeem_logic_program_state: AccountInfo<'info>,

    /// LP mint token to deposit
    #[account()]
    pub reserve_mint: Box<Account<'info, Mint>>,

    /// Token account for vault reserve tokens
    #[account(init, payer = payer, seeds = [tranche_config.key().as_ref(), reserve_mint.key().as_ref()], bump, token::authority = tranche_authority, token::mint = reserve_mint)]
    pub reserve: Box<Account<'info, TokenAccount>>,

    /// Senior tranche mint
    #[account(init, payer = payer, mint::decimals = input_data.tranche_mint_decimals, mint::authority = tranche_authority)]
    pub senior_tranche_mint: Box<Account<'info, Mint>>,

    /// Junior tranche mint
    #[account(init, payer = payer, mint::decimals = input_data.tranche_mint_decimals, mint::authority = tranche_authority)]
    pub junior_tranche_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}
```

Noteworthy accounts are:

- `tranche_config`: the account created to store the state of the tranche configuration
- `tranche_authority`: the PDA account used to sign instructions
- `rate_program` and `rate_program_state`: the rate plugin and the its state, those accounts are saved in the tranche configuration in order avoid hack involving different plugins
- `redeem_logic_program` and `redeem_logic_program_state`: same as before for the redeem logic plugin
- `reserve_mint` and `reserve`: the underlying asset swapped for tranche positions and its token account, the latter owned by the program
- `senior_tranche_mint` and `junior_tranche_mint`: the two mints created for senior and junior positions
