use crate::*;
use near_sdk::json_types::{U128, U64};
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{testing_env, PromiseResult, RuntimeFeesConfig, VMConfig};
use std::collections::HashMap;

const T0: u64 = 10 * 24 * 3600 * 1_000_000_000; // block timestamp in ns

fn acct(name: &str) -> AccountId {
    name.parse().unwrap()
}

fn builder(predecessor: &str) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(acct("mpip.near"))
        .predecessor_account_id(acct(predecessor))
        .block_timestamp(T0);
    builder
}

fn new_contract() -> MpipContract {
    testing_env!(builder("admin.near").build());
    MpipContract::new(
        acct("admin.near"),
        acct("operator.near"),
        acct("meta.near"),
        acct("metavote.near"),
        U64(7 * 24 * 3600 * 1000),
        U128(1),
        U128(0),
        2500,
    )
}

fn contract_with_proposal() -> MpipContract {
    let mut contract = new_contract();
    contract.internal_create_proposal(
        0,
        acct("creator.near"),
        "title".to_string(),
        "short".to_string(),
        "body".to_string(),
        "data".to_string(),
        "extra".to_string(),
    );
    contract
}

/// runs a callback that reads one promise result holding a U128 (JSON string)
fn with_promise_result(value: &str) {
    testing_env!(
        builder("mpip.near").build(),
        VMConfig::test(),
        RuntimeFeesConfig::test(),
        HashMap::default(),
        vec![PromiseResult::Successful(format!("\"{}\"", value).into_bytes())]
    );
}

#[test]
fn admin_check_uses_the_direct_caller() {
    let contract = new_contract();
    testing_env!(builder("admin.near").build());
    contract.assert_only_admin();
}

#[test]
#[should_panic(expected = "Only the admin can call this function.")]
fn admin_check_ignores_the_signer() {
    let contract = new_contract();
    // the admin signed the transaction, but a contract called us
    testing_env!(builder("attacker.near")
        .signer_account_id(acct("admin.near"))
        .build());
    contract.assert_only_admin();
}

#[test]
fn proposal_creator_is_the_proposer_not_the_callback_caller() {
    let contract = contract_with_proposal();
    assert_eq!(contract.internal_get_proposal(&0).creator_id, acct("creator.near"));
}

#[test]
#[should_panic(expected = "total voting power is zero")]
fn voting_period_cannot_start_with_zero_total_voting_power() {
    let mut contract = contract_with_proposal();
    with_promise_result("0");
    contract.start_voting_period_callback(0);
}

#[test]
fn voting_period_starts_with_voting_power() {
    let mut contract = contract_with_proposal();
    with_promise_result("1000");
    contract.start_voting_period_callback(0);
    assert_eq!(
        contract.internal_get_proposal(&0).v_power_quorum_to_reach,
        Some(250)
    );
}

#[test]
#[should_panic(expected = "Account has already voted")]
fn vote_callback_rejects_a_second_vote_from_the_same_voter() {
    let mut contract = contract_with_proposal();
    with_promise_result("1000");
    contract.start_voting_period_callback(0);

    with_promise_result("500");
    contract.vote_proposal_callback(0, acct("alice.near"), VoteType::For, U128(0), String::new());
    // a second vote transaction that passed the first-receipt check concurrently
    with_promise_result("500");
    contract.vote_proposal_callback(0, acct("alice.near"), VoteType::For, U128(0), String::new());
}

#[test]
fn vote_callback_counts_a_single_vote() {
    let mut contract = contract_with_proposal();
    with_promise_result("1000");
    contract.start_voting_period_callback(0);
    with_promise_result("500");
    contract.vote_proposal_callback(0, acct("alice.near"), VoteType::For, U128(0), String::new());
    assert_eq!(contract.internal_get_proposal_vote(0).for_votes, 500);
}
