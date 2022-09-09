# Lifecycle

The lifecycle of a Vyper Core contract can be deconstructed with the following calls (please note that this is an high level overview, refer to the following chapters for an in depth walkthrough).

## Initialization

Initializing Vyper Core means creating a `tranche_configuration`: a Solana account containing all the parameters of a vyper "session". All the following calls to the program are conditioned to this account state and will change it eventually.

## Deposit

During the deposit the caller (a user or another Solana program) deposits some reserve tokens inside vyper. During the deposit the caller can decide to deposit acting as `Side A` or as `Side B`, those two sides can express different meanings accordingly to the application logic. For example in some applications positions can rappresent different risk profiles (Profile A = risk adverse and Profile B = yield lover), in others can be just the two bet sides.

During the deposit instruction Vyper Core mint back to the user some new tokens rappresenting his position. Side A tokens are different from Side B tokens. Vyper Core uses the two plugins, rate and redeem logic, in order to decide how many tokens the program needs to mint.

## Redeem

The redeem instruction contains the reverse logic of the deposit. The user deposits his position tokens and the original reserve tokens are withdrawn back to the original owner.

Again, in order to decide how many tokens needs to be withdrawn the two plugins take place.
