from typing import NamedTuple


INTEREST_SPLIT = 0.20

assert INTEREST_SPLIT >= 0
assert INTEREST_SPLIT <= 1


class TrancheQuantity(NamedTuple):
    senior: float
    junior: float


def redeem(
    old_tranche_quantity: TrancheQuantity,
    old_reserve_fair_value: float,
    new_reserve_fair_value: float,
) -> TrancheQuantity:

    # default in the past
    if old_reserve_fair_value == 0:
        return old_tranche_quantity

    # positive return, share proceeds
    if new_reserve_fair_value > old_reserve_fair_value:
        senior = (
            old_tranche_quantity.senior
            * old_reserve_fair_value
            / new_reserve_fair_value
            * (
                1
                + (new_reserve_fair_value / old_reserve_fair_value - 1)
                * (1 - INTEREST_SPLIT)
            )
        )

    else:
        # total loss
        if new_reserve_fair_value == 0:
            senior = old_tranche_quantity.senior + old_tranche_quantity.junior

        # partial loss
        else:
            senior = min(
                old_tranche_quantity.senior + old_tranche_quantity.junior,
                old_tranche_quantity.senior
                * old_reserve_fair_value
                / new_reserve_fair_value,
            )

    # max(0, ..) should be superfluos
    junior = max(0, old_tranche_quantity.senior + old_tranche_quantity.junior - senior)

    # approx true due to rounding
    assert senior + junior == old_tranche_quantity.senior + old_tranche_quantity.junior

    return TrancheQuantity(senior, junior)
