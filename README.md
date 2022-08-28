<p align="center">
  <a href="https://vyperprotocol.io">
    <img alt="Vyper Protocol" src="https://github.com/vyper-protocol/branding/blob/main/medium-logo.png" width="250" />
  </a>
</p>

Vyper Core works as a set of smart contract which can take any SPL token and a custom payoff function, and redistribute the tokens to match the payoff upon certain conditions. For example, people can deposit farming LP tokens, which after some time (e.g. a week) are redistributed to reflect the the impermanent loss vs fees generated.

There are three main smart contracts:

- **Vyper Core**: manages position IOUs creation and redemption, accepts only fungible tokens (e.g. LP tokens or cTokens). It redistributes collateral deposited consuming data from the rate calculator and redeem logic contracts
- **Rate Calculator**: updates the fair price of the collateral deposited (e.g. USD value of LP token). Supports up to 10 different underlyings for sophisticated payoffs
- **Redeem Logic**: payoff formula which specifies how collateral should be distributed, based on initial collateral deposited, initial prices, final prices, and other parameters (e.g. strike, duration)

# Repository Structure

Following the Vyper suite

## Solana Programs

| Name                            | Type                | Version | Path                                   |
| ------------------------------- | ------------------- | ------- | -------------------------------------- |
| **Vyper Core**                  | Core Primitive      | `0.1.0` | `programs/vyper-core`                  |
| **Rate Mock**                   | Rate Plugin         | `0.1.0` | `programs/rate-mock`                   |
| **Rate Switchboard**            | Rate Plugin         | `0.1.0` | `programs/rate-switchboard`            |
| **Redeem Logic Lending**        | Redeem Logic Plugin | `0.1.0` | `programs/redeem-logic-lending`        |
| **Redeem Logic Lending Fee**    | Redeem Logic Plugin | `0.1.0` | `programs/redeem-logic-lending-fee`    |
| **Redeem Logic Farming**        | Redeem Logic Plugin | `0.1.0` | `programs/redeem-logic-farming`        |
| **Redeem Logic Vanilla Option** | Redeem Logic Plugin | `0.1.0` | `programs/redeem-logic-vanilla-option` |
| **Redeem Logic Fila**           | Redeem Logic Plugin | `0.1.0` | `programs/redeem-logic-fila`           |

## Rust Libraries

| Name         | Version | Path                |
| ------------ | ------- | ------------------- |
| Vyper Utils  | `0.1.0` | `libs/vyper-utils`  |
| Vyper Macros | `0.1.0` | `libs/vyper-macros` |

## Typescript SDK

We're currently working on a typescript sdk for frontend integrations. This is still a WIP, but it's available at the path `/sdk`.

Once finished it'll be published as a npm module.

# Setup, Build, and Test

First, install dependencies:

```
$ yarn install
```

And install Anchor by following the [instructions here](https://github.com/coral-xyz/anchor/blob/master/docs/src/getting-started/installation.md).

Build the program:

```
$ anchor build
```

Finally, run the tests:

```
$ cargo test
$ anchor test
```

# Documentation

General Vyper documentation can be found [here](https://docs.vyperprotocol.io/).

# Getting Help

Join [our Discord channel](https://discord.gg/KYaXgwetcK) and post a message
