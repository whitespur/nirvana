use crate::numbers::CoarseNumber;
use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
#[derive(Accounts)]
pub struct InitMoneyMarket<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    pub mint: Account<'info, Mint>,

    #[account(init, token::mint = mint, token::authority = nirv_center_authority, payer = signer)]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [
            b"mm1".as_ref(),
            mint.key().as_ref(),
            nirv_center.key().as_ref()
        ],
        bump,
        payer = signer
    )]
    pub money_market: Account<'info, MoneyMarket>,

    /// The account containing the price information for the token.
    /// CHECK: This is a high-trust action
    pub oracle_price: AccountInfo<'info>, // I would at least check owner of it

    /// The account containing the metadata about the token being referenced
    /// CHECK: This is a high-trust action
    pub oracle_product: AccountInfo<'info>, // I would at least check owner of it

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.signer))]
pub fn handler(
    ctx: Context<InitMoneyMarket>,
    bump: u8,
    rfv_coefficient: CoarseNumber,
    for_amm: bool,
    for_prana: bool,
) -> Result<()> {
    let money_market = &mut ctx.accounts.money_market;

    money_market.enabled = true;
    money_market.nirv_center = ctx.accounts.nirv_center.key();
    money_market.risk_free_value_coefficient = rfv_coefficient;
    money_market.token_account = ctx.accounts.token_account.key();
    money_market.mint = ctx.accounts.mint.key();
    money_market.decimals = ctx.accounts.mint.decimals;
    money_market.for_amm = for_amm;
    money_market.for_prana = for_prana;
    money_market.pyth_oracle_metadata = ctx.accounts.oracle_product.key();
    money_market.pyth_oracle_price = ctx.accounts.oracle_price.key();
    money_market.bump = bump;

    Ok(())
}
