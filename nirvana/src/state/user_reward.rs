use crate::errors::ErrorCode;
use crate::numbers::{Decimalable, PreciseNumber, ALMS, ANA, NIRV};
use crate::state::{NirvCenterConfigV3, PriceFieldV2};
use anchor_lang::prelude::*;
use rust_decimal::prelude::*;
use std::ops::Mul;

#[account]
#[derive(Default, Debug)]
pub struct UserReward {
    // I dont like that you dont store bump inside accounts this requires you to resend it every time
    // You want to revalidate seeds of account but you are also not doing that so you didnt hit this problem
    // Overall is good to validate seeds of account but its not necessary
    /// The user's reward index
    pub index: PreciseNumber,

    /// User account that "owns" this
    pub owner: Pubkey,

    /// How much ANA is staked
    pub staked_amount: ANA,

    /// How much NIRV is borrowed
    pub borrowed_nirv: NIRV,

    /// How much pre-ana is staged to claim?
    pub staged_pre_ana_rewards: ANA,

    /// How much pre-ana is staged as fees?
    pub staged_pre_ana_fees: ANA,
}

#[allow(dead_code)]
impl UserReward {
    pub fn calc_rewards_before_fees(&self, central_reward_index: PreciseNumber) -> ANA {
        let index_delta = central_reward_index - self.index;

        // Get the share of the reward pot
        let share = self
            .staked_amount
            .to_decimal()
            .checked_mul(index_delta.into())
            .unwrap()
            .round_dp_with_strategy(ANA::SCALE, RoundingStrategy::ToZero);

        ANA::from_decimal(share)
    }

    pub fn calc_rewards_and_fees(
        &self,
        config: &NirvCenterConfigV3,
        price_field: &PriceFieldV2,
    ) -> (ANA, ANA) {
        let borrow_utilization = self.get_nirv_borrow_utilization(price_field).unwrap();

        let total_rewards = self.calc_rewards_before_fees(config.reward_index);

        let (rewards_less_fees, fees) =
            config.calc_nirv_debt_fee(total_rewards, borrow_utilization);

        (rewards_less_fees, fees)
    }

    /// Claim prana rewards
    pub fn claim_prana_rewards(
        &mut self,
        config: &mut NirvCenterConfigV3,
        price_field: &PriceFieldV2,
        total_alms_staked: ALMS,
    ) -> (ANA, ANA) {
        let (new_rewards, new_fees) = self.calc_rewards_and_fees(config, price_field);

        let total_rewards = new_rewards + self.staged_pre_ana_rewards;
        let total_fees = new_fees + self.staged_pre_ana_fees;

        config.collect_prana_fee(total_fees, total_alms_staked);

        self.reset_rewards(config.reward_index);

        (total_rewards, total_fees)
    }

    /// Stage the un-claimed rewards from the central index
    pub fn stage_rewards(&mut self, config: &NirvCenterConfigV3, price_field: &PriceFieldV2) {
        // get new rewards
        let central_reward_index = config.reward_index;
        let (new_rewards, new_fees) = self.calc_rewards_and_fees(config, price_field);

        // stage them
        self.staged_pre_ana_rewards = self.staged_pre_ana_rewards + new_rewards;
        self.staged_pre_ana_fees = self.staged_pre_ana_fees + new_fees;

        // update the index
        self.index = central_reward_index;
    }

    /// Add to staked amount
    pub fn stake_ana(
        &mut self,
        amount: ANA,
        config: &NirvCenterConfigV3,
        price_field: &PriceFieldV2,
    ) {
        // stage the unclaimed rewards
        self.stage_rewards(config, price_field);

        self.staked_amount += amount;
    }

    /// Unstake ANA
    pub fn unstake_ana(
        &mut self,
        amount: ANA,
        config: &mut NirvCenterConfigV3,
        price_field: &PriceFieldV2,
        total_alms_staked: ALMS,
    ) -> Result<(ANA, ANA)> {
        // stage the unclaimed rewards
        self.stage_rewards(config, price_field);

        self.staked_amount -= amount;

        let borrow_limit = self.get_nirv_borrow_limit(price_field);

        if self.borrowed_nirv.to_decimal() > borrow_limit {
            return Err(ErrorCode::InsufficientStakedANAToBackBorrowedNIRV.into());
        }

        // Calculate fee
        let (amount_less_fee, fee) = config.collect_ana_unstake_fee(amount, total_alms_staked);

        Ok((amount_less_fee, fee))
    }

    /// The maximum amount of NIRV that can be borrowed
    pub fn get_nirv_borrow_limit(&self, price_field: &PriceFieldV2) -> Decimal {
        let staked_ana = self.staked_amount.to_decimal();
        let floor_price = price_field.floor_price.to_decimal();

        // Since tokens out, round down
        let mut max = staked_ana
            .mul(floor_price)
            .round_dp_with_strategy(NIRV::SCALE, RoundingStrategy::ToZero);

        max.rescale(NIRV::SCALE);

        max
    }

    /// The ratio of borrowed amount against borrow limit
    pub fn get_nirv_borrow_utilization(&self, price_field: &PriceFieldV2) -> Result<Decimal> {
        let borrowed_nirv = self.borrowed_nirv.to_decimal();

        if borrowed_nirv == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        let borrow_limit = self.get_nirv_borrow_limit(price_field);
        if borrowed_nirv > borrow_limit {
            return Err(ErrorCode::BorrowedAmountLargerThanLimit.into());
        }

        borrowed_nirv
            .checked_div(borrow_limit)
            .ok_or(ErrorCode::InvalidBorrowUtilization.into())
    }

    /// Add to borrowed nirv amount
    pub fn borrow_nirv(&mut self, amount: NIRV, price_field: &PriceFieldV2) -> Result<()> {
        self.borrowed_nirv += amount;

        let borrow_limit = self.get_nirv_borrow_limit(price_field);

        if self.borrowed_nirv.to_decimal() > borrow_limit {
            return Err(ErrorCode::InsufficientStakedANAToBorrowNIRV.into());
        }

        Ok(())
    }

    /// Remove from borrowed nirv amount
    pub fn repay_nirv(&mut self, amount: NIRV) -> Result<()> {
        if amount.to_decimal() > self.borrowed_nirv.to_decimal() {
            return Err(ErrorCode::RepayNIRVMoreThanBorrowed.into());
        }

        self.borrowed_nirv -= amount;

        Ok(())
    }

    pub fn reset_rewards(&mut self, central_reward_index: PreciseNumber) {
        self.index = central_reward_index;
        self.staged_pre_ana_rewards = ANA::ZERO;
        self.staged_pre_ana_fees = ANA::ZERO;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn calc_new_rewards_zero() {
        // 0.001
        let index = PreciseNumber { val: 1_000_000_000 };

        let user_reward = UserReward {
            ..Default::default()
        };
        let rewards = user_reward.calc_rewards_before_fees(index);
        assert_eq!(rewards.val, 0);

        let user_reward = UserReward {
            index: PreciseNumber { val: 1_000_000 },
            ..Default::default()
        };
        let rewards = user_reward.calc_rewards_before_fees(index);
        assert_eq!(rewards.val, 0);
    }

    #[test]
    fn calc_new_rewards() {
        // 0.001
        let index = PreciseNumber { val: 2_000_000_000 };
        let user_reward = UserReward {
            index: PreciseNumber { val: 1_000_000_000 },
            staked_amount: ANA { val: 3_000_000 },
            ..Default::default()
        };
        let rewards = user_reward.calc_rewards_before_fees(index);
        assert_eq!(rewards.val, 3_000);

        let user_reward = UserReward {
            index: PreciseNumber { val: 2_000_000_000 },
            staked_amount: ANA { val: 3_000_000 },
            ..Default::default()
        };
        let rewards = user_reward.calc_rewards_before_fees(index);
        assert_eq!(rewards.val, 0);
    }
}
