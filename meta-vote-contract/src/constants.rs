use near_sdk::{BorshStorageKey, CryptoHash, Gas, NearToken};

pub const ONE_MPDAO: u128 = 1_000_000; // MPDAO has 6 decimals
pub const E18: u128 = 1_000_000_000_000_000_000; // to convert 6 decimals to 24 decimals
pub const ONE_NEAR: NearToken = NearToken::from_near(1).as_yoctonear(); // 1 NEAR in yoctoNEAR

/// Amount of gas for fungible token transfers.
///

/// ARF
/// pub const GAS_FOR_FT_TRANSFER: Gas = Gas(47 * TGAS);
/// pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(11 * TGAS);
/// This is no longer valid in near-sdk v5 because Gas is no longer a tuple struct.
/// The correct way is to use:

pub const GAS_FOR_FT_TRANSFER: Gas = Gas::from_tgas(47);
pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas::from_tgas(11);

//ARF
//In near-sdk 5.x, the derive macro #[derive(BorshStorageKey)] needs to know which crate borsh and near_sdk use, because of the internal decoupling they did in the new version.

#[derive(BorshStorageKey)]
#[near(serializers = [borsh, json])]
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
}
