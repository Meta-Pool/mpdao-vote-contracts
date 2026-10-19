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

use near_sdk::json_types::U64;

const DAY_MS: u64 = 24 * 3600 * 1000;
const DAY_NS: u64 = 24 * 3600 * 1_000_000_000;

fn t0_ms() -> u64 {
    T0 / 1_000_000
}

fn delegated() -> String {
    crate::internal::DELEGATED_CONTRACT_CODE.to_string()
}

fn contract_with_mpip() -> MetaVoteContract {
    let mut contract = new_contract();
    testing_env!(context("owner.near").attached_deposit(1).build());
    contract.set_mpip_contract_id(acct("mpip.near"));
    add_voter(&mut contract, "alice.near");
    add_voter(&mut contract, "bob.near");
    contract
}

/// the MPIP contract locks `voter` until `until_ms` and returns the voting power it reads
fn mpip_lock(contract: &mut MetaVoteContract, voter: &str, until_ms: u64) -> u128 {
    testing_env!(context("mpip.near").attached_deposit(1).build());
    contract
        .lock_and_get_mpip_voting_power(voter.to_string(), U64(until_ms))
        .0
}

/// alice delegates 40e24 voting power to bob at T0
fn alice_delegates_to_bob(contract: &mut MetaVoteContract) {
    testing_env!(context("alice.near").build());
    contract.vote(U128(40 * 10u128.pow(24)), delegated(), "bob.near".to_string());
}

#[test]
#[should_panic(expected = "Only the owner can call this function.")]
fn only_the_owner_sets_the_mpip_contract() {
    let mut contract = new_contract();
    testing_env!(context("alice.near").attached_deposit(1).build());
    contract.set_mpip_contract_id(acct("mpip.near"));
}

#[test]
#[should_panic(expected = "Only the MPIP contract can lock voting power.")]
fn only_the_mpip_contract_can_lock() {
    let mut contract = contract_with_mpip();
    testing_env!(context("attacker.near").attached_deposit(1).build());
    contract.lock_and_get_mpip_voting_power("alice.near".to_string(), U64(t0_ms() + 7 * DAY_MS));
}

#[test]
fn lock_reports_the_mpip_voting_power_and_records_the_lock() {
    let mut contract = contract_with_mpip();
    let until = t0_ms() + 7 * DAY_MS;
    let vp = mpip_lock(&mut contract, "alice.near", until);
    let expected = crate::utils::calculate_voting_power(100 * ONE_MPDAO, 60)
        + crate::utils::calculate_voting_power(100 * ONE_MPDAO, 120);
    assert_eq!(vp, expected);
    assert_eq!(contract.get_mpip_lock_until("alice.near".to_string()).0, until);
    assert_eq!(contract.get_mpip_lock_until("bob.near".to_string()).0, 0);
}

#[test]
fn lock_is_capped_and_never_shortened() {
    let mut contract = contract_with_mpip();
    mpip_lock(&mut contract, "alice.near", t0_ms() + 1000 * DAY_MS);
    assert_eq!(
        contract.get_mpip_lock_until("alice.near".to_string()).0,
        t0_ms() + 90 * DAY_MS
    );
    mpip_lock(&mut contract, "bob.near", t0_ms() + 7 * DAY_MS);
    mpip_lock(&mut contract, "bob.near", t0_ms() + 3 * DAY_MS);
    assert_eq!(
        contract.get_mpip_lock_until("bob.near".to_string()).0,
        t0_ms() + 7 * DAY_MS
    );
}

#[test]
#[should_panic(expected = "locked by an MPIP vote")]
fn voter_cannot_unlock_after_voting() {
    let mut contract = contract_with_mpip();
    mpip_lock(&mut contract, "alice.near", t0_ms() + 7 * DAY_MS);
    testing_env!(context_at("alice.near", T0 + DAY_NS).build());
    contract.unlock_position(0);
}

#[test]
#[should_panic(expected = "locked by an MPIP vote")]
fn voter_cannot_partially_unlock_after_voting() {
    let mut contract = contract_with_mpip();
    mpip_lock(&mut contract, "alice.near", t0_ms() + 7 * DAY_MS);
    testing_env!(context_at("alice.near", T0 + DAY_NS).build());
    contract.unlock_partial_position(0, U128(10 * ONE_MPDAO));
}

#[test]
#[should_panic(expected = "locked by an MPIP vote")]
fn voter_cannot_delegate_after_voting() {
    let mut contract = contract_with_mpip();
    mpip_lock(&mut contract, "alice.near", t0_ms() + 7 * DAY_MS);
    testing_env!(context_at("alice.near", T0 + DAY_NS).build());
    contract.vote(U128(10u128.pow(24)), delegated(), "bob.near".to_string());
}

#[test]
#[should_panic(expected = "locked by an MPIP vote")]
fn delegate_cannot_lose_delegated_power_while_locked() {
    let mut contract = contract_with_mpip();
    alice_delegates_to_bob(&mut contract);
    mpip_lock(&mut contract, "bob.near", t0_ms() + 10 * DAY_MS);
    // the 7-day undelegation wait is over, but bob voted and is still locked
    testing_env!(context_at("alice.near", T0 + 8 * DAY_NS).build());
    contract.unvote(delegated(), "bob.near".to_string());
}

#[test]
fn everything_works_again_after_the_lock_expires() {
    let mut contract = contract_with_mpip();
    alice_delegates_to_bob(&mut contract);
    mpip_lock(&mut contract, "bob.near", t0_ms() + 10 * DAY_MS);
    testing_env!(context_at("alice.near", T0 + 11 * DAY_NS).build());
    contract.unvote(delegated(), "bob.near".to_string());
    contract.unlock_position(0);
    testing_env!(context_at("bob.near", T0 + 11 * DAY_NS).build());
    contract.unlock_position(0);
}

#[test]
fn voters_without_a_lock_are_not_affected() {
    let mut contract = contract_with_mpip();
    testing_env!(context("alice.near").build());
    contract.unlock_position(0);
    contract.unlock_partial_position(1, U128(10 * ONE_MPDAO));
}
