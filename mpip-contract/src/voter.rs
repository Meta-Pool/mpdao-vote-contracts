use crate::utils::generate_hash_id;
use crate::*;

#[near(serializers = [borsh])]
pub struct Voter {
    pub votes: UnorderedMap<MpipId, Vote>,
}

#[near(serializers = [json])]
pub struct VoterJson {
    pub votes: Vec<VoteJson>,
}

impl Voter {
    pub(crate) fn new(voter_id: &VoterId) -> Self {
        Voter {
            votes: UnorderedMap::new(StorageKey::Votes {
                hash_id: generate_hash_id(voter_id.to_string()),
            }),
        }
    }
    pub(crate) fn to_json(&self, voter_id: VoterId) -> VoterJson {
        let mut _votes = Vec::<VoteJson>::new();
        for (_mpip_id, vote) in self.votes.iter() {
            _votes.push(vote.to_json(voter_id.clone()));
        }

        VoterJson { votes: _votes }
    }
}
