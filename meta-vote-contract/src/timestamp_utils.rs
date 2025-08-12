use crate::types::{ContractAddress, VotableObjId};
use near_sdk::{env, log, CryptoHash};

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

    /// Refresh only the STALE votes for a given voter_id.
    /// Returns how many timestamps were updated.
    pub(crate) fn refresh_only_stale_votes(&mut self, voter_id: &String) -> u32 {
        // Check if the voter exists in the registry
        let Some(voter) = self.voters.get(voter_id) else {
            log!("⚠️ {} has no votes", voter_id);
            return 0;
        };

        let current_timestamp = env::block_timestamp_ms();
        let mut refreshed = 0u32;

        // Iterate through all vote positions for this voter
        for contract_address in voter.vote_positions.keys_as_vector().iter() {
            // Only refresh if the contract_address is 'metastaking.app'
            if contract_address != "metastaking.app" {
                continue;
            }

            if let Some(votes_for_address) = voter.vote_positions.get(&contract_address) {
                // Iterate each votable object ID
                for votable_object_id in votes_for_address.keys_as_vector().iter() {
                    // Refresh only if the vote is considered stale (None → stale)
                    if self.verify_vote_is_stale(voter_id, &contract_address, &votable_object_id) {
                        let hash_key = Self::compose_key(voter_id, &contract_address, &votable_object_id);
                        self.timestamp_storage.insert(&hash_key, &current_timestamp);
                        refreshed += 1;
                    }
                }
            }
        }

        log!("🔄 Refreshed {} STALE vote timestamps for {}", refreshed, voter_id);
        refreshed
    }
    /// Returns true if the vote is stale (older than threshold), or if there's no timestamp.
    pub(crate) fn verify_vote_is_stale(
        &self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) -> bool {
        let timestamp = self.get_vote_timestamp(voter_id, contract_address, votable_object_id);
        let current_timestamp = env::block_timestamp_ms();
        let sixty_days_ms = 60 * 24 * 60 * 60 * 1000; // 60 days in milliseconds

        match timestamp {
            None => true, // No timestamp means it's from before timestamp tracking was implemented
            Some(vote_timestamp) => {
                // Vote is stale if it's at least 60 days old
                current_timestamp >= vote_timestamp + sixty_days_ms
            }
        }
    }
}
