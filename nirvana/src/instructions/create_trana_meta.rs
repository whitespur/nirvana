use crate::numbers::{ArbitraryNumber, PreciseNumber, ANA};
use crate::state::*;
use crate::utils::{admin, is_debug};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
#[instruction(vesting_length_seconds: i64, sensitivity: PreciseNumber, max_discount_ratio: PreciseNumber, bump: u8)]
pub struct CreateTranaMeta<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = mint.key() == money_market.mint
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [
            b"trana_v1".as_ref(),
            vesting_length_seconds.to_string().as_bytes().as_ref(),
            mint.key().as_ref(),
            nirv_center.key().as_ref()
        ],
        bump,
        payer = authority,
    )]
    pub trana: Box<Account<'info, TranaMeta>>,

    #[account(
        mut,
        constraint = money_market.nirv_center == nirv_center.key())
    ]
    pub money_market: Box<Account<'info, MoneyMarket>>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(
    ctx: Context<CreateTranaMeta>,
    vesting_length_seconds: u64,
    sensitivity: PreciseNumber,
    max_discount_ratio: PreciseNumber,
    _bump: u8,
) -> Result<()> {
    let trana_account = &mut ctx.accounts.trana;
    let nirv_center = &ctx.accounts.nirv_center;
    let money_market = &ctx.accounts.money_market;

    let scale = money_market.decimals;

    trana_account.nirv_center = nirv_center.key();
    trana_account.vesting_length_seconds = vesting_length_seconds;
    trana_account.sensitivity = sensitivity;
    trana_account.max_discount_ratio = max_discount_ratio;
    trana_account.ana_outstanding = ANA::ZERO;
    trana_account.total_bought = ArbitraryNumber { val: 0, scale };
    trana_account.money_market = money_market.key();
    // TODO - do not enable by default
    trana_account.enabled = true;

    Ok(())
}
