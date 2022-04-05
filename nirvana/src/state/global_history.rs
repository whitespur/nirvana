use std::cmp;

use crate::numbers::{ANA, NIRV};
use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct GlobalHistory {
    /// Total volume in round dollars
    pub volume_usd: u64,
    /// Net quantity of ANA purchased
    pub net_purchased_ana: ANA,
    /// High watermark for ANA supply
    pub all_time_high_ana_supply: ANA,
    /// PRANA minted, including for fees
    pub prana_minted: ANA,
    /// Total prANA rewards dropped
    pub total_prana_rewards: ANA,
    /// Total prANA staged, yet to be claimed
    pub staged_prana_rewards: ANA,
    /// PRANA executed
    pub prana_executed: ANA,
    /// PRANA purchased with NIRV
    pub prana_purchased: ANA,
    /// Total NIRV minted
    pub nirv_minted: NIRV,
    /// Total NIRV repaid
    pub nirv_repaid: NIRV,

    pub bump: u8,
}

impl GlobalHistory {
    pub fn buy_ana(&mut self, round_dollars: u64, amount_ana: ANA, current_supply_ana: ANA) {
        self.volume_usd += round_dollars;
        self.net_purchased_ana += amount_ana;

        // TODO: Implement Ord for ANA
        self.all_time_high_ana_supply = ANA::from_u64(cmp::max(
            self.all_time_high_ana_supply.val,
            (current_supply_ana + amount_ana).val,
        ));
    }

    pub fn sell_ana(&mut self, round_dollars: u64, amount_ana: ANA) {
        self.volume_usd += round_dollars;
        self.net_purchased_ana -= amount_ana;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn buy_sell_ana() {
        let mut g = GlobalHistory {
            ..Default::default()
        };

        g.buy_ana(10, ANA::new(1), ANA::ZERO);

        assert_eq!(g.all_time_high_ana_supply, ANA::new(1));
        assert_eq!(g.volume_usd, 10);
        assert_eq!(g.net_purchased_ana, ANA::new(1));

        g.buy_ana(10, ANA::new(1), ANA::new(1));

        assert_eq!(g.all_time_high_ana_supply, ANA::new(2));
        assert_eq!(g.volume_usd, 20);
        assert_eq!(g.net_purchased_ana, ANA::new(2));

        g.sell_ana(10, ANA::new(1));

        assert_eq!(g.all_time_high_ana_supply, ANA::new(2));
        assert_eq!(g.volume_usd, 30);
        assert_eq!(g.net_purchased_ana, ANA::new(1));
    }
}
