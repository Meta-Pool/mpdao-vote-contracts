use crate::types::{ContractAddress, VotableObjId};
use near_sdk::{env, CryptoHash};

impl crate::MetaVoteContract {
    // *******************
    // * Timestamp Utils *
    // *******************

    /// Compose a cryptographic hash key from voter_id, contract_address, and votable_object_id
    pub(crate) fn compose_key(
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) -> CryptoHash {
        let key_data = format!("{}:{}:{}", voter_id, contract_address, votable_object_id);
        env::sha256_array(key_data.as_bytes())
    }

    /// Store a timestamp with a hashed key combining voter_id, contract_address, and votable_object_id
    pub(crate) fn store_vote_timestamp(
        &mut self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        let hash_key = Self::compose_key(voter_id, contract_address, votable_object_id);
        let timestamp = env::block_timestamp_ms();
        self.timestamp_storage.insert(&hash_key, &timestamp);
    }

    /// Get the timestamp for a specific vote
    pub(crate) fn get_vote_timestamp(
        &self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) -> Option<u64> {
        let hash_key = Self::compose_key(voter_id, contract_address, votable_object_id);
        self.timestamp_storage.get(&hash_key)
    }

    /// Remove timestamp when unvoting
    pub(crate) fn remove_vote_timestamp(
        &mut self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        let hash_key = Self::compose_key(voter_id, contract_address, votable_object_id);
        self.timestamp_storage.remove(&hash_key);
    }

    /// Refresh the timestamps of all votes for a specific voter
    pub(crate) fn refresh_all_vote_timestamps(&mut self, voter_id: &String) {
        let voter = self.internal_get_voter_or_panic(voter_id);
        let current_timestamp = env::block_timestamp_ms();

        // Iterate through all vote positions for this voter
        for contract_address in voter.vote_positions.keys_as_vector().iter() {
            // Only refresh if the contract_address is 'metastaking.app'
            if contract_address != "metastaking.app" {
                continue;
            }

            if let Some(votes_for_address) = voter.vote_positions.get(&contract_address) {
                for votable_object_id in votes_for_address.keys_as_vector().iter() {
                    let hash_key = Self::compose_key(voter_id, &contract_address, &votable_object_id);
                    self.timestamp_storage.insert(&hash_key, &current_timestamp);
                }
            }
        }
    }
}
