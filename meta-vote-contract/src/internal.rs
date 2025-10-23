use crate::*;

pub const DELEGATED_CONTRACT_CODE: &str = "delegated";

struct VotePos {
    pub votable_address: String,
    pub votable_object_id: String,
    pub voting_power: u128,
}

impl MetaVoteContract {
    pub(crate) fn assert_only_owner(&self) {
        require!(
            self.owner_id == env::predecessor_account_id(),
            "Only the owner can call this function."
        );
    }
    pub(crate) fn assert_operator(&self) {
        require!(
            self.operator_id == env::predecessor_account_id(),
            "Only the operator can call this function."
        );
    }

    pub(crate) fn assert_min_deposit_amount(&self, amount: Balance) {
        assert!(
            amount >= self.min_deposit_amount,
            "Minimum deposit amount is {} mpDAO.",
            self.min_deposit_amount
        );
    }

    /// get the delegated vote amount for a delegate (user_id == votable object_id)
    /// in order to delegate votes, a user "votes" for the delegate, and that vp is assigned to the delegate
    pub(crate) fn internal_get_delegated_vp(&self, user_id: &VotableObjId) -> u128 {
        match self.votes.get(&DELEGATED_CONTRACT_CODE.into()) {
            Some(object) => object.get(&user_id).unwrap_or(0_u128),
            None => 0_u128,
        }
    }

    pub(crate) fn internal_add_delegated_voting_power(
        &mut self,
        delegate_id: &String,
        voting_power: u128,
    ) {
        let mut delegate = self.internal_get_voter_or_panic(&delegate_id);
        delegate.available_voting_power += voting_power;
        // save delegate
        self.voters.insert(&delegate_id, &delegate);
    }

    pub(crate) fn internal_remove_delegated_voting_power(
        &mut self,
        delegate_id: &String,
        voting_power: u128,
    ) {
        let delegate = self.voters.get(&delegate_id);
        if let Some(mut delegate) = delegate {
            delegate.available_voting_power =
                delegate.available_voting_power.saturating_sub(voting_power);
            // save delegate
            self.voters.insert(&delegate_id, &delegate);
        }
    }

    /// call this after altering locking positions to ensure enough free voting power
    /// existed before the change
    pub(crate) fn internal_common_update_available(
        &self,
        require: bool,
        voter_id: &String,
        voter: &mut Voter,
    ) {
        let full_voting_power = self.internal_get_delegated_vp(voter_id) + voter.sum_locked_vp();
        let used_voting_power = voter.sum_used_votes();
        if require {
            require!(
            // here the user is unlocking their own positions, so self.delegated_vp(&voter_id) does not applies
            full_voting_power >= used_voting_power,
            "Not enough free voting power to unlock! You need to free more vp by removing votes.",
        );
        }
        // update available vp just in case
        voter.available_voting_power = full_voting_power.saturating_sub(used_voting_power);
    }

    /// call this to recompute available voting power after altering locking positions
    /// without checking for sufficiency (saturating_sub is used)
    pub(crate) fn update_vp_available(&self, voter_id: &String, voter: &mut Voter) {
        self.internal_common_update_available(false, voter_id, voter);
    }

    /// call this after reducing locking positions to ensure enough free voting power
    /// existed before the change
    pub(crate) fn require_vp_available(&self, voter_id: &String, voter: &mut Voter) {
        self.internal_common_update_available(true, voter_id, voter);
    }

    /// Recomputes the voter's available voting power from scratch.
    pub(crate) fn adjust_voter_voting_power(&mut self, voter_id: &String, voter: &mut Voter) {
        // HANDLE VOTING POWER ADJUSTMENT
        let mut used_voting_power = voter.sum_used_votes();
        let self_voting_power = voter.sum_locked_vp();
        let new_voting_power: u128 = self_voting_power + self.internal_get_delegated_vp(voter_id);
        // while more votes than voting power, remove votes
        if used_voting_power > new_voting_power {
            // get all voting positions sorted by voting power (ascending)
            let mut vote_positions_by_power = std::collections::BTreeMap::new();
            for address in voter.vote_positions.keys_as_vector().iter() {
                let pos = voter.vote_positions.get(&address).unwrap();
                for obj in pos.keys_as_vector().iter() {
                    let voting_power = pos.get(&obj).unwrap();
                    vote_positions_by_power.insert(
                        (voting_power, address.clone(), obj.clone()),
                        VotePos {
                            votable_address: address.as_str().to_string(),
                            votable_object_id: obj,
                            voting_power,
                        },
                    );
                }
            }

            // remove votes starting with the smaller ones by voting power (BTreeMap is naturally sorted)
            for (_, vote_pos) in vote_positions_by_power.iter() {
                if used_voting_power <= new_voting_power {
                    break;
                }
                self.internal_remove_voting_position(
                    voter_id,
                    voter,
                    &vote_pos.votable_address,
                    &vote_pos.votable_object_id,
                );
                log!(
                    "Removed vote for {} / {} of {} vp",
                    vote_pos.votable_address,
                    vote_pos.votable_object_id,
                    vote_pos.voting_power
                );
                used_voting_power -= vote_pos.voting_power;
            }
        }
        voter.available_voting_power = new_voting_power - used_voting_power;
    }

    /// Inner method to get or create a Voter.
    pub(crate) fn internal_get_voter(&self, voter_id: &String) -> Voter {
        self.voters.get(&voter_id).unwrap_or(Voter::new(&voter_id))
    }
    pub(crate) fn internal_get_voter_or_panic(&self, voter_id: &String) -> Voter {
        match self.voters.get(&voter_id) {
            Some(a) => a,
            _ => panic!("invalid voter_id {}", voter_id),
        }
    }

    fn internal_get_total_votes_for_address(
        &self,
        contract_address: &String,
    ) -> UnorderedMap<VotableObjId, u128> {
        self.votes
            .get(&contract_address)
            .unwrap_or(UnorderedMap::new(StorageKey::ContractVotes {
                hash_id: generate_hash_id(contract_address),
            }))
    }

    pub(crate) fn internal_increase_total_votes(
        &mut self,
        voting_power: u128,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        let mut votes_for_address = self.internal_get_total_votes_for_address(&contract_address);
        let mut votes = votes_for_address.get(&votable_object_id).unwrap_or(0_u128);
        votes += voting_power;

        votes_for_address.insert(&votable_object_id, &votes);
        self.votes.insert(&contract_address, &votes_for_address);
    }

    pub(crate) fn state_internal_decrease_total_votes_for_address(
        &mut self,
        voting_power: u128,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        let mut votes_for_address = self.internal_get_total_votes_for_address(&contract_address);
        let mut votes = votes_for_address
            .get(&votable_object_id)
            .expect("Cannot decrease if the Contract Address has no Votable Object.");
        require!(votes >= voting_power, "Decreasing total is too large.");
        votes -= voting_power;

        if votes == 0 {
            votes_for_address.remove(&votable_object_id);
        } else {
            votes_for_address.insert(&votable_object_id, &votes);
        }

        if votes_for_address.is_empty() {
            self.votes.remove(&contract_address);
        } else {
            self.votes.insert(&contract_address, &votes_for_address);
        }
    }

    // ***************************
    // * Claimable mpDao & stNear *
    // ***************************

    fn add_claimable(
        claimable_map: &mut UnorderedMap<String, u128>,
        total_unclaimed: &mut u128,
        account: &String,
        amount: u128,
    ) {
        let existing_claimable_amount = claimable_map.get(account).unwrap_or_default();
        claimable_map.insert(account, &(existing_claimable_amount + amount));
        // keep contract total
        *total_unclaimed += amount;
    }

    fn remove_claimable(
        claimable_map: &mut UnorderedMap<String, u128>,
        total_unclaimed: &mut u128,
        account: &String,
        amount: u128,
        token: &str,
    ) {
        let existing_claimable_amount = claimable_map.get(&account).unwrap_or_default();
        assert!(
            existing_claimable_amount >= amount,
            "you don't have enough claimable {}",
            token
        );
        let after_remove = existing_claimable_amount - amount;
        if after_remove == 0 {
            // 0 means remove
            claimable_map.remove(&account)
        } else {
            claimable_map.insert(&account, &after_remove)
        };
        // keep contract total
        *total_unclaimed -= amount;
    }

    pub(crate) fn add_claimable_mpdao(&mut self, account: &String, amount: u128) {
        assert!(amount > 0);
        Self::add_claimable(
            &mut self.claimable_mpdao,
            &mut self.total_unclaimed_mpdao,
            account,
            amount,
        );
    }

    pub(crate) fn add_claimable_stnear(&mut self, account: &String, amount: u128) {
        assert!(amount > 0);
        Self::add_claimable(
            &mut self.claimable_stnear,
            &mut self.total_unclaimed_stnear,
            account,
            amount,
        );
    }

    pub(crate) fn remove_claimable_mpdao(&mut self, account: &String, amount: u128) {
        Self::remove_claimable(
            &mut self.claimable_mpdao,
            &mut self.total_unclaimed_mpdao,
            account,
            amount,
            "mpDAO",
        );
    }

    pub(crate) fn remove_claimable_stnear(&mut self, account: &String, amount: u128) {
        Self::remove_claimable(
            &mut self.claimable_stnear,
            &mut self.total_unclaimed_stnear,
            account,
            amount,
            "stNEAR",
        );
    }

    pub(crate) fn claim_stnear_internal(
        &mut self,
        voter_id: &String,
        receiver_id: &AccountId,
        amount: u128,
    ) -> Promise {
        // remove claim
        self.remove_claimable_stnear(&voter_id, amount);
        // transfer to destination
        self.transfer_claimable_stnear_to_receiver(&voter_id, receiver_id, amount)
    }

    pub(crate) fn claim_and_bond_internal(
        &mut self,
        account: &String,
        beneficiary_id: &String,
        amount: u128,
        locking_period: u16,
    ) {
        assert!(
            locking_period >= self.min_claim_and_bond_days,
            "Minimum claim and bond period is {} days",
            self.min_claim_and_bond_days
        );
        self.assert_min_deposit_amount(amount);
        self.remove_claimable_mpdao(&account, amount);
        // get beneficiary voter
        let mut beneficiary_voter = self.internal_get_voter(&beneficiary_id);
        // create/update locking position
        self.deposit_locking_position(
            amount,
            locking_period,
            &beneficiary_id,
            &mut beneficiary_voter,
        );
    }
}
