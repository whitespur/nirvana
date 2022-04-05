use crate::errors::ErrorCode;
use crate::numbers::NIRV;
use crate::state::*;
use crate::utils::is_debug;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token::{self, *};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct RepayNirv<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

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

    pub token_program: Program<'info, Token>,
}

#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(ctx: Context<RepayNirv>, amount: u64) -> Result<()> {
    let nirv_balance = ctx.accounts.user_nirv.amount;
    if amount > nirv_balance {
        return Err(ErrorCode::RepayNIRVMoreThanHeld.into());
    }

    token::burn(ctx.accounts.burn_context(), amount)?;

    ctx.accounts
        .user_reward_index
        .repay_nirv(NIRV::from_u64(amount))?;

    Ok(())
}

impl<'info> RepayNirv<'info> {
    fn burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.mint_nirv.to_account_info(),
                to: self.user_nirv.to_account_info(),
                authority: self.authority.to_account_info(),
            },
        )
    }
}
