use crate::numbers::CoarseNumber;
use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct FeeConfig {
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

    pub bump: u8,
}
