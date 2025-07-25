use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, near, require, AccountId, BorshStorageKey};

#[near(serializers = [borsh])]
pub enum StorageKey {
    Records,
    RecordsPerUser { account_hash: Vec<u8> },
}

#[near(serializers = [borsh, json])]
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

#[derive(BorshStorageKey)]
#[near(contract_state, serializers = [borsh])]
pub struct TrackerContract {
    pub records_per_user: UnorderedMap<AccountId, UserRecords>,
}

impl Default for TrackerContract {
    fn default() -> Self {
        Self {
            records_per_user: UnorderedMap::new(b"r".to_vec()),
        }
    }
}

#[near]
impl TrackerContract {
    #[init]
    pub fn new() -> Self {
        Self {
            records_per_user: UnorderedMap::new(b"r".to_vec()),
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
            records: Vector::new(env::sha256(voter_id.as_bytes())),
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

        let mut new_vec = Vector::new(env::sha256(voter_id.as_bytes()));
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
        // Only owner can remove their own position
        require!(
            env::predecessor_account_id() == voter_id,
            "Only the owner of the vote can remove it"
        );

        if let Some(mut user_records) = self.records_per_user.get(&voter_id) {
            let mut kept: Vec<VoteRecord> = vec![];

            for record in user_records.records.iter() {
                if record.contract_address != contract_address || record.votable_object_id != votable_object_id {
                    kept.push(record);
                }
            }

            let mut new_vec = Vector::new(env::sha256(voter_id.as_bytes()));
            for r in kept {
                new_vec.push(&r);
            }

            self.records_per_user
                .insert(&voter_id, &UserRecords { records: new_vec });
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
                let mut new_vec = Vector::new(env::sha256(voter_id.as_bytes()));
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
}
