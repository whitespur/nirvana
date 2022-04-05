use anchor_lang::prelude::*;
use rust_decimal::prelude::*;

use crate::numbers::{CoarseNumber, Decimalable};

#[account]
#[derive(Default)]
pub struct MoneyMarket {
    /// Link to NirvCenter
    pub nirv_center: Pubkey,

    /// Can this token be used for the AMM
    pub for_amm: bool,

    /// Can this token be used for realizing prANA?
    pub for_prana: bool,

    /// Can this token be used for trANA?
    pub for_trana: bool,

    /// Whether this market is active
    pub enabled: bool,

    /// How much "risk free value" is this asset valued for?
    pub risk_free_value_coefficient: CoarseNumber,

    /// The account where a Pyth oracle keeps the updated price of the token.
    pub pyth_oracle_price: Pubkey,

    /// The account where a Pyth oracle keeps metadata about the token.
    pub pyth_oracle_metadata: Pubkey,

    /// Mint for underlying asset
    pub mint: Pubkey,

    /// Amount of decimals
    pub decimals: u8,

    /// Token account for underlying
    pub token_account: Pubkey,

    pub bump: u8,
}

impl MoneyMarket {
    pub fn rfv_coefficient_into_decimal(&self) -> Decimal {
        self.risk_free_value_coefficient.to_decimal()
    }
}
