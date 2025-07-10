use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VoterJSON {
    pub voter_id: String,
    pub balance_in_contract: U128String,
    pub locking_positions: Vec<LockingPositionJSON>, // sum here to get total voting power
    pub voting_power: U128String,                    // available voting power
    pub vote_positions: Vec<VotePositionJSON>,       // sum here to get used voting power
}
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Voter {
    pub balance: MpDAOAmount,
    pub locking_positions: Vector<LockingPosition>,
    pub available_voting_power: u128, // available voting power, equals to sum(lp.voting_power)-sum(vp.voting_power)
    pub vote_positions: UnorderedMap<ContractAddress, UnorderedMap<VotableObjId, u128>>,
}

impl Voter {
    pub(crate) fn new(id: &String) -> Self {
        Self {
            balance: 0,
            locking_positions: Vector::new(StorageKey::LockingPosition {
                hash_id: generate_hash_id(id),
            }),
            available_voting_power: 0,
            vote_positions: UnorderedMap::new(StorageKey::VotePosition {
                hash_id: generate_hash_id(id),
            }),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.balance == 0 && self.locking_positions.is_empty()
    }

    pub(crate) fn sum_locked(&self) -> MpDAOAmount {
        let mut result = 0_u128;
        for locking_position in self.locking_positions.iter() {
            if locking_position.is_locked() {
                result += locking_position.amount;
            }
        }
        result
    }

    pub(crate) fn sum_unlocking(&self) -> MpDAOAmount {
        let mut result = 0_u128;
        for locking_position in self.locking_positions.iter() {
            if locking_position.is_unlocking() {
                result += locking_position.amount;
            }
        }
        result
    }

    pub(crate) fn sum_unlocked(&self) -> MpDAOAmount {
        let mut result = 0_u128;
        for locking_position in self.locking_positions.iter() {
            if locking_position.is_unlocked() {
                result += locking_position.amount;
            }
        }
        result
    }

    // /** sum all voting power from locked positions */
    pub(crate) fn sum_voting_power(&self) -> u128 {
        self.locking_positions
            .iter()
            .filter(|locking_position| locking_position.is_locked())
            .map(|locking_position| locking_position.voting_power)
            .sum()
    }

    pub(crate) fn sum_used_votes(&self) -> u128 {
        let mut result = 0_u128;
        for map in self.vote_positions.values() {
            result += map.values().sum::<u128>();
        }
        result
    }

    pub(crate) fn find_locked_position(&self, unbond_days: Days) -> Option<u64> {
        let mut index = 0_u64;
        for locking_position in self.locking_positions.iter() {
            if locking_position.locking_period == unbond_days && locking_position.is_locked() {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub(crate) fn get_position(&self, index: PositionIndex) -> LockingPosition {
        self.locking_positions
            .get(index)
            .expect("Index out of range!")
    }

    pub(crate) fn remove_position(&mut self, index: PositionIndex) {
        self.locking_positions.swap_remove(index);
    }

    //ARF
    //In NEAR SDK 5.x, the type passed as an argument to .new(...) in UnorderedMap, Vector, etc., must implement the IntoStorageKey trait, which in turn requires BorshSerialize.
    //Although you already have #[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)] applied, this is not enough on its own if you are passing a variant of the enum that contains data
    //When you do this, hash_id has a dynamic value (Vec<u8> or similar), and BorshStorageKey does not automatically generate the BorshSerialize implementation for variants with non-trivial fields unless you explicitly tell it to.
    //Use .try_to_vec().unwrap() como workaround converts the enum to a byte array (via Borsh), which does implement IntoStorageKey. It's a valid and secure solution, even used by the NEAR team.

    pub(crate) fn get_vote_position_for_address(
        &self,
        voter_id: &VoterId,
        contract_address: &ContractAddress,
    ) -> UnorderedMap<VotableObjId, u128> {
        let id = format!("{}-{}", voter_id.to_string(), contract_address.as_str());
        self.vote_positions.get(&contract_address).unwrap_or(
            UnorderedMap::new(StorageKey::VoterVotes {
                hash_id: generate_hash_id(&id),
            })
            .try_to_vec()
            .unwrap(),
        )
    }

    pub(crate) fn get_unlocked_position_indexes(&self) -> Vec<PositionIndex> {
        let mut result = Vec::new();
        for index in 0..self.locking_positions.len() {
            let locking_position = self
                .locking_positions
                .get(index)
                .expect("Locking position not found!");
            if locking_position.is_unlocked() {
                result.push(index);
            }
        }
        result
    }

    pub(crate) fn to_json(&self, voter_id: &VoterId) -> VoterJSON {
        let mut locking_positions = Vec::<LockingPositionJSON>::new();
        for index in 0..self.locking_positions.len() {
            let pos = self.locking_positions.get(index).unwrap();
            locking_positions.push(pos.to_json(Some(index)));
        }

        let mut vote_positions = Vec::<VotePositionJSON>::new();
        for address in self.vote_positions.keys_as_vector().iter() {
            let pos = self.vote_positions.get(&address).unwrap();
            for obj in pos.keys_as_vector().iter() {
                let value = pos.get(&obj).unwrap();
                vote_positions.push(VotePositionJSON {
                    votable_address: address.as_str().to_string(),
                    votable_object_id: obj,
                    voting_power: value.into(),
                });
            }
        }

        VoterJSON {
            voter_id: voter_id.to_string(),
            balance_in_contract: self.balance.into(),
            locking_positions,
            voting_power: self.available_voting_power.into(),
            vote_positions,
        }
    }

    // clear SEVERAL fully unlocked positions
    pub fn clear_fully_unlocked_positions(&mut self, position_index_list: Vec<PositionIndex>) {
        let mut position_index_list = position_index_list;
        position_index_list.sort();
        position_index_list.reverse();
        for index in position_index_list {
            let locking_position = self.get_position(index);
            // only if it is fully unlocked
            if locking_position.is_unlocked() {
                self.balance += locking_position.amount;
                self.remove_position(index);
            }
        }
    }
}
