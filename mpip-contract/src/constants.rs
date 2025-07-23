use near_sdk::{near, BorshStorageKey, CryptoHash, Gas};

pub const ONE_HUNDRED: u16 = 10_000;

/// Amount of gas for fungible token transfers.
pub const GAS_FOR_GET_VOTING_POWER: Gas = Gas::from_tgas(10);
pub const GAS_FOR_RESOLVE_VOTE: Gas = Gas::from_tgas(11);

#[derive(BorshStorageKey)]
#[near(serializers = [borsh, json])]
pub enum StorageKey {
    Mpips,
    HasVoted { hash_id: CryptoHash },
    MpipVotes,
    Voters,
    Proposers,
    Votes { hash_id: CryptoHash },
}
