use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, BorshStorageKey, CryptoHash, Gas};

pub const ONE_MPDAO: u128 = 1_000_000; // MPDAO has 6 decimals
pub const E18: u128 = 1_000_000_000_000_000_000; // to convert 6 decimals to 24 decimals
pub const TGAS: u64 = 1_000_000_000_000;

pub fn near_as_account_id() -> AccountId {
    AccountId::new_unchecked("near".into())
}

pub const SECONDS_IN_MS: u64 = 1000;
pub const MINUTES_IN_MS: u64 = 60 * SECONDS_IN_MS;

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(47 * TGAS);
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(11 * TGAS);
/// Restoring a sold position touches voter storage, so the callback needs more than a balance refund.
pub const GAS_FOR_SELL_CALLBACK: Gas = Gas(30 * TGAS);

/// Lock length at which selling a position keeps 10% of the free-mpDAO USDT price.
pub const SELL_DISCOUNT_ANCHOR_DAYS: u16 = 300;
/// Discount versus free mpDAO at `SELL_DISCOUNT_ANCHOR_DAYS`. 9000 bps = 90%.
pub const SELL_DISCOUNT_AT_ANCHOR_BPS: u16 = 9_000;
/// Short locks never keep more than 60% of the protected USDT price. 4000 bps = 40%.
pub const SELL_DISCOUNT_FLOOR_BPS: u16 = 4_000;
/// Daily observations kept for the sell price. The payout uses the median.
pub const SELL_PRICE_HISTORY_DAYS: usize = 7;
/// The USDT price used for sells may rise at most this much from one accepted day to the next.
pub const SELL_PRICE_MAX_DAILY_RISE_BPS: u16 = 1_000;
/// Hard cap on USDT paid out for sold positions per UTC day. 6 decimals, so this is $2,000.
pub const SELL_DAILY_USDT_CAP: u128 = 2_000_000_000;
pub const MS_PER_DAY: u64 = 24 * 60 * MINUTES_IN_MS;
/// `ft_transfer_call` msg that credits the USDT sell reserve without buying mpDAO.
pub const FUND_USDT_RESERVE_MSG: &str = "fund-reserve";

/// IMPORTANT 🚨: DO NOT REORDER OR REMOVE VARIANTS.
/// APPEND NEW VARIANTS ONLY AT THE END.
/// Breaking this will corrupt mainnet state.
#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
pub enum StorageKey {
    LockingPosition { hash_id: CryptoHash },
    VotePosition { hash_id: CryptoHash },
    Voters,
    Votes,
    ContractVotes { hash_id: CryptoHash },
    VoterVotes { hash_id: CryptoHash },
    Claimable,
    ClaimableStNear,
    AirdropData,
    EvmDelegates,
    EvmDelegationSignatures,
    EvmPreDelegation,
    TimestampStorage,
    TokenInfo,
    MpdaoPrices,
    ClaimableUnlockedMpdao,
    RealizedEthMirrors,
}
