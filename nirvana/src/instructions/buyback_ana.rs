use crate::errors::ErrorCode;
use crate::numbers::Decimalable;
use crate::numbers::ANA;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token;
use anchor_spl::token::Burn;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::Transfer;
use rust_decimal::prelude::*;
use std::ops::Mul;

#[derive(Accounts)]
pub struct BuybackAna<'info> {
    pub authority: Signer<'info>, // I dont like calling it authority since
    // in most of cases thats how we call program signer.
    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    pub mint_u: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = user_u.mint == mint_u.key(),
        constraint = user_u.owner == authority.key()
    )]
    pub user_u: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = treasury_u.owner == nirv_center.key(),
        constraint = treasury_u.mint == mint_u.key()
    )]
    pub treasury_u: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = mint_ana.key() == config.mint_ana
    )]
    pub mint_ana: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = user_ana.mint == mint_ana.key(),
        constraint = user_ana.owner == authority.key()
    )]
    pub user_ana: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = price_field.nirv_center == nirv_center.key(),
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_field.bump
    )]
    pub price_field: Account<'info, PriceFieldV2>,

    pub token_program: Program<'info, Token>,
}
pub fn handler(ctx: Context<BuybackAna>, ana_amount: u64) -> Result<()> {
    let ana_balance = ctx.accounts.user_ana.amount;

    if ana_amount > ana_balance {
        return Err(ErrorCode::InsufficientAnaToBuyback.into());
    }

    let payback_decimals = ctx.accounts.mint_u.decimals.to_u32().unwrap();
    let floor = ctx.accounts.price_field.floor_price.to_decimal();
    let ana_amount_d = ANA::from_u64(ana_amount).to_decimal();

    // Since tokens out, round down
    let mut payback_amount = ana_amount_d
        .mul(floor)
        .round_dp_with_strategy(payback_decimals, RoundingStrategy::ToZero);

    payback_amount.rescale(payback_decimals);
    let payback_amount = payback_amount.mantissa().to_u64().unwrap();
    let payback_treasury_balance = ctx.accounts.treasury_u.amount;

    if payback_amount > payback_treasury_balance {
        return Err(ErrorCode::InsufficientTreasuryUToWithdraw.into());
    }

    token::burn(ctx.accounts.burn_ana_context(), ana_amount)?;

    token::transfer(
        ctx.accounts
            .pay_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        payback_amount,
    )?;

    Ok(())
}

impl<'info> BuybackAna<'info> {
    fn burn_ana_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.mint_ana.to_account_info(),
                to: self.user_ana.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn pay_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.treasury_u.to_account_info(),
                to: self.user_u.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
