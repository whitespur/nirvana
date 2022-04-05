use crate::errors::ErrorCode;
use crate::numbers::ALMS;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::Transfer;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct UnstakeAlms<'info> {
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
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = mint_alms.key() == config.mint_alms
    )]
    pub mint_alms: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_token_alms.mint == mint_alms.key(),
        constraint = user_token_alms.owner == authority.key()
    )]
    pub user_token_alms: Account<'info, TokenAccount>,

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
        constraint = stake_pool_alms.key() == config.stake_pool_alms
    )]
    pub stake_pool_alms: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<UnstakeAlms>, amount: u64) -> Result<()> {
    ctx.accounts.sufficent_staked_alms(amount)?;

    // TODO: stage the unclaimed fees

    ctx.accounts
        .fee_collector
        .unstake_alms(ALMS::from_u64(amount));

    // Transfer ALMS to user
    token::transfer(
        ctx.accounts
            .transfer_user_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        amount,
    )?;

    Ok(())
}

impl<'info> UnstakeAlms<'info> {
    fn sufficent_staked_alms(&self, amount: u64) -> Result<()> {
        let balance = self.fee_collector.staked_alms;

        if amount > balance.val {
            return Err(ErrorCode::InsufficientStakedALMSToUnstake.into());
        }
        Ok(())
    }

    fn transfer_user_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.stake_pool_alms.to_account_info(),
                to: self.user_token_alms.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
