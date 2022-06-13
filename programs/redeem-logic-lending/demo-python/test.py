from lending import INTEREST_SPLIT, redeem, TrancheQuantity
from pytest import approx


assert INTEREST_SPLIT == 0.20


def test_flat_returns():
    old_tranche = TrancheQuantity(1, 1)
    old_reserve = 1
    new_reserve = 1

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(1)
    assert new_tranche.junior == approx(1)


def test_positive_returns():
    old_tranche = TrancheQuantity(1, 1)
    old_reserve = 60
    new_reserve = 75

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(0.96)
    assert new_tranche.junior == approx(1.04)


def test_positive_returns_senior_imbalance():
    old_tranche = TrancheQuantity(100, 1)
    old_reserve = 60
    new_reserve = 75

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(96)
    assert new_tranche.junior == approx(5)


def test_positive_returns_junior_imbalance():
    old_tranche = TrancheQuantity(1, 100)
    old_reserve = 100
    new_reserve = 125

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(0.96)
    assert new_tranche.junior == approx(100.04)


def test_negative_returns():
    old_tranche = TrancheQuantity(1, 1)
    old_reserve = 80
    new_reserve = 64

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(1.25)
    assert new_tranche.junior == approx(0.75)


def test_negative_returns_senior_imbalance():
    old_tranche = TrancheQuantity(100, 1)
    old_reserve = 60
    new_reserve = 48

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(101)
    assert new_tranche.junior == approx(0)


def test_negative_returns_junior_imbalance():
    old_tranche = TrancheQuantity(1, 100)
    old_reserve = 100
    new_reserve = 80

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(1.25)
    assert new_tranche.junior == approx(99.75)


def test_junior_wipeout():
    old_tranche = TrancheQuantity(1, 1)
    old_reserve = 100
    new_reserve = 50

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(2)
    assert new_tranche.junior == approx(0)


def test_junior_wipeout_senior_partial():
    old_tranche = TrancheQuantity(1, 1)
    old_reserve = 10
    new_reserve = 2.5

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(2)
    assert new_tranche.junior == approx(0)


def test_junior_wipeout_senior_wipeout():
    old_tranche = TrancheQuantity(1, 1)
    old_reserve = 13
    new_reserve = 0

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(2)
    assert new_tranche.junior == approx(0)


def test_past_wipeout():
    old_tranche = TrancheQuantity(10, 1)
    old_reserve = 0
    new_reserve = 0

    new_tranche = redeem(old_tranche, old_reserve, new_reserve)
    assert new_tranche.senior == approx(10)
    assert new_tranche.junior == approx(1)
