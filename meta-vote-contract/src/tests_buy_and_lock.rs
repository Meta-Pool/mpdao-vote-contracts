use crate::*;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{testing_env, PromiseResult, RuntimeFeesConfig, VMConfig};
use std::collections::HashMap;

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

fn run_burn_callback(contract: &mut MetaVoteContract, result: PromiseResult, amount: u128) {
    testing_env!(
        context("metavote.near").build(),
        VMConfig::test(),
        RuntimeFeesConfig::test(),
        HashMap::default(),
        vec![result]
    );
    contract.after_burn_sold_mpdao(&acct("alice.near"), U128(amount));
}

#[test]
fn successful_burn_leaves_the_sell_inventory_unchanged() {
    let mut contract = new_contract();
    contract.mpdao_avail_to_sell = 7;
    run_burn_callback(&mut contract, PromiseResult::Successful(vec![]), 1_000_000);
    assert_eq!(contract.mpdao_avail_to_sell, 7);
}

#[test]
fn failed_burn_adds_the_sold_mpdao_to_the_sell_inventory() {
    let mut contract = new_contract();
    contract.mpdao_avail_to_sell = 7;
    run_burn_callback(&mut contract, PromiseResult::Failed, 1_000_000);
    assert_eq!(contract.mpdao_avail_to_sell, 7 + 1_000_000);
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

// ---- security: buy_lock_and_vote beneficiary / voting limit ----

fn contract_with_near_buys() -> MetaVoteContract {
    let mut contract = new_contract();
    testing_env!(context("owner.near").attached_deposit(1).build());
    contract.set_token_info(&near_as_account_id(), 24);
    contract.enable_token(&near_as_account_id(), true);
    contract.mpdao_avail_to_sell = 1_000_000 * ONE_MPDAO;
    testing_env!(context("operator.near").attached_deposit(1).build());
    contract.update_mpdao_prices(vec![buy_and_lock::UpdatePriceJsonItem {
        token_contract: near_as_account_id(),
        mpdao_per_token_e9: near_sdk::json_types::U64(462_768_483_257),
        paused: false,
    }]);
    contract
}

fn buy(
    contract: &mut MetaVoteContract,
    who: &str,
    near: u128,
    beneficiary: Option<&str>,
    addr: Option<&str>,
) {
    testing_env!(context(who).attached_deposit(near).build());
    contract.buy_lock_and_vote(
        30,
        beneficiary.map(|s| s.to_string()),
        addr.map(|s| s.to_string()),
        addr.map(|_| "x".to_string()),
    );
}

#[test]
#[should_panic(expected = "Cannot vote on behalf of another account")]
fn cannot_vote_as_beneficiary() {
    let mut contract = contract_with_near_buys();
    buy(&mut contract, "victim.near", ONE_NEAR, None, None);
    buy(&mut contract, "attacker.near", ONE_NEAR, Some("victim.near"), Some("junk.near"));
}

#[test]
#[should_panic(expected = "Minimum deposit amount")]
fn near_buy_below_min_deposit_rejected() {
    let mut contract = contract_with_near_buys();
    buy(&mut contract, "attacker.near", ONE_NEAR / 100_000, Some("victim.near"), None);
}

#[test]
fn gift_lock_without_vote_still_works() {
    let mut contract = contract_with_near_buys();
    buy(&mut contract, "dao.near", ONE_NEAR, Some("victim.near"), None);
    assert_eq!(contract.internal_get_voter(&"victim.near".to_string()).locking_positions.len(), 1);
}

#[test]
fn full_voter_can_still_add_votes_to_existing_address() {
    let mut contract = contract_with_near_buys(); // max_voting_positions = 5
    buy(&mut contract, "v.near", 10 * ONE_NEAR, None, None);
    testing_env!(context("v.near").build());
    for i in 0..5 {
        contract.vote(U128(ONE_MPDAO), format!("a{}.near", i), "x".to_string());
    }
    // already at the max: more power on an existing address must work
    contract.vote(U128(ONE_MPDAO), "a0.near".to_string(), "x".to_string());
}

#[test]
#[should_panic(expected = "Cannot exceed 5 voting positions")]
fn full_voter_cannot_add_new_address() {
    let mut contract = contract_with_near_buys();
    buy(&mut contract, "v.near", 10 * ONE_NEAR, None, None);
    testing_env!(context("v.near").build());
    for i in 0..6 {
        contract.vote(U128(ONE_MPDAO), format!("a{}.near", i), "x".to_string());
    }
}
