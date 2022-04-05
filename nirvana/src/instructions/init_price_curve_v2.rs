use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitPriceCurveV2<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        init,
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump,
        payer=payer
    )]
    pub price_curve: Account<'info, PriceFieldV2>,
    pub system_program: Program<'info, System>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.payer))]
pub fn handler(ctx: Context<InitPriceCurveV2>) -> Result<()> {
    let b = ctx.bumps.get("price_curve").unwrap();
    let x = &mut ctx.accounts.price_curve;
    x.nirv_center = ctx.accounts.nirv_center.key();
    x.bump = *b;
    Ok(())
}
