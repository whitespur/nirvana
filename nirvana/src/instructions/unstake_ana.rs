use crate::errors::ErrorCode;
use crate::numbers::{ALMS, ANA};
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
pub struct UnstakeAna<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        constraint = mint_ana.key() == config.mint_ana
    )]
    pub mint_ana: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_token_ana.mint == mint_ana.key(),
        constraint = user_token_ana.owner == authority.key()
    )]
    pub user_token_ana: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_reward_index.owner == authority.key(),
        seeds = [
            b"userreward_v2".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump = user_reward_index.bump
    )]
    pub user_reward_index: Account<'info, UserRewardV2>,

    #[account(
        mut,
        constraint = stake_pool_ana.key() == config.stake_pool_ana
    )]
    pub stake_pool_ana: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = ana_fee_account.key() == config.ana_fee_account
    )]
    pub ana_fee_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        constraint = price_field.nirv_center == nirv_center.key(),
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_field.bump
    )]
    pub price_field: Account<'info, PriceFieldV2>,

    #[account(
        constraint = stake_pool_alms.key() == config.stake_pool_alms
    )]
    pub stake_pool_alms: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<UnstakeAna>, amount: u64) -> Result<()> {
    // shouldn't we compound rewards first?
    ctx.accounts.sufficent_staked_ana(amount)?; // Only this line protects from entire program falling apart
                                                // Imo add checked math
    let amount = ANA::from_u64(amount);

    let total_alms_staked = ALMS::from_u64(ctx.accounts.stake_pool_alms.amount);

    // decrease user's staked amount
    let (amount_less_fee, fee) = ctx.accounts.user_reward_index.unstake_ana(
        amount,
        &mut ctx.accounts.config,
        &ctx.accounts.price_field,
        total_alms_staked,
    )?;

    // Transfer ANA to user
    token::transfer(
        ctx.accounts
            .transfer_user_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        amount_less_fee.into(),
    )?;

    // Transfer ANA to fee account
    token::transfer(
        ctx.accounts
            .transfer_fee_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        fee.into(),
    )?;

    Ok(())
}

impl<'info> UnstakeAna<'info> {
    fn sufficent_staked_ana(&self, amount: u64) -> Result<()> {
        let balance = self.user_reward_index.staked_amount;

        if amount > balance.val {
            return Err(ErrorCode::InsufficientStakedANAToUnstake.into());
        }
        Ok(())
    }

    fn transfer_user_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.stake_pool_ana.to_account_info(),
                to: self.user_token_ana.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }

    fn transfer_fee_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.stake_pool_ana.to_account_info(),
                to: self.ana_fee_account.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
