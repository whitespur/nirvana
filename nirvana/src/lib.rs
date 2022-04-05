use crate::utils::admin;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
mod bond_math;
mod bootstrap_math;
mod decimal;
mod errors;
mod instructions;
mod numbers;
mod price_math;
mod state;
mod utils;

use instructions::*;
use numbers::{CoarseNumber, PreciseNumber, ANA};
use state::*;
use utils::is_debug;
declare_id!("nirkXSE28jCQoK8SmKWqxXz3L9vbSRGKS24iYJj8Aeo");
#[program]
pub mod nirvana {
    use anchor_spl::token::{self, MintTo};

    use super::*;

    pub fn init_nirv_center(
        ctx: Context<InitNirvCenter>,
        config: NirvCenterInitConfig,
    ) -> Result<()> {
        instructions::init_nirv_center::handler(ctx, config)
    }

    pub fn init_nirv_center_config(
        ctx: Context<InitNirvCenterConfig>,
        config: InitNirvCenterConfigArg,
    ) -> Result<()> {
        instructions::init_nirv_center_config::handler(ctx, config)
    }

    /// Start the bootstrapping pool
    pub fn start_bootstrapping(
        ctx: Context<StartBootstrapping>,
        start_time: u64,
        duration: u64,
        price_offset: PreciseNumber,
    ) -> Result<()> {
        instructions::start_bootstrapping::handler(ctx, start_time, duration, price_offset)
    }

    pub fn set_price_field_params(
        ctx: Context<SetPriceFieldParams>,
        ramp_start: ANA,
        ramp_width: ANA,
        ramp_height: PreciseNumber,
        main_slope: PreciseNumber,
        floor_price: PreciseNumber,
    ) -> Result<()> {
        instructions::set_price_field_params::handler(
            ctx,
            ramp_start,
            ramp_width,
            ramp_height,
            main_slope,
            floor_price,
        )
    }

    /// Create a new money market
    pub fn init_money_market(
        ctx: Context<InitMoneyMarket>,
        bump: u8,
        rfv_coefficient: CoarseNumber,
        for_amm: bool,
        for_prana: bool,
    ) -> Result<()> {
        instructions::init_money_market::handler(ctx, bump, rfv_coefficient, for_amm, for_prana)
    }

    /// Swap ANA from the AMM
    pub fn swap(
        ctx: Context<Swap>,
        amount_of_ana: ANA,
        expected_total_price: u64,
        is_buy: bool,
        clock_override: u64,
    ) -> Result<()> {
        instructions::swap::handler(
            ctx,
            amount_of_ana,
            expected_total_price,
            clock_override,
            is_buy,
        )
    }

    /// Set whether the treasury account can be used for ANA buyback
    pub fn set_treasury_account_is_for_amm(
        ctx: Context<SetTreasuryAccountForAmm>,
        is_for_amm: bool,
    ) -> Result<()> {
        instructions::set_treasury_account_for_amm::handler(ctx, is_for_amm)
    }

    /// Set whether the treasury account can be used for prana
    pub fn set_treasury_account_is_for_prana(
        ctx: Context<SetTreasuryAccountForPrana>,
        is_for_prana: bool,
    ) -> Result<()> {
        instructions::set_treasury_account_for_prana::handler(ctx, is_for_prana)
    }

    pub fn create_trana_meta(
        ctx: Context<CreateTranaMeta>,
        vesting_length_seconds: u64,
        sensitivity: PreciseNumber,
        max_discount_ratio: PreciseNumber,
        bond_bump: u8,
    ) -> Result<()> {
        instructions::create_trana_meta::handler(
            ctx,
            vesting_length_seconds,
            sensitivity,
            max_discount_ratio,
            bond_bump,
        )
    }

    pub fn stake_ana(ctx: Context<StakeAna>, amount: u64) -> Result<()> {
        instructions::stake_ana::handler(ctx, amount)
    }

    pub fn unstake_ana(ctx: Context<UnstakeAna>, amount: u64) -> Result<()> {
        instructions::unstake_ana::handler(ctx, amount)
    }

    pub fn stake_alms(ctx: Context<StakeAlms>, amount: u64) -> Result<()> {
        instructions::stake_alms::handler(ctx, amount)
    }

    pub fn unstake_alms(ctx: Context<UnstakeAlms>, amount: u64) -> Result<()> {
        instructions::unstake_alms::handler(ctx, amount)
    }

    pub fn mint_pre_ana(ctx: Context<MintPreAna>, amount: u64) -> Result<()> {
        instructions::mint_pre_ana::handler(ctx, amount)
    }

    #[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
    #[access_control(is_debug(&ctx.accounts.nirv_center))]
    pub fn mint_nirv(ctx: Context<MintNirv>, amount: u64) -> Result<()> {
        instructions::mint_nirv::handler(ctx, amount)
    }

    pub fn mint_alms(ctx: Context<MintAlms>, amount: u64) -> Result<()> {
        instructions::mint_alms::handler(ctx, amount)
    }

    pub fn borrow_nirv(ctx: Context<BorrowNirv>, amount: u64) -> Result<()> {
        instructions::borrow_nirv::handler(ctx, amount)
    }

    pub fn repay_nirv(ctx: Context<RepayNirv>, amount: u64) -> Result<()> {
        instructions::repay_nirv::handler(ctx, amount)
    }

    pub fn realize_pre_ana(ctx: Context<RealizePreAna>, amount: u64) -> Result<()> {
        instructions::realize_pre_ana::handler(ctx, amount)
    }

    pub fn purchase_trana(
        ctx: Context<PurchaseTrana>,
        payment_u64: u64,
        max_offered_price_u64: u64,
    ) -> Result<()> {
        instructions::purchase_trana::handler(ctx, payment_u64, max_offered_price_u64)
    }

    /// Creates an empty bond account for the user
    pub fn initialize_user_trana_contract_account(
        ctx: Context<InitializeUserTranaContractAccount>,
        bump: u8,
        bond_index: u8,
    ) -> Result<()> {
        instructions::initialize_user_trana_contract_account::handler(ctx, bump, bond_index)
    }

    pub fn redeem_trana(ctx: Context<RedeemTrana>) -> Result<()> {
        instructions::redeem_trana::handler(ctx)
    }

    /// Burn ANA in exchange for reserve tokens
    pub fn buyback_ana(ctx: Context<BuybackAna>, amount: u64) -> Result<()> {
        instructions::buyback_ana::handler(ctx, amount)
    }

    pub fn set_trana_max_discount_ratio(
        ctx: Context<SetTranaMaxDiscount>,
        max_discount_ratio: PreciseNumber,
    ) -> Result<()> {
        instructions::set_trana_max_discount::handler(ctx, max_discount_ratio)
    }

    pub fn drop_rewards(ctx: Context<Reward>) -> Result<()> {
        instructions::reward::handler(ctx)
    }

    pub fn set_reward_rate(ctx: Context<SetRewardRate>, reward_rate: PreciseNumber) -> Result<()> {
        instructions::set_reward_rate::handler(ctx, reward_rate)
    }

    pub fn set_mint_ana(ctx: Context<SetMintAna>) -> Result<()> {
        instructions::set_mint_ana::handler(ctx)
    }

    pub fn set_mint_pre_ana(ctx: Context<SetMintPreAna>) -> Result<()> {
        instructions::set_mint_pre_ana::handler(ctx)
    }

    pub fn set_nirv_loan_origination_fee(
        ctx: Context<SetNirvLoanOriginationFee>,
        fee: CoarseNumber,
    ) -> Result<()> {
        instructions::set_nirv_loan_origination_fee::handler(ctx, fee)
    }

    pub fn set_nirv_debt_fee(ctx: Context<SetNirvDebtFee>, fee: CoarseNumber) -> Result<()> {
        instructions::set_nirv_debt_fee::handler(ctx, fee)
    }

    pub fn set_instant_buy_fee(ctx: Context<SetInstantBuyFee>, fee: CoarseNumber) -> Result<()> {
        instructions::set_instant_buy_fee::handler(ctx, fee)
    }

    pub fn set_sell_fee(ctx: Context<SetSellFee>, fee: CoarseNumber) -> Result<()> {
        instructions::set_sell_fee::handler(ctx, fee)
    }

    pub fn set_trana_buy_fee(ctx: Context<SetTranaBuyFee>, fee: CoarseNumber) -> Result<()> {
        instructions::set_trana_buy_fee::handler(ctx, fee)
    }

    pub fn set_unstake_fee(ctx: Context<SetUnstakeFee>, fee: CoarseNumber) -> Result<()> {
        instructions::set_unstake_fee::handler(ctx, fee)
    }

    pub fn set_bond_bcv(ctx: Context<SetBondBcv>, sensitivity: PreciseNumber) -> Result<()> {
        instructions::set_trana_sensitivity::handler(ctx, sensitivity)
    }

    pub fn set_treasury_account_rfv(
        ctx: Context<SetTreasuryAccountRfv>,
        rfv: CoarseNumber,
    ) -> Result<()> {
        instructions::set_treasury_account_rfv::handler(ctx, rfv)
    }

    pub fn set_trana_enabled(ctx: Context<SetTranaEnabled>, is_enabled: bool) -> Result<()> {
        instructions::set_trana_enabled::handler(ctx, is_enabled)
    }

    pub fn reward(ctx: Context<Reward>) -> Result<()> {
        instructions::reward::handler(ctx)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        instructions::claim_reward::handler(ctx)
    }

    pub fn initialize_user_reward_index(
        ctx: Context<InitializeUserReward>,
        bump: u8,
    ) -> Result<()> {
        instructions::initialize_user_reward::handler(ctx, bump)
    }

    pub fn initialize_fee_collector(ctx: Context<InitializeFeeCollector>, bump: u8) -> Result<()> {
        instructions::initialize_fee_collector::handler(ctx, bump)
    }

    pub fn set_debug_mode(ctx: Context<SetDebugMode>, debug_mode: bool) -> Result<()> {
        instructions::set_debug_mode::handler(ctx, debug_mode)
    }

    /// Public method for issuing rewards
    pub fn reward_by_time(ctx: Context<RewardByTime>) -> Result<()> {
        instructions::reward_by_time::handler(ctx)
    }

    /// Commitments are for the pre-commit to the bootstrap event
    pub fn init_commitment(ctx: Context<InitCommitment>, bump: u8) -> Result<()> {
        instructions::init_commitment::handler(ctx, bump)
    }

    /// Set commitment level for the pre-commit to the boostrap event
    pub fn set_commitment(ctx: Context<SetCommitment>, target_spend_usd: u64) -> Result<()> {
        instructions::set_commitment::handler(ctx, target_spend_usd)
    }

    /// The commitment meta holds the pre-commit info about the pre-bootstrap event
    pub fn init_commitment_meta(
        ctx: Context<InitCommitmentMeta>,
        bump: u8,
        start_time: i64,
        early_bird_end: i64,
        end: i64,
    ) -> Result<()> {
        instructions::init_commitment_meta::handler(ctx, bump, start_time, early_bird_end, end)
    }

    pub fn set_commitment_meta(
        ctx: Context<SetCommitmentMeta>,
        start_time: i64,
        early_bird_end: i64,
        end: i64,
    ) -> Result<()> {
        instructions::set_commitment_meta::handler(ctx, start_time, early_bird_end, end)
    }

    /// History is used to track a user's state
    pub fn init_history(ctx: Context<InitHistory>, bump: u8) -> Result<()> {
        instructions::init_history::handler(ctx, bump)
    }

    /// Global History for the whole state
    pub fn init_global_history(ctx: Context<InitGlobalHistory>) -> Result<()> {
        instructions::init_global_history::handler(ctx)
    }

    /// Claim the rewards for the bootstrap event
    pub fn claim_lbp_rewards(ctx: Context<ClaimLbpRewards>) -> Result<()> {
        instructions::claim_lbp_rewards::handler(ctx)
    }

    pub fn close_config_v2(ctx: Context<CloseConfigV2>) -> Result<()> {
        instructions::close_config_v2::handler(ctx)
    }

    pub fn init_fee_config(ctx: Context<InitFeeConfig>) -> Result<()> {
        instructions::init_fee_config::handler(ctx)
    }

    pub fn set_fee_config(ctx: Context<SetFeeConfig>, arg: FeeConfigArg) -> Result<()> {
        instructions::set_fee_config::handler(ctx, arg)
    }

    pub fn init_price_curve_v2(ctx: Context<InitPriceCurveV2>) -> Result<()> {
        instructions::init_price_curve_v2::handler(ctx)
    }

    #[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.payer))]
    #[access_control(is_debug(&ctx.accounts.nirv_center))]
    pub fn mint_ana(ctx: Context<MintAna>, amount: u64) -> Result<()> {
        let accounts_recv = MintTo {
            mint: ctx.accounts.mint_ana.to_account_info(),
            to: ctx.accounts.token_ana.to_account_info(),
            authority: ctx.accounts.nirv_center_authority.to_account_info(),
        };

        let token_program = ctx.accounts.token_program.to_account_info();
        let ctx_mint = CpiContext::new(token_program, accounts_recv);

        token::mint_to(
            ctx_mint.with_signer(&[&ctx.accounts.nirv_center.authority_seeds()]),
            amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct MintAna<'info> {
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mint_ana: Account<'info, Mint>,

    #[account(
        mut,
        constraint = token_ana.mint == mint_ana.key(),
    )]
    pub token_ana: Account<'info, TokenAccount>,

    #[account(
        constraint = config.mint_ana == mint_ana.key(),
    )]
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
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}
