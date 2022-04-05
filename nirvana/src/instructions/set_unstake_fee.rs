use crate::numbers::CoarseNumber;
use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetUnstakeFee<'info> {
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
pub fn handler(ctx: Context<SetUnstakeFee>, fee: CoarseNumber) -> Result<()> {
    let config = &mut ctx.accounts.config;

    config.unstake_fee = fee;

    Ok(())
}
