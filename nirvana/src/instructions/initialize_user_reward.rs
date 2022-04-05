use crate::numbers::ANA;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;

#[derive(Accounts)]
pub struct InitializeUserReward<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    #[account(
        init,
        seeds = [
            b"userreward_v2".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump,
        payer=authority
    )]
    pub user_reward: Account<'info, UserRewardV2>,
    pub system_program: Program<'info, System>,
}
pub fn handler(ctx: Context<InitializeUserReward>, bump: u8) -> Result<()> {
    let user_reward = &mut ctx.accounts.user_reward;

    user_reward.owner = ctx.accounts.authority.key();
    // set the user reward index to the current index of the nirv center
    user_reward.index = ctx.accounts.config.reward_index;
    user_reward.staked_amount = ANA::ZERO;
    user_reward.staged_pre_ana_rewards = ANA::ZERO;
    user_reward.staged_pre_ana_fees = ANA::ZERO;
    user_reward.bump = bump;

    Ok(())
}
