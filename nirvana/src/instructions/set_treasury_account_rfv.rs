use crate::numbers::CoarseNumber;
use crate::state::*;
use crate::utils::admin;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetTreasuryAccountRfv<'info> {
    pub nirv_center: Account<'info, NirvCenter>,

    #[account(
        mut,
        constraint = money_market.nirv_center == nirv_center.key())
    ]
    pub money_market: Account<'info, MoneyMarket>,
    pub authority: Signer<'info>,
}
#[access_control(admin(&ctx.accounts.nirv_center, &ctx.accounts.authority))]
pub fn handler(ctx: Context<SetTreasuryAccountRfv>, rfv: CoarseNumber) -> Result<()> {
    let money_market = &mut ctx.accounts.money_market;
    money_market.risk_free_value_coefficient = rfv;
    Ok(())
}
