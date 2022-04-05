use crate::errors::ErrorCode;
use crate::numbers::Decimalable;
use crate::numbers::ANA;
use crate::price_math::PriceCalculator;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token::{self, *};
use rust_decimal::prelude::*;

#[derive(Accounts)]
pub struct RealizePreAna<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        constraint = money_market.nirv_center == nirv_center.key(),
    )]
    // So here is interesting case if there is more than one money market this might be spoofed
    // Imo you should add seeds check to all PDAs.
    pub money_market: Box<Account<'info, MoneyMarket>>,

    #[account(
        mut,
        constraint = user_ana.mint == mint_ana.key(),
        constraint = user_ana.owner == authority.key(),
    )]
    pub user_ana: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = mint_ana.key() == config.mint_ana,
    )]
    pub mint_ana: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_pre_ana.mint == mint_pre_ana.key(),
        constraint = user_pre_ana.owner == authority.key(),
    )]
    pub user_pre_ana: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = config.mint_pre_ana == mint_pre_ana.key(),
    )]
    pub mint_pre_ana: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_u.owner == authority.key(),
    )]
    pub user_u: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = treasury_u.key() == money_market.token_account, // This is only reference preventing spoofing 
    )]
    pub treasury_u: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = price_field.nirv_center == nirv_center.key(),
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_field.bump,
    )]
    pub price_field: Box<Account<'info, PriceFieldV2>>,

    pub token_program: Program<'info, Token>,
}
pub fn handler(ctx: Context<RealizePreAna>, pre_ana_amount: u64) -> Result<()> {
    let floor = ctx.accounts.price_field.floor_price.to_decimal();

    ctx.accounts.sufficient_prana(pre_ana_amount)?;
    // Check whether this token account is acceptable for prANA realization
    ctx.accounts.can_realize_prana()?;

    // Round up
    let payment_amount = ANA::from_u64(pre_ana_amount)
        .to_decimal()
        .checked_mul(floor)
        .unwrap()
        .round_dp_with_strategy(6, RoundingStrategy::AwayFromZero)
        .mantissa()
        .to_u64()
        .unwrap();

    // update the price calculator
    ctx.accounts
        .price_field
        .increase_supply_with_no_price_impact(ANA::from_u64(pre_ana_amount));

    // Burn the prANA
    token::burn(ctx.accounts.burn_context(), pre_ana_amount)?;

    // Transfer the payment
    token::transfer(ctx.accounts.transfer_context(), payment_amount)?;

    // Mint new ANA to the user
    token::mint_to(
        ctx.accounts
            .mint_to_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        pre_ana_amount,
    )?;

    Ok(())
}

impl<'info> RealizePreAna<'info> {
    fn sufficient_prana(&self, amount: u64) -> Result<()> {
        let pre_ana_balance = self.user_pre_ana.amount;
        if amount > pre_ana_balance {
            return Err(ErrorCode::InsufficientPreAnaToRealize.into());
        }
        Ok(())
    }
    fn can_realize_prana(&self) -> Result<()> {
        if !self.money_market.for_prana {
            return Err(ErrorCode::TokenAccountNotForPrana.into());
        }
        Ok(())
    }

    fn mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint_ana.to_account_info(),
                to: self.user_ana.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }

    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_u.to_account_info(),
                to: self.treasury_u.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.mint_pre_ana.to_account_info(),
                to: self.user_pre_ana.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }
}
