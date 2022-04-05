use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

use crate::numbers::{Decimalable, PreciseNumber, ANA};

#[account]
#[derive(Default)]
/// A commitment represents a pre-bootstrap target of USD to spend
pub struct Commitment {
    /// Owner of this commitment
    pub owner: Pubkey,

    /// Central state record
    pub commitment_meta: Pubkey,

    /// what is the reward percentage
    pub reward_index: PreciseNumber,

    /// how much USD targeted to spend
    pub target_spend_usd: u64,

    /// Has the reward been claimed?
    pub claimed_and_dead: bool,

    pub bump: u8,
}

impl Commitment {
    pub fn update_reward_rate(&mut self, delta: u64, new_rate: PreciseNumber) {
        if delta == 0 {
            return;
        }
        let old_rate = self.reward_index.to_decimal();
        let new_rate = new_rate.to_decimal();
        let old_spend = Decimal::from_u64(self.target_spend_usd).unwrap();
        let new_spend = Decimal::from_u64(delta).unwrap();
        let a = old_rate * old_spend;
        let b = new_rate * new_spend;
        let sum_rates = a + b;
        let total = old_spend + new_spend;
        let new_ratio = sum_rates.checked_div(total).unwrap();
        self.reward_index = PreciseNumber::from_decimal(new_ratio);
    }

    pub fn escrow_amount(&self) -> u64 {
        self.target_spend_usd / 100
    }

    pub fn reward_amount(&self, bootstrap_avg_price: Decimal) -> ANA {
        let ana_bought_during_bootstrap = Decimal::from_u64(self.target_spend_usd)
            .unwrap()
            .checked_div(bootstrap_avg_price)
            .unwrap();
        let amount_rewarded = ana_bought_during_bootstrap * self.reward_index.to_decimal();
        ANA::from_decimal(amount_rewarded)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const TWENTY: PreciseNumber = PreciseNumber {
        val: 200_000_000_000,
    };

    const TEN: PreciseNumber = PreciseNumber {
        val: 100_000_000_000,
    };

    #[test]
    fn test_update_reward_rate_at_zero() {
        let mut c = Commitment {
            ..Default::default()
        };

        c.update_reward_rate(0, PreciseNumber { val: 0 });
        assert_eq!(c.reward_index.val, 0);
    }

    #[test]
    fn test_update_reward_rate_from_zero() {
        let mut c = Commitment {
            reward_index: PreciseNumber::ZERO,
            ..Default::default()
        };

        c.update_reward_rate(100, TWENTY);
        assert_eq!(c.reward_index.val, TWENTY.val);
    }

    #[test]
    fn test_update_reward_rate_same_rate() {
        let mut c = Commitment {
            reward_index: TWENTY,
            target_spend_usd: 100,
            ..Default::default()
        };

        c.update_reward_rate(999, TWENTY);
        assert_eq!(c.reward_index.val, TWENTY.val);
    }

    #[test]
    fn test_update_reward_rate_different_rate() {
        let mut c = Commitment {
            reward_index: TWENTY,
            target_spend_usd: 100,
            ..Default::default()
        };

        c.update_reward_rate(100, TEN);
        assert_eq!(c.reward_index.val, 150_000_000_000);

        c.update_reward_rate(100, TEN);
        assert_eq!(c.reward_index.val, 125_000_000_000);

        let mut c = Commitment {
            reward_index: TWENTY,
            target_spend_usd: 100,
            ..Default::default()
        };

        c.update_reward_rate(25, TEN);
        assert_eq!(c.reward_index.val, 180_000_000_000);
    }

    #[test]
    fn test_reward_amount() {
        let c = Commitment {
            reward_index: TWENTY,
            target_spend_usd: 100,
            ..Default::default()
        };

        let avg_price = Decimal::new(1, 0);
        let amount = c.reward_amount(avg_price);

        assert_eq!(amount, ANA { val: 20_000_000 });

        let c = Commitment {
            reward_index: TWENTY,
            target_spend_usd: 100,
            ..Default::default()
        };

        let avg_price = Decimal::new(2, 0);
        let amount = c.reward_amount(avg_price);

        assert_eq!(amount, ANA { val: 10_000_000 });
    }
}
