use crate::numbers::PreciseNumber;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitCommitment<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>, // Should add seed check here

    #[account(
        constraint = commitment_meta.nirv_center == nirv_center.key()
    )]
    pub commitment_meta: Account<'info, CommitmentMeta>,

    #[account(
        init,
        seeds = [
            b"commitment".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump,
        payer=authority
    )]
    pub commitment: Account<'info, Commitment>,

    pub system_program: Program<'info, System>,
}
/// Commitments are per-user accounts to log their commitments to participate during the launch
pub fn handler(ctx: Context<InitCommitment>, bump: u8) -> Result<()> {
    let x = &mut ctx.accounts.commitment;

    x.bump = bump;
    x.claimed_and_dead = false;
    x.commitment_meta = ctx.accounts.commitment_meta.key();
    x.owner = ctx.accounts.authority.key();
    x.reward_index = PreciseNumber::ZERO;

    Ok(())
}
