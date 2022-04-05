use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseConfigV2<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        mut,
        close = payer,
        seeds = [
            b"config_v2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfig>>,

    pub system_program: Program<'info, System>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.payer))]
pub fn handler(ctx: Context<CloseConfigV2>) -> Result<()> {
    Ok(())
}
