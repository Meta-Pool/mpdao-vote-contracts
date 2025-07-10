use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
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

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
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
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VotePositionJSON {
    pub votable_address: String,
    pub votable_object_id: String,
    pub voting_power: U128,
}
