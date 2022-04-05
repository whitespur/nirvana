use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetFeeConfig<'info> {
    pub signer: Signer<'info>,

    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        mut,
        seeds = [
            b"fee_config".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump=fee_config.bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,
}

#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.signer))]
pub fn handler(ctx: Context<SetFeeConfig>, arg: FeeConfigArg) -> Result<()> {
    let x = &mut ctx.accounts.fee_config;
    msg!(
        "Setting fee config
    unstake: {}
    sell: {}
    buy: {}
    trana: {}
    loan_origination: {}
    debt: {}
    ",
        arg.unstake_fee,
        arg.sell_fee,
        arg.instant_buy_fee,
        arg.trana_buy_fee,
        arg.nirv_loan_origination_fee,
        arg.nirv_debt_fee
    );
    x.unstake_fee.val = arg.unstake_fee;
    x.sell_fee.val = arg.sell_fee;
    x.instant_buy_fee.val = arg.instant_buy_fee;
    x.trana_buy_fee.val = arg.trana_buy_fee;
    x.nirv_loan_origination_fee.val = arg.nirv_loan_origination_fee;
    x.nirv_debt_fee.val = arg.nirv_debt_fee;

    Ok(())
}

#[derive(Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct FeeConfigArg {
    /// Fee paid in ANA for unstaking
    pub unstake_fee: u64,

    /// Fee for selling ANA (not applied when at floor)
    pub sell_fee: u64,

    /// Fee for buying ANA
    pub instant_buy_fee: u64,

    /// Fee paid in ANA for buying trANA
    pub trana_buy_fee: u64,

    /// Fee of NIRV for loan origination
    /// This is a percentage of NIRV sent to the fee account
    pub nirv_loan_origination_fee: u64,

    /// Fee paid in prANA for NIRV debt
    /// This is a percentage reduction of prANA rewards to the owner
    pub nirv_debt_fee: u64,
}
