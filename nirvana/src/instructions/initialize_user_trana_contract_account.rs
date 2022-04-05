use crate::state::*;
use crate::utils::is_debug;
use anchor_lang::prelude::*;
use anchor_lang::Key;

#[derive(Accounts)]
#[instruction(bump: u8, index: u8)]
pub struct InitializeUserTranaContractAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds = [
            b"userbond".as_ref(),
            &[index],
            nirv_center.key().as_ref(),
            payer.key().as_ref(),
        ],
        bump,
        payer=payer
    )]
    pub user_trana: Account<'info, UserTranaContract>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub nirv_center: Account<'info, NirvCenter>,
}
#[access_control(is_debug(&ctx.accounts.nirv_center))]
pub fn handler(
    ctx: Context<InitializeUserTranaContractAccount>,
    _bump: u8,
    _index: u8,
) -> Result<()> {
    let trana = &mut ctx.accounts.user_trana;
    trana.available = true;
    trana.user = ctx.accounts.payer.key();

    Ok(())
}
