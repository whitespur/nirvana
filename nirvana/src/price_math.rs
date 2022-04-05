use rust_decimal::prelude::*;

use crate::numbers::{CoarseNumber, Decimalable, ANA};

pub trait PriceCalculator {
    /// Calculates total liquidity in pool, given supply
    fn liquidity(&self, supply: ANA) -> u64;

    /// Calculates the price at a specific supply point
    /// Returns in RFV
    fn price_for_supply(&self, supply: ANA) -> Decimal;

    /// Would the supply be at the floor price?
    fn at_floor(&self, supply: ANA) -> bool;

    /// Resets the start of the slippage to the input supply if necessary
    /// Returns true if it reset, meaning that the floor was reached
    fn reset_slippage_start_point_if_needed(&mut self, supply: ANA) -> bool;

    /// Increase supply in terms of tokens added
    fn increase_supply_with_no_price_impact(&mut self, token_amount: ANA) -> ();

    /// Decrease supply in terms of tokens removed
    fn decrease_supply_with_no_price_impact(&mut self, token_amount: ANA) -> ();
}

pub fn calc_total_cost_for_amount<T: PriceCalculator>(
    current_supply: ANA,
    amount: ANA,
    money_risk_free_value_factor: CoarseNumber,
    price_field: &T,
    is_buy: bool,
    bootstrap_offset: Decimal,
) -> Decimal {
    let rounding_strategy = if is_buy {
        RoundingStrategy::AwayFromZero
    } else {
        RoundingStrategy::ToZero
    };
    let target_supply = if is_buy {
        current_supply + amount
    } else {
        current_supply - amount
    };

    let price = calc_price(
        target_supply,
        money_risk_free_value_factor,
        price_field,
        is_buy,
    );

    price
        .checked_add(bootstrap_offset)
        .unwrap()
        .checked_mul(amount.into())
        .unwrap()
        .round_dp_with_strategy(12, rounding_strategy)
}

pub fn calc_price<T: PriceCalculator>(
    target_supply: ANA,
    money_risk_free_value_factor: CoarseNumber,
    price_field: &T,
    is_buy: bool,
) -> Decimal {
    // round up if buy, down if sell
    let rounding_strategy = if is_buy {
        RoundingStrategy::AwayFromZero
    } else {
        RoundingStrategy::ToZero
    };

    // get price
    let price = price_field.price_for_supply(target_supply);

    // scale by rfv factor
    let rfv_factor = money_risk_free_value_factor.to_decimal();
    let price = price
        .checked_mul(rfv_factor)
        .unwrap()
        .round_dp_with_strategy(12, rounding_strategy);

    price
}

// #[cfg(test)]
// mod test {
//     use crate::{numbers::CoarseNumber, price_math::calc_price, state::PriceFieldV1};

//     use super::*;

//     #[test]
//     fn test_get_price_same_scale() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };

//         let money_risk_free_value_factor = CoarseNumber { val: 1_000_000 };

//         let mut price = calc_price(1, money_risk_free_value_factor, &price_field, true);
//         price.rescale(6);
//         assert_eq!(price.mantissa(), 1_000_001);

//         let mut price = calc_price(500_000, money_risk_free_value_factor, &price_field, true);
//         price.rescale(6);
//         assert_eq!(price.mantissa(), 1_500_000);
//     }

//     #[test]
//     fn test_get_price_different_scale() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };

//         let money_risk_free_value_factor = CoarseNumber { val: 1_000_000 };
//         let money_scale = 9;

//         let mut price = calc_price(1, money_risk_free_value_factor, &price_field, true);
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 1_000_001_000);

//         let mut price = calc_price(500_000, money_risk_free_value_factor, &price_field, true);
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 1_500_000_000);

//         let money_scale = 3;
//         let mut price = calc_price(500_000, money_risk_free_value_factor, &price_field, true);
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 1_500);

//         let mut price = calc_price(4_000_000, money_risk_free_value_factor, &price_field, true);
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 5_000);
//     }

//     #[test]
//     fn test_total_amount_buying() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };

//         let money_risk_free_value_factor = CoarseNumber { val: 1_000_000 };
//         let money_scale = 3;

//         let mut price = calc_total_cost_for_amount(
//             1_000_000,
//             0,
//             money_risk_free_value_factor,
//             &price_field,
//             true,
//         );
//         assert_eq!(price.mantissa(), 0);

//         price = calc_total_cost_for_amount(
//             0,
//             1_000_000,
//             money_risk_free_value_factor,
//             &price_field,
//             true,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 2_000);

//         price = calc_total_cost_for_amount(
//             1_000_000,
//             1_000_000,
//             money_risk_free_value_factor,
//             &price_field,
//             true,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 3_000);

//         price = calc_total_cost_for_amount(
//             1_000_000,
//             2_000_000,
//             money_risk_free_value_factor,
//             &price_field,
//             true,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 8_000);

//         price = calc_total_cost_for_amount(
//             1_000_000,
//             500_000,
//             money_risk_free_value_factor,
//             &price_field,
//             true,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 1_250);
//     }

//     #[test]
//     #[should_panic]
//     fn test_total_amount_selling_too_much() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };
//         let money_risk_free_value_factor = CoarseNumber { val: 1_000_000 };
//         calc_total_cost_for_amount(
//             0,
//             1_000_000,
//             money_risk_free_value_factor,
//             &price_field,
//             false,
//         );
//     }

//     #[test]
//     fn test_total_amount_selling() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };

//         let money_risk_free_value_factor = CoarseNumber { val: 1_000_000 };
//         let money_scale = 3;
//         let is_buy = false;

//         let mut price = calc_total_cost_for_amount(
//             1_000_000,
//             0,
//             money_risk_free_value_factor,
//             &price_field,
//             is_buy,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 0);

//         price = calc_total_cost_for_amount(
//             1_000_000,
//             1_000_000,
//             money_risk_free_value_factor,
//             &price_field,
//             is_buy,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 1_000);

//         price = calc_total_cost_for_amount(
//             2_000_000,
//             2_000_000,
//             money_risk_free_value_factor,
//             &price_field,
//             is_buy,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 2_000);

//         price = calc_total_cost_for_amount(
//             1_000_000,
//             500_000,
//             money_risk_free_value_factor,
//             &price_field,
//             is_buy,
//         );
//         price.rescale(money_scale);
//         assert_eq!(price.mantissa(), 750);
//     }

//     #[test]
//     fn test_price_for_supply_at_0() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(0);

//         assert_eq!(price.to_u64().unwrap(), 1);

//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 9_000_000_000_000,
//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(0);

//         assert_eq!(price.to_u64().unwrap(), 9);

//         let price_field = PriceFieldV1 {
//             main_slope: 99999,
//             floor_price: 0,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(0);

//         assert_eq!(price.to_u64().unwrap(), 0);
//     }

//     #[test]
//     #[should_panic]
//     fn test_price_panics_when_large() {
//         let price_field = PriceFieldV1 {
//             main_slope: 2_000_000_000_000,
//             floor_price: 2_000_000_000_000,

//             ..Default::default()
//         };
//         price_field.price_for_supply(u64::MAX - 1).to_u64().unwrap();
//     }

//     #[test]
//     fn test_price_for_supply_at_1() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(1_000_000);
//         assert_eq!(price.mantissa(), 2_000_000);

//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 9_000_000_000_000,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(1_000_000);
//         assert_eq!(price.mantissa(), 10_000_000);

//         let price_field = PriceFieldV1 {
//             main_slope: 2_000_000_000_000,
//             floor_price: 9_000_000_000_000,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(1_000_000);
//         assert_eq!(price.mantissa(), 11_000_000);

//         let price_field = PriceFieldV1 {
//             main_slope: 0,
//             floor_price: 0,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(1);
//         assert_eq!(price.mantissa(), 0);
//     }

//     #[test]
//     fn test_price_for_supply_at_n() {
//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 1_000_000_000_000,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(10_000_000);

//         assert_eq!(price.mantissa(), 11_000_000);

//         let price_field = PriceFieldV1 {
//             main_slope: 2_000_000_000_000,
//             floor_price: 9_000_000_000_000,

//             ..Default::default()
//         };

//         let price = price_field.price_for_supply(10_000_000);
//         assert_eq!(price.mantissa(), 29_000_000);

//         let price_field = PriceFieldV1 {
//             main_slope: 0,
//             floor_price: 0,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(10);
//         assert_eq!(price.mantissa(), 0);

//         let price_field = PriceFieldV1 {
//             main_slope: 1_000_000_000_000,
//             floor_price: 0,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(10);
//         assert_eq!(price.mantissa(), 10);

//         let price_field = PriceFieldV1 {
//             main_slope: 500_000_000_000,
//             floor_price: 0,

//             ..Default::default()
//         };
//         let price = price_field.price_for_supply(10);
//         assert_eq!(price.mantissa(), 5);
//     }
// }
