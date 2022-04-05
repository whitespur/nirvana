use crate::errors::ErrorCode;
use crate::state::*;
use crate::utils::{admin, is_debug};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetTranaEnabled<'info> {
    #[account(
        mut,
        constraint = trana_meta.nirv_center == nirv_center.key() @ErrorCode::UnauthorizedBondMetaAccess
    )]
    pub trana_meta: Account<'info, TranaMeta>,

    pub nirv_center: Account<'info, NirvCenter>,

    pub signer: Signer<'info>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.signer))]
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(ctx: Context<SetTranaEnabled>, is_enabled: bool) -> Result<()> {
    let trana_meta = &mut ctx.accounts.trana_meta;
    trana_meta.enabled = is_enabled;
    Ok(())
}
