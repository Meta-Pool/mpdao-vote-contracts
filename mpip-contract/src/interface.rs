use crate::types::{MpipId, VoterId};
use crate::*;
use near_sdk::json_types::{U128, U64};
use near_sdk::{ext_contract, AccountId};

#[allow(dead_code)]
#[ext_contract(ext_metavote)]
pub trait ExtMetaVote {
    // should call a fn to get all voting power (used or not) get_user_total_voting_power
    fn get_available_voting_power(&self, voter_id: VoterId);
    //fn get_all_locking_positions(&self, voter_id: VoterId);
    fn get_total_voting_power(&self);
    fn get_mpip_voting_power(&self, voter_id: VoterId);
    // same as get_mpip_voting_power, and locks the voter's voting power until lock_until_ms
    fn lock_and_get_mpip_voting_power(&mut self, voter_id: VoterId, lock_until_ms: U64);
}

#[allow(dead_code)]
#[ext_contract(ext_self)]
pub trait SelfMetaVote {
    fn vote_proposal_callback(
        &mut self,
        mpip_id: MpipId,
        voting_power: U128,
        voter_id: AccountId,
        vote_type: VoteType,
    );

    fn create_proposal_callback(
        &mut self,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    );
}
