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

fn add_vote_position(voter: &mut Voter, id: &str) {
    let address = "metastaking.app".to_string();
    let mut votes = voter.get_vote_position_for_address(&id.to_string(), &address);
    votes.insert(&"validator.near".to_string(), &1);
    voter.vote_positions.insert(&address, &votes);
}

#[test]
fn fresh_voter_is_empty() {
    testing_env!(context("alice.near").build());
    let voter = Voter::new(&"alice.near".to_string());
    assert!(voter.is_empty());
}

#[test]
fn voter_with_a_vote_is_not_empty() {
    testing_env!(context("alice.near").build());
    let mut voter = Voter::new(&"alice.near".to_string());
    add_vote_position(&mut voter, "alice.near");
    assert!(!voter.is_empty());
}

#[test]
fn voter_with_available_voting_power_is_not_empty() {
    testing_env!(context("alice.near").build());
    let mut voter = Voter::new(&"alice.near".to_string());
    voter.available_voting_power = 1;
    assert!(!voter.is_empty());
}

#[test]
fn withdraw_keeps_a_voter_that_still_has_votes() {
    let mut contract = new_contract();
    let id = "bob.near".to_string();
    let mut voter = Voter::new(&id);
    voter.balance = 5 * ONE_MPDAO;
    add_vote_position(&mut voter, "bob.near");
    contract.voters.insert(&id, &voter);

    testing_env!(context("bob.near").attached_deposit(1).build());
    contract.withdraw(vec![], U128(0));

    let kept = contract.voters.get(&id).expect("voter with votes must not be deleted");
    assert_eq!(kept.balance, 0);
}

#[test]
fn withdraw_deletes_a_voter_with_nothing_left() {
    let mut contract = new_contract();
    let id = "carol.near".to_string();
    let mut voter = Voter::new(&id);
    voter.balance = 5 * ONE_MPDAO;
    contract.voters.insert(&id, &voter);

    testing_env!(context("carol.near").attached_deposit(1).build());
    contract.withdraw(vec![], U128(0));

    assert!(contract.voters.get(&id).is_none());
}
