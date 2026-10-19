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

#[test]
#[should_panic(expected = "position should be locked in order to unlock it")]
fn unlock_position_twice_panics() {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    testing_env!(context("alice.near").build());
    contract.unlock_position(0);
    contract.unlock_position(0);
}

#[test]
fn unlock_position_reduces_total_voting_power_once() {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    testing_env!(context("alice.near").build());
    let vp_60_days = crate::utils::calculate_voting_power(100 * ONE_MPDAO, 60);
    let before = contract.total_voting_power;
    contract.unlock_position(0);
    assert_eq!(contract.total_voting_power, before - vp_60_days);
}

#[test]
#[should_panic(expected = "position should be locked in order to unlock it")]
fn unlock_partial_position_after_unlock_panics() {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    testing_env!(context("alice.near").build());
    contract.unlock_position(0);
    contract.unlock_partial_position(0, U128(10 * ONE_MPDAO));
}

#[test]
fn operator_can_repair_total_voting_power() {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    add_voter(&mut contract, "bob.near");
    let per_voter = crate::utils::calculate_voting_power(100 * ONE_MPDAO, 60)
        + crate::utils::calculate_voting_power(100 * ONE_MPDAO, 120);
    let page_total = contract.get_locked_voting_power_page(0, 10).0;
    assert_eq!(page_total, per_voter * 2);

    contract.total_voting_power = 0; // drained by repeated unlocks
    testing_env!(context("operator.near").build());
    contract.operator_set_total_voting_power(U128(page_total));
    assert_eq!(contract.total_voting_power, page_total);
}

#[test]
#[should_panic(expected = "Only the operator can call this function.")]
fn only_operator_can_set_total_voting_power() {
    let mut contract = new_contract();
    testing_env!(context("alice.near").build());
    contract.operator_set_total_voting_power(U128(1));
}
