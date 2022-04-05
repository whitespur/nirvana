use crate::bond_math::bond_discount;
use crate::errors::ErrorCode;
use crate::numbers::ArbitraryNumber;
use crate::numbers::Decimalable;
use crate::numbers::{ALMS, ANA};
use crate::price_math::PriceCalculator;
use crate::state::*;
use crate::utils::is_debug;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token::{self, *};
use rust_decimal::prelude::*;
use std::convert::TryInto;

#[derive(Accounts)]
pub struct PurchaseTrana<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    #[account(
        mut,
        constraint = mint_ana.key() == config.mint_ana
    )]
    pub mint_ana: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = mint_u.key() == money_market.mint
    )]
    pub mint_u: Account<'info, Mint>,

    #[account(
        constraint = money_market.nirv_center == nirv_center.key(),
    )]
    pub money_market: Box<Account<'info, MoneyMarket>>,

    #[account(
        mut,
        constraint = trana_meta.nirv_center == nirv_center.key(),
        constraint = trana_meta.money_market == money_market.key()
    )]
    pub trana_meta: Box<Account<'info, TranaMeta>>,

    #[account(mut)]
    /// CHECK: User token account must be usable to transfer tokens
    pub user_token: AccountInfo<'info>, // Just check mint.

    #[account(
        mut,
        constraint = treasury_account.key() == money_market.token_account
    )]
    /// CHECK: Treasury account key is matched to money_market
    pub treasury_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = ana.key() == config.treasury_ana
    )]
    /// CHECK: Ana account key is matched to NirvCenter
    pub ana: AccountInfo<'info>,

    #[account(
        mut,
        constraint = ana_fee_account.key() == config.ana_fee_account
    )]
    /// CHECK: Key is matched to nirv center
    pub ana_fee_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = user_trana.available @ErrorCode::UnavailableBondAccount,
        constraint = user_trana.user == authority.key() @ErrorCode::UnauthorizedBondAccount
    )]
    pub user_trana: Box<Account<'info, UserTranaContract>>,

    #[account(
        mut,
        constraint = price_field.nirv_center == nirv_center.key(),
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_field.bump,
    )]
    pub price_field: Account<'info, PriceFieldV2>,

    #[account(
        constraint = stake_pool_alms.key() == config.stake_pool_alms
    )]
    pub stake_pool_alms: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(
    ctx: Context<PurchaseTrana>,
    payment_u64: u64,
    max_offered_price_u64: u64,
) -> Result<()> {
    let trana_meta = &ctx.accounts.trana_meta;
    let pf = &ctx.accounts.price_field;
    let money_market = &ctx.accounts.money_market;

    let payment = Decimal::new(
        payment_u64.try_into().unwrap(),
        money_market.decimals.into(),
    );

    let ana_supply = ANA::from_u64(ctx.accounts.mint_ana.supply);
    let floor = pf.floor_price.to_decimal();
    let native_base_price = pf.price_for_supply(ana_supply);

    let discount_ratio = bond_discount(&trana_meta, &native_base_price, &floor);
    let discount_ratio_complement = Decimal::ONE.checked_sub(discount_ratio).unwrap();

    // Apply discount
    let native_min_price = native_base_price
        .checked_mul(discount_ratio_complement)
        .unwrap();

    // Control for floor
    let native_min_price = native_min_price.max(floor);
    msg!("native trana price {}", native_min_price);

    let max_offered_price = Decimal::new(
        max_offered_price_u64.try_into().unwrap(),
        money_market.decimals.into(),
    );

    if max_offered_price < native_min_price {
        return Err(ErrorCode::BondPriceNotMet.into());
    }

    // Calculate amount of ANA purchased
    let ana_bought = payment
        .checked_div(native_min_price)
        .unwrap()
        .round_dp_with_strategy(ANA::SCALE, RoundingStrategy::ToZero);
    let ana_bought = ANA::from_decimal(ana_bought);

    let total_alms_staked = ALMS::from_u64(ctx.accounts.stake_pool_alms.amount);

    // Calculate ANA fee
    let (ana_bought_less_fee, fee) = ctx
        .accounts
        .config
        .collect_trana_buy_fee(ana_bought, total_alms_staked);

    // Transfer payment to to treasury account
    token::transfer(ctx.accounts.transfer_payment_context(), payment_u64)?;

    // Send the ANA to a treasury escrow holder
    token::mint_to(
        ctx.accounts
            .mint_escrow_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        ana_bought_less_fee.into(),
    )?;

    // Send ANA fee to fee account
    token::mint_to(
        ctx.accounts
            .mint_fee_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        fee.into(),
    )?;

    // Update price field with new ANA supply
    ctx.accounts
        .price_field
        .increase_supply_with_no_price_impact(ana_bought.into());

    // increase the outstanding ANA
    let trana_meta = &mut ctx.accounts.trana_meta;
    trana_meta.ana_outstanding += ana_bought_less_fee;

    // increase total bought, for bookkeeping purposes
    trana_meta.total_bought.val = trana_meta
        .total_bought
        .val
        .checked_add(payment_u64)
        .unwrap();

    // Configure the user's trana contract
    let now = Clock::get()?.unix_timestamp;
    let price_in_underlying =
        ArbitraryNumber::from_decimal(native_min_price, money_market.decimals.into());

    let user_trana = &mut ctx.accounts.user_trana;
    user_trana.available = false;
    user_trana.amount_ana = ana_bought;
    user_trana.price_in_underlying = price_in_underlying;
    user_trana.redeemed_amount = ANA::ZERO;
    user_trana.start_time = now;
    user_trana.trana_meta = trana_meta.key();
    user_trana.end_time = now + trana_meta.vesting_length_seconds.to_i64().unwrap();
    user_trana.user = ctx.accounts.authority.key();

    Ok(())
}

impl<'info> PurchaseTrana<'info> {
    fn transfer_payment_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token.to_account_info(),
                to: self.treasury_account.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }

    fn mint_escrow_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.ana.to_account_info(),
                mint: self.mint_ana.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }

    fn mint_fee_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.ana_fee_account.to_account_info(),
                mint: self.mint_ana.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
