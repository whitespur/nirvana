use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetCommitmentMeta<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        mut,
        seeds = [
            b"commitment_meta".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump = commitment_meta.bump,
    )]
    pub commitment_meta: Box<Account<'info, CommitmentMeta>>,
}

/// The commitment metadata is used for the pre-commit phase of the launch
#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
pub fn handler(
    ctx: Context<SetCommitmentMeta>,
    start_time: i64,
    early_bird_end: i64,
    end: i64,
) -> Result<()> {
    let x = &mut ctx.accounts.commitment_meta;

    x.start_time = start_time;
    x.early_bird_end = early_bird_end;
    x.end = end;

    msg!(
        "Start: {}, EarlyBirdEnd: {}, End: {}",
        start_time,
        early_bird_end,
        end
    );

    Ok(())
}
