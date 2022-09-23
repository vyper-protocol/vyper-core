# Rate PoolV2

The PoolV2 rate plugin read data from a standard farming pool v2 accounts, following the Uniswap v2 architecture.

## Initialize

The accounts used during initialization are:

```rust
#[derive(Accounts)]
pub struct InitializeContext<'info> {
    /// Signer account
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = RateState::LEN)]
    pub rate_data: Box<Account<'info, RateState>>,

    /// CHECK: the pool id
    #[account()]
    pub pool: AccountInfo<'info>,

    /// Mint of the lp tokens
    #[account(mint::authority = pool)]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// CHECK: Base mint, for a SOL/USDC pool this is SOL
    #[account()]
    pub base_mint: Box<Account<'info, Mint>>,

    /// CHECK: Quote mint, for a SOL/USDC pool this is USDC
    #[account()]
    pub quote_mint: Box<Account<'info, Mint>>,

    /// Base token account, for a SOL/USDC pool this is the SOL token account
    #[account(token::mint = base_mint, token::authority = pool)]
    pub base_token_account: Box<Account<'info, TokenAccount>>,

    /// Quote token account, for a SOL/USDC pool this is the USDC token account
    #[account(token::mint = quote_mint, token::authority = pool)]
    pub quote_token_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
}
```

In a standard SOL/USDC pool the meaningful accounts above are:

- `pool`: the account storing the pool data, it's the owner of the token accounts and the mint authority for lp tokens
- `lp_mint`: mint for LP tokens
- `base_mint`: for a SOL/USDC pool thats the SOL mint
- `quote_mint`: for a SOL/USDC pool thats the USDC mint
- `base_token_account`: for a SOL/USDC pool thats the SOL token account owned by the pool
- `quote_token_account`: for a SOL/USDC pool thats the USDC token account owned by the pool

## Refresh

During the refresh instruction the plugin calculate the used prices and serialize it in the `rate_state::fair_value` taking care of update the `refreshed_slot`.

## Price calculation

Considering the pool v2 architecture the prices are calculated as:

\\[ LP = \frac{supplyUSDC * 2}{supplyLP} \\]

and

\\[ SOL/USDC = \frac{supplyUSDC }{supplySOL} \\]

the output of the plugin will respect the above order, LP price first, base price then.
