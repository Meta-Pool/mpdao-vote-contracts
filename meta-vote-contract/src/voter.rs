use crate::*;
#[derive(Debug)]
#[near(serializers = [json])]
pub struct VoterJSON {
    pub voter_id: String,
    pub balance_in_contract: U128,
    pub locking_positions: Vec<LockingPositionJSON>, // sum here to get total voting power
    pub voting_power: U128,                          // available voting power
    pub vote_positions: Vec<VotePositionJSON>,       // sum here to get used voting power
}

//voting_power
//This field defines the amount of voting power assigned to this position.
//It is a numeric value used to calculate the impact of votes within the system.
//created_at
//This field stores the timestamp in milliseconds since the Unix epoch when this position was created.
//It is useful for auditing, sorting positions by age, or filtering positions based on their creation date.
//was_revalidated
//This field is a boolean indicator that signals whether the position has been revalidated.
//Revalidation may be necessary to ensure the position remains valid according to the current system rules.
#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct VotePosition {
    pub voting_power: u128,
    pub created_at: EpochMillis,
}

//The change in the Voter structure modifies the vote_positions field. Previously, it used UnorderedMap<ContractAddress, UnorderedMap<VotableObjId, u128>>, where the value for each VotableObjId was a simple numeric voting power (u128). Now, it uses UnorderedMap<ContractAddress, UnorderedMap<VotableObjId, VotePosition>>, where the value is a VotePosition structure.
//Previous Structure
//Voting power was stored as a simple numeric value (u128).
//This limited the ability to store additional information about each voting position.
//New Structure (VotePosition)
//Each voting position is now represented by a VotePosition structure, which includes:
//voting_power: The assigned voting power.
//created_at: The creation timestamp in milliseconds since the Unix epoch.
//was_revalidated: A boolean indicating whether the position was revalidated.
#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct Voter {
    pub balance: MpDAOAmount,
    pub locking_positions: Vector<LockingPosition>,
    pub available_voting_power: u128, // available voting power, equals to sum(lp.voting_power)-sum(vp.voting_power)
    pub vote_positions: UnorderedMap<ContractAddress, UnorderedMap<VotableObjId, VotePosition>>,
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

    //In this version, the vote_positions field now stores values of type VotePosition, which are structures containing detailed information about each voting position.
    //The function accesses the voting_power field of each VotePosition and sums these values to calculate the total voting power used.
    pub(crate) fn sum_used_votes(&self) -> u128 {
        let mut result = 0_u128;
        for map in self.vote_positions.values() {
            result += map.values().map(|v| v.voting_power).sum::<u128>();
        }
        result
    }

    pub(crate) fn find_locked_position(&self, unbond_days: Days) -> Option<u32> {
        let mut index = 0_u32;
        for locking_position in self.locking_positions.iter() {
            if locking_position.locking_period == unbond_days && locking_position.is_locked() {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    pub(crate) fn get_position(&self, index: PositionIndex) -> LockingPosition {
        self.locking_positions.get(index).expect("IndexOutOfRange").clone()
    }

    pub(crate) fn remove_position(&mut self, index: PositionIndex) {
        self.locking_positions.swap_remove(index);
    }

    //The get_vote_position_for_address function has been updated to handle the new VotePosition structure. Previously, vote_positions stored simple numeric values (u128) for voting power. Now, it stores detailed VotePosition structures.
    pub(crate) fn get_vote_position_for_address(
        &self,
        voter_id: &VoterId,
        contract_address: &ContractAddress,
    ) -> UnorderedMap<VotableObjId, VotePosition> {
        let id = format!("{}-{}", voter_id.to_string(), contract_address.as_str());

        self.vote_positions
            .get(contract_address)
            .unwrap_or(UnorderedMap::new(StorageKey::VoterVotes {
                hash_id: generate_hash_id(&id),
            }))
    }

    pub(crate) fn get_unlocked_position_indexes(&self) -> Vec<PositionIndex> {
        let mut result = Vec::new();
        for index in 0..self.locking_positions.len() {
            let locking_position = self.locking_positions.get(index).expect("Locking position not found!");
            if locking_position.is_unlocked() {
                result.push(index);
            }
        }
        result
    }

    //The to_json function has been updated to reflect the new structure of VotePosition. Previously, voting positions were stored as simple numeric values (u128). Now, they are represented by the VotePosition structure, which includes additional metadata such as voting_power, created_at, and was_revalidated.
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
                    voting_power: value.voting_power.into(),
                    created_at: value.created_at,
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
