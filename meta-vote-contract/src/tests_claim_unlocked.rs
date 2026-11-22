use crate::*;
use near_sdk::json_types::U128;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;

fn acct(name: &str) -> AccountId {
    name.parse().unwrap()
}

const T0: u64 = 10 * 24 * 3600 * 1_000_000_000;
const ETH: &str = "aabbccddeeff00112233445566778899aabbccdd";

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

#[test]
fn delegated_claim_unlocked_with_unbond_days_locks_for_the_delegate() {
    let mut contract = new_contract();
    let pseudo_id = utils::pseudo_near_address(&ETH.to_string());
    contract
        .evm_delegates
        .insert(&"alice.near".to_string(), &vec![ETH.to_string()]);
    contract
        .claimable_unlocked_mpdao
        .insert(&pseudo_id, &(10 * ONE_MPDAO));
    contract.total_unclaimed_unlocked_mpdao = 10 * ONE_MPDAO;

    testing_env!(context("alice.near").attached_deposit(1).build());
    contract.delegated_claim_unlocked_mpdao(ETH.to_string(), U128(10 * ONE_MPDAO), Some(30));

    // the lock belongs to alice, not to the uncontrolled pseudo account
    assert_eq!(contract.get_all_locking_positions("alice.near".to_string()).len(), 1);
    assert_eq!(contract.get_all_locking_positions(pseudo_id).len(), 0);
}

#[test]
fn own_claim_unlocked_with_unbond_days_still_locks_for_caller() {
    let mut contract = new_contract();
    contract
        .claimable_unlocked_mpdao
        .insert(&"bob.near".to_string(), &(10 * ONE_MPDAO));
    contract.total_unclaimed_unlocked_mpdao = 10 * ONE_MPDAO;
    testing_env!(context("bob.near").build());
    contract.claim_unlocked_mpdao(U128(10 * ONE_MPDAO), Some(30));
    assert_eq!(contract.get_all_locking_positions("bob.near".to_string()).len(), 1);
}
