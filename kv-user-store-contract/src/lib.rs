use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, near, AccountId, BorshStorageKey};

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
    pub action: String, // "vote", "rebalance", "unvote"
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

        let record = VoteRecord {
            timestamp,
            contract_address,
            votable_object_id,
            voting_power,
            action,
        };

        let mut user_records = self.records_per_user.get(&voter_id).unwrap_or_else(|| UserRecords {
            records: Vector::new(env::sha256(voter_id.as_bytes())),
        });

        user_records.records.push(&record);
        self.records_per_user.insert(&voter_id, &user_records);
    }

    /// View para verificar que se haya grabado correctamente
    pub fn get_records_for_user(&self, account_id: AccountId) -> Vec<VoteRecord> {
        match self.records_per_user.get(&account_id) {
            Some(user_records) => user_records.records.to_vec(),
            None => vec![],
        }
    }
}
