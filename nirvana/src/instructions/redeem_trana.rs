use crate::errors::ErrorCode;
use crate::state::*;
use crate::utils::is_debug;
use anchor_lang::prelude::*;
use anchor_lang::Key;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::Transfer;

#[derive(Accounts)]
pub struct RedeemTrana<'info> {
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
        constraint = mint_ana.key() == config.mint_ana
    )]
    pub mint_ana: Account<'info, Mint>,

    #[account(
        mut,
        constraint = user_trana.trana_meta == trana_meta.key(),
        constraint = user_trana.user == authority.key()
    )]
    pub user_trana: Box<Account<'info, UserTranaContract>>,

    #[account(
        mut,
        constraint = trana_meta.nirv_center == nirv_center.key()
    )]
    pub trana_meta: Box<Account<'info, TranaMeta>>,

    #[account(
        mut,
        constraint = user_ana.mint == mint_ana.key(),
        constraint = user_ana.owner == authority.key()
    )]
    pub user_ana: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = treasury_ana.key() == config.treasury_ana
    )]
    pub treasury_ana: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(ctx: Context<RedeemTrana>) -> Result<()> {
    ctx.accounts.trana_available()?;

    let now = Clock::get()?.unix_timestamp;
    let left_to_redeem = ctx.accounts.user_trana.get_left_to_redeem(now);

    ctx.accounts.user_trana.update_redeemed(left_to_redeem);

    // there is nothing left in the bond
    if left_to_redeem.val == 0 {
        msg!("Attempted redemption of already vested bond");
        return Ok(());
    }

    // Transfer the left to redeem amount ANA
    token::transfer(
        ctx.accounts
            .transfer_context()
            .with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
        left_to_redeem.into(),
    )?;

    // Decrease outstanding ANA
    ctx.accounts.trana_meta.sub_ana_outstanding(left_to_redeem);

    Ok(())
}

impl<'info> RedeemTrana<'info> {
    fn trana_available(&self) -> Result<()> {
        // Bond is in initial state
        if self.user_trana.available == true {
            return Err(ErrorCode::RedeemUnusedBond.into());
        }
        Ok(())
    }

    fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.treasury_ana.to_account_info(),
                to: self.user_ana.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }
}
