use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitHistory<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        init,
        seeds = [
            b"history".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump,
        payer=authority
    )]
    pub history: Account<'info, History>,

    pub system_program: Program<'info, System>,
}
/// History is a per-user account to log their activity with the protocol
pub fn handler(ctx: Context<InitHistory>, bump: u8) -> Result<()> {
    let x = &mut ctx.accounts.history;

    x.bump = bump;
    x.nirv_center = ctx.accounts.nirv_center.key();
    x.authority = ctx.accounts.authority.key();

    Ok(())
}
