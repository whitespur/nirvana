use crate::errors::ErrorCode;
use crate::numbers::ANA;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct RewardByTime<'info> {
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
        constraint = mint_ana.key() == config.mint_ana,
    )]
    pub mint_ana: Account<'info, Mint>,

    #[account(
        constraint = stake_pool_ana.key() == config.stake_pool_ana
    )]
    pub stake_pool_ana: Account<'info, TokenAccount>,
}
pub fn handler(ctx: Context<RewardByTime>) -> Result<()> {
    let reward_interval_seconds = ctx.accounts.config.prana_reward_interval_seconds;
    let clock = Clock::get()?;
    let ts = clock.unix_timestamp.unsigned_abs();

    // Reward once per 5 minutes
    require!(
        ts > (ctx.accounts.config.time_of_last_prana_reward
            + reward_interval_seconds.unsigned_abs()),
        ErrorCode::RewardTooSoon
    );

    let ana_supply = ANA::from_u64(ctx.accounts.mint_ana.supply);
    let total_staked = ANA::from_u64(ctx.accounts.stake_pool_ana.amount);

    ctx.accounts
        .config
        .drop_prana_reward(ana_supply, total_staked, reward_interval_seconds);
    ctx.accounts.config.time_of_last_prana_reward = ts;

    Ok(())
}
