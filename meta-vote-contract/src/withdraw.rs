use crate::interface::*;
use crate::*;
//use near_contract_standards::fungible_token::receiver;
use near_contract_standards::fungible_token::Balance;
use near_sdk::{assert_one_yocto, json_types::U128, near_bindgen, Promise, PromiseResult};

#[near_bindgen]
impl MetaVoteContract {
    // ************
    // * Withdraw *
    // ************

    fn internal_withdraw(
        &mut self,
        voter_id: &String,
        position_index_list: Vec<PositionIndex>,
        optional_amount_to_withdraw: Option<u128>,
    ) {
        assert_one_yocto();
        let mut voter = self.internal_get_voter_or_panic(&voter_id);
        // Clear locking positions, and increase the voter balance.
        if position_index_list.len() > 0 {
            voter.clear_fully_unlocked_positions(position_index_list);
        }
        let total_to_withdraw = optional_amount_to_withdraw.unwrap_or(voter.balance);
        require!(total_to_withdraw > 0, "Nothing to withdraw.");
        assert!(
            voter.balance >= total_to_withdraw,
            "Not enough balance. You have {} mpDAO in balance, required {}.",
            voter.balance,
            total_to_withdraw
        );

        voter.balance -= total_to_withdraw;

        if voter.is_empty() {
            self.voters.remove(&voter_id);
            log!("GODSPEED: {} is no longer part of Meta Vote!", &voter_id);
        } else {
            self.voters.insert(&voter_id, &voter);
        }
        self.transfer_mpdao_to_voter(voter_id, total_to_withdraw);
    }

    #[payable]
    pub fn withdraw(
        &mut self,
        position_index_list: Vec<PositionIndex>,
        amount_from_balance: U128String,
    ) {
        let amount_to_withdraw = amount_from_balance.0;
        let voter_id = env::predecessor_account_id().as_str().to_string();
        self.internal_withdraw(
            &voter_id,
            position_index_list,
            if amount_to_withdraw == 0 {
                None
            } else {
                Some(amount_to_withdraw)
            },
        );
    }

    #[payable]
    pub fn withdraw_all(&mut self) {
        let voter_id = env::predecessor_account_id().as_str().to_string();
        let voter = self.internal_get_voter_or_panic(&voter_id);
        let position_index_list = voter.get_unlocked_position_indexes();
        self.internal_withdraw(&voter_id, position_index_list, None);
    }

    // *************************
    // * Internals & callbacks *
    // *************************

    pub(crate) fn transfer_mpdao_to_voter(&mut self, voter_id: &String, amount: MpDAOAmount) {
        ext_ft::ext(self.mpdao_token_contract_address.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(voter_id.clone(), U128::from(amount), None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .after_transfer_mpdao_callback(voter_id.clone(), U128::from(amount)),
            );
    }

    #[private]
    pub fn after_transfer_mpdao_callback(&mut self, voter_id: VoterId, amount: U128) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "WITHDRAW: {} mpDAO transfer to {}",
                    amount,
                    voter_id.to_string()
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} mpDAO not transferred. Recovering {} state.",
                    amount,
                    &voter_id.to_string()
                );
                self.restore_transfer_to_mpdao(amount, voter_id);
            }
        };
    }

    fn restore_transfer_to_mpdao(&mut self, amount: Balance, voter_id: VoterId) {
        let mut voter = self.internal_get_voter(&voter_id);
        voter.balance += amount;
        self.voters.insert(&voter_id, &voter);
    }

    /// This transfer is only to claim available stNEAR
    pub(crate) fn transfer_claimable_stnear_to_receiver(
        &self,
        source_voter: &String,
        receiver: &String,
        amount: Balance,
    ) -> Promise {
        ext_ft::ext(self.stnear_token_contract_address.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(receiver.clone(), U128::from(amount), None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .after_transfer_stnear_callback(&source_voter, &receiver, U128::from(amount)),
            )
    }

    #[private]
    pub fn after_transfer_stnear_callback(
        &mut self,
        source_voter: &String,
        receiver: &String,
        amount: U128,
    ) {
        let amount = amount.0;
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                log!(
                    "{} WITHDRAWN {} stNEAR to {}",
                    source_voter,
                    amount,
                    receiver
                );
            }
            PromiseResult::Failed => {
                log!(
                    "FAILED: {} stNEAR not transferred. Recovering {} state.",
                    amount,
                    source_voter
                );
                self.add_claimable_stnear(source_voter, amount);
            }
        };
    }
}
