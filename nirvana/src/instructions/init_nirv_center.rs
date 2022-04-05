use crate::{
    numbers::{CoarseNumber, PreciseNumber},
    state::*,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitNirvCenter<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(init, payer=payer)]
    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        init,
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump,
        payer = payer
    )]
    pub price_curve: Account<'info, PriceFieldV2>,

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

pub fn handler(ctx: Context<InitNirvCenter>, config: NirvCenterInitConfig) -> Result<()> {
    let config_bump = ctx.bumps.get("config").unwrap();
    let price_curve_bump = ctx.bumps.get("price_curve").unwrap();

    let address = ctx.accounts.nirv_center.key();
    let (authority, signer_authority_bump) =
        Pubkey::find_program_address(&[address.as_ref()], ctx.program_id);

    let nc = &mut ctx.accounts.nirv_center;
    let c = &mut ctx.accounts.config;
    nc.policy_owner = ctx.accounts.payer.to_account_info().key();

    nc.signer_authority = authority;
    msg!("Authority: {}", authority);

    nc.signer_authority_seed = address;
    nc.signer_authority_bump = [signer_authority_bump];
    nc.config = c.key();

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

    c.price_curve = ctx.accounts.price_curve.key();
    ctx.accounts.price_curve.nirv_center = address;
    ctx.accounts.price_curve.bump = *price_curve_bump;

    c.prana_reward_rate = PreciseNumber { val: 1_000_000_000 };
    c.reward_index = PreciseNumber::ZERO;

    let now = Clock::get()?.unix_timestamp.unsigned_abs();
    c.time_of_last_prana_reward = now;
    c.bump = *config_bump;

    Ok(())
}

#[derive(Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct NirvCenterInitConfig {
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
