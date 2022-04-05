use crate::numbers::ANA;
use crate::state::*;
use crate::utils::admin;
use crate::utils::is_debug;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token::Mint;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct Reward<'info> {
    pub authority: Signer<'info>,

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

    #[account(
        constraint = mint_ana.key() == config.mint_ana,
    )]
    pub mint_ana: Account<'info, Mint>,

    #[account(
        constraint = stake_pool_ana.key() == config.stake_pool_ana
    )]
    pub stake_pool_ana: Account<'info, TokenAccount>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(ctx: Context<Reward>) -> Result<()> {
    msg!("Rate: {}", ctx.accounts.config.prana_reward_rate.val);
    let ana_supply = ANA::from_u64(ctx.accounts.mint_ana.supply);
    msg!("ANA supply: {}", ana_supply.val);
    let total_staked = ANA::from_u64(ctx.accounts.stake_pool_ana.amount);
    msg!("Total staked: {}", total_staked.val);
    ctx.accounts
        .config
        .drop_prana_reward(ana_supply, total_staked, 24 * 60 * 60);
    Ok(())
}
