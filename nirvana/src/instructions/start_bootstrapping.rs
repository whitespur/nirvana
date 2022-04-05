use crate::numbers::PreciseNumber;
use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct StartBootstrapping<'info> {
    pub authority: Signer<'info>,

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
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
pub fn handler(
    ctx: Context<StartBootstrapping>,
    start_time: u64,
    duration: u64,
    price_offset: PreciseNumber,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    let clock = Clock::get()?;
    let now = if start_time == 0 {
        clock.unix_timestamp.unsigned_abs()
    } else {
        start_time
    };
    msg!(
        "Starting bootstrap at {}.  Duration: {}. Price offset: {}",
        now,
        duration,
        price_offset.val
    );
    config.start_bootstrap(now, duration, price_offset);

    Ok(())
}
