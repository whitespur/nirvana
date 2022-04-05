use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

use crate::{
    numbers::{Decimalable, PreciseNumber, ANA},
    price_math::PriceCalculator,
};

/// A PriceFieldV2 is a kind of curve with 3 segments, including a floor
/// [0 - ramp_start] = constant floor price
/// [ramp_start, ramp_width] = a straight line that is hypotenuse of the ramp dimensions
/// [ramp_start + ramp_width, Inf] = a straight line with slope main_slope
#[account]
#[derive(Default, Debug)]
pub struct PriceFieldV2 {
    /// how wide is the ramp
    pub ramp_width: ANA,
    /// how tall is the ramp
    pub ramp_height: PreciseNumber,
    /// how far forward from the ramp's start is the current price point
    pub ramp_start: ANA,
    /// slippage slope after the ramp ends
    pub main_slope: PreciseNumber,
    /// flat floor price for supply 0 - ramp
    pub floor_price: PreciseNumber,
    pub nirv_center: Pubkey,
    pub bump: u8,
}

impl PriceCalculator for PriceFieldV2 {
    fn increase_supply_with_no_price_impact(&mut self, token_amount: ANA) -> () {
        self.ramp_start += token_amount;
    }

    fn decrease_supply_with_no_price_impact(&mut self, token_amount: ANA) -> () {
        self.ramp_start -= token_amount;
    }

    /// Calculates total liquidity in pool, given supply
    fn liquidity(&self, _supply: ANA) -> u64 {
        // TODO
        0
    }

    fn reset_slippage_start_point_if_needed(&mut self, supply: ANA) -> bool {
        if self.at_floor(supply) {
            self.ramp_start = supply;
            return true;
        }

        false
    }

    fn at_floor(&self, supply: ANA) -> bool {
        self.price_for_supply(supply) == self.floor_price.to_decimal()
    }

    /// Calculates the price at a specific supply point
    /// Returns in RFV
    fn price_for_supply(&self, supply: ANA) -> Decimal {
        // if the supply is behind the ramp start, then it is the floor
        if supply.val < self.ramp_start.val {
            return self.floor_price.to_decimal();
        }
        let offset_from_ramp_start = supply - self.ramp_start;

        let offset_from_ramp_start = offset_from_ramp_start.to_decimal();
        let floor = self.floor_price.to_decimal();
        let ramp_width = self.ramp_width.to_decimal();
        let ramp_height = self.ramp_height.to_decimal();

        // if the supply is in the ramp
        if offset_from_ramp_start <= ramp_width && ramp_width > Decimal::ZERO {
            let ramp_slope = ramp_height.checked_div(ramp_width).unwrap();
            // Round up
            return offset_from_ramp_start
                .checked_mul(ramp_slope)
                .unwrap()
                .checked_add(floor)
                .unwrap()
                .round_dp_with_strategy(12, RoundingStrategy::AwayFromZero);
        }

        // floor + ramp_height
        let vert_offset = floor.checked_add(ramp_height).unwrap();
        // get supply after ramp ends
        let offset_after_ramp_end = offset_from_ramp_start.checked_sub(ramp_width).unwrap();

        offset_after_ramp_end
            .checked_mul(self.main_slope.into())
            .unwrap()
            .checked_add(vert_offset)
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn increase() {
        let mut pf = PriceFieldV2 {
            ..Default::default()
        };
        pf.increase_supply_with_no_price_impact(ANA { val: 100 });
        assert_eq!(pf.ramp_start.val, 100);

        pf.increase_supply_with_no_price_impact(ANA { val: 100 });
        assert_eq!(pf.ramp_start.val, 200);
    }

    #[test]
    fn calculates_price_for_supply() {
        let pf = PriceFieldV2 {
            ..Default::default()
        };

        let p = pf.price_for_supply(ANA::ZERO);
        assert_eq!(p.to_u64().unwrap(), 0);

        let p = pf.price_for_supply(ANA::new(100));
        assert_eq!(p.to_u64().unwrap(), 0);

        let pf = PriceFieldV2 {
            floor_price: PreciseNumber::from_decimal(Decimal::new(1, 0)),
            ..Default::default()
        };

        let p = pf.price_for_supply(ANA::new(0));
        assert_eq!(p.to_u64().unwrap(), 1);

        let p = pf.price_for_supply(ANA::new(100));
        assert_eq!(p.to_u64().unwrap(), 1);

        let pf = PriceFieldV2 {
            ramp_start: ANA::new(100),
            ramp_width: ANA::new(100),
            ramp_height: PreciseNumber::from_decimal(Decimal::new(100, 0)),
            floor_price: PreciseNumber::from_decimal(Decimal::new(1, 0)),
            ..Default::default()
        };

        // before the ramp
        let p = pf.price_for_supply(ANA::new(0));
        assert_eq!(p.to_u64().unwrap(), 1);

        // at the start of the ramp
        let p = pf.price_for_supply(ANA::new(100));
        assert_eq!(p.to_u64().unwrap(), 1);

        // in the middle of the ramp
        let p = pf.price_for_supply(ANA::new(150));
        assert_eq!(p.to_u64().unwrap(), 51);

        // at the end of the ramp
        let p = pf.price_for_supply(ANA::new(200));
        assert_eq!(p.to_u64().unwrap(), 101);

        // beyond the end of the ramp
        let p = pf.price_for_supply(ANA::new(300));
        assert_eq!(p.to_u64().unwrap(), 101);

        let pf = PriceFieldV2 {
            ramp_start: ANA::new(100),
            ramp_width: ANA::new(100),
            ramp_height: PreciseNumber::from_decimal(Decimal::new(100, 0)),
            floor_price: PreciseNumber::from_decimal(Decimal::new(1, 0)),
            main_slope: PreciseNumber::from_decimal(Decimal::new(1, 0)),
            ..Default::default()
        };

        // beyond the end of the ramp
        let p = pf.price_for_supply(ANA::new(300));
        assert_eq!(p.to_u64().unwrap(), 201);
    }
}
