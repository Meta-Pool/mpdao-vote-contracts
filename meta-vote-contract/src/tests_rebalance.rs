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

/// alice already has `amount` vp delegated to bob. Built through the internal writer because
/// the public vote path no longer accepts new delegations.
fn setup_delegation() -> (MetaVoteContract, u128) {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    add_voter(&mut contract, "bob.near");
    let voter_id = "alice.near".to_string();
    let mut alice = contract.internal_get_voter(&voter_id);
    let amount = 40 * 10u128.pow(24);
    let delegate = "bob.near".to_string();
    let contract_address = delegated();
    contract.internal_create_voting_position(
        &voter_id,
        &mut alice,
        amount,
        &contract_address,
        &delegate,
    );
    contract.voters.insert(&voter_id, &alice);
    (contract, amount)
}

#[test]
#[should_panic(expected = "Vote delegation is deprecated")]
fn new_vote_delegation_is_rejected() {
    let mut contract = new_contract();
    add_voter(&mut contract, "alice.near");
    add_voter(&mut contract, "bob.near");
    testing_env!(context("alice.near").build());
    contract.vote(U128(10u128.pow(24)), delegated(), "bob.near".to_string());
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
#[should_panic(expected = "Vote delegation is deprecated")]
fn rebalance_up_is_rejected() {
    let (mut contract, amount) = setup_delegation();
    testing_env!(context_at("alice.near", T0 + DAY_NS).build());
    contract.rebalance(U128(amount + 10u128.pow(24)), delegated(), "bob.near".to_string());
}

#[test]
fn operator_can_force_undelegate_before_7_days() {
    let (mut contract, _) = setup_delegation();
    testing_env!(context_at("operator.near", T0 + DAY_NS).build());
    let removed = contract.operator_force_undelegate(vec![(
        "alice.near".to_string(),
        "bob.near".to_string(),
    )]);
    assert_eq!(removed, 1);
    let alice = contract.internal_get_voter(&"alice.near".to_string());
    assert_eq!(alice.sum_delegated_away_vp(), 0);
    assert_eq!(
        contract.internal_get_delegated_vp(&"bob.near".to_string()),
        0
    );
}

#[test]
#[should_panic(expected = "Only the owner or the operator can call this function.")]
fn delegator_cannot_force_undelegate() {
    let (mut contract, _) = setup_delegation();
    testing_env!(context_at("alice.near", T0 + DAY_NS).build());
    contract.operator_force_undelegate(vec![(
        "alice.near".to_string(),
        "bob.near".to_string(),
    )]);
}

// ---- vote freeze (lock_votes_in) must hold for partial decreases too ----

const GRANTS: &str = "initiatives";
const FROZEN_OBJ: &str = "grant #11 final";
const OTHER_OBJ: &str = "grant #12 final";

fn contract_with_frozen_vote(freeze_end_ms: u64) -> (MetaVoteContract, u128) {
    let mut contract = new_contract();
    let voter_id = "carol.near".to_string();
    let mut voter = contract.internal_get_voter(&voter_id);
    contract.internal_create_locking_position(&mut voter, 100 * ONE_MPDAO, 60);
    contract.voters.insert(&voter_id, &voter);
    let power = contract.internal_get_voter(&voter_id).available_voting_power;
    testing_env!(context("carol.near").build());
    contract.vote(U128(power / 2), GRANTS.to_string(), FROZEN_OBJ.to_string());
    contract.vote(U128(power - power / 2), GRANTS.to_string(), OTHER_OBJ.to_string());
    testing_env!(context("operator.near").build());
    contract.set_lock_in_vote_filters(freeze_end_ms, 11, Some(GRANTS.to_string()));
    (contract, power)
}

fn frozen_until() -> u64 {
    T0 / 1_000_000 + 20 * 24 * 3600 * 1000
}

#[test]
#[should_panic(expected = "you can not remove votes here until timestamp_ms")]
fn unvote_is_blocked_by_the_freeze() {
    let (mut contract, _) = contract_with_frozen_vote(frozen_until());
    testing_env!(context("carol.near").build());
    contract.unvote(GRANTS.to_string(), FROZEN_OBJ.to_string());
}

#[test]
#[should_panic(expected = "you can not remove votes here until timestamp_ms")]
fn partial_rebalance_decrease_is_blocked_by_the_freeze() {
    let (mut contract, _) = contract_with_frozen_vote(frozen_until());
    testing_env!(context("carol.near").build());
    contract.rebalance(U128(1), GRANTS.to_string(), FROZEN_OBJ.to_string());
}

#[test]
#[should_panic(expected = "you can not remove votes here until timestamp_ms")]
fn rebalance_to_zero_is_blocked_by_the_freeze() {
    let (mut contract, _) = contract_with_frozen_vote(frozen_until());
    testing_env!(context("carol.near").build());
    contract.rebalance(U128(0), GRANTS.to_string(), FROZEN_OBJ.to_string());
}

#[test]
fn rebalance_decrease_on_another_object_is_not_affected_by_the_freeze() {
    let (mut contract, power) = contract_with_frozen_vote(frozen_until());
    testing_env!(context("carol.near").build());
    contract.rebalance(U128(1), GRANTS.to_string(), OTHER_OBJ.to_string());
    assert_eq!(
        contract.internal_get_voter(&"carol.near".to_string()).available_voting_power,
        power - power / 2 - 1
    );
}

/// Legacy self-delegation plus a smaller app vote. Numbers are chosen so the stale
/// reload used to shrink the app vote (and then the outer save put it back).
fn setup_self_delegation() -> (MetaVoteContract, u128, u128, u128) {
    let mut contract = new_contract();
    let voter_id = "alice.near".to_string();
    let mut alice = contract.internal_get_voter(&voter_id);
    contract.internal_create_locking_position(&mut alice, 100 * ONE_MPDAO, 60);
    let locked = alice.available_voting_power;
    contract.voters.insert(&voter_id, &alice);

    // Self-delegation first, while the whole lock is still free. It adds and spends
    // the same power, so available stays at `locked`.
    let self_vp = locked * 60 / 100;
    let mut alice = contract.internal_get_voter(&voter_id);
    let available_before = alice.available_voting_power;
    let contract_address = delegated();
    contract.internal_create_voting_position(
        &voter_id,
        &mut alice,
        self_vp,
        &contract_address,
        &voter_id,
    );
    // Creating the delegation also reloads alice from storage and saves that copy.
    // Put back the voter that holds the new vote, with available unchanged.
    alice.available_voting_power = available_before;
    contract.voters.insert(&voter_id, &alice);

    let app_vp = locked / 2;
    testing_env!(context("alice.near").build());
    contract.vote(U128(app_vp), "app.near".to_string(), "prop".to_string());
    (contract, locked, self_vp, app_vp)
}

#[test]
fn unvoting_a_self_delegation_keeps_other_votes() {
    let (mut contract, _, _, app_vp) = setup_self_delegation();
    let available = contract
        .internal_get_voter(&"alice.near".to_string())
        .available_voting_power;
    testing_env!(context_at("alice.near", T0 + 8 * DAY_NS).build());
    contract.unvote(delegated(), "alice.near".to_string());

    let alice = contract.internal_get_voter(&"alice.near".to_string());
    assert_eq!(alice.sum_delegated_away_vp(), 0);
    assert_eq!(alice.available_voting_power, available);
    assert_eq!(
        contract
            .get_total_votes("app.near".to_string(), "prop".to_string())
            .0,
        app_vp
    );
    // the app vote is still removable; the stale clawback used to make this panic
    contract.unvote("app.near".to_string(), "prop".to_string());
    assert_eq!(
        contract
            .get_total_votes("app.near".to_string(), "prop".to_string())
            .0,
        0
    );
}

#[test]
fn reducing_a_self_delegation_does_not_free_voting_power() {
    let (mut contract, _, self_vp, app_vp) = setup_self_delegation();
    let available = contract
        .internal_get_voter(&"alice.near".to_string())
        .available_voting_power;
    let kept = self_vp / 2;
    testing_env!(context_at("alice.near", T0 + 8 * DAY_NS).build());
    contract.rebalance(U128(kept), delegated(), "alice.near".to_string());

    let alice = contract.internal_get_voter(&"alice.near".to_string());
    assert_eq!(alice.sum_delegated_away_vp(), kept);
    assert_eq!(alice.available_voting_power, available);
    assert_eq!(
        contract.internal_get_delegated_vp(&"alice.near".to_string()),
        kept
    );
    assert_eq!(
        contract
            .get_total_votes("app.near".to_string(), "prop".to_string())
            .0,
        app_vp
    );
}

#[test]
fn rebalance_decrease_works_after_the_freeze_ends() {
    let (mut contract, power) = contract_with_frozen_vote(frozen_until());
    testing_env!(context_at("carol.near", (frozen_until() + 1) * 1_000_000).build());
    contract.rebalance(U128(1), GRANTS.to_string(), FROZEN_OBJ.to_string());
    assert_eq!(
        contract.internal_get_voter(&"carol.near".to_string()).available_voting_power,
        power / 2 - 1
    );
}
