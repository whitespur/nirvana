use crate::numbers::ANA;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token;
use anchor_spl::token::*;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct StakeAna<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    #[account(
        constraint = mint_ana.key() == config.mint_ana
    )]
    pub mint_ana: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = user_reward_index.owner == authority.key(),
        constraint = user_reward_index.to_account_info().owner == program_id,
        seeds = [
            b"userreward_v2".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump = user_reward_index.bump
    )]
    pub user_reward_index: Box<Account<'info, UserRewardV2>>,

    #[account(
        mut,
        constraint = user_token_ana.owner == authority.key()
    )]
    pub user_token_ana: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = stake_pool_ana.key() == config.stake_pool_ana
    )]
    pub stake_pool_ana: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = price_field.nirv_center == nirv_center.key(),
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_field.bump
    )]
    pub price_field: Box<Account<'info, PriceFieldV2>>,

    pub token_program: Program<'info, Token>,
}
pub fn handler(ctx: Context<StakeAna>, amount: u64) -> Result<()> {
    // update how much the user has staked
    ctx.accounts.user_reward_index.stake_ana(
        ANA::from_u64(amount),
        &mut ctx.accounts.config,
        &ctx.accounts.price_field,
    );

    token::transfer(ctx.accounts.transfer_context(), amount)?;

    Ok(())
}

impl<'info> StakeAna<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token_ana.to_account_info(),
                to: self.stake_pool_ana.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }
}
