use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitGlobalHistory<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        init,
        seeds = [
            b"globalhistory".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump,
        payer=payer
    )]
    pub global_history: Account<'info, GlobalHistory>,
    pub system_program: Program<'info, System>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.payer))]
pub fn handler(ctx: Context<InitGlobalHistory>) -> Result<()> {
    let b = ctx.bumps.get("global_history").unwrap();
    let x = &mut ctx.accounts.global_history;
    x.bump = *b;
    Ok(())
}
