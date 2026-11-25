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

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::PromiseResult;

fn failed_sale_callback(
    contract: &mut MetaVoteContract,
    locking_days: u16,
    unlocking_started_at: u64,
    voting_power: u128,
) {
    let usdt = acct("usdt.near");
    contract.token_info.insert(
        &usdt,
        &TokenInfo {
            enabled: true,
            token_decimals: 6,
            amount_received: 0,
        },
    );
    testing_env!(
        context("metavote.near").build(),
        near_sdk::VMConfig::test(),
        near_sdk::RuntimeFeesConfig::test(),
        Default::default(),
        vec![PromiseResult::Failed]
    );
    contract.after_sell_locking_position(
        &acct("alice.near"),
        U128(10 * ONE_MPDAO),
        locking_days,
        U128(voting_power),
        unlocking_started_at,
        U128(5_000_000),
        &usdt,
        0,
    );
}

fn fill_all_locking_slots(contract: &mut MetaVoteContract) {
    // 5 slots (max_locking_positions) filled with gifted locks of other durations
    for days in [30u16, 60, 90, 120, 150] {
        testing_env!(context("mpdao.near").build());
        contract.ft_on_transfer(
            acct("gifter.near"),
            U128(MIN_GIFT_LOCK),
            format!("[\"alice.near\",{days}]"),
        );
    }
    assert_eq!(contract.get_all_locking_positions("alice.near".to_string()).len(), 5);
}

#[test]
fn failed_sale_of_locked_position_is_restored_even_if_slots_are_full() {
    let mut contract = new_contract();
    fill_all_locking_slots(&mut contract);
    failed_sale_callback(&mut contract, 200, 0, 0);
    let positions = contract.get_all_locking_positions("alice.near".to_string());
    assert_eq!(positions.len(), 6);
    assert!(positions.iter().any(|p| p.amount.0 == 10 * ONE_MPDAO && p.locking_period == 200));
}

#[test]
fn failed_sale_of_unlocking_position_is_restored_even_if_slots_are_full() {
    let mut contract = new_contract();
    fill_all_locking_slots(&mut contract);
    failed_sale_callback(&mut contract, 200, T0 / 1_000_000, 1);
    let positions = contract.get_all_locking_positions("alice.near".to_string());
    assert_eq!(positions.len(), 6);
}

#[test]
fn failed_sale_merges_into_existing_lock_of_same_duration() {
    let mut contract = new_contract();
    fill_all_locking_slots(&mut contract);
    failed_sale_callback(&mut contract, 60, 0, 0);
    let positions = contract.get_all_locking_positions("alice.near".to_string());
    assert_eq!(positions.len(), 5);
}
