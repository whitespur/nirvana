use crate::{
    errors::ErrorCode,
    numbers::{PreciseNumber, ALMS, ANA},
    price_math::{calc_total_cost_for_amount, PriceCalculator},
    state::*,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
use rust_decimal::prelude::*;

#[derive(Accounts)]
pub struct Swap<'info> {
    pub signer: Signer<'info>,

    pub nirv_center: Box<Account<'info, NirvCenter>>,

    #[account(
        constraint = nirv_center_authority.key() == nirv_center.signer_authority
    )]
    /// CHECK - Just a pubkey
    pub nirv_center_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            b"config_v3".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, NirvCenterConfigV3>>,

    #[account(
        mut,
        constraint = mint_ana.key() == config.mint_ana,
    )]
    pub mint_ana: Box<Account<'info, Mint>>,

    /// CHECK: User must sign to move funds
    #[account(
        mut,
        constraint = user_ana_token_account.mint == mint_ana.key()
    )]
    pub user_ana_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: User must sign to move funds
    #[account(
        mut,
        constraint = user_money_token_account.mint == money_market.mint
    )]
    pub user_money_token_account: Box<Account<'info, TokenAccount>>,

    /// Token account for treasury money
    #[account(
        mut,
        constraint = treasury_token_account.key() == money_market.token_account,
        constraint = treasury_token_account.mint == money_market.mint,
    )]
    pub treasury_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        has_one = nirv_center,
        seeds = [
            b"mm1".as_ref(),
            treasury_token_account.mint.as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = money_market.bump,
    )]
    pub money_market: Box<Account<'info, MoneyMarket>>,

    #[account(
        mut,
        has_one = nirv_center,
        seeds = [
            b"pf2".as_ref(),
            nirv_center.key().as_ref()
        ],
        bump = price_calculator.bump
    )]
    pub price_calculator: Box<Account<'info, PriceFieldV2>>,

    /// CHECK: Key is matched to NirvCenter
    #[account(
        mut,
        constraint = ana_fee_account.key() == config.ana_fee_account,
        constraint = ana_fee_account.mint == mint_ana.key()
    )]
    pub ana_fee_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = history.authority == signer.key(),
        constraint = history.nirv_center == nirv_center.key(),
        seeds = [
            b"history".as_ref(),
            nirv_center.key().as_ref(),
            signer.key().as_ref()
        ],
        bump = history.bump,
    )]
    pub history: Box<Account<'info, History>>,

    #[account(
        mut,
        seeds = [
            b"globalhistory".as_ref(),
            nirv_center.key().as_ref(),
        ],
        bump = global_history.bump,
    )]
    pub global_history: Box<Account<'info, GlobalHistory>>,

    #[account(
        constraint = stake_pool_alms.key() == config.stake_pool_alms,
        constraint = stake_pool_alms.mint == config.mint_alms
    )]
    pub stake_pool_alms: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<Swap>,
    amount_of_ana: ANA,
    expected_total_cost: u64,
    clock_override: u64,
    is_buy: bool,
) -> Result<()> {
    msg!("Is buy: {}", is_buy);
    msg!("ANA: {}", amount_of_ana.val);
    msg!("Expected cost: {}", expected_total_cost);

    ctx.accounts.can_swap()?;

    let total_alms_staked = ALMS::from_u64(ctx.accounts.stake_pool_alms.amount);

    // ana_less_fees is how much ANA is bought, or sold.
    // Fees are either minted to the fee account, or transfered from the seller
    let (ana_less_fees, fees) =
        ctx.accounts
            .config
            .collect_ana_swap_fee(is_buy, amount_of_ana, total_alms_staked);

    let now = ctx.accounts.now(clock_override);
    let is_bootstrapping = ctx.accounts.config.is_bootstrapping(now);

    // Get total cost for amount of ANA
    // if buying, calculate cost based on ANA minted
    // if selling, calculate cost based on ANA - fees
    let total_cost_d = if is_buy {
        ctx.accounts.total_cost(now, true, amount_of_ana)
    } else {
        ctx.accounts.total_cost(now, false, ana_less_fees)
    };

    msg!("Total cost: {}", total_cost_d);

    // update history
    let round_dollars = total_cost_d
        .round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_u64()
        .unwrap();

    ctx.accounts
        .update_history(round_dollars, is_buy, is_bootstrapping, amount_of_ana);

    // Convert to integer
    let total_cost = total_cost_d.mantissa().to_u64().unwrap();

    // Update the new price of ANA
    // NOTE: This does not include the bootstrapping offset
    let new_ana_supply = ctx.accounts.new_ana_supply(is_buy, amount_of_ana);
    let price_for_unit = ctx
        .accounts
        .price_calculator
        .price_for_supply(new_ana_supply.into());
    let price_for_unit = PreciseNumber::from_decimal(price_for_unit);
    ctx.accounts.config.current_ana_price_usd = price_for_unit;

    if is_buy {
        ctx.accounts
            .buy(expected_total_cost, total_cost, ana_less_fees, fees)?;
    } else {
        // selling ANA
        ctx.accounts
            .sell(expected_total_cost, total_cost, ana_less_fees, fees)?;

        ctx.accounts
            .price_calculator
            .reset_slippage_start_point_if_needed(ANA::from_u64(new_ana_supply).into());
    }

    Ok(())
}

impl<'info> Swap<'info> {
    fn can_swap(&self) -> Result<()> {
        if !self.money_market.enabled {
            return Err(error!(ErrorCode::MoneyMarketNotEnabled));
        }

        if !self.money_market.for_amm {
            return Err(error!(ErrorCode::MoneyMarketNotEnabled));
        }

        Ok(())
    }

    fn update_history(
        &mut self,
        round_dollars: u64,
        is_buy: bool,
        is_bootstrapping: bool,
        amount_of_ana: ANA,
    ) {
        if is_buy {
            self.global_history.buy_ana(
                round_dollars,
                amount_of_ana,
                ANA::from_u64(self.mint_ana.supply),
            );

            if is_bootstrapping {
                self.history
                    .buy_during_bootstrap(round_dollars, amount_of_ana);
            } else {
                self.history.buy(round_dollars);
            }
        } else {
            self.global_history.sell_ana(round_dollars, amount_of_ana);

            if is_bootstrapping {
                self.history
                    .sell_during_bootstrap(round_dollars, amount_of_ana);
            } else {
                self.history.sell(round_dollars);
            }
        }
    }

    fn total_cost(&self, now: u64, is_buy: bool, amount_of_ana: ANA) -> Decimal {
        let bootstrap_params = self.config.to_bootstrap_params();
        let money_scale = self.money_market.decimals;
        let current_supply = self.mint_ana.supply;
        let mut price_offset = bootstrap_params.current_offset(now);
        price_offset.rescale(money_scale.into());
        msg!("Price offset {}", price_offset);

        // If buying, round price up
        // If selling, round price down
        let rounding_strategy = if is_buy {
            RoundingStrategy::AwayFromZero
        } else {
            RoundingStrategy::ToZero
        };

        // Total cost for amount of ANA
        let mut total_cost_d = calc_total_cost_for_amount(
            ANA::from_u64(current_supply),
            amount_of_ana,
            self.money_market.risk_free_value_coefficient,
            &self.price_calculator.to_owned().into_inner(),
            is_buy,
            price_offset,
        )
        .round_dp_with_strategy(money_scale.into(), rounding_strategy);

        // Scale the precision of the underyling token
        total_cost_d.rescale(money_scale.into());

        total_cost_d
    }

    fn new_ana_supply(&self, is_buy: bool, amount_of_ana: ANA) -> u64 {
        let current_supply = self.mint_ana.supply;
        if is_buy {
            current_supply.checked_add(amount_of_ana.into()).unwrap()
        } else {
            current_supply.checked_sub(amount_of_ana.into()).unwrap()
        }
    }
    /// Allow admins to override the clock
    /// used when testing
    fn now(&self, clock_override: u64) -> u64 {
        if self.nirv_center.policy_owner == self.signer.key() && clock_override != 0 {
            clock_override
        } else {
            let clock = Clock::get().unwrap();
            clock.unix_timestamp.unsigned_abs()
        }
    }

    fn sell(&self, expected_cost: u64, cost: u64, ana_less_fees: ANA, fees: ANA) -> Result<()> {
        if expected_cost > cost {
            return Err(ErrorCode::SlippageExceededForSell.into());
        }

        // Burn ANA
        token::burn(
            self.burn_ana_context()
                .with_signer(&[&self.nirv_center.authority_seeds()]),
            ana_less_fees.into(),
        )?;

        // Transfer fee to account
        token::transfer(self.transfer_fee_context(), fees.into())?;

        // Transfer payment
        token::transfer(
            self.refund_context()
                .with_signer(&[&self.nirv_center.authority_seeds()]),
            cost,
        )?;

        Ok(())
    }

    fn buy(&self, expected_cost: u64, cost: u64, ana_less_fees: ANA, fees: ANA) -> Result<()> {
        if expected_cost < cost {
            return Err(ErrorCode::SlippageExceededForBuy.into());
        }
        // transfer payment
        token::transfer(self.pay_context(), cost)?;

        // give ANA
        token::mint_to(
            self.mint_to_user_context()
                .with_signer(&[&self.nirv_center.authority_seeds()]),
            ana_less_fees.into(),
        )?;

        // collect fee
        token::mint_to(
            self.mint_fee_context()
                .with_signer(&[&self.nirv_center.authority_seeds()]),
            fees.into(),
        )?;

        Ok(())
    }

    fn mint_to_user_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.user_ana_token_account.to_account_info(),
                mint: self.mint_ana.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }

    fn mint_fee_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                to: self.ana_fee_account.to_account_info(),
                mint: self.mint_ana.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }

    fn transfer_fee_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                to: self.ana_fee_account.to_account_info(),
                from: self.user_ana_token_account.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

    fn pay_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.user_money_token_account.to_account_info(),
                to: self.treasury_token_account.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }

    fn refund_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.treasury_token_account.to_account_info(),
                to: self.user_money_token_account.to_account_info(),
                authority: self.nirv_center_authority.to_account_info(),
            },
        )
    }

    fn burn_ana_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.mint_ana.to_account_info(),
                to: self.user_ana_token_account.to_account_info(),
                authority: self.signer.to_account_info(),
            },
        )
    }
}
