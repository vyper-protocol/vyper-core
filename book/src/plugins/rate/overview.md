# Rate Plugins

Rate plugins are used to fetch data from 3rd party feeds. They store their state in a plugin account that will be deserialized from vyper core to read the serialized data.

The minimum account information needed are:

```rust
#[account]
pub struct RateState {
    pub fair_value: [[u8; 16]; 10],
    pub refreshed_slot: u64,
}
```

Where the first field represent the fair value in Decimal format and the refreshed_slot the slot where the information have been read.

Despite the fact that rate plugins can have whatever instruction set they want (with whatever naming), is a best practice to have something like the following:

```rust
pub fn initialize(ctx: Context<InitializeContext>) -> Result<()> {
    // rate state initialization
}

pub fn refresh(ctx: Context<InitializeContext>) -> Result<()> {
    // information read from 3rd party feed
    // serialization in the rate state
    // refreshed_slot update
}
```

In the following chapter we'll cover some rate_plugin both from a tech and a use case point of view.
