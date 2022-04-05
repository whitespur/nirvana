use anchor_lang::prelude::*;

use crate::numbers::{PreciseNumber, ANA};
use crate::state::{NirvCenter, PriceFieldV2};
use crate::utils::admin;

#[derive(Accounts)]
pub struct SetPriceFieldParams<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        mut,
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_field.bump,
    )]
    pub price_field: Account<'info, PriceFieldV2>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
pub fn handler(
    ctx: Context<SetPriceFieldParams>,
    ramp_start: ANA,
    ramp_width: ANA,
    ramp_height: PreciseNumber,
    main_slope: PreciseNumber,
    floor_price: PreciseNumber,
) -> Result<()> {
    let pf = &mut ctx.accounts.price_field;
    pf.ramp_start = ramp_start;
    pf.ramp_width = ramp_width;
    pf.ramp_height = ramp_height;
    pf.main_slope = main_slope;
    pf.floor_price = floor_price;

    Ok(())
}
