use crate::numbers::PreciseNumber;
use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetRewardRate<'info> {
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
pub fn handler(ctx: Context<SetRewardRate>, reward_rate: PreciseNumber) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.prana_reward_rate = reward_rate;
    Ok(())
}
