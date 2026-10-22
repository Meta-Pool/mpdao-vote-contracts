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

const DAY_NS: u64 = 24 * 3600 * 1_000_000_000;

fn delegated() -> String {
    crate::internal::DELEGATED_CONTRACT_CODE.to_string()
}

/// alice delegates `amount` vp to bob at T0, returns the amount
fn setup_delegation() -> (MetaVoteContract, u128) {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    add_voter(&mut contract, "bob.near");
    testing_env!(context("alice.near").build());
    let amount = 40 * 10u128.pow(24);
    contract.vote(U128(amount), delegated(), "bob.near".to_string());
    (contract, amount)
}

#[test]
#[should_panic(expected = "Cannot unvote delegated votes until 7 days have passed")]
fn rebalance_down_inside_7_days_panics() {
    let (mut contract, amount) = setup_delegation();
    testing_env!(context_at("alice.near", T0 + DAY_NS).build());
    contract.rebalance(U128(amount / 2), delegated(), "bob.near".to_string());
}

#[test]
fn rebalance_down_after_7_days_works_and_does_not_restart_the_clock() {
    let (mut contract, amount) = setup_delegation();
    testing_env!(context_at("alice.near", T0 + 8 * DAY_NS).build());
    contract.rebalance(U128(amount / 2), delegated(), "bob.near".to_string());
    // the clock was not refreshed by the decrease, so a full unvote is still allowed
    contract.unvote(delegated(), "bob.near".to_string());
    let alice = contract.internal_get_voter(&"alice.near".to_string());
    assert_eq!(alice.sum_delegated_away_vp(), 0);
}

#[test]
fn rebalance_up_still_works_inside_7_days() {
    let (mut contract, amount) = setup_delegation();
    testing_env!(context_at("alice.near", T0 + DAY_NS).build());
    contract.rebalance(U128(amount + 10u128.pow(24)), delegated(), "bob.near".to_string());
    let alice = contract.internal_get_voter(&"alice.near".to_string());
    assert_eq!(alice.sum_delegated_away_vp(), amount + 10u128.pow(24));
}
