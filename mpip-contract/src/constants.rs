use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{BorshStorageKey, CryptoHash, Gas};

pub const TGAS: u64 = 1_000_000_000_000;

pub const ONE_HUNDRED: u16 = 10_000;

/// Max sizes (bytes) of user supplied strings. Their storage is paid by the contract.
pub const MAX_TITLE_LEN: usize = 200;
pub const MAX_SHORT_DESCRIPTION_LEN: usize = 1_000;
pub const MAX_BODY_LEN: usize = 30_000;
pub const MAX_DATA_LEN: usize = 5_000;
pub const MAX_EXTRA_LEN: usize = 5_000;
pub const MAX_MEMO_LEN: usize = 1_000;

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_GET_VOTING_POWER: Gas = Gas(10 * TGAS);
pub const GAS_FOR_RESOLVE_VOTE: Gas = Gas(11 * TGAS);
/// One close step re-reads this many voters, then the operator continues.
pub const RECONCILE_BATCH: u64 = 10;
pub const GAS_FOR_RECONCILE_CALLBACK: Gas = Gas(30 * TGAS);

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
pub enum StorageKey {
    Mpips,
    HasVoted{ hash_id: CryptoHash },
    MpipVotes,
    Voters,
    Proposers,
    Votes { hash_id: CryptoHash },
}