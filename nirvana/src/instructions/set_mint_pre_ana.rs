use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
#[derive(Accounts)]
pub struct SetMintPreAna<'info> {
    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        mut,
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    pub mint_pre_ana: Account<'info, Mint>,

    pub signer: Signer<'info>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.signer))]
pub fn handler(ctx: Context<SetMintPreAna>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.mint_pre_ana = ctx.accounts.mint_pre_ana.key();
    Ok(())
}
