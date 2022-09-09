# Tranche Configuration

The core data of Vyper resides in the tranche configuration account.

Following the Rust struct

```rust
#[account]
pub struct TrancheConfig {
    pub reserve_mint: Pubkey,
    pub reserve: Pubkey,

    pub tranche_data: TrancheData,

    pub senior_tranche_mint: Pubkey,
    pub junior_tranche_mint: Pubkey,

    pub tranche_authority: Pubkey,

    pub authority_seed: Pubkey,
    pub authority_bump: [u8; 1],

    pub owner: Pubkey,

    pub rate_program: Pubkey,
    pub rate_program_state: Pubkey,

    pub redeem_logic_program: Pubkey,
    pub redeem_logic_program_state: Pubkey,

    pub version: [u8; 3],

    pub created_at: i64,
}
```

Where fields have the following meaning:

- `reserve_mint`: the mint of the tokens accepted during the deposits
- `reserve`: the token account storing the reserve tokens
- `tranche_data`: the parameters of the current tranche configuration, see below
- `senior_tranche_mint` and `junior_tranche_mint`: the mint addressed of the two positions, please note that senior and junior are just casual names, some other programs name similar concept as sideA and sideB
- `authority_seed` and `authority_bump`: seed and bump of the PDA authority account, used to let Vyper sign CPIs
- `owner`: wallet with super privileges, only the `tranche_configuration` owner can change some parameters of the state
- `rate_program` and `rate_program_state`: rate plugin and its state
- `redeem_logic_program` and `redeem_logic_program_state`: redeem logic plugin and its state
- `version`: version of the program, with semver
- `created_at`: creation timestamp

## Tranche Data

One important field of the tranche configuration is the tranche_data, following the rust struct:

```rust
pub struct TrancheData {
    pub deposited_quantity: [u64; 2],
    pub fee_to_collect_quantity: u64,
    pub reserve_fair_value: ReserveFairValue,
    pub tranche_fair_value: TrancheFairValue,
    halt_flags: u16,
    owner_restricted_ix: u16,
    pub deposit_cap: [Option<u64>; 2],
}
```

Where fields have the following meaning:

- `deposited_quantity`: the total quantity of reserve tokens deposited on vyper
- `fee_to_collect_quantity`: the amount of fees available to accrue from vyper
- `reserve_fair_value`: the fair value of the reserve token
- `tranche_fair_value`: the fair value of the tranches token, senior and junior
- `halt_flags`: a bitmask representing the halted operations, only the owner calling the `update_tranche_data` instruction can change this value
- `owner_restricted_ix`: a bitmask representing the instructions reserved to the owner
- `deposit cap`: a cap to the deposits, both per senior and junior side

## Tranche Fair Value

The tranche fair value and reserve fair value have a very similar shape, following the rust structs for the tranche_fair_value:

```rust
pub struct TrancheFairValue {
    pub value: [[u8; 16]; 2],
    pub slot_tracking: SlotTracking,
}

pub struct SlotTracking {
    last_update: LastUpdate,
    pub stale_slot_threshold: u64,
}

pub struct LastUpdate {
    slot: u64,
}
```

Where the fields express:

- `TrancheFairValue::value`: the current 2 Decimal values saved
- `SlotTracking::stale_slot_threshold`: the threshold used to declare a value stale, please note that to prevent using stale values Vyper checks the `LastUpdate::slot` against this threshold. In order to update the `LastUpdate::slot` users can use the `update_tranche_fair_value` instruction.
- `LastUpdate::slot`: the slot where the update occurred
