use crate::numbers::CoarseNumber;
use crate::state::*;
use crate::utils::{admin, is_debug};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetTranaBuyFee<'info> {
    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        mut,
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    pub signer: Signer<'info>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.signer))]
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(ctx: Context<SetTranaBuyFee>, fee: CoarseNumber) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.trana_buy_fee = fee;

    Ok(())
}
