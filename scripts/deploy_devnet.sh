#!/bin/bash

# vyper core
anchor deploy -p vyper-core --provider.cluster d --provider.wallet ~/Dev/VyperWallets/vyper-program-authority/authority.json

# rate mock plugin
# anchor deploy -p rate-mock --provider.cluster d --provider.wallet ~/Dev/VyperWallets/vyper-program-authority/authority.json

# redeem logic fee plugin
# anchor deploy -p redeem-logic-lending-fee --provider.cluster d --provider.wallet ~/Dev/VyperWallets/vyper-program-authority/authority.json

# # # # # # # # # # # # # # # # # # 
# RECOVERY
# # # # # # # # # # # # # # # # # # 

# solana-keygen recover -o ./ephemeral-kp.json prompt:// 
# solana program deploy --buffer ./ephemeral-kp.json --upgrade-authority ~/Dev/VyperWallets/vyper-core-program-authority/authority.json -k ~/Dev/VyperWallets/vyper-core-program-authority/authority.json ./target/deploy/vyper_core.so
