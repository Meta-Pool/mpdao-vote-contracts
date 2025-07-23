use crate::types::{MpipId, VoterId};
use near_sdk::json_types::U128;
use near_sdk::{near, AccountId};

/// Vote Types
#[derive(Debug, Clone, PartialEq)]
#[near(serializers = [borsh, json])]
pub enum VoteType {
    Against,
    For,
    Abstain,
}

#[derive(Debug)]
#[near(serializers = [json])]
pub struct VoteJson {
    pub mpip_id: MpipId,

    pub voter_id: AccountId,
    pub vote_type: VoteType,

    pub voting_power: U128,
    pub memo: String,
    // pub already_withdrawn: bool
}

#[derive(PartialEq, Clone)]
#[near(serializers = [borsh, json])]
pub struct Vote {
    pub mpip_id: MpipId,
    pub vote_type: VoteType,
    pub voting_power: u128,
    pub memo: String,
    // pub already_withdrawn: bool
}

impl Vote {
    pub(crate) fn new(
        mpip_id: MpipId,
        _vote_type: VoteType,
        _voting_power: u128,
        _memo: String,
    ) -> Self {
        Vote {
            mpip_id,
            vote_type: _vote_type,
            voting_power: _voting_power,
            memo: _memo,
            // already_withdrawn: false
        }
    }
    pub(crate) fn to_json(&self, voter_id: VoterId) -> VoteJson {
        VoteJson {
            mpip_id: self.mpip_id.clone(),
            voter_id: voter_id.clone(),
            vote_type: self.vote_type.clone(),
            voting_power: U128::from(self.voting_power),
            memo: self.memo.clone(),
            // already_withdrawn: self.already_withdrawn.clone()
        }
    }
}

// impl Deref for Vote {
//     type Target = MpipId;

//     fn deref(&self) -> &Self::Target {
//         &self.mpip_id
//     }
// }
