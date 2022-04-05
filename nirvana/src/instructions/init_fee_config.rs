use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitFeeConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        init,
        seeds = [
            b"fee_config".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump,
        payer=payer
    )]
    pub fee_config: Account<'info, FeeConfig>,
    pub system_program: Program<'info, System>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.payer))]
pub fn handler(ctx: Context<InitFeeConfig>) -> Result<()> {
    let b = ctx.bumps.get("fee_config").unwrap();
    let x = &mut ctx.accounts.fee_config;
    x.bump = *b;
    Ok(())
}
