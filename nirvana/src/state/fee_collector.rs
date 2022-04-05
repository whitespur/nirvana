use crate::numbers::{PreciseNumber, ALMS, ANA, NIRV};
use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct FeeCollector {
    /// The user's fee indexes
    pub nirv_fee_index: PreciseNumber,
    pub ana_fee_index: PreciseNumber,
    pub prana_fee_index: PreciseNumber,

    /// User account that "owns" this
    pub owner: Pubkey,

    /// How much ALMS is staked
    pub staked_alms: ALMS,

    /// How much ana is staged to claim?
    pub staged_ana: ANA,

    /// How much nirv is staged to claim?
    pub staged_nirv: NIRV,

    /// How much pre-ana is staged to claim?
    pub staged_pre_ana: ANA,

    pub bump: u8,
}

impl FeeCollector {
    pub fn stake_alms(&mut self, amount: ALMS) {
        self.staked_alms += amount;
    }

    pub fn unstake_alms(&mut self, amount: ALMS) {
        self.staked_alms -= amount;
    }
}
