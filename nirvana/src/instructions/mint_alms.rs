use crate::state::*;
use crate::utils::admin;
use crate::utils::is_debug;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token::{self, *};

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct MintAlms<'info> {
    pub authority: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

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
        constraint = mint_alms.key() == config.mint_alms
    )]
    pub mint_alms: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_alms.mint == mint_alms.key()
    )]
    pub user_alms: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(ctx: Context<MintAlms>, amount: u64) -> Result<()> {
    token::mint_to(
        ctx.accounts
            .mint_to_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        amount,
    )?;

    Ok(())
}

impl<'info> MintAlms<'info> {
    fn mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint_alms.to_account_info(),
                to: self.user_alms.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
