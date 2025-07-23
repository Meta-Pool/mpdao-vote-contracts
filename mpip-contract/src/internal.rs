use crate::utils::get_current_epoch_millis;
use crate::*;
use near_sdk::json_types::U128;
use near_sdk::{env, require, NearToken, PromiseResult};

impl MpipContract {
    pub(crate) fn assert_only_admin(&self) {
        require!(
            self.admin_id == env::signer_account_id(),
            "Only the admin can call this function."
        );
    }

    pub(crate) fn assert_only_operator(&self) {
        require!(
            self.admin_id == env::signer_account_id()
                || self.operator_id == env::signer_account_id(),
            "Only the operator or admin can call this function."
        );
    }

    pub(crate) fn assert_only_operator_or_creator(&self, mpip_id: MpipId) {
        let proposal = self.internal_get_proposal(&mpip_id);
        require!(
            self.operator_id == env::signer_account_id()
                || proposal.creator_id == env::signer_account_id(),
            "Only the admin or proposal creator can call this function."
        );
    }

    pub(crate) fn assert_only_creator(&self, mpip_id: MpipId) {
        let proposal = self.internal_get_proposal(&mpip_id);
        require!(
            proposal.creator_id == env::signer_account_id(),
            "Only the proposal creator can call this function."
        );
    }

    pub(crate) fn assert_open_for_new_mpips(&self) {
        require!(
            self.open_for_new_mpips == true,
            "Contract not open for new proposals"
        );
    }

    pub(crate) fn internal_has_voted(&self, mpip_id: &MpipId, voter_id: &VoterId) -> bool {
        let voter = self.voters.get(&voter_id);
        match voter {
            Some(_voter) => !_voter.votes.get(&mpip_id).is_none(),
            None => false,
        }
    }

    pub(crate) fn assert_has_not_voted(&self, mpip_id: MpipId, account_id: VoterId) {
        require!(
            !self.internal_has_voted(&mpip_id, &account_id),
            "Account has already voted"
        );
    }

    pub(crate) fn assert_has_voted(&self, mpip_id: MpipId, account_id: VoterId) {
        require!(
            self.internal_has_voted(&mpip_id, &account_id),
            "Account has not voted"
        );
    }

    pub(crate) fn internal_check_proposal_threshold(&self, voting_power: u128) -> bool {
        self.min_voting_power_amount <= voting_power
    }

    pub(crate) fn internal_proposal_is_active(&self, mpip_id: MpipId) -> bool {
        let proposal = self.internal_get_proposal(&mpip_id);
        // check if proposal has vote_start_timestamp
        match proposal.vote_start_timestamp {
            Some(date) => {
                get_current_epoch_millis() <= date
                    && !proposal.draft
                    && !proposal.canceled
                    && !proposal.executed
            }
            None => !proposal.draft && !proposal.canceled && !proposal.executed,
        }
    }

    pub(crate) fn internal_proposal_is_active_or_draft(&self, mpip_id: MpipId) -> bool {
        let proposal = self.internal_get_proposal(&mpip_id);
        // check if proposal has vote_start_timestamp
        match proposal.vote_start_timestamp {
            Some(date) => {
                get_current_epoch_millis() <= date && !proposal.canceled && !proposal.executed
            }
            None => !proposal.canceled && !proposal.executed,
        }
    }

    pub(crate) fn internal_proposal_is_on_voting(&self, mpip_id: &MpipId) -> bool {
        let proposal = self.internal_get_proposal(&mpip_id);
        log!("proposal {} on voting?", proposal.mpip_id.to_string());
        // check if proposal has vote_start_timestamp
        match proposal.vote_start_timestamp {
            Some(date) => {
                get_current_epoch_millis() >= date
                    && get_current_epoch_millis() <= proposal.vote_end_timestamp.unwrap()
                    && !proposal.draft
                    && !proposal.canceled
                    && !proposal.executed
            }
            None => false,
        }
    }

    pub(crate) fn assert_proposal_threshold(&self, voting_power: u128) {
        require!(
            self.internal_check_proposal_threshold(voting_power),
            "Proposal threshold does not reached"
        )
    }

    pub(crate) fn assert_proposal_is_active_or_draft(&self, mpip_id: MpipId) {
        require!(
            self.internal_proposal_is_active_or_draft(mpip_id),
            "Proposal is not active or in draft state"
        )
    }

    pub(crate) fn assert_proposal_is_on_voting(&self, mpip_id: &MpipId) {
        require!(
            self.internal_proposal_is_on_voting(&mpip_id),
            "Proposal is not on voting period"
        )
    }

    pub(crate) fn internal_get_proposal(&self, mpip_id: &MpipId) -> Mpip {
        self.proposals
            .get(&mpip_id)
            .expect("MPIP Id does not exist")
    }

    pub(crate) fn internal_get_proposal_vote(&self, mpip_id: MpipId) -> ProposalVote {
        self.votes
            .get(&mpip_id)
            .unwrap_or(ProposalVote::new(&mpip_id))
    }

    pub(crate) fn internal_get_voter_vote(&self, mpip_id: &MpipId, voter_id: &VoterId) -> Vote {
        let voter = self
            .voters
            .get(&voter_id)
            .expect("Voter has not active votes");
        voter
            .votes
            .get(mpip_id)
            .expect("Voter has no voted on this proposal")
    }

    pub(crate) fn internal_get_total_voting_power_from_promise(&self) -> Balance {
        require!(
            env::promise_results_count() == 1,
            "This is a callback method."
        );

        match env::promise_result(0) {
            PromiseResult::Failed => env::panic_str("Meta Vote is not available!"),
            PromiseResult::Successful(result) => {
                let v_power = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                v_power.0
            }
        }
    }

    pub(crate) fn internal_get_user_total_voting_power_from_promise(&self) -> Balance {
        require!(
            env::promise_results_count() == 1,
            "This is a callback method."
        );

        match env::promise_result(0) {
            PromiseResult::Failed => env::panic_str("Meta Vote is not available!"),
            PromiseResult::Successful(result) => {
                let locking_positions =
                    near_sdk::serde_json::from_slice::<Vec<LockingPositionJSON>>(&result).unwrap();
                let mut result: Balance = 0;
                for index in 0..locking_positions.len() {
                    let locking_position = locking_positions
                        .get(index)
                        .expect("Locking position not found!");
                    if locking_position.is_locked {
                        result += locking_position.voting_power.0;
                    }
                }
                result
            }
        }
    }

    pub(crate) fn internal_get_quorum(&self, total_voting_power: u128) -> u128 {
        total_voting_power * u128::from(self.quorum_floor) / 100 / 100
    }

    pub(crate) fn internal_is_quorum_reached(&self, mpip_id: MpipId) -> bool {
        // let quorum = self.internal_get_quorum(total_voting_power);
        let proposal_vote = self.internal_get_proposal_vote(mpip_id);
        let proposal = self.internal_get_proposal(&mpip_id);
        let quorum = match proposal.v_power_quorum_to_reach {
            Some(quorum) => quorum,
            None => panic!("Proposal quorum has not been set"),
        };
        quorum <= proposal_vote.for_votes + proposal_vote.abstain_votes
    }

    pub(crate) fn internal_get_voter(&self, voter_id: &VoterId) -> Voter {
        self.voters.get(&voter_id).unwrap_or(Voter::new(&voter_id))
    }

    pub(crate) fn internal_get_proposer(&self, proposer_id: AccountId) -> Vec<MpipId> {
        self.proposers
            .get(&proposer_id)
            .unwrap_or(Vec::<MpipId>::new())
    }

    pub(crate) fn assert_proposal_storage_is_covered(&self) {
        assert!(
            env::attached_deposit() >= NearToken::from_yoctonear(self.mpip_storage_near),
            "The required NEAR to create a proposal is {}",
            self.mpip_storage_near
        );
    }
}
