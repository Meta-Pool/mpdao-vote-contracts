use crate::{constants::*, locking_position::*, utils::*};
use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{unordered_map::UnorderedMap, Vector},
    env, log, near_bindgen, require,
    store::LookupMap,
    AccountId, Balance, PanicOnDefault, Promise, ONE_NEAR,
};
use types::*;
use voter::Voter;

mod constants;
mod deposit;
mod evm_delegate;
mod interface;
mod internal;
mod locking_position;
mod migrate;
mod types;
mod utils;
mod view;
mod voter;
mod withdraw;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MetaVoteContract {
    pub owner_id: AccountId,
    pub operator_id: AccountId,
    pub voters: UnorderedMap<String, Voter>,
    pub votes: UnorderedMap<ContractAddress, UnorderedMap<VotableObjId, u128>>,
    pub min_unbond_period: Days,
    pub max_unbond_period: Days,
    pub min_deposit_amount: MpDAOAmount,
    pub max_locking_positions: u8,
    pub max_voting_positions: u8,
    pub mpdao_token_contract_address: AccountId, // governance tokens
    pub total_voting_power: u128,

    // mpdao as rewards
    pub claimable_mpdao: UnorderedMap<String, u128>,
    pub accumulated_mpdao_distributed_for_claims: u128, // accumulated total mpDAO distributed
    pub total_unclaimed_mpdao: u128,                    // currently unclaimed mpDAO

    // stNear as rewards
    pub stnear_token_contract_address: AccountId,
    pub claimable_stnear: UnorderedMap<String, u128>,
    pub accum_distributed_stnear_for_claims: u128, // accumulated total stNEAR distributed
    pub total_unclaimed_stnear: u128,              // currently unclaimed stNEAR

    // association with other blockchain addresses, users' encrypted data
    pub registration_cost: u128,
    pub associated_user_data: UnorderedMap<String, String>, // account => encrypted_data

    // upgrade from prev governance token
    pub prev_governance_contract: String,

    pub evm_delegates: UnorderedMap<String, Vec<EvmAddress>>,
    pub evm_pre_delegation: LookupMap<EvmAddress, (String, EvmSignature)>,
    pub evm_delegation_signatures: LookupMap<EvmAddress, (String, EvmSignature)>,

    pub lock_votes_in_end_timestamp_ms: u64,
    pub lock_votes_in_address: Option<String>,
    pub lock_votes_in_numeric_id: u16,

    pub mpdao_per_near_e24: u128,
    pub mpdao_avail_to_sell: u128,
}

#[near_bindgen]
impl MetaVoteContract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        operator_id: AccountId,
        min_unbond_period: Days,
        max_unbond_period: Days,
        min_deposit_amount: U128String,
        max_locking_positions: u8,
        max_voting_positions: u8,
        mpdao_token_contract_address: AccountId,
        stnear_token_contract_address: AccountId,
        registration_cost: U128String,
        prev_governance_contract: String,
    ) -> Self {
        require!(!env::state_exists(), "The contract is already initialized");
        require!(
            min_unbond_period < max_unbond_period,
            "Review the min and max locking period"
        );
        Self {
            owner_id,
            operator_id,
            voters: UnorderedMap::new(StorageKey::Voters),
            votes: UnorderedMap::new(StorageKey::Votes),
            min_unbond_period,
            max_unbond_period,
            min_deposit_amount: MpDAOAmount::from(min_deposit_amount),
            max_locking_positions,
            max_voting_positions,
            mpdao_token_contract_address,
            total_voting_power: 0,
            accumulated_mpdao_distributed_for_claims: 0,
            total_unclaimed_mpdao: 0,
            claimable_mpdao: UnorderedMap::new(StorageKey::Claimable),
            stnear_token_contract_address,
            claimable_stnear: UnorderedMap::new(StorageKey::ClaimableStNear),
            accum_distributed_stnear_for_claims: 0,
            total_unclaimed_stnear: 0,
            registration_cost: registration_cost.0,
            associated_user_data: UnorderedMap::new(StorageKey::AirdropData),
            prev_governance_contract,
            evm_delegates: UnorderedMap::new(StorageKey::EvmDelegates),
            evm_delegation_signatures: LookupMap::new(StorageKey::EvmDelegationSignatures),
            evm_pre_delegation: LookupMap::new(StorageKey::EvmPreDelegation),
            lock_votes_in_end_timestamp_ms: 0,
            lock_votes_in_address: None,
            lock_votes_in_numeric_id: 0,
            mpdao_per_near_e24: 0,
            mpdao_avail_to_sell: 0,
        }
    }

    // ***************
    // * owner config
    // ***************
    #[payable]
    pub fn set_stnear_contract(&mut self, stnear_contract: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.stnear_token_contract_address = stnear_contract;
    }
    #[payable]
    pub fn set_operator_id(&mut self, operator_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.operator_id = operator_id;
    }
    #[payable]
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.owner_id = owner_id;
    }

    // *******************************
    // * Register for Airdrops/Gifts *
    // *******************************

    // for airdrops/rewards
    pub fn update_registration_cost(&mut self, new_cost: U128String) {
        self.assert_only_owner();
        self.registration_cost = new_cost.0;
    }

    // for airdrops/rewards
    pub fn get_registration_cost(&self) -> U128String {
        self.registration_cost.into()
    }

    pub fn check_if_registered_for_airdrops(&self, account_id: &String) -> bool {
        self.associated_user_data.get(account_id).is_some()
    }
    // "registerd" (sic) kept sintax error for backward compat, same fn as the one above  // cspell:disable-line cspell:disable-next-line
    pub fn check_if_user_is_registerd(&self, account_id: &String) -> bool {
        self.check_if_registered_for_airdrops(account_id)
    }

    #[payable]
    pub fn update_airdrop_user_data(&mut self, encrypted_data: &String) {
        assert!(
            env::attached_deposit() == self.registration_cost,
            "Pay {} yoctos for the registration cost",
            self.registration_cost
        );
        self.associated_user_data
            .insert(&env::predecessor_account_id().into(), encrypted_data);
    }

    /// Returns a single airdrop data / associated user data
    pub fn get_airdrop_account(&self, account_id: &String) -> String {
        self.associated_user_data.get(&account_id).unwrap()
    }
    pub fn operator_multi_set_airdrop_data(&mut self, data: Vec<(String, String)>) {
        self.assert_operator();
        for user_data in data {
            // set associated user data
            self.associated_user_data.insert(&user_data.0, &user_data.1);
        }
    }

    /// Returns a list of airdrop data / associated user data
    pub fn get_airdrop_accounts(&self, from_index: u32, limit: u32) -> Vec<(String, String)> {
        let keys = self.associated_user_data.keys_as_vector();
        let voters_len = keys.len() as u64;
        let start = from_index as u64;
        let limit = limit as u64;
        let mut results = Vec::<(String, String)>::new();
        for index in start..std::cmp::min(start + limit, voters_len) {
            let voter_id = keys.get(index).unwrap();
            let airdrop_data = self.associated_user_data.get(&voter_id).unwrap();
            results.push((voter_id.to_string(), airdrop_data));
        }
        results
    }

    // ****************
    // * claim & Lock *
    // ****************

    // claim mpDAO and create/update a locking position
    pub fn claim_and_lock(&mut self, amount: U128String, locking_period: u16) {
        let amount = amount.0;
        self.assert_min_deposit_amount(amount);
        let voter_id: String = env::predecessor_account_id().into();
        self.remove_claimable_mpdao(&voter_id, amount);
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        // create/update locking position
        self.deposit_locking_position(amount, locking_period, &voter_id, &mut voter);
    }

    // claim stNear
    pub fn claim_stnear(&mut self, amount: U128String) -> Promise {
        let amount = amount.0;
        let voter_id = env::predecessor_account_id().to_string();
        self.remove_claimable_stnear(&voter_id, amount);
        let receiver = voter_id.clone();
        self.transfer_stnear_to_voter(&voter_id, &receiver, amount)
    }

    // *************
    // * Unlocking *
    // *************

    pub fn unlock_position(&mut self, index: PositionIndex) {
        let voter_id: String = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let mut locking_position = voter.get_position(index);

        let voting_power = locking_position.voting_power;
        assert!(
            voter.available_voting_power >= voting_power,
            "Not enough free voting power to unlock! You have {}, required {}.",
            voter.available_voting_power,
            voting_power
        );

        log!(
            "UNLOCK: {} unlocked position {}.",
            &voter_id.to_string(),
            index
        );
        locking_position.unlocking_started_at = Some(get_current_epoch_millis());
        voter.locking_positions.replace(index, &locking_position);
        voter.available_voting_power -= voting_power;
        self.total_voting_power = self.total_voting_power.saturating_sub(voting_power);
        self.voters.insert(&voter_id, &voter);
    }

    pub fn unlock_partial_position(&mut self, index: PositionIndex, amount: U128String) {
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let mut locking_position = voter.get_position(index);

        let locking_period = locking_position.locking_period;
        let amount = MpDAOAmount::from(amount);

        // If the amount equals the total, then the unlock is not partial.
        if amount == locking_position.amount {
            return self.unlock_position(index);
        }
        require!(locking_position.amount > amount, "Amount too large!");
        assert!(
            (locking_position.amount - amount) >= self.min_deposit_amount,
            "A locking position cannot have less than {} mpDAO",
            self.min_deposit_amount
        );
        assert_at_least_1_mpdao(amount);
        let remove_voting_power = utils::calculate_voting_power(amount, locking_period);
        assert!(
            locking_position.voting_power >= remove_voting_power,
            "Not enough free voting power to unlock! Locking position has {}, required {}.",
            locking_position.voting_power,
            remove_voting_power
        );
        assert!(
            voter.available_voting_power >= remove_voting_power,
            "Not enough free voting power to unlock! You have {}, required {}.",
            voter.available_voting_power,
            remove_voting_power
        );

        log!(
            "UNLOCK: {} partially unlocked position {}.",
            &voter_id.to_string(),
            index
        );
        // Create a NEW unlocking position
        self.create_unlocking_position(&mut voter, amount, locking_period, remove_voting_power);

        // Decrease current locking position
        locking_position.voting_power -= remove_voting_power;
        locking_position.amount -= amount;
        assert_at_least_1_mpdao(locking_position.amount);
        voter.locking_positions.replace(index, &locking_position);

        voter.available_voting_power -= remove_voting_power;
        self.total_voting_power = self.total_voting_power.saturating_sub(remove_voting_power);
        self.voters.insert(&voter_id, &voter);
    }

    // ********************************
    // * extend locking position days *
    // ********************************

    pub fn locking_position_extend_days(&mut self, index: PositionIndex, new_locking_period: Days) {
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let mut locking_position = voter.get_position(index);

        // position should be locked
        require!(
            locking_position.unlocking_started_at.is_none(),
            "position should be locked in order to extend time"
        );
        require!(
            new_locking_period > locking_position.locking_period,
            "new auto-lock period should be greater than previous one"
        );

        log!(
            "EXTEND-TIME: {} position #{} {} days",
            &voter_id.to_string(),
            index,
            new_locking_period
        );

        let old_voting_power = locking_position.voting_power;
        let new_voting_power =
            utils::calculate_voting_power(locking_position.amount, new_locking_period);

        // update to new total-voting-power (add delta)
        self.total_voting_power += new_voting_power - old_voting_power;

        // update to new voter-voting-power (add delta)
        voter.available_voting_power += new_voting_power - old_voting_power;

        // update position
        locking_position.locking_period = new_locking_period;
        locking_position.voting_power = new_voting_power;

        // save
        voter.locking_positions.replace(index, &locking_position);
        self.voters.insert(&voter_id, &voter);
    }

    // ***********
    // * Re-Lock *
    // ***********

    pub fn relock_position(
        &mut self,
        index: PositionIndex,
        locking_period: Days,
        amount_from_balance: U128String,
    ) {
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let locking_position = voter.get_position(index);

        // Check voter balance and unlocking position amount.
        let amount_from_balance = amount_from_balance.0;
        assert!(
            voter.balance >= amount_from_balance,
            "Not enough balance. You have {} mpDAO in balance, required {}.",
            voter.balance,
            amount_from_balance
        );
        // Check if position is unlocking.
        require!(
            locking_position.unlocking_started_at.is_some(),
            "Cannot re-lock a locked position."
        );

        let now = get_current_epoch_millis();
        let unlocking_date = locking_position.unlocking_started_at.unwrap()
            + locking_position.locking_period_millis();

        if now < unlocking_date {
            // Position is still in the **unlocking** period, (unlocking_date is in the future)
            let remaining = unlocking_date - now;
            assert!(
                remaining < days_to_millis(locking_period),
                "The new locking period should be greater than {} days.",
                millis_to_days(remaining)
            );
        }

        log!(
            "RELOCK: {} relocked position {}.",
            &voter_id.to_string(),
            index
        );
        let amount = locking_position.amount + amount_from_balance;
        voter.remove_position(index);
        voter.balance -= amount_from_balance;
        self.deposit_locking_position(amount, locking_period, &voter_id, &mut voter);
    }

    pub fn relock_partial_position(
        &mut self,
        index: PositionIndex,
        amount_from_position: U128String,
        locking_period: Days,
        amount_from_balance: U128String,
    ) {
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let mut locking_position = voter.get_position(index);

        // Check voter balance and unlocking position amount.
        let amount_from_balance = amount_from_balance.0;
        let amount_from_position = amount_from_position.0;
        assert!(
            voter.balance >= amount_from_balance,
            "Not enough balance. You have {} mpDAO in balance, required {}.",
            voter.balance,
            amount_from_balance
        );
        assert!(
            locking_position.amount >= amount_from_position,
            "Locking position amount is not enough. Locking position has {} mpDAO, required {}.",
            locking_position.amount,
            amount_from_position
        );
        let amount = amount_from_balance + amount_from_position;
        assert!(
            amount >= self.min_deposit_amount,
            "A locking position cannot have less than {} mpDAO.",
            self.min_deposit_amount
        );
        // Check if position is unlocking.
        require!(
            locking_position.unlocking_started_at.is_some(),
            "Cannot re-lock a locked position."
        );

        let now = get_current_epoch_millis();
        let unlocking_date = locking_position.unlocking_started_at.unwrap()
            + locking_position.locking_period_millis();

        if now < unlocking_date {
            // Position is **unlocking**.
            let remaining = unlocking_date - now;
            assert!(
                remaining < days_to_millis(locking_period),
                "The new locking period should be greater than {} days.",
                millis_to_days(remaining)
            );

            let new_amount = locking_position.amount - amount_from_position;
            assert!(
                amount >= self.min_deposit_amount,
                "A locking position cannot have less than {} mpDAO.",
                self.min_deposit_amount
            );
            assert!(new_amount > 0, "Use relock_position() function instead.");

            locking_position.amount = new_amount;
            locking_position.voting_power =
                utils::calculate_voting_power(new_amount, locking_position.locking_period);
            voter.locking_positions.replace(index, &locking_position);
        } else {
            voter.balance += locking_position.amount - amount_from_position;
            voter.remove_position(index);
        }

        log!(
            "RELOCK: {} partially relocked position {}.",
            &voter_id.to_string(),
            index
        );
        voter.balance -= amount_from_balance;
        self.deposit_locking_position(amount, locking_period, &voter_id, &mut voter);
    }

    pub fn relock_from_balance(&mut self, locking_period: Days, amount_from_balance: U128String) {
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);

        let amount = amount_from_balance.0;
        assert!(
            voter.balance >= amount,
            "Not enough balance. You have {} mpDAO in balance, required {}.",
            voter.balance,
            amount
        );
        assert!(
            amount >= self.min_deposit_amount,
            "A locking position cannot have less than {} mpDAO.",
            self.min_deposit_amount
        );

        log!("RELOCK: {} relocked position.", &voter_id.to_string());
        voter.balance -= amount;
        self.deposit_locking_position(amount, locking_period, &voter_id, &mut voter);
    }

    // ******************
    // * Clear Position *
    // ******************

    // clear SEVERAL fully unlocked positions for env::predecessor_account_id()
    // and increases their balance
    pub fn clear_locking_position(&mut self, position_index_list: Vec<PositionIndex>) {
        require!(position_index_list.len() > 0, "Index list is empty.");
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        voter.clear_fully_unlocked_positions(position_index_list);
        self.voters.insert(&voter_id, &voter);
    }

    // **********
    // * Voting *
    // **********

    pub fn vote(
        &mut self,
        voting_power: U128String,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        self.internal_vote(
            &env::predecessor_account_id().as_str().to_string(),
            voting_power,
            contract_address,
            votable_object_id,
        )
    }

    fn internal_vote(
        &mut self,
        voter_id: &String,
        voting_power: U128String,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let voting_power = u128::from(voting_power);

        self.internal_create_voting_position(
            &voter_id,
            &mut voter,
            voting_power,
            &contract_address,
            &votable_object_id,
        );

        // save voter info
        self.voters.insert(&voter_id, &voter);

        log!(
            "VOTE: {} gave {} votes for object {} at address {}.",
            &voter_id,
            voting_power.to_string(),
            &votable_object_id,
            contract_address.as_str()
        );
    }

    fn internal_create_voting_position(
        &mut self,
        voter_id: &String,
        voter: &mut Voter,
        voting_power: u128,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        assert!(
            voter.available_voting_power >= voting_power,
            "Not enough free voting power. You have {}, requested {}.",
            voter.available_voting_power,
            voting_power
        );
        assert!(
            voter.vote_positions.len() <= self.max_voting_positions as u64,
            "Cannot exceed {} voting positions.",
            self.max_voting_positions
        );

        let mut votes_for_address =
            voter.get_vote_position_for_address(&voter_id, &contract_address);
        let mut votes = votes_for_address.get(&votable_object_id).unwrap_or(0_u128);

        voter.available_voting_power -= voting_power;
        votes += voting_power;
        votes_for_address.insert(&votable_object_id, &votes);
        voter
            .vote_positions
            .insert(&contract_address, &votes_for_address);

        // Update Meta Vote state.
        self.internal_increase_total_votes(voting_power, &contract_address, &votable_object_id);
    }

    pub fn rebalance(
        &mut self,
        voting_power: U128String,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        let voter_id = env::predecessor_account_id().to_string();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        let voting_power = u128::from(voting_power);

        let mut votes_for_address =
            voter.get_vote_position_for_address(&voter_id, &contract_address);
        let mut votes = votes_for_address
            .get(&votable_object_id)
            .expect("Rebalance not allowed for nonexisting Votable Object.");

        require!(
            votes != voting_power,
            "Cannot rebalance to same Voting Power."
        );
        if voting_power == 0 {
            return self.unvote(contract_address, votable_object_id);
        }

        if votes < voting_power {
            // Increase votes.
            let additional_votes = voting_power - votes;
            assert!(
                voter.available_voting_power >= additional_votes,
                "Not enough free voting power to unlock! You have {}, required {}.",
                voter.available_voting_power,
                additional_votes
            );
            voter.available_voting_power -= additional_votes;
            votes += additional_votes;

            log!(
                "VOTE: {} increased to {} votes for object {} at address {}.",
                &voter_id,
                voting_power.to_string(),
                &votable_object_id,
                contract_address.as_str()
            );

            self.internal_increase_total_votes(
                additional_votes,
                &contract_address,
                &votable_object_id,
            );
        } else {
            // Decrease votes.
            let remove_votes = votes - voting_power;
            voter.available_voting_power += remove_votes;
            votes -= remove_votes;

            log!(
                "VOTE: {} decreased to {} votes for object {} at address {}.",
                &voter_id,
                voting_power.to_string(),
                &votable_object_id,
                contract_address.as_str()
            );

            self.state_internal_decrease_total_votes_for_address(
                remove_votes,
                &contract_address,
                &votable_object_id,
            );
        }
        votes_for_address.insert(&votable_object_id, &votes);
        voter
            .vote_positions
            .insert(&contract_address, &votes_for_address);
        self.voters.insert(&voter_id, &voter);
    }

    fn internal_remove_voting_position(
        &mut self,
        voter_id: &String,
        voter: &mut Voter,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        // update this voter struct
        let mut user_votes_for_app =
            voter.get_vote_position_for_address(&voter_id, &contract_address);
        let user_vote_for_object = user_votes_for_app
            .get(&votable_object_id)
            .expect("Cannot unvote a Votable Object without votes.");

        voter.available_voting_power += user_vote_for_object; // available voting power
        user_votes_for_app.remove(&votable_object_id);

        if user_votes_for_app.is_empty() {
            voter.vote_positions.remove(&contract_address);
        } else {
            voter
                .vote_positions
                .insert(&contract_address, &user_votes_for_app);
        }
        // Update Meta Vote global state unordered maps
        self.state_internal_decrease_total_votes_for_address(
            user_vote_for_object,
            &contract_address,
            &votable_object_id,
        );

        log!(
            "UNVOTE: {} unvoted object {} at address {}.",
            &voter_id,
            &votable_object_id,
            contract_address.as_str()
        );
    }

    pub fn unvote(&mut self, contract_address: ContractAddress, votable_object_id: VotableObjId) {
        let voter_id = env::predecessor_account_id().as_str().to_string();
        self.internal_unvote(&voter_id, &contract_address, &votable_object_id)
    }

    fn internal_unvote(
        &mut self,
        voter_id: &String,
        contract_address: &ContractAddress,
        votable_object_id: &VotableObjId,
    ) {
        // verify if the votes are locked (for example last 48hs of grants voting up to 20 days after)
        if let Some(lock_votes_in_address) = &self.lock_votes_in_address {
            if self.lock_votes_in_end_timestamp_ms > env::block_timestamp_ms()
                && lock_votes_in_address == contract_address
            {
                let votable_object_id_filter = format!(" #{} ", self.lock_votes_in_numeric_id);
                if votable_object_id.contains(&votable_object_id_filter) {
                    panic!(
                        "you can not remove votes here until timestamp_ms {}",
                        self.lock_votes_in_end_timestamp_ms
                    )
                }
            }
        }

        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        self.internal_remove_voting_position(
            &voter_id,
            &mut voter,
            &contract_address,
            &votable_object_id,
        );
        // save voter
        self.voters.insert(&voter_id, &voter);
    }

    // *********
    // * Admin *
    // *********
    pub fn set_lock_in_vote_filters(
        &mut self,
        end_timestamp_ms: u64,
        votable_numeric_id: u16,
        votable_address: Option<String>,
    ) {
        self.assert_operator();
        self.lock_votes_in_end_timestamp_ms = end_timestamp_ms;
        self.lock_votes_in_address = votable_address;
        self.lock_votes_in_numeric_id = votable_numeric_id;
    }
    pub fn get_lock_in_vote_filters(self) -> (u64, Option<String>, u16) {
        (
            self.lock_votes_in_end_timestamp_ms,
            self.lock_votes_in_address,
            self.lock_votes_in_numeric_id,
        )
    }

    #[payable]
    pub fn update_mpdao_per_near_e24(&mut self, mpdao_per_near_e24: U128String) {
        self.assert_operator();
        // sanity check
        assert!(
            mpdao_per_near_e24.0 >= ONE_NEAR,
            "mpdao_per_near_e24 should be greater than ONE per NEAR"
        );
        self.mpdao_per_near_e24 = mpdao_per_near_e24.0;
    }

    #[payable]
    pub fn update_mpdao_avail_to_sell(&mut self, mpdao_avail_to_sell: U128String) {
        self.assert_only_owner();
        self.mpdao_avail_to_sell = mpdao_avail_to_sell.0;
    }

    #[payable]
    pub fn buy_lock_and_vote(
        &mut self,
        days: u16,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        // sanity check
        assert!(
            self.mpdao_per_near_e24 >= ONE_NEAR,
            "invalid mpdao_per_near_e24"
        );
        let amount_near = env::attached_deposit();
        assert!(
            amount_near >= ONE_NEAR / 100,
            "Minimum deposit amount is 0.01 NEAR."
        );
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let mut voter = self.internal_get_voter(&voter_id);
        let mpdao_amount_e24 = proportional(amount_near, self.mpdao_per_near_e24, ONE_NEAR);
        let mpdao_amount = mpdao_amount_e24 / E18;

        assert!(
            self.mpdao_avail_to_sell >= mpdao_amount,
            "Not enough mpDAO available to sell."
        );
        self.mpdao_avail_to_sell -= mpdao_amount;

        self.deposit_locking_position(mpdao_amount, days, &voter_id, &mut voter);

        let voting_power = utils::calculate_voting_power(mpdao_amount, days);
        log!(
            "buy_lock_and_vote: {} {} yNEAR => {} mpDAO",
            voter_id,
            amount_near,
            mpdao_amount
        );
        self.internal_vote(
            &voter_id,
            voting_power.into(),
            contract_address,
            votable_object_id,
        )
    }

    // If extra NEAR balance (from buy_lock_and_vote)
    // transfer to owner
    pub fn transfer_extra_balance(&mut self) -> U128String {
        let storage_cost = env::storage_usage() as u128 * env::storage_byte_cost();
        let extra_balance = env::account_balance() - storage_cost;
        if extra_balance >= 6 * ONE_NEAR {
            // only if there's more than 6 NEAR to transfer, and leave 5 extra NEAR to backup the storage an extra 500kb
            let extra = extra_balance - 5 * ONE_NEAR;
            Promise::new(self.owner_id.clone()).transfer(extra);
            return extra.into();
        }
        return 0.into();
    }

    #[payable]
    pub fn update_min_unbond_period(&mut self, new_min_unbond_period: Days) {
        assert_one_yocto();
        self.assert_only_owner();
        self.min_unbond_period = new_min_unbond_period;
    }
    #[payable]
    pub fn update_max_unbond_period(&mut self, new_max_unbond_period: Days) {
        assert_one_yocto();
        self.assert_only_owner();
        self.max_unbond_period = new_max_unbond_period;
    }
    #[payable]
    pub fn set_prev_gov_contract(&mut self, contract_id: String) {
        assert_one_yocto();
        self.assert_only_owner();
        self.prev_governance_contract = contract_id;
    }

    // user-started migration of locking-positions from prev-governance-contract
    // tuple Vec is (mpdao_amount,unbond_days)
    // the old gov contract has a flag to block users from migrating
    // the same position more than once
    pub fn migration_create_lps(
        &mut self,
        voter_id: &String,
        locking_positions: Vec<(U128String, u16)>,
        encrypted_associated_user_data: Option<String>,
    ) {
        require!(
            env::predecessor_account_id().to_string() == self.prev_governance_contract,
            "Only the old gov contract can call this function."
        );
        let mut voter = self.internal_get_voter(&voter_id);
        // create locking positions
        for lp in &locking_positions {
            // migrate with new voting power calculation
            // amount is in META w/24 decimals, convert to mpDAO w/6 decimals
            let mpdao_amount = lp.0 .0 / 1_000_000_000_000_000_000;
            let unbond_days = lp.1;
            self.deposit_locking_position(mpdao_amount, unbond_days, &voter_id, &mut voter);
        }
        // Note: deposit_locking_position saves voter
        // migration of associated data (but does not update)
        if let Some(associated_user_data) = encrypted_associated_user_data {
            if self
                .associated_user_data
                .get(&env::predecessor_account_id().into())
                .is_none()
            {
                self.associated_user_data
                    .insert(&env::predecessor_account_id().into(), &associated_user_data);
            }
        }
    }

    // bot-managed mirroring of locking positions in ethereum and l2s
    // tuple Vec is (unbond_days, mpdao_amount)
    pub fn operator_mirror_lps(
        &mut self,
        external_address: EvmAddress,
        locking_positions: Vec<(u16, U128String)>,
    ) {
        self.assert_operator();
        // external mirrored addresses are in the form of [address].evmp.near
        // example for an eth based address: f1552d1d7CD279A7B766F431c5FaC49A2fb6e361.evmp.near
        // evmp.near is controlled by the dao. No external user can create a xxx.evmp.near account
        let voter_id = utils::pseudo_near_address(&external_address);
        let mut voter = self.internal_get_voter(&voter_id);

        // HANDLE VOTING POWER
        let mut used_voting_power = voter.sum_used_votes();
        let prev_voting_power = voter.available_voting_power + used_voting_power;
        // check if the new voting power is enough for all existing votes
        let new_voting_power: u128 = locking_positions
            .iter()
            .map(|i| calculate_voting_power(i.1 .0, i.0))
            .sum();
        log!(
            "MIRROR: prev_vp {} new_vp {} used_vp {}.",
            prev_voting_power,
            new_voting_power,
            used_voting_power
        );
        // while more votes than voting power, remove votes
        while used_voting_power > new_voting_power {
            let first_voted_app_key: String = voter.vote_positions.keys_as_vector().get(0).unwrap();
            let first_voted_app_data = voter.vote_positions.get(&first_voted_app_key).unwrap();
            let first_voted_object_key = first_voted_app_data.keys_as_vector().get(0).unwrap();
            let used_voting_power_to_remove =
                first_voted_app_data.get(&first_voted_object_key).unwrap();
            // this fn manages all other accumulators that need to be updated when removing votes
            self.internal_remove_voting_position(
                &voter_id,
                &mut voter,
                &first_voted_app_key,
                &first_voted_object_key,
            );
            used_voting_power -= used_voting_power_to_remove;
        }
        // HANDLE LOCKING POSITIONS
        // first clear all
        voter.locking_positions.clear();
        // when creating the voting position, power is added to available_voting_power
        // so zero that too
        voter.available_voting_power = 0;
        // Note: internal_create_locking_position also adds to the contract total voting power
        // create locking positions
        for lp in &locking_positions {
            // create mirrored locking position
            let unbond_days = lp.0;
            let mpdao_amount = lp.1 .0;
            self.internal_create_locking_position(&mut voter, mpdao_amount, unbond_days);
        }

        // update user available_voting_power (to the amount added, remove the used)
        voter.available_voting_power -= used_voting_power;
        // also update contract total (new vp was already added, remove old only)
        self.total_voting_power = self.total_voting_power - prev_voting_power;

        // save voter
        self.voters.insert(&voter_id, &voter);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests;
