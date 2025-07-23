use crate::*;

// /////////////////
// Comment struct //
// /////////////////

#[near(serializers = [json, borsh])]
pub enum MpipState {
    Draft,  // proposer share the idea. Giving awareness from the community via discussion or poll
    Active, // reviewed and accepted by managers
    VotingProcess, // on voting process
    Accepted, // accepted by votes
    Rejected, // rejected by votes
    Executed, // proposal executed, performing on-chain actions
    Canceled, // canceled by manager after community awareness
}

#[near(serializers = [json])]
pub struct MpipJSON {
    pub mpip_id: MpipId,
    pub title: String,
    pub short_description: String,
    pub body: String,
    // pub comments: String,
    pub data: String,
    pub extra: String,

    pub creator_id: AccountId,
    pub vote_start_timestamp: Option<EpochMillis>,
    pub vote_end_timestamp: Option<EpochMillis>,
    pub draft: bool,
    pub executed: bool,
    pub canceled: bool,

    pub v_power_quorum_to_reach: Option<U128>,
}

#[near(serializers = [borsh, json])]
pub struct Mpip {
    pub mpip_id: MpipId,
    pub title: String,
    pub short_description: String,
    pub body: String,
    // pub comments: String,
    pub data: String,
    pub extra: String,
    pub creator_id: AccountId,
    pub vote_start_timestamp: Option<EpochMillis>,
    pub vote_end_timestamp: Option<EpochMillis>,
    pub draft: bool,
    pub executed: bool,
    pub canceled: bool,
    pub v_power_quorum_to_reach: Option<VotingPower>,
}

impl Mpip {
    pub(crate) fn new(
        id: MpipId,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    ) -> Self {
        Mpip {
            mpip_id: id,
            title: title,
            short_description: short_description,
            body: body,
            // comments: "".to_string(),
            data: data,
            extra: extra,
            creator_id: env::signer_account_id(),
            vote_end_timestamp: None,
            vote_start_timestamp: None,
            draft: true,
            executed: false,
            canceled: false,
            v_power_quorum_to_reach: None,
        }
    }

    pub(crate) fn to_json(&self) -> MpipJSON {
        let quorum_to_reach = match self.v_power_quorum_to_reach {
            Some(quorum_to_reach) => Some(U128::from(quorum_to_reach)),
            None => None,
        };
        MpipJSON {
            mpip_id: self.mpip_id.clone(),
            title: self.title.clone(),
            body: self.body.clone(),
            short_description: self.short_description.clone(),
            // comments: self.comments.clone(),
            data: self.data.clone(),
            extra: self.extra.clone(),
            creator_id: self.creator_id.clone(),
            vote_end_timestamp: self.vote_end_timestamp.clone(),
            vote_start_timestamp: self.vote_start_timestamp.clone(),
            executed: self.executed,
            canceled: self.canceled,
            draft: self.draft,
            v_power_quorum_to_reach: quorum_to_reach,
        }
    }
}

impl MpipContract {
    pub(crate) fn internal_create_proposal(
        &mut self,
        mpip_id: MpipId,
        title: String,
        short_description: String,
        body: String,
        data: String,
        extra: String,
    ) -> MpipId {
        let proposal = Mpip::new(mpip_id, title, short_description, body, data, extra);
        self.proposals.insert(&mpip_id, &proposal);
        let mut proposer = self.internal_get_proposer(proposal.creator_id.clone());
        proposer.push(mpip_id);
        self.proposers.insert(&proposal.creator_id, &proposer);
        // let voter = self.internal_get_voter(&proposal.creator_id);
        // voter.used_voting_power += self.min_voting_power_amount;
        // self.voters.insert(&proposal.creator_id, &voter);
        proposal.mpip_id.into()
    }

    pub(crate) fn internal_get_proposal_state(&self, mpip_id: MpipId) -> MpipState {
        let proposal = self.internal_get_proposal(&mpip_id);
        if proposal.executed {
            return MpipState::Executed;
        } else if proposal.canceled {
            return MpipState::Canceled;
        } else if proposal.draft {
            return MpipState::Draft;
        }

        if self.internal_proposal_is_on_voting(&mpip_id) {
            return MpipState::VotingProcess;
        } else if self.internal_proposal_is_active(mpip_id) {
            return MpipState::Active;
        }

        if self.internal_is_quorum_reached(mpip_id) && self.get_proposal_vote_succeeded(mpip_id) {
            return MpipState::Accepted;
        } else {
            return MpipState::Rejected;
        }
    }
}
