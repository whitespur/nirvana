use crate::numbers::{ArbitraryNumber, ANA};
use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct UserTranaContract {
    /// owner of the bond
    pub user: Pubkey,

    /// Is the bond open for re-use
    pub available: bool,

    /// Link to trana metadata
    pub trana_meta: Pubkey,

    /// total ANA tokens
    pub amount_ana: ANA,

    /// how many ANA tokens have been redeemed
    pub redeemed_amount: ANA,

    /// price in underlying money
    pub price_in_underlying: ArbitraryNumber,

    /// when did the bond start
    pub start_time: i64,

    /// when will the bond end
    pub end_time: i64,
}

impl UserTranaContract {
    /// How much ANA is yet to be redeemed?
    pub fn get_left_to_redeem(&self, now: i64) -> ANA {
        if self.available {
            return ANA::ZERO;
        }

        let amount = self.amount_ana;

        if amount == self.redeemed_amount {
            return ANA::ZERO;
        }

        // Get the amount that can be redeemed at this time
        let end = self.end_time;
        let start = self.start_time;

        let progress = now.checked_sub(start).unwrap().unsigned_abs();
        // let progress = Decimal::from_u64(progress).unwrap();

        let span = end.checked_sub(start).unwrap().unsigned_abs();
        // let span = Decimal::from_u64(span).unwrap();

        let mut redeemable = amount
            .val
            .checked_mul(progress)
            .unwrap()
            .checked_div(span)
            .unwrap();

        // How much is redeemable at this moment?
        // let redeemable = amount
        //     .to_decimal()
        //     .checked_mul(progress)
        //     .unwrap()
        //     .checked_div(span)
        //     .unwrap();

        // Get rid of dust and ensure the redeemable amount
        // never goes over the total amount
        let dust = 100u64;

        if redeemable.checked_add(dust.into()).unwrap() > amount.val {
            redeemable = amount.val;
        }
        let left_to_redeem = redeemable
            .checked_sub(self.redeemed_amount.val)
            .unwrap_or(0);

        ANA::from_u64(left_to_redeem)
    }

    /// Given an amount left to redeem, update the user trana
    pub fn update_redeemed(&mut self, left_to_redeem: ANA) {
        let redeemed_amount = self
            .redeemed_amount
            .val
            .checked_add(left_to_redeem.val)
            .unwrap();
        self.redeemed_amount = ANA::from_u64(redeemed_amount);

        if self.redeemed_amount == self.amount_ana {
            self.available = true;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_left_redeem() {
        let amount_ana = ANA::ONE;
        let half_amount = ANA::from_u64(ANA::ONE.val / 2);
        let mut b = UserTranaContract {
            amount_ana,
            redeemed_amount: ANA::ZERO,
            start_time: 0,
            end_time: 32000,
            ..Default::default()
        };

        assert_eq!(b.get_left_to_redeem(0), ANA::ZERO);
        assert_eq!(b.get_left_to_redeem(32000), amount_ana);
        assert_eq!(b.get_left_to_redeem(32001), amount_ana);
        assert_eq!(b.get_left_to_redeem(31999), amount_ana);
        assert_eq!(b.get_left_to_redeem(16000), half_amount);

        b.redeemed_amount = half_amount;

        assert_eq!(b.get_left_to_redeem(32000), half_amount);
        assert_eq!(b.get_left_to_redeem(32001), half_amount);
        assert_eq!(b.get_left_to_redeem(31999), half_amount);
        assert_eq!(b.get_left_to_redeem(16000), ANA::ZERO);

        b.redeemed_amount = amount_ana;

        assert_eq!(b.get_left_to_redeem(32000).val, 0);
        assert_eq!(b.get_left_to_redeem(32001).val, 0);
        assert_eq!(b.get_left_to_redeem(31999).val, 0);
        assert_eq!(b.get_left_to_redeem(16000).val, 0);
    }
}
