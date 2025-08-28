use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use uint::construct_uint;

pub type U128String = U128;
pub type VoterId = String;
pub type Days = u16;
pub type MpDAOAmount = u128;
pub type ContractAddress = String;
pub type VotableObjId = String;

pub type EvmAddress = String;
pub type EvmSignature = String;

pub type EpochMillis = u64;
pub type PositionIndex = u64;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LockingPositionJSON {
    pub index: Option<PositionIndex>,
    pub amount: U128,
    pub locking_period: Days, // unbond_period, kept as locking_period for backwards compat
    pub voting_power: U128,
    pub unlocking_started_at: Option<EpochMillis>,
    pub is_unlocked: bool,
    pub is_unlocking: bool,
    pub is_locked: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct VotableObjectJSON {
    pub votable_contract: String,
    pub id: VotableObjId,
    pub current_votes: U128,
    pub vote_timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VotePositionJSON {
    pub votable_address: String,
    pub votable_object_id: String,
    pub voting_power: U128,
    pub vote_timestamp: u64,
}

/// Represents a single vote position to be removed as stale, used in batch operations.
///
/// We use a named struct instead of raw tuples because NEAR's JSON ABI does not reliably
/// support deserialization of tuples from external JSON callers (e.g., NEAR CLI, NEAR Blocks).
///
/// This struct enables clear, unambiguous argument passing and full compatibility with
/// frontend tools and JSON-based inputs. It also allows consuming the input by value
/// without requiring `.clone()`, keeping the batch removal process efficient.
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StaleVoteInput {
    pub voter_id: VoterId,
    pub contract_address: ContractAddress,
    pub votable_object_id: VotableObjId,
}
