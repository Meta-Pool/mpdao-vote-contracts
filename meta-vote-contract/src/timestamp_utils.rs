use crate::types::{ContractAddress, VotableObjId};
use near_sdk::{env, CryptoHash};

// Constants for timestamp calculations
/// 60 days in milliseconds
const SIXTY_DAYS_MS: u64 = 60 * 24 * 60 * 60 * 1000;
/// timestamp implementation date Aug 1st, 2025
const DEFAULT_TIMESTAMP: u64 = 1754006400000;

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
    ) -> u64 {
        let hash_key = Self::compose_key(voter_id, contract_address, votable_object_id);
        self.timestamp_storage
            .get(&hash_key)
            .unwrap_or(DEFAULT_TIMESTAMP)
    }

    /// Remove timestamp record when unvoting
    pub(crate) fn remove_vote_timestamp(
        &mut self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        let hash_key = Self::compose_key(voter_id, contract_address, votable_object_id);
        self.timestamp_storage.remove(&hash_key);
    }

    /// Returns true if the vote is stale (older than threshold), or if there's no timestamp.
    pub(crate) fn verify_vote_is_stale(
        &self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) -> bool {
        let vote_timestamp = self.get_vote_timestamp(voter_id, contract_address, votable_object_id);
        // Vote is stale if it's at least 60 days old
        env::block_timestamp_ms() >= vote_timestamp + SIXTY_DAYS_MS
    }
}
