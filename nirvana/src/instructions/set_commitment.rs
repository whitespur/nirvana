use crate::errors::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use std::cmp::{max, min};

#[derive(Accounts)]
pub struct SetCommitment<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        mut,
        constraint = commitment.owner == authority.key()
    )]
    pub commitment: Account<'info, Commitment>,

    #[account(
        mut,
        constraint = commitment_meta.nirv_center == nirv_center.key(),
        constraint = commitment_meta.usdc_escrow_token_account == escrow_account.key(),
    )]
    pub commitment_meta: Account<'info, CommitmentMeta>,

    #[account(mut)]
    pub escrow_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_usdc_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<SetCommitment>, target_spend_usd: u64) -> Result<()> {
    let c = &mut ctx.accounts.commitment;
    let cm = &mut ctx.accounts.commitment_meta;
    let usdc_denominator = cm.usdc_denominator();
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    msg!("Target commit: {}", target_spend_usd);
    msg!("Current commit: {}", c.target_spend_usd);
    msg!("Current reward rate: {}", c.reward_index.val);

    if cm.start_time > now {
        return Err(error!(ErrorCode::CommitmentPeriodNotBegun));
    }

    let is_increase = target_spend_usd > c.target_spend_usd;
    let delta = max(target_spend_usd, c.target_spend_usd)
        .checked_sub(min(target_spend_usd, c.target_spend_usd))
        .unwrap();
    // 1% escrow
    // This might overflow leading to vulnerability
    // Especially since user  is passing target_spend_usd
    let escrow_change = delta.checked_mul(usdc_denominator).unwrap() / 100;

    // If not increasing target, then reduce escrow
    // Else increase escrow & update rate
    if !is_increase {
        // Return escrow change amount
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow_account.to_account_info(),
                    to: ctx.accounts.user_usdc_account.to_account_info(),
                    authority: ctx.accounts.nirv_center_authority.to_account_info(),
                },
            )
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
            escrow_change,
        )?;

        // update total amount on commitment metadata
        cm.sub(delta);
    } else {
        // send user funds to escrow
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    to: ctx.accounts.escrow_account.to_account_info(),
                    from: ctx.accounts.user_usdc_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            escrow_change,
        )?;

        let new_rate = cm.get_rate(now);
        // If paying more, then the reward rate for the user should change
        c.update_reward_rate(delta, new_rate);

        // update total amount on commitment metadata
        cm.add(delta);
    }

    // update the commitment
    c.target_spend_usd = target_spend_usd;

    msg!("New reward rate: {}", c.reward_index.val);

    Ok(())
}
