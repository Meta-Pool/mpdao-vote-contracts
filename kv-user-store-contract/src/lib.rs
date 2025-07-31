use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near, require, AccountId, BorshStorageKey, IntoStorageKey, PanicOnDefault};

#[near(serializers = [borsh])]
#[derive(BorshStorageKey)]
pub enum StorageKey {
    RecordsPerUser,
    RecordsVector { account_hash: Vec<u8> },
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct VoteRecord {
    pub timestamp: u64,
    pub contract_address: String,
    pub votable_object_id: String,
    pub voting_power: U128,
    pub action: String, // "vote", "revalidate"
}

#[near(serializers = [borsh])]
pub struct UserRecords {
    pub records: Vector<VoteRecord>,
}

#[derive(BorshStorageKey, PanicOnDefault)]
#[near(contract_state, serializers = [borsh])]
pub struct TrackerContract {
    pub records_per_user: UnorderedMap<AccountId, UserRecords>,
}

#[near(serializers = [borsh, json])]
pub struct RawVoteInputWithVoter {
    pub voter_id: AccountId,
    pub timestamp: u64,
    pub contract_address: String,
    pub votable_object_id: String,
    pub voting_power: U128,
    pub action: String,
}
#[near(serializers = [borsh, json])]
pub struct VoteKey {
    pub voter_id: AccountId,
    pub contract_address: String,
    pub votable_object_id: String,
}

#[near]
impl TrackerContract {
    #[init]
    pub fn new() -> Self {
        Self {
            records_per_user: UnorderedMap::new(StorageKey::RecordsPerUser),
        }
    }

    pub fn register_vote_event(
        &mut self,
        voter_id: AccountId,
        contract_address: String,
        votable_object_id: String,
        voting_power: U128,
        action: String,
    ) {
        let timestamp = env::block_timestamp_ms();
        let new_power = voting_power.0;

        let mut user_records = self.records_per_user.get(&voter_id).unwrap_or_else(|| UserRecords {
            records: Vector::new(
                StorageKey::RecordsVector {
                    account_hash: env::sha256(voter_id.as_bytes()),
                }
                .into_storage_key(),
            ),
        });

        let mut accumulated_power = new_power;
        let mut kept: Vec<VoteRecord> = vec![];

        for record in user_records.records.iter() {
            if record.contract_address == contract_address && record.votable_object_id == votable_object_id {
                accumulated_power += record.voting_power.0;
                // Replace the position for a new one with the sum of both voting powers
            } else {
                kept.push(record);
            }
        }

        let mut new_vec = Vector::new(
            StorageKey::RecordsVector {
                account_hash: env::sha256(voter_id.as_bytes()),
            }
            .into_storage_key(),
        );
        for r in kept {
            new_vec.push(&r);
        }

        new_vec.push(&VoteRecord {
            timestamp,
            contract_address,
            votable_object_id,
            voting_power: U128(accumulated_power),
            action,
        });

        self.records_per_user
            .insert(&voter_id, &UserRecords { records: new_vec });
    }

    pub fn remove_vote_event(&mut self, voter_id: AccountId, contract_address: String, votable_object_id: String) {
        const BOT_ACCOUNT: &str = "bot-account.testnet";
        const CONTRACT_A: &str = "mpdao-vote-v004.testnet";

        let caller = env::predecessor_account_id();

        // Authorized if caller is bot, the contract A, or the voter themselves
        let is_authorized = caller.as_str() == BOT_ACCOUNT || caller.as_str() == CONTRACT_A || caller == voter_id;

        require!(is_authorized, "Caller is not authorized to remove vote event");

        if let Some(mut user_records) = self.records_per_user.get(&voter_id) {
            let mut kept: Vec<VoteRecord> = vec![];

            for record in user_records.records.iter() {
                if record.contract_address != contract_address || record.votable_object_id != votable_object_id {
                    kept.push(record);
                }
            }

            let mut new_vec = Vector::new(
                StorageKey::RecordsVector {
                    account_hash: env::sha256(voter_id.as_bytes()),
                }
                .into_storage_key(),
            );
            for r in kept {
                new_vec.push(&r);
            }

            self.records_per_user
                .insert(&voter_id, &UserRecords { records: new_vec });

            log!(
                "Removed vote event: {} / {} for {}",
                contract_address,
                votable_object_id,
                voter_id
            );
        } else {
            log!("No records found for {}", voter_id);
        }
    }

    pub fn get_records_for_user(&self, account_id: AccountId) -> Vec<VoteRecord> {
        match self.records_per_user.get(&account_id) {
            Some(user_records) => user_records.records.to_vec(),
            None => vec![],
        }
    }

    pub fn revalidate_vote_event(&mut self, voter_id: AccountId, contract_address: String, votable_object_id: String) {
        // Only owner can revalidate
        require!(
            env::predecessor_account_id() == voter_id,
            "Only the owner of the vote can revalidate it"
        );

        if let Some(mut user_records) = self.records_per_user.get(&voter_id) {
            let mut updated = false;
            let mut kept: Vec<VoteRecord> = vec![];

            for record in user_records.records.iter() {
                if record.contract_address == contract_address && record.votable_object_id == votable_object_id {
                    // Replace with new timestamp and action revalidate
                    kept.push(VoteRecord {
                        timestamp: env::block_timestamp_ms(),
                        contract_address: record.contract_address.clone(),
                        votable_object_id: record.votable_object_id.clone(),
                        voting_power: record.voting_power,
                        action: "revalidate".to_string(),
                    });
                    updated = true;
                } else {
                    kept.push(record);
                }
            }

            if updated {
                let mut new_vec = Vector::new(
                    StorageKey::RecordsVector {
                        account_hash: env::sha256(voter_id.as_bytes()),
                    }
                    .into_storage_key(),
                );
                for r in kept {
                    new_vec.push(&r);
                }

                self.records_per_user
                    .insert(&voter_id, &UserRecords { records: new_vec });
            } else {
                env::panic_str("Vote position not found");
            }
        } else {
            env::panic_str("User not found");
        }
    }

    pub fn get_all_vote_records(&self) -> Vec<(AccountId, VoteRecord)> {
        let mut result = vec![];

        for (account_id, user_records) in self.records_per_user.iter() {
            for record in user_records.records.iter() {
                result.push((account_id.clone(), record));
            }
        }

        result
    }

    pub fn get_vote_timestamp(
        &self,
        account_id: AccountId,
        contract_address: String,
        votable_object_id: String,
    ) -> Option<u64> {
        if let Some(user_records) = self.records_per_user.get(&account_id) {
            for record in user_records.records.iter() {
                if record.contract_address == contract_address && record.votable_object_id == votable_object_id {
                    return Some(record.timestamp);
                }
            }
        }
        None
    }
    /// Registers multiple vote positions for multiple users efficiently.
    /// Only meant to be used via CLI, allowing arbitrary timestamps.
    pub fn register_batch_vote_events_multi_user(&mut self, votes: Vec<RawVoteInputWithVoter>) {
        use std::collections::HashMap;

        // Group votes by voter_id
        let mut grouped: HashMap<AccountId, Vec<RawVoteInputWithVoter>> = HashMap::new();
        for vote in votes {
            grouped.entry(vote.voter_id.clone()).or_insert_with(Vec::new).push(vote);
        }

        // Process each user independently
        for (voter_id, user_votes) in grouped {
            let account_hash = env::sha256(voter_id.as_bytes());

            // Load existing records if present
            let mut existing_records: Vec<VoteRecord> = if let Some(user_records) = self.records_per_user.get(&voter_id)
            {
                user_records.records.to_vec()
            } else {
                vec![]
            };

            // Insert existing records into a map (keyed by contract + object)
            let mut records_map: HashMap<(String, String), VoteRecord> = HashMap::new();
            for record in existing_records {
                records_map.insert(
                    (record.contract_address.clone(), record.votable_object_id.clone()),
                    record,
                );
            }

            // Overwrite or add new votes into the map
            for vote in user_votes {
                let key = (vote.contract_address.clone(), vote.votable_object_id.clone());
                records_map.insert(
                    key,
                    VoteRecord {
                        timestamp: vote.timestamp,
                        contract_address: vote.contract_address,
                        votable_object_id: vote.votable_object_id,
                        voting_power: vote.voting_power,
                        action: vote.action,
                    },
                );
            }

            // Rebuild the Vector from the merged records
            let mut new_vec = Vector::new(StorageKey::RecordsVector { account_hash }.into_storage_key());

            for record in records_map.values() {
                new_vec.push(record);
            }

            // Save back to storage
            self.records_per_user
                .insert(&voter_id, &UserRecords { records: new_vec });

            log!("Registered {} votes for {}", records_map.len(), voter_id);
        }
    }

    pub fn remove_vote_events_batch(&mut self, purge_requests: Vec<VoteKey>) {
        use std::collections::HashMap;

        const BOT_ACCOUNT: &str = "bot-account.testnet";
        const CONTRACT_A: &str = "mpdao-vote-v004.testnet";

        let caller = env::predecessor_account_id();

        // Group purge requests by user
        let mut grouped: HashMap<AccountId, Vec<(String, String)>> = HashMap::new();
        for req in purge_requests {
            let is_authorized =
                caller.as_str() == BOT_ACCOUNT || caller.as_str() == CONTRACT_A || caller == req.voter_id;
            require!(is_authorized, &format!("Caller not authorized for {}", req.voter_id));

            grouped
                .entry(req.voter_id.clone())
                .or_default()
                .push((req.contract_address, req.votable_object_id));
        }

        for (voter_id, positions) in grouped {
            if let Some(user_records) = self.records_per_user.get(&voter_id) {
                let mut kept = vec![];
                for record in user_records.records.iter() {
                    let should_remove = positions
                        .iter()
                        .any(|(addr, obj)| record.contract_address == *addr && record.votable_object_id == *obj);

                    if !should_remove {
                        kept.push(record);
                    }
                }

                let mut new_vec = Vector::new(
                    StorageKey::RecordsVector {
                        account_hash: env::sha256(voter_id.as_bytes()),
                    }
                    .into_storage_key(),
                );

                for r in kept {
                    new_vec.push(&r);
                }

                self.records_per_user
                    .insert(&voter_id, &UserRecords { records: new_vec });
                log!("Removed {} vote(s) for {}", positions.len(), voter_id);
            }
        }
    }
}
