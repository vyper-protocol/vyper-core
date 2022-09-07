# Used Libraries

While Vyper Core has been completely developer _in house_ it uses a lot of 3rd party libraries.

- [Anchor Framework](https://docs.rs/anchor-lang/latest/anchor_lang/): used to improved security, serialization and **a lot** more
- [rust_decimal](https://docs.rs/rust_decimal/latest/): used for Decimal calculation
- [bitflags](https://docs.rs/bitflags/latest/): for bit flags
- [boolinator](https://docs.rs/boolinator/latest/): because the `ok_or_else` is fun
- [solana-security-txt](https://docs.rs/solana-security-txt/latest/): to include in the chain deployment a security disclaimeer

For a compete list refer to the programs Cargo.toml dependencies.
