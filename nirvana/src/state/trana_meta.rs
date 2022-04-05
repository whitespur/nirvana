use crate::numbers::{ArbitraryNumber, PreciseNumber, ANA};
use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct TranaMeta {
    /// Link to NirvCentr
    pub nirv_center: Pubkey,

    /// Whether this bond is active
    pub enabled: bool,

    /// controls the slope of the price function
    pub sensitivity: PreciseNumber,

    /// Max discount ratio
    pub max_discount_ratio: PreciseNumber,

    /// Money market metadata
    pub money_market: Pubkey,

    /// How much ANA is yet to vest
    pub ana_outstanding: ANA,

    /// How many of the underlying tokens have been purchased
    pub total_bought: ArbitraryNumber,

    /// Maximum vesting time
    pub vesting_length_seconds: u64,
}

impl TranaMeta {
    pub fn sub_ana_outstanding(&mut self, a: ANA) {
        self.ana_outstanding -= a;
    }
}
