use crate::*;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;

fn acct(name: &str) -> AccountId {
    name.parse().unwrap()
}

const T0: u64 = 10 * 24 * 3600 * 1_000_000_000;


fn context(predecessor: &str) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(acct("metavote.near"))
        .predecessor_account_id(acct(predecessor))
        .block_timestamp(T0);
    builder
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

fn add_voter_with_lock(contract: &mut MetaVoteContract, id: &str) {
    let voter_id = id.to_string();
    let mut voter = contract.internal_get_voter(&voter_id);
    contract.internal_create_locking_position(&mut voter, 100 * ONE_MPDAO, 30);
    contract.voters.insert(&voter_id, &voter);
}

fn configure_mpip(contract: &mut MetaVoteContract) {
    testing_env!(context("owner.near").attached_deposit(1).build());
    contract.set_mpip_contract_id(acct("mpip.near"));
}

fn mpip_registers_hold(contract: &mut MetaVoteContract, voter: &str, until_ms: u64) -> u128 {
    testing_env!(context("mpip.near").attached_deposit(1).build());
    contract
        .mpip_vote_power_and_hold(voter.to_string(), near_sdk::json_types::U64(until_ms))
        .0
}

#[test]
#[should_panic(expected = "MPIP vote in progress")]
fn cannot_sell_a_position_while_an_mpip_vote_is_in_progress() {
    let mut contract = new_contract();
    add_voter_with_lock(&mut contract, "bob.near");
    configure_mpip(&mut contract);
    let vp = mpip_registers_hold(&mut contract, "bob.near", T0 / 1_000_000 + 7 * 24 * 3600 * 1000);
    assert!(vp > 0);
    testing_env!(context("bob.near").attached_deposit(1).build());
    contract.sell_locking_position(0);
}

#[test]
fn hold_ends_with_the_ballot() {
    let mut contract = new_contract();
    add_voter_with_lock(&mut contract, "bob.near");
    configure_mpip(&mut contract);
    let end = T0 / 1_000_000 + 1000;
    mpip_registers_hold(&mut contract, "bob.near", end);
    assert_eq!(contract.get_mpip_sell_hold("bob.near".to_string()).0, end);
    // after the end the hold check passes (the sale then fails later for pricing reasons, not the hold)
    testing_env!(context("bob.near").block_timestamp((end + 1) * 1_000_000).build());
    MetaVoteContract::assert_not_on_mpip_hold("bob.near");
}

#[test]
fn a_later_shorter_vote_does_not_shorten_the_hold() {
    let mut contract = new_contract();
    add_voter_with_lock(&mut contract, "bob.near");
    configure_mpip(&mut contract);
    mpip_registers_hold(&mut contract, "bob.near", 5_000_000_000_000);
    mpip_registers_hold(&mut contract, "bob.near", 4_000_000_000_000);
    assert_eq!(contract.get_mpip_sell_hold("bob.near".to_string()).0, 5_000_000_000_000);
}

#[test]
fn account_without_mpip_vote_is_not_held() {
    let contract = new_contract();
    assert_eq!(contract.get_mpip_sell_hold("carol.near".to_string()).0, 0);
    testing_env!(context("carol.near").build());
    MetaVoteContract::assert_not_on_mpip_hold("carol.near");
}

#[test]
#[should_panic(expected = "Only the MPIP contract can call this function.")]
fn only_the_mpip_contract_can_register_a_hold() {
    let mut contract = new_contract();
    configure_mpip(&mut contract);
    testing_env!(context("mallory.near").attached_deposit(1).build());
    contract.mpip_vote_power_and_hold("bob.near".to_string(), near_sdk::json_types::U64(u64::MAX));
}

#[test]
#[should_panic(expected = "MPIP contract is not configured")]
fn hold_requires_the_mpip_contract_to_be_configured() {
    let mut contract = new_contract();
    testing_env!(context("mpip.near").attached_deposit(1).build());
    contract.mpip_vote_power_and_hold("bob.near".to_string(), near_sdk::json_types::U64(1));
}
