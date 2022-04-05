use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetDebugMode<'info> {
    #[account(mut)]
    pub nirv_center: Account<'info, NirvCenter>,
}

pub fn handler(ctx: Context<SetDebugMode>, debug_mode: bool) -> Result<()> {
    let nirv_center = &mut ctx.accounts.nirv_center;

    nirv_center.debug_mode = debug_mode;
    Ok(())
}
