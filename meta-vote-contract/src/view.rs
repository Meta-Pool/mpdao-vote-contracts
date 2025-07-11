use crate::types::U128Json;
use crate::types::*;
use crate::{voter::VoterJSON, MetaVoteContract, MetaVoteContractExt, StorageKey};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    collections::UnorderedMap,
    json_types::{U128, U64},
    near_bindgen,
};
use schemars::JsonSchema;
type U128String = U128;

/**********************/
/*   View functions   */
/**********************/

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractInfoJson {
    pub owner_id: String,
    pub operator_id: String,
    pub voter_count: u64,
    pub min_unbond_period: u16,
    pub min_claim_and_bond_days: u16,
    pub max_unbond_period: u16,
    pub min_deposit_amount: U128Json,
    pub max_locking_positions: u8,
    pub max_voting_positions: u8,
    pub mpdao_token_contract_address: String,
    pub stnear_token_contract_address: String,
    pub registration_cost: U128Json,
    pub prev_governance_contract: String,
    pub accumulated_mpdao_distributed_for_claims: U128Json,
    pub total_unclaimed_mpdao: U128Json,
    pub accum_distributed_stnear_for_claims: U128Json,
    pub total_unclaimed_stnear: U128Json,
    pub evm_delegates_count: u64,
    pub mpdao_per_near_e24: U128Json,
    pub mpdao_avail_to_sell: U128Json,
}

impl MetaVoteContract {
    pub fn get_owner_id(&self) -> String {
        self.owner_id.to_string()
    }
    pub fn get_operator_id(&self) -> String {
        self.operator_id.to_string()
    }

    pub fn get_contract_info(&self) -> ContractInfoJson {
        ContractInfoJson {
            owner_id: self.owner_id.as_str().into(),
            operator_id: self.operator_id.as_str().into(),
            voter_count: self.voters.len(),
            min_unbond_period: self.min_unbond_period,
            min_claim_and_bond_days: self.min_claim_and_bond_days.into(),
            max_unbond_period: self.max_unbond_period,
            min_deposit_amount: self.min_deposit_amount.into(),
            max_locking_positions: self.max_locking_positions,
            max_voting_positions: self.max_voting_positions,
            mpdao_token_contract_address: self.mpdao_token_contract_address.as_str().into(),
            stnear_token_contract_address: self.stnear_token_contract_address.as_str().into(),
            registration_cost: self.registration_cost.into(),
            prev_governance_contract: self.prev_governance_contract.as_str().into(),
            accumulated_mpdao_distributed_for_claims: self
                .accumulated_mpdao_distributed_for_claims
                .into(),
            total_unclaimed_mpdao: self.total_unclaimed_mpdao.into(),
            accum_distributed_stnear_for_claims: self.accum_distributed_stnear_for_claims.into(),
            total_unclaimed_stnear: self.total_unclaimed_stnear.into(),
            evm_delegates_count: self.evm_delegates.len(),
            mpdao_per_near_e24: self.mpdao_per_near_e24.into(),
            mpdao_avail_to_sell: self.mpdao_avail_to_sell.into(),
        }
    }

    pub fn get_voters_count(&self) -> U64 {
        self.voters.len().into()
    }

    /** all users accumulated vp */
    pub fn get_total_voting_power(&self) -> U128String {
        self.total_voting_power.into()
    }

    // get all information for a single voter: voter + locking-positions + voting-positions
    pub fn get_voter_info(&self, voter_id: &String) -> VoterJSON {
        if let Some(voter) = self.voters.get(voter_id) {
            voter.to_json(voter_id)
        } else {
            VoterJSON {
                voter_id: voter_id.to_string(),
                locking_positions: vec![],
                balance_in_contract: 0.into(),
                voting_power: 0.into(),
                vote_positions: vec![],
            }
        }
    }

    // get all information for multiple voters, by index: Vec<voter + locking-positions + voting-positions>
    pub fn get_voters(&self, from_index: u32, limit: u32) -> Vec<VoterJSON> {
        let keys = self.voters.keys_as_vector();
        let voters_len = keys.len() as u64;
        let start = from_index as u64;
        let limit = limit as u64;

        let mut results = Vec::<VoterJSON>::new();
        for index in start..std::cmp::min(start + limit, voters_len) {
            let voter_id = keys.get(index).unwrap();
            let voter = self.voters.get(&voter_id).unwrap();
            results.push(voter.to_json(&voter_id));
        }
        results
    }

    pub fn get_balance(&self, voter_id: VoterId) -> U128String {
        let voter = self.internal_get_voter(&voter_id);
        let balance = voter.balance + voter.sum_unlocked();
        balance.into()
    }

    pub fn get_claimable_mpdao(&self, voter_id: &VoterId) -> U128String {
        self.claimable_mpdao
            .get(&voter_id)
            .unwrap_or_default()
            .into()
    }
    // kept to not break public interface
    pub fn get_claimable_meta(&self, voter_id: &VoterId) -> U128String {
        self.get_claimable_mpdao(voter_id)
    }

    pub fn get_claimable_stnear(&self, voter_id: &VoterId) -> U128String {
        self.claimable_stnear
            .get(&voter_id)
            .unwrap_or_default()
            .into()
    }

    // get all claims
    fn internal_get_claims(
        &self,
        map: &UnorderedMap<VoterId, u128>,
        from_index: u32,
        limit: u32,
    ) -> Vec<(String, U128String)> {
        let mut results = Vec::<(String, U128String)>::new();
        let keys = map.keys_as_vector();
        let start = from_index as u64;
        let limit = limit as u64;
        for index in start..std::cmp::min(start + limit, keys.len()) {
            let voter_id = keys.get(index).unwrap();
            let amount = map.get(&voter_id).unwrap();
            results.push((voter_id, amount.into()));
        }
        results
    }

    // get all stNEAR claims
    pub fn get_stnear_claims(&self, from_index: u32, limit: u32) -> Vec<(String, U128String)> {
        self.internal_get_claims(&self.claimable_stnear, from_index, limit)
    }

    // get all mpDAO claims
    pub fn get_claims(&self, from_index: u32, limit: u32) -> Vec<(String, U128String)> {
        self.internal_get_claims(&self.claimable_mpdao, from_index, limit)
    }

    pub fn get_locked_balance(&self, voter_id: VoterId) -> U128String {
        let voter = self.internal_get_voter(&voter_id);
        voter.sum_locked().into()
    }

    pub fn get_unlocking_balance(&self, voter_id: VoterId) -> U128String {
        let voter = self.internal_get_voter(&voter_id);
        voter.sum_unlocking().into()
    }

    pub fn voter_total_voting_power(&self, voter_id: VoterId) -> U128String {
        let voter = self.internal_get_voter(&voter_id);
        voter.sum_voting_power().into()
    }

    pub fn get_available_voting_power(&self, voter_id: VoterId) -> U128String {
        let voter = self.internal_get_voter(&voter_id);
        voter.available_voting_power.into()
    }

    pub fn get_used_voting_power(&self, voter_id: VoterId) -> U128String {
        let voter = self.internal_get_voter(&voter_id);
        voter.sum_used_votes().into()
    }

    pub fn get_locking_period(&self) -> (Days, Days) {
        (self.min_unbond_period, self.max_unbond_period)
    }

    // all locking positions for a voter
    pub fn get_all_locking_positions(&self, voter_id: VoterId) -> Vec<LockingPositionJSON> {
        let mut result = Vec::new();
        let voter = self.internal_get_voter(&voter_id);
        for index in 0..voter.locking_positions.len() {
            let locking_position = voter
                .locking_positions
                .get(index)
                .expect("Locking position not found!");
            result.push(locking_position.to_json(Some(index)));
        }
        result
    }

    pub fn get_locking_position(
        &self,
        index: PositionIndex,
        voter_id: VoterId,
    ) -> Option<LockingPositionJSON> {
        let voter = self.internal_get_voter(&voter_id);
        match voter.locking_positions.get(index) {
            Some(locking_position) => Some(locking_position.to_json(Some(index))),
            None => None,
        }
    }

    // votes by app and votable_object
    pub fn get_total_votes(
        &self,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) -> U128String {
        let votes = match self.votes.get(&contract_address) {
            Some(object) => object.get(&votable_object_id).unwrap_or(0_u128),
            None => 0_u128,
        };
        votes.into()
    }

    // votes by app (contract)
    // returns [[votable_bj_id, vote_amount],[votable_bj_id, vote_amount]...]
    pub fn get_votes_by_app(&self, app_or_contract_address: String) -> Vec<(String, U128String)> {
        let objects = self
            .votes
            .get(&app_or_contract_address)
            .unwrap_or(UnorderedMap::new(StorageKey::Votes));

        let mut results = Vec::new();
        for (id, voting_power) in objects.iter() {
            results.push((id, voting_power.into()))
        }
        results
    }

    // votes by app (deprecated, use get_votes_by_app)
    pub fn get_votes_by_contract(
        &self,
        contract_address: ContractAddress,
    ) -> Vec<VotableObjectJSON> {
        let objects = self
            .votes
            .get(&contract_address)
            .unwrap_or(UnorderedMap::new(StorageKey::Votes));

        let mut results: Vec<VotableObjectJSON> = Vec::new();
        for (id, applied_voting_power) in objects.iter() {
            results.push(VotableObjectJSON {
                votable_contract: contract_address.to_string(),
                id,
                current_votes: applied_voting_power.into(),
            })
        }
        results.sort_by_key(|v| v.current_votes.0);
        results
    }
    // given a voter, total votes per app + object_id
    pub fn get_votes_by_voter(&self, voter_id: VoterId) -> Vec<VotableObjectJSON> {
        let mut results: Vec<VotableObjectJSON> = Vec::new();
        let voter = self.internal_get_voter(&voter_id);
        for contract_address in voter.vote_positions.keys_as_vector().iter() {
            let votes_for_address = voter.vote_positions.get(&contract_address).unwrap();
            for (id, applied_voting_power) in votes_for_address.iter() {
                results.push(VotableObjectJSON {
                    votable_contract: contract_address.to_string(),
                    id,
                    current_votes: applied_voting_power.into(),
                })
            }
        }
        results.sort_by_key(|v| v.current_votes.0);
        results
    }

    pub fn get_votes_for_object(
        &self,
        voter_id: VoterId,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) -> U128String {
        let voter = self.internal_get_voter(&voter_id);
        let votes = match voter.vote_positions.get(&contract_address) {
            Some(votes_for_address) => votes_for_address.get(&votable_object_id).unwrap_or(0_u128),
            None => 0_u128,
        };
        votes.into()
    }

    // query current meta ready for distribution
    pub fn get_total_unclaimed_mpdao(&self) -> U128String {
        self.total_unclaimed_mpdao.into()
    }
    // kept to not break public interface
    pub fn get_total_unclaimed_meta(&self) -> U128String {
        self.get_total_unclaimed_mpdao()
    }
    // query total_distributed mpdao for claims
    pub fn get_accumulated_mpdao_distributed_for_claims(&self) -> U128String {
        self.accumulated_mpdao_distributed_for_claims.into()
    }
    // kept to not break public interface
    pub fn get_accumulated_distributed_for_claims(&self) -> U128String {
        self.get_accumulated_mpdao_distributed_for_claims()
    }
}
