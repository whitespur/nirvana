use crate::numbers::{PreciseNumber, ALMS, ANA, NIRV};
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::Key;

#[derive(Accounts)]
pub struct InitializeFeeCollector<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        init,
        seeds = [
            b"feecollector".as_ref(),
            nirv_center.key().as_ref(),
            authority.key().as_ref()
        ],
        bump,
        payer=authority
    )]
    pub fee_collector: Account<'info, FeeCollector>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeFeeCollector>, bump: u8) -> Result<()> {
    let fee_collector = &mut ctx.accounts.fee_collector;

    fee_collector.owner = ctx.accounts.authority.key();
    fee_collector.nirv_fee_index = PreciseNumber::ZERO;
    fee_collector.ana_fee_index = PreciseNumber::ZERO;
    fee_collector.prana_fee_index = PreciseNumber::ZERO;
    fee_collector.staked_alms = ALMS::ZERO;
    fee_collector.staged_ana = ANA::ZERO;
    fee_collector.staged_nirv = NIRV::ZERO;
    fee_collector.staged_pre_ana = ANA::ZERO;
    fee_collector.bump = bump;

    Ok(())
}
