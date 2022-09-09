# Rate Switchboard

The [switchboard](https://switchboard.xyz/) rate plugin reads data from a switchboard aggregator. Some example aggregator can be found [here](https://switchboard.xyz/explorer).

A switchboard aggregator is a tool super flexible, it can give informations reagarding both financial and non, for example:

- [SOL_USD aggregator](https://switchboard.xyz/explorer/3/GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR)
- [London temperature](https://switchboard.xyz/explorer/2/DQ2v1xuxTampfvKK5kp8oJDoZBRyHZUfYwUVUX32eBBd)

Composing multiple aggregators together and using them in a Vyper RateSwitchboard Plugin can open possibilities to thousands opportunities.

Following an overview of the plugin instructions.

## Initialize

During inizialization we declare which aggregator we're going to use in our application, those aggregators are serialized in the rate state, that has a struct similar to the following:

```rust
#[account]
pub struct RateState {
    pub fair_value: [[u8; 16]; 10],
    pub refreshed_slot: u64,
    pub switchboard_aggregators: [Option<Pubkey>; 10],
}
```

The rate_state account can store up to 10 switchboard aggregators.

## Refresh

During the refresh instruction the plugin read the values inside the aggregator and serialize it in the `rate_state::fair_value` taking care of update the `refreshed_slot`.
