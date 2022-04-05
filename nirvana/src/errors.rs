use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Commitment period not begun")]
    CommitmentPeriodNotBegun,

    #[msg("Unauthorized bond creation for Nirv Center")]
    UnauthorizedBondCreation,

    #[msg("Insufficient ANA to stake")]
    InsufficientANAToStake,

    #[msg("Insufficient funds to buy bond")]
    InsufficientFundsToBuyBond,

    #[msg("Insufficient Staked ANA to unstake")]
    InsufficientStakedANAToUnstake,

    #[msg("Submitted price too low for bond")]
    BondPriceNotMet,

    #[msg("Unauthorized bond redemption")]
    UnauthorizedBondRedemption,

    #[msg("Cannot use active bond account")]
    UnavailableBondAccount,

    #[msg("Unauthorized bond account")]
    UnauthorizedBondAccount,

    #[msg("Attempting to redeem unused bond")]
    RedeemUnusedBond,

    #[msg("Insufficient ANA in user account for buyback")]
    InsufficientAnaToBuyback,

    #[msg("Insufficient underlying tokens in the treasury for buyback")]
    InsufficientTreasuryUToWithdraw,

    #[msg("Insufficient prANA in user account to realize")]
    InsufficientPreAnaToRealize,

    #[msg("Insufficient U token in user account to realize preANA")]
    InsufficientUTokenToRealizePreAna,

    #[msg("Decimal scale is different")]
    DifferentScale,

    #[msg("Insufficient balance to drop rewards")]
    InsufficientRewardBalance,

    #[msg("Unauthorized access to nirv center")]
    Unauthorized,

    #[msg("Unauthorized bond_meta access")]
    UnauthorizedBondMetaAccess,

    #[msg("Token account not usable for prANA realization")]
    TokenAccountNotForPrana,

    #[msg("Token account not found in treasury")]
    TokenAccountNotFoundInTreasury,

    #[msg("Cannot unstake without claiming rewards")]
    UnstakingWithUnclaimedRewards,

    #[msg("Cannot stake without claiming rewards")]
    StakingWithUnclaimedRewards,

    #[msg("Unauthorized ANA token account access")]
    UnauthorizedAnaAccountAccess,

    #[msg("Must be in debug mode")]
    DebugRequired,

    #[msg("Reward must wait longer")]
    RewardTooSoon,

    #[msg("Insufficient payment for ANA purchase")]
    SlippageExceededForBuy,

    #[msg("Slippage exceeded for ANA sale")]
    SlippageExceededForSell,

    #[msg("Commitment target not met")]
    CommitmentTargetNotMet,

    #[msg("Unable to claim reward on dead commitment")]
    CommitmentAlreadyClaimed,

    #[msg("Bootstrapping event not ended")]
    BootstrappingNotEnded,

    #[msg("Money market not enabled")]
    MoneyMarketNotEnabled,

    #[msg("Money market not for AMM")]
    MoneyMarketNotForAMM,

    #[msg("Insufficient staked ANA to borrow NIRV")]
    InsufficientStakedANAToBorrowNIRV,

    #[msg("Insufficient staked ANA to back borrowed NIRV")]
    InsufficientStakedANAToBackBorrowedNIRV,

    #[msg("Repay NIRV amount more than amount held")]
    RepayNIRVMoreThanHeld,

    #[msg("Repay NIRV amount more than amount borrowed")]
    RepayNIRVMoreThanBorrowed,

    #[msg("Insufficient Staked ALMS to unstake")]
    InsufficientStakedALMSToUnstake,

    #[msg("Borrowed NIRV amount is larger than borrow limit")]
    BorrowedAmountLargerThanLimit,

    #[msg("Invalid NIRV borrow utilization")]
    InvalidBorrowUtilization,
}
