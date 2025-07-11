use crate::*;
use uint::construct_uint;

pub type VoterId = String;
pub type Days = u16;
pub type MpDAOAmount = u128;
pub type ContractAddress = String;
pub type VotableObjId = String;
pub type EvmAddress = String;
pub type EvmSignature = String;
pub type EpochMillis = u64;
pub type PositionIndex = u32;

construct_uint! {
    /// 256-bit unsigned integer
    pub struct U256(4);
}
#[derive(Debug)]
#[near(serializers = [json])]
pub struct LockingPositionJSON {
    pub index: Option<PositionIndex>,
    pub amount: U128,
    pub locking_period: Days,
    pub voting_power: U128,
    pub unlocking_started_at: Option<EpochMillis>,
    pub is_unlocked: bool,
    pub is_unlocking: bool,
    pub is_locked: bool,
}

#[near(serializers = [json])]
pub struct VotableObjectJSON {
    pub votable_contract: String,
    pub id: VotableObjId,
    pub current_votes: U128,
}

#[derive(Debug)]
#[near(serializers = [json])]
pub struct VotePositionJSON {
    pub votable_address: String,
    pub votable_object_id: String,
    pub voting_power: U128,
}
