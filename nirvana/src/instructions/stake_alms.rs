use crate::numbers::ALMS;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token;
use anchor_spl::token::*;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct StakeAlms<'info> {
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
        constraint = mint_alms.key() == config.mint_alms
    )]
    pub mint_alms: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = fee_collector.owner == authority.key(),
        constraint = fee_collector.to_account_info().owner == program_id,
        seeds = [
            b"feecollector".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump = fee_collector.bump,
    )]
    pub fee_collector: Box<Account<'info, FeeCollector>>,

    #[account(
        mut,
        constraint = user_token_alms.owner == authority.key()
    )]
    pub user_token_alms: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = stake_pool_alms.key() == config.stake_pool_alms
    )]
    pub stake_pool_alms: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
pub fn handler(ctx: Context<StakeAlms>, amount: u64) -> Result<()> {
    // TODO: stage the unclaimed rewards

    token::transfer(ctx.accounts.transfer_context(), amount)?;

    ctx.accounts
        .fee_collector
        .stake_alms(ALMS::from_u64(amount));

    Ok(())
}

impl<'info> StakeAlms<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_token_alms.to_account_info(),
                to: self.stake_pool_alms.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }
}
