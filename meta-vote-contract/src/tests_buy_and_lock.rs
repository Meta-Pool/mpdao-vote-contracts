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

use near_sdk::{PromiseResult, RuntimeFeesConfig, VMConfig};
use std::collections::HashMap;

fn run_callback(contract: &mut MetaVoteContract, result: PromiseResult, sent: u128) {
    testing_env!(
        context("metavote.near").build(),
        VMConfig::test(),
        RuntimeFeesConfig::test(),
        HashMap::default(),
        vec![result]
    );
    contract.resolve_transfer_received_tokens(&acct("usdt.near"), U128(sent));
}

fn contract_with_received(amount: u128) -> MetaVoteContract {
    let mut contract = new_contract();
    contract.token_info.insert(
        &acct("usdt.near"),
        &TokenInfo {
            enabled: true,
            token_decimals: 6,
            amount_received: amount,
        },
    );
    contract
}

#[test]
fn failed_transfer_keeps_amount_received() {
    let mut contract = contract_with_received(5_000);
    run_callback(&mut contract, PromiseResult::Failed, 5_000);
    assert_eq!(
        contract.token_info.get(&acct("usdt.near")).unwrap().amount_received,
        5_000
    );
}

#[test]
fn successful_transfer_subtracts_only_the_amount_sent() {
    // 2_000 were credited by a new buy while the transfer of 5_000 was in flight
    let mut contract = contract_with_received(7_000);
    run_callback(&mut contract, PromiseResult::Successful(vec![]), 5_000);
    assert_eq!(
        contract.token_info.get(&acct("usdt.near")).unwrap().amount_received,
        2_000
    );
}
