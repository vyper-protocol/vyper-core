# Appendix: Rust Decimal Representation

In Vyper Core all the decimal values are represented using [rust_decimal](https://docs.rs/rust_decimal/latest/) library. This helped a lot with the precision and float calculations.

Unfortunately the problem occurred on account serialization. At the time of writing anchor doesn't support 3rd party lib structs in serializing accounts during IDL generation. Thats why something like the following will result in a corrupted idl.

```rust
use rust_decimal::Decimal;

#[account]
pub struct RedeemLogicConfig {
    pub is_call: bool,
    pub is_linear: bool,
    pub strike: Decimal, // <- this will cause the corrupted idl
    pub owner: Pubkey,
}
```

That's why we were forced to keep the decimal values serialized as bytes array, so the above struct resulted in:

```rust
#[account]
pub struct RedeemLogicConfig {
    pub is_call: bool,
    pub is_linear: bool,
    pub strike: [u8; 16], // <- this is the result of Decimal::serialize()
    pub owner: Pubkey,
}
```

Of course all the times we need to read the value and use them we have to deserialize the values:

```rust
let strike_dec: Decimal = Decimal::deserialize(ctx.accounts.redeem_logic_config.strike);
```

## Javascript support

The above solution helped a lot and soved all the rust-related stuff. The auto generated idl is now fine, but all the Decimal values are still represented as byte array, that's resulted in a challenge when we needed to show those values in some frontend application.

We decided so to develop a js library to help doing the above. The library is available open source [here](https://github.com/vyper-protocol/rust-decimal-wrapper).

And with the library help the challenge was solved:

```ts
import { RustDecimalWrapper } from "@vyper-protocol/rust-decimal-wrapper";

// receive from an external application the bytes serialized with rust_decimal
const bytes = new Uint8Array([0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

// create a RustDecimalWrapper around the bytes
const wrapper = new RustDecimalWrapper(bytes);

// get the original value
const value = wrapper.toNumber();

// value is 0.1
```

More on this in [this twitter thread](https://twitter.com/vanderlinde____/status/1564308344759656448?s=20&t=On0XpLKNN4vaz7e7MoVCOQ),
