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
#[should_panic(expected = "Unbound period must be between")]
fn extend_beyond_max_unbond_period_panics() {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    testing_env!(context("alice.near").build());
    contract.locking_position_extend_days(0, 301);
}

#[test]
fn extend_to_max_unbond_period_works() {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    testing_env!(context("alice.near").build());
    let old_vp = crate::utils::calculate_voting_power(100 * ONE_MPDAO, 60);
    let new_vp = crate::utils::calculate_voting_power(100 * ONE_MPDAO, 300);
    let before = contract.total_voting_power;
    contract.locking_position_extend_days(0, 300);
    assert_eq!(contract.total_voting_power, before + new_vp - old_vp);
    let voter = contract.internal_get_voter(&"alice.near".to_string());
    assert_eq!(voter.get_position(0).locking_period, 300);
}
