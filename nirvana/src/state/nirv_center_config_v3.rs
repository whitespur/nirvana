use crate::bootstrap_math::BootstrapParams;
use crate::numbers::{CoarseNumber, Decimalable, PreciseNumber, ALMS, ANA, NIRV};
use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct NirvCenterConfigV3 {
    /// Mint for ANA
    pub mint_ana: Pubkey,

    /// Mint for Pre-ANA
    pub mint_pre_ana: Pubkey,

    /// Mint for NIRV
    pub mint_nirv: Pubkey,

    /// Mint for ALMS
    pub mint_alms: Pubkey,

    /// Escrow Token Account for ANA, used for unvested trana contracts
    pub treasury_ana: Pubkey,

    /// Fee account for ANA
    pub ana_fee_account: Pubkey,

    /// Fee account for prANA
    pub prana_fee_account: Pubkey,

    /// Fee account for NIRV
    pub nirv_fee_account: Pubkey,

    /// NIRV fee index
    ///
    /// Tracks the NIRV fees collected per each ALMS staked.
    /// This is used to calculate the fee distribution among
    /// ALMS staked the same way PRANA rewards are distributed
    /// among ANA stakers.
    pub nirv_fee_index: PreciseNumber,

    /// ANA fee index
    ///
    /// Tracks the ANA fees collected per each ALMS staked.
    /// This is used to calculate the fee distribution among
    /// ALMS staked the same way PRANA rewards are distributed
    /// among ANA stakers.
    pub ana_fee_index: PreciseNumber,

    /// PRANA fee index
    ///
    /// Tracks the PRANA fees collected per each ALMS staked.
    /// This is used to calculate the fee distribution among
    /// ALMS staked the same way PRANA rewards are distributed
    /// among ANA stakers.
    pub prana_fee_index: PreciseNumber,

    /// PRANA reward index
    ///
    /// The reward index tracks the amount of prANA tokens rewards
    /// available per each ANA staked. The index is initialized to zero.
    /// Every time there a rewards drop we update the prANA per ANA.
    ///
    /// index = index + (prANA rewards dropped / current total ANA staked)
    ///
    /// When a user stakes ANA the following is recorded:
    ///
    /// * The amount of staked ANA
    /// * The current reward index
    ///
    /// This establishes a baseline for the rewards already distributed
    /// before the user stakes. This means the claimable rewards are
    /// calculated by using the increase of index, which represents
    /// the rewards distributed since the user initially staked.
    ///
    /// prANA rewards = ANA staked * (current index - user stake index )
    pub reward_index: PreciseNumber,

    /// Token Account for staking ANA
    pub stake_pool_ana: Pubkey,

    /// Stake pool for ALMS tokens
    pub stake_pool_alms: Pubkey,

    /// PRANA Reward distribution rate
    ///
    /// This coefficient is applied to the global supply of ANA
    /// to calculate the amount of PRANA to be minted as staking rewards
    /// This value connected with the prana_reward_interval_seconds computes the
    /// total amount of PRANA created per time unit
    pub prana_reward_rate: PreciseNumber,

    /// PRANA reward interval seconds
    ///
    /// The minimum interval in between dropping PRANA rewards
    pub prana_reward_interval_seconds: i64,

    /// Timestamp of most recent PRANA reward drop
    pub time_of_last_prana_reward: u64,

    /// Bootstrap starting price offset
    pub bs_start_offset: PreciseNumber,

    /// Bootstrap duration seconds
    pub bs_duration_seconds: u64,

    /// Bootstrap start time seconds
    pub bs_start_time_seconds: u64,

    /// Current price of ANA in USD
    /// not including any bootstrap offset
    pub current_ana_price_usd: PreciseNumber,

    /// Fee paid in ANA for unstaking
    pub unstake_fee: CoarseNumber,

    /// Fee for selling ANA (not applied when at floor)
    pub sell_fee: CoarseNumber,

    /// Fee for buying ANA
    pub instant_buy_fee: CoarseNumber,

    /// Fee paid in ANA for buying trANA
    pub trana_buy_fee: CoarseNumber,

    /// Fee of NIRV for loan origination
    /// This is a percentage of NIRV sent to the fee account
    pub nirv_loan_origination_fee: CoarseNumber,

    /// Fee paid in prANA for NIRV debt
    /// This is a percentage reduction of prANA rewards to the owner
    pub nirv_debt_fee: CoarseNumber,

    /// How much prANA can be redeemed per hour?
    pub max_prana_per_hour: u64,

    /// Max NIRV loan ratio
    ///
    /// The amount of NIRV that can be loaned is
    /// MIN(floor value, % of ANA collateral)
    /// where max_nirv_loan_ratio is this percentage
    /// 50% would allow max levering of 2x while 75% would allow up to 4x.
    pub max_nirv_loan_ratio: CoarseNumber,

    /// The Price Curve parameters
    pub price_curve: Pubkey,

    pub bump: u8,
}

impl NirvCenterConfigV3 {
    pub fn start_bootstrap(&mut self, now: u64, duration: u64, price_offset: PreciseNumber) {
        self.bs_start_time_seconds = now;
        self.bs_duration_seconds = duration;
        self.bs_start_offset = price_offset;
    }

    pub fn bs_end_time_seconds(&self) -> u64 {
        self.bs_duration_seconds
            .checked_add(self.bs_start_time_seconds)
            .unwrap()
    }

    pub fn bootstrapping_ended(&self, now: u64) -> bool {
        now > self.bs_end_time_seconds()
    }

    pub fn is_bootstrapping(&self, now: u64) -> bool {
        now > self.bs_start_time_seconds && now < self.bs_end_time_seconds()
    }

    pub fn to_bootstrap_params(&self) -> BootstrapParams {
        BootstrapParams {
            start_offset: self.bs_start_offset,
            start_time_seconds: self.bs_start_time_seconds,
            duration_seconds: self.bs_duration_seconds,
        }
    }

    pub fn collect_ana_fee(&mut self, fee_amount: ANA, total_alms_staked: ALMS) {
        if total_alms_staked == ALMS::ZERO {
            return;
        }

        let index_increase = fee_amount
            .to_decimal()
            .checked_div(total_alms_staked.into())
            .unwrap()
            .round_dp_with_strategy(PreciseNumber::SCALE, RoundingStrategy::ToZero);
        self.ana_fee_index += PreciseNumber::from_decimal(index_increase);
    }

    pub fn collect_ana_swap_fee(
        &mut self,
        is_buy: bool,
        swap_amount: ANA,
        total_alms_staked: ALMS,
    ) -> (ANA, ANA) {
        let fee_rate = if is_buy {
            self.instant_buy_fee.to_decimal()
        } else {
            self.sell_fee.to_decimal()
        };
        let fee = swap_amount
            .to_decimal()
            .checked_mul(fee_rate)
            .unwrap()
            .round_dp_with_strategy(ANA::SCALE, RoundingStrategy::ToZero);
        let amount_less_fee = swap_amount.to_decimal().checked_sub(fee).unwrap();

        self.collect_ana_fee(ANA::from_decimal(fee), total_alms_staked);

        (ANA::from_decimal(amount_less_fee), ANA::from_decimal(fee))
    }

    pub fn collect_trana_buy_fee(
        &mut self,
        buy_amount: ANA,
        total_alms_staked: ALMS,
    ) -> (ANA, ANA) {
        let fee_rate = self.trana_buy_fee.to_decimal();
        let fee = buy_amount
            .to_decimal()
            .checked_mul(fee_rate)
            .unwrap()
            .round_dp_with_strategy(ANA::SCALE, RoundingStrategy::ToZero);
        let amount_less_fee = buy_amount.to_decimal().checked_sub(fee).unwrap();

        self.collect_ana_fee(ANA::from_decimal(fee), total_alms_staked);

        (ANA::from_decimal(amount_less_fee), ANA::from_decimal(fee))
    }

    pub fn collect_ana_unstake_fee(
        &mut self,
        unstake_amount: ANA,
        total_alms_staked: ALMS,
    ) -> (ANA, ANA) {
        let unstake_fee = self.unstake_fee.to_decimal();
        let fee = unstake_amount
            .to_decimal()
            .checked_mul(unstake_fee)
            .unwrap()
            .round_dp_with_strategy(ANA::SCALE, RoundingStrategy::ToZero);
        let amount_less_fee = unstake_amount.to_decimal().checked_sub(fee).unwrap();

        self.collect_ana_fee(ANA::from_decimal(fee), total_alms_staked);

        (ANA::from_decimal(amount_less_fee), ANA::from_decimal(fee))
    }

    pub fn collect_nirv_fee(&mut self, fee_amount: NIRV, total_alms_staked: ALMS) {
        if total_alms_staked == ALMS::ZERO {
            return;
        }

        let index_increase = fee_amount
            .to_decimal()
            .checked_div(total_alms_staked.into())
            .unwrap()
            .round_dp_with_strategy(PreciseNumber::SCALE, RoundingStrategy::ToZero);
        self.nirv_fee_index += PreciseNumber::from_decimal(index_increase);
    }

    pub fn collect_nirv_origination_fee(
        &mut self,
        requested_amount: NIRV,
        total_alms_staked: ALMS,
    ) -> (NIRV, NIRV) {
        let origination_fee = self.nirv_loan_origination_fee.to_decimal();
        let fee = requested_amount
            .to_decimal()
            .checked_mul(origination_fee)
            .unwrap()
            .round_dp_with_strategy(NIRV::SCALE, RoundingStrategy::ToZero);
        let amount_less_fee = requested_amount.to_decimal().checked_sub(fee).unwrap();

        self.collect_nirv_fee(NIRV::from_decimal(fee), total_alms_staked);

        (NIRV::from_decimal(amount_less_fee), NIRV::from_decimal(fee))
    }

    pub fn collect_prana_fee(&mut self, fee_amount: ANA, total_alms_staked: ALMS) {
        if total_alms_staked == ALMS::ZERO {
            return;
        }

        let index_increase = fee_amount
            .to_decimal()
            .checked_div(total_alms_staked.into())
            .unwrap()
            .round_dp_with_strategy(PreciseNumber::SCALE, RoundingStrategy::ToZero);
        self.prana_fee_index += PreciseNumber::from_decimal(index_increase);
    }

    /// Calculate a fee from a user's PRANA rewards.
    ///
    /// The fee is calculated by multiplying the users nirv borrow utilization
    /// by a nirv debt fee rate. For example, if a user stakes 100 ANA then
    /// borrows 50 NIRV when the ANA floor is $1 then the borrow utilization is
    /// 50%. Assuming a debt fee of 4% that means this user's effective
    /// debt fee would be 4% * 50% = 2%. When the user is eligible to claim
    /// 100 PRANA they will receive 98 PRANA and will be charged a 2 PRANA fee.
    pub fn calc_nirv_debt_fee(&self, total_reward: ANA, borrow_utilization: Decimal) -> (ANA, ANA) {
        let rate = self.nirv_debt_fee.to_decimal();
        let fee = total_reward
            .to_decimal()
            .checked_mul(rate)
            .unwrap()
            .checked_mul(borrow_utilization)
            .unwrap()
            .round_dp_with_strategy(ANA::SCALE, RoundingStrategy::ToZero);

        let reward_less_fee = total_reward.to_decimal().checked_sub(fee).unwrap();

        (ANA::from_decimal(reward_less_fee), ANA::from_decimal(fee))
    }

    pub fn drop_prana_reward(
        &mut self,
        ana_supply: ANA,
        total_ana_staked: ANA,
        reward_interval_seconds: i64,
    ) {
        // TODO - use time since last drop to calculate amount that should be dropped
        // get propotion of reward interval for daily rate
        let seconds_in_day = 24 * 60 * 60;
        let reward_interval_ratio = Decimal::new(reward_interval_seconds, 0)
            .checked_div(Decimal::new(seconds_in_day, 0))
            .unwrap();

        let reward = self
            .prana_reward_rate
            .to_decimal()
            .checked_mul(reward_interval_ratio)
            .unwrap()
            .checked_mul(ana_supply.into())
            .unwrap()
            .round_dp_with_strategy(ANA::SCALE, RoundingStrategy::ToZero);

        let dropped_amount = ANA::from_decimal(reward);
        msg!("Dropped amount: {}", dropped_amount.val);

        if total_ana_staked.val == 0 {
            return;
        }

        let reward_index_add = dropped_amount
            .to_decimal()
            .checked_div(total_ana_staked.into())
            .unwrap()
            .round_dp_with_strategy(PreciseNumber::SCALE, RoundingStrategy::ToZero);

        msg!("Reward index delta: {}", reward_index_add);
        let reward_index_add = PreciseNumber::from_decimal(reward_index_add);
        self.reward_index += reward_index_add;
        //TODO: subtract NIRV debt fee and add it to nirv fee index
    }
}
