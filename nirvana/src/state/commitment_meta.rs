use anchor_lang::prelude::*;

use crate::numbers::PreciseNumber;

#[account]
#[derive(Default)]
pub struct CommitmentMeta {
    /// Foreign key to nirv_center
    pub nirv_center: Pubkey,
    /// NirvCenter owned escrow account
    pub usdc_escrow_token_account: Pubkey,
    /// Decimals for escrow token type
    pub escrow_decimals: u8,
    /// Total amount committed, just for accounting purposes
    pub total_in_round_dollars: u64,
    /// When does the commitment period begin?
    pub start_time: i64,
    /// When does the early bird period end?
    pub early_bird_end: i64,
    /// when does the commitment period end?
    pub end: i64,

    pub bump: u8,
}

impl CommitmentMeta {
    pub fn add(&mut self, amount: u64) {
        self.total_in_round_dollars = self.total_in_round_dollars.checked_add(amount).unwrap();
    }

    pub fn sub(&mut self, amount: u64) {
        self.total_in_round_dollars = self.total_in_round_dollars.checked_sub(amount).unwrap();
    }
    pub fn usdc_denominator(&self) -> u64 {
        10u64.pow(self.escrow_decimals.into())
    }

    pub fn get_rate(&self, now: i64) -> PreciseNumber {
        if now < self.early_bird_end {
            PreciseNumber {
                val: 200_000_000_000,
            }
        } else if now < self.end {
            PreciseNumber {
                val: 150_000_000_000,
            }
        } else {
            PreciseNumber::ZERO
        }
    }
}
