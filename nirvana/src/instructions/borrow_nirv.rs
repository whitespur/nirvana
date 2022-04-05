use crate::numbers::{ALMS, NIRV};
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token::{self, *};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct BorrowNirv<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

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
        mut,
        constraint = mint_nirv.key() == config.mint_nirv
    )]
    pub mint_nirv: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_nirv.mint == mint_nirv.key()
    )]
    pub user_nirv: Account<'info, TokenAccount>,

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

    #[account(
        mut,
        constraint = nirv_fee_account.key() == config.nirv_fee_account
    )]
    pub nirv_fee_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
pub fn handler(ctx: Context<BorrowNirv>, amount: u64) -> Result<()> {
    let total_alms_staked = ALMS::from_u64(ctx.accounts.stake_pool_alms.amount);

    let requested_nirv = NIRV::from_u64(amount);

    let (amount_less_fee, fee) = ctx
        .accounts
        .config
        .collect_nirv_origination_fee(requested_nirv, total_alms_staked);

    ctx.accounts
        .user_reward_index
        .borrow_nirv(requested_nirv, &ctx.accounts.price_field)?;

    // Transfer NIRV to fee account
    token::mint_to(
        ctx.accounts
            .mint_to_fee_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        fee.into(),
    )?;

    // Transfer NIRV to user
    token::mint_to(
        ctx.accounts
            .mint_to_user_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        amount_less_fee.into(),
    )?;

    Ok(())
}

impl<'info> BorrowNirv<'info> {
    fn mint_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint_nirv.to_account_info(),
                to: self.user_nirv.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
    fn mint_to_fee_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint_nirv.to_account_info(),
                to: self.nirv_fee_account.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
