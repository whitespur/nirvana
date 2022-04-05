use crate::numbers::ALMS;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    pub authority: Signer<'info>,

    #[account(
        constraint = config.mint_ana == mint_ana.key(),
        constraint = config.mint_pre_ana == mint_pre_ana.key()
    )]
    pub nirv_center: Box<Account<'info, NirvCenter>>,

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
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        constraint = mint_ana.key() == config.mint_ana
    )]
    pub mint_ana: Account<'info, Mint>,

    #[account(
        mut,
        constraint = mint_pre_ana.key() == config.mint_pre_ana
    )]
    pub mint_pre_ana: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_pre_ana_account.owner == authority.key()
    )]
    pub user_pre_ana_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_reward.owner == authority.key(),
        constraint = user_reward.to_account_info().owner == program_id, // This check actually breaks composability // NB do you need it ? This is redundant imo
        seeds = [
            b"userreward_v2".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump = user_reward.bump
    )]
    pub user_reward: Account<'info, UserRewardV2>,

    #[account(
        constraint = price_field.nirv_center == nirv_center.key(),
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_field.bump
    )]
    pub price_field: Box<Account<'info, PriceFieldV2>>,

    #[account(
        constraint = stake_pool_alms.key() == config.stake_pool_alms
    )]
    pub stake_pool_alms: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = prana_fee_account.key() == config.prana_fee_account
    )]
    pub prana_fee_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

/// Claim the rewarded prana
pub fn handler(ctx: Context<ClaimReward>) -> Result<()> {
    let total_alms_staked = ALMS::from_u64(ctx.accounts.stake_pool_alms.amount);

    let (reward_less_fee, fee) = ctx.accounts.user_reward.claim_prana_rewards(
        &mut ctx.accounts.config,
        &ctx.accounts.price_field,
        total_alms_staked,
    );

    token::mint_to(
        ctx.accounts
            .mint_to_fee_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        fee.into(),
    )?;

    token::mint_to(
        ctx.accounts
            .mint_to_user_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        reward_less_fee.into(),
    )?;

    Ok(())
}

impl<'info> ClaimReward<'info> {
    fn mint_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint_pre_ana.to_account_info(),
                to: self.user_pre_ana_account.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
    fn mint_to_fee_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint_pre_ana.to_account_info(),
                to: self.prana_fee_account.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
