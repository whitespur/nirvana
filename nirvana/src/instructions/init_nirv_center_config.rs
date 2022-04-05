use crate::utils::admin;
use crate::{
    numbers::{CoarseNumber, PreciseNumber},
    state::*,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitNirvCenterConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        init,
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump,
        payer = payer
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.payer))]
pub fn handler(ctx: Context<InitNirvCenterConfig>, config: InitNirvCenterConfigArg) -> Result<()> {
    let c = &mut ctx.accounts.config;
    let config_bump = ctx.bumps.get("config").unwrap();
    let now = Clock::get()?.unix_timestamp.unsigned_abs();

    c.mint_nirv = config.mint_nirv;
    c.mint_ana = config.mint_ana;
    c.mint_pre_ana = config.mint_prana;
    c.mint_alms = config.mint_alms;

    c.ana_fee_account = config.fee_ana;
    c.nirv_fee_account = config.fee_nirv;
    c.prana_fee_account = config.fee_prana;

    c.stake_pool_alms = config.stake_pool_alms;
    c.stake_pool_ana = config.stake_pool_ana;
    c.treasury_ana = config.escrow_ana;

    c.nirv_debt_fee = CoarseNumber::from_u64(config.nirv_debt_fee);
    c.nirv_loan_origination_fee = CoarseNumber::from_u64(config.nirv_loan_origination_fee);
    c.trana_buy_fee = CoarseNumber::from_u64(config.trana_buy_fee);
    c.instant_buy_fee = CoarseNumber::from_u64(config.instant_buy_fee);
    c.sell_fee = CoarseNumber::from_u64(config.sell_fee);
    c.unstake_fee = CoarseNumber::from_u64(config.unstake_fee);

    c.prana_reward_interval_seconds = config.prana_reward_interval_seconds;

    c.prana_reward_rate = PreciseNumber { val: 1_000_000_000 };
    c.reward_index = PreciseNumber::ZERO;

    c.time_of_last_prana_reward = now;
    c.bump = *config_bump;

    Ok(())
}

#[derive(Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct InitNirvCenterConfigArg {
    mint_ana: Pubkey,
    mint_prana: Pubkey,
    mint_alms: Pubkey,
    mint_nirv: Pubkey,

    fee_ana: Pubkey,
    fee_prana: Pubkey,
    fee_nirv: Pubkey,

    stake_pool_ana: Pubkey,
    stake_pool_alms: Pubkey,
    escrow_ana: Pubkey,

    nirv_debt_fee: u64,
    nirv_loan_origination_fee: u64,
    trana_buy_fee: u64,
    instant_buy_fee: u64,
    sell_fee: u64,
    unstake_fee: u64,

    prana_reward_interval_seconds: i64,
}
