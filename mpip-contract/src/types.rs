use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use uint::construct_uint;

pub type MpipId = u32;

pub type VoterId = AccountId;
pub type VotingPower = u128;
pub type ContractAddress = AccountId;
pub type EpochMillis = u64;
pub type BasisPoints = u16;
pub type PositionIndex = u64;
use near_sdk::json_types::U128;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LockingPositionJSON {
    pub index: Option<PositionIndex>,
    pub amount: U128,
    pub locking_period: u16,
    pub voting_power: U128,
    pub unlocking_started_at: Option<EpochMillis>,
    pub is_unlocked: bool,
    pub is_unlocking: bool,
    pub is_locked: bool,
}
