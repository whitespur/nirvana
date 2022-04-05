use crate::numbers::{Decimalable, ANA, NIRV};
use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

#[account]
#[derive(Default)]
/// User history
/// Includes aggregated data points about user's history
pub struct History {
    /// central state
    pub nirv_center: Pubkey,

    /// user account
    pub authority: Pubkey,

    /// how much buy/sell volume has the user conducted
    pub total_volume_usd: u64,

    /// how much net spend
    pub net_spent_usd: u64,

    /// how much historical prana has the user earned
    pub total_prana_earned: u64,

    /// how much USDC spent during the boostrap
    pub bootstrap_net_spent_usd: u64,

    /// how much net ANA during the bootstrap
    pub bootstrap_net_ana: ANA,

    /// how much lifetime total NIRV has been borrowed
    pub total_nirv_borrowed: NIRV,

    /// how much lifetime total NIRV has been repaid
    pub total_nirv_repaid: NIRV,

    pub bump: u8,
}

impl History {
    pub fn buy(&mut self, usd_amount: u64) {
        self.net_spent_usd += usd_amount;
        self.total_volume_usd += usd_amount;
    }

    pub fn sell(&mut self, usd_amount: u64) {
        self.net_spent_usd = self.net_spent_usd.checked_sub(usd_amount).unwrap();
        self.total_volume_usd += usd_amount;
    }

    pub fn buy_during_bootstrap(&mut self, usd_amount: u64, ana_amount: ANA) {
        self.buy(usd_amount);
        self.bootstrap_net_spent_usd += usd_amount;
        self.bootstrap_net_ana += ana_amount;
    }

    pub fn sell_during_bootstrap(&mut self, usd_amount: u64, ana_amount: ANA) {
        self.sell(usd_amount);
        self.bootstrap_net_spent_usd = self
            .bootstrap_net_spent_usd
            .checked_sub(usd_amount)
            .unwrap();
        self.bootstrap_net_ana -= ana_amount;
    }

    /// Get average price spent on ANA during the bootstrap period
    pub fn bootstrap_avg_price(&self) -> Decimal {
        let net_spend = Decimal::from_u64(self.bootstrap_net_spent_usd).unwrap();
        let ana_bought = self.bootstrap_net_ana.to_decimal();

        net_spend.checked_div(ana_bought).unwrap()
    }
}
