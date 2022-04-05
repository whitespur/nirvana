use std::cmp;

use crate::{
    numbers::{Decimalable, PreciseNumber, ANA},
    state::TranaMeta,
};
use rust_decimal::prelude::*;

pub fn bond_discount(bond: &TranaMeta, ana_price: &Decimal, floor: &Decimal) -> Decimal {
    let bond_max_discount_ratio = bond.max_discount_ratio;
    let max_discount_ratio = compress_max_discount(ana_price, floor, bond_max_discount_ratio);

    bond_discount_ratio(bond.sensitivity, bond.ana_outstanding, max_discount_ratio)
}

fn bond_discount_ratio(
    bond_sensitivity: PreciseNumber,
    ana_outstanding: ANA,
    max_discount_ratio: PreciseNumber,
) -> Decimal {
    let max_discount_ratio = max_discount_ratio.to_decimal();
    // the demand is the outstanding ANA * sensitivity
    let demand = ana_outstanding
        .to_decimal()
        .checked_mul(bond_sensitivity.into())
        .unwrap()
        .round_dp_with_strategy(12, RoundingStrategy::AwayFromZero);

    // the discount is the max discount - demand
    // ie, more demand is tantamount to less discount
    let discount = max_discount_ratio.checked_sub(demand).unwrap();

    discount
}

/// Checks the discount of the floor relative to ANA,
/// and returns the lesser of this versus the bonds discount
fn compress_max_discount(
    ana_price: &Decimal,
    floor: &Decimal,
    bond_max_discount_ratio: PreciseNumber,
) -> PreciseNumber {
    // Round up
    let floor_ratio = floor
        .checked_div(*ana_price)
        .unwrap()
        .round_dp_with_strategy(PreciseNumber::SCALE, RoundingStrategy::AwayFromZero);
    let floor_discount = Decimal::ONE.checked_sub(floor_ratio).unwrap();

    let floor_discount = PreciseNumber::from_decimal(floor_discount);

    PreciseNumber {
        val: cmp::min(floor_discount.val, bond_max_discount_ratio.val),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_max_discount_at_zero() {
        let ana = Decimal::new(1, 0);
        let floor = Decimal::new(1, 0);
        let max_discount_ratio = PreciseNumber::from_decimal(Decimal::new(5, 1));
        let max_discount = compress_max_discount(&ana, &floor, max_discount_ratio);
        assert_eq!(max_discount.val, 0);
    }

    #[test]
    fn test_no_compress_max_discount() {
        // Floor is 90% beneath ANA price
        let ana = Decimal::new(10, 0);
        let floor = Decimal::new(1, 0);

        // Bond max discount is 50%
        let max_discount_ratio = PreciseNumber::from_decimal(Decimal::new(5, 1));

        // Compressed max discount is 50%
        let max_discount = compress_max_discount(&ana, &floor, max_discount_ratio);
        assert_eq!(max_discount.val, 500_000_000_000);
    }

    #[test]
    fn test_compress_max_discount() {
        // Floor is 25% beneath ANA price
        let ana = Decimal::new(4, 0);
        let floor = Decimal::new(3, 0);

        // Bond max discount is 50%
        let max_discount_ratio = PreciseNumber::from_decimal(Decimal::new(5, 1));

        // compressed max discount is 25%
        let max_discount = compress_max_discount(&ana, &floor, max_discount_ratio);
        assert_eq!(max_discount.val, 250_000_000_000);
    }
}
