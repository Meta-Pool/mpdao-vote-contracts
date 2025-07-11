use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use schemars::{
    schema::{InstanceType, Schema, SchemaObject},
    JsonSchema,
};
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

/// Wrapper that allows using U128 with JsonSchema
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct U128Json(pub U128);

impl JsonSchema for U128Json {
    fn schema_name() -> String {
        "U128".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            ..Default::default()
        })
    }
}

// U128 to U128Json conversion
impl From<U128> for U128Json {
    fn from(value: U128) -> Self {
        U128Json(value)
    }
}

// u128 primitive to U128Json conversion
use near_sdk::serde_json::Value;
impl From<u128> for U128Json {
    fn from(value: u128) -> Self {
        U128Json(U128::from(value))
    }
}

// Extract u128 primite value from U128Json
impl From<U128Json> for u128 {
    fn from(wrapper: U128Json) -> Self {
        wrapper.0 .0
    }
}

// Directly extract `U128` from `U128Json`
impl From<U128Json> for U128 {
    fn from(wrapper: U128Json) -> Self {
        wrapper.0
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct LockingPositionJSON {
    pub index: Option<PositionIndex>,
    pub amount: U128Json,
    pub locking_period: Days,
    pub voting_power: U128Json,
    pub unlocking_started_at: Option<EpochMillis>,
    pub is_unlocked: bool,
    pub is_unlocking: bool,
    pub is_locked: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct VotableObjectJSON {
    pub votable_contract: String,
    pub id: VotableObjId,
    pub current_votes: U128Json,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct VotePositionJSON {
    pub votable_address: String,
    pub votable_object_id: String,
    pub voting_power: U128Json,
}
