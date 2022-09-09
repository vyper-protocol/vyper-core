# Plugins

The real value and flexibility of Vyper is its underlyng plugin system. At its core `vyper_core` program is quite simple, the real power comes from the plugin attached and used.

The vyper plugin system is the most and foremost attempt of Vyper Labs to provide real composability to the Solana ecosystem.

## Plugin Families

A vyper configuration except from vyper parameters is composed by

- 1x rate plugin
- 1x redeem logic plugin

Together those plugins are used in the `update_tranche_fair_value` instruction in order to calculate the fair value between reserve tokens and tranches tokens.

Keeping track of the slot where the tranche fair value is updated vyper can decide if the value is stale and needs a new update or not. Usually application refresh the fair value everytime a `deposit` or `redeem` instruction is needed.

## Plugin interfaces

In order to work correctly the plugin developers needs to stick to some rules in order give a new plugin the correct interface, this way Vyper knows how to call them and how to deserialize values.

In the following chapters we'll cover both the interfaces and some real market plugin currently used in production.
