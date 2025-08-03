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

    /// Verify that a vote timestamp is either None or at least 30 days old
    pub(crate) fn verify_vote_is_stale(
        &self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) -> bool {
        let timestamp = self.get_vote_timestamp(voter_id, contract_address, votable_object_id);

        match timestamp {
            None => true, // No timestamp means it's from before timestamp tracking was implemented
            Some(vote_timestamp) => {
                let current_timestamp = env::block_timestamp_ms();
                let thirty_days_ms = 30 * 24 * 60 * 60 * 1000; // 30 days in milliseconds

                // Vote is stale if it's at least 30 days old
                current_timestamp >= vote_timestamp + thirty_days_ms
            }
        }
    }
}
