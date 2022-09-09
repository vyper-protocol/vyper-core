# Redeem Logic Vanilla Option

The redeem logic vanilla option can simulate options on chain, where the senior tranches owners and junior tranches owners are the two sides of the option.

During initialization some parameters can be decided and serialized in the redeem logic state account

```rust
#[account]
pub struct RedeemLogicConfig {
    pub is_call: bool,
    pub is_linear: bool,

    pub strike: [u8; 16],
    pub owner: Pubkey,
}
```

Where the fields express:

- `is_call`: true if call, false if put
- `is_linear`: true if linear, false if inverse
- `strike`: strike value in Decimal format
- `owner`: owner of the configuration with the capability to update internal state
