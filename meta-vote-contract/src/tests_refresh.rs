use crate::*;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;

fn acct(name: &str) -> AccountId {
    name.parse().unwrap()
}

const T0: u64 = 10 * 24 * 3600 * 1_000_000_000; // block timestamp in ns

fn context_at(predecessor: &str, timestamp_ns: u64) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(acct("metavote.near"))
        .predecessor_account_id(acct(predecessor))
        .block_timestamp(timestamp_ns);
    builder
}

fn context(predecessor: &str) -> VMContextBuilder {
    context_at(predecessor, T0)
}

fn new_contract() -> MetaVoteContract {
    testing_env!(context("owner.near").build());
    MetaVoteContract::new(
        acct("owner.near"),
        acct("operator.near"),
        30,
        300,
        U128(ONE_MPDAO),
        5,
        5,
        acct("mpdao.near"),
        acct("stnear.near"),
        U128(0),
        "prev.near".to_string(),
    )
}

/// voter with two locked positions: 100 mpDAO for 60 days (index 0) and 100 mpDAO for 120 days (index 1)
fn add_voter(contract: &mut MetaVoteContract, id: &str) {
    let voter_id = id.to_string();
    let mut voter = contract.internal_get_voter(&voter_id);
    contract.internal_create_locking_position(&mut voter, 100 * ONE_MPDAO, 60);
    contract.internal_create_locking_position(&mut voter, 100 * ONE_MPDAO, 120);
    contract.voters.insert(&voter_id, &voter);
}

const FEE: u128 = ONE_NEAR / 100;

fn voter_with_validator_vote(contract: &mut MetaVoteContract, id: &str) {
    add_voter(contract, id);
    testing_env!(context(id).build());
    contract.vote(
        U128(10u128.pow(24)),
        "metastaking.app".to_string(),
        "validator.near".to_string(),
    );
}

#[test]
#[should_panic(expected = "Only the voter or the operator can refresh vote timestamps.")]
fn third_party_cannot_refresh() {
    let mut contract = new_contract();
    voter_with_validator_vote(&mut contract, "alice.near");
    testing_env!(context("bob.near").attached_deposit(FEE).build());
    contract.refresh_vote_timestamps(&acct("alice.near"));
}

#[test]
fn voter_can_refresh_own_votes() {
    let mut contract = new_contract();
    voter_with_validator_vote(&mut contract, "alice.near");
    testing_env!(context("alice.near").attached_deposit(FEE).build());
    assert_eq!(contract.refresh_vote_timestamps(&acct("alice.near")), 1);
}

#[test]
fn operator_can_refresh_votes() {
    let mut contract = new_contract();
    voter_with_validator_vote(&mut contract, "alice.near");
    testing_env!(context("operator.near").attached_deposit(FEE).build());
    assert_eq!(contract.refresh_vote_timestamps(&acct("alice.near")), 1);
}
