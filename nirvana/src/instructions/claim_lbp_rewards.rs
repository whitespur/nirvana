use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::state::{Commitment, CommitmentMeta, History, NirvCenter, NirvCenterConfigV3};

#[event]
struct ClaimLbpRewardsEvent {
    commitment: Pubkey,
    authority: Pubkey,
    amount_rewarded: u64,
    amount_spent_usd: u64,
}

#[derive(Accounts)]
pub struct ClaimLbpRewards<'info> {
    pub authority: Signer<'info>,
    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    #[account(
        mut,
        constraint = commitment.owner == authority.key(),
        constraint = commitment.commitment_meta == commitment_meta.key(),
        seeds = [
            b"commitment".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump = commitment.bump
    )]
    pub commitment: Box<Account<'info, Commitment>>,

    #[account(
        constraint = commitment_meta.nirv_center == nirv_center.key(),
        seeds = [
            b"commitment_meta".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump = commitment_meta.bump
    )]
    pub commitment_meta: Box<Account<'info, CommitmentMeta>>,

    #[account(
        constraint = history.authority == authority.key(),
        seeds = [
            b"history".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump = history.bump
    )]
    pub history: Box<Account<'info, History>>,

    #[account(
        mut,
        constraint = usdc_escrow_token_account.key() == commitment_meta.usdc_escrow_token_account
    )]
    pub usdc_escrow_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_usdc_account.owner == authority.key()
    )]
    pub user_usdc_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_prana_account.owner == authority.key()
    )]
    pub user_prana_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = mint_prana.key() == config.mint_pre_ana
    )]
    pub mint_prana: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

impl<'info> ClaimLbpRewards<'info> {
    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.usdc_escrow_token_account.to_account_info(),
                to: self.user_usdc_account.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }

    fn mint_reward_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint_prana.to_account_info(),
                to: self.user_prana_account.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}

/// Claim the reward for the Liquidity Bootstrapping Pool
pub fn handler(ctx: Context<ClaimLbpRewards>) -> Result<()> {
    let commitment = &ctx.accounts.commitment;
    let history = &ctx.accounts.history;
    let nirv_center = &ctx.accounts.nirv_center;
    let config = &ctx.accounts.config;

    if commitment.claimed_and_dead {
        return Err(ErrorCode::CommitmentAlreadyClaimed.into());
    }

    if commitment.target_spend_usd < history.net_spent_usd {
        return Err(ErrorCode::CommitmentTargetNotMet.into());
    }

    // If still bootstrapping, or earlier than boostrapping, return
    let now = Clock::get()?.unix_timestamp.unsigned_abs();
    if !config.bootstrapping_ended(now) {
        return Err(ErrorCode::BootstrappingNotEnded.into());
    }

    // Return escrow balance
    token::transfer(
        ctx.accounts
            .transfer_context()
            .with_signer(&[&nirv_center.authority_seeds()]),
        commitment.target_spend_usd * ctx.accounts.commitment_meta.usdc_denominator() / 100,
    )?;

    // Mint reward prANA to user
    let bootstrap_avg_price = history.bootstrap_avg_price();
    let amount_rewarded = commitment.reward_amount(bootstrap_avg_price).val;
    msg!("Target spend: {}", commitment.target_spend_usd);
    msg!("Bootstrap avg price: {}", bootstrap_avg_price);
    msg!("Amount rewarded: {}", amount_rewarded);

    token::mint_to(
        ctx.accounts
            .mint_reward_context()
            .with_signer(&[&nirv_center.authority_seeds()]),
        amount_rewarded,
    )?;

    emit!(ClaimLbpRewardsEvent {
        commitment: ctx.accounts.commitment.key(),
        authority: ctx.accounts.authority.key(),
        amount_rewarded,
        amount_spent_usd: commitment.target_spend_usd
    });

    // the commitment is over
    ctx.accounts.commitment.claimed_and_dead = true;
    // There is also case of closing account and getting rent back

    Ok(())
}
