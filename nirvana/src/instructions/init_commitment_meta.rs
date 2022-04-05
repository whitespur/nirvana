use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitCommitmentMeta<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        init,
        seeds = [
            b"commitment_meta".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump,
        payer=authority
    )]
    pub commitment_meta: Box<Account<'info, CommitmentMeta>>,

    #[account(init, token::authority = nirv_center_authority, token::mint = usdc_mint, payer = authority)]
    pub usdc_escrow_token_account: Box<Account<'info, TokenAccount>>,

    pub usdc_mint: Box<Account<'info, Mint>>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// The commitment metadata is used for the pre-commit phase of the launch
#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
pub fn handler(
    ctx: Context<InitCommitmentMeta>,
    bump: u8,
    start_time: i64,
    early_bird_end: i64,
    end: i64,
) -> Result<()> {
    let x = &mut ctx.accounts.commitment_meta;

    x.bump = bump;
    x.nirv_center = ctx.accounts.nirv_center.key();
    x.start_time = start_time;
    x.early_bird_end = early_bird_end;
    x.end = end;
    x.usdc_escrow_token_account = ctx.accounts.usdc_escrow_token_account.key();
    x.escrow_decimals = ctx.accounts.usdc_mint.decimals;

    msg!(
        "Start: {}, EarlyBirdEnd: {}, End: {}",
        start_time,
        early_bird_end,
        end
    );

    Ok(())
}
