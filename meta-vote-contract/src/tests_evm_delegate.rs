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

fn pre_delegate(contract: &mut MetaVoteContract, account: &str) {
    testing_env!(context(account).attached_deposit(1).build());
    contract.pre_delegate_evm_address(ETH.to_string(), format!("sig-{account}"));
}

#[test]
fn operator_confirms_expected_account() {
    let mut contract = new_contract();
    pre_delegate(&mut contract, "alice.near");
    testing_env!(context("operator.near").attached_deposit(1).build());
    contract.operator_confirm_delegated_evm_address(ETH.to_string(), "alice.near".to_string());
    assert_eq!(
        contract.evm_delegates.get(&"alice.near".to_string()).unwrap(),
        vec![ETH.to_string()]
    );
}

#[test]
#[should_panic(expected = "belongs to mallory.near, expected alice.near")]
fn overwritten_pre_delegation_is_not_confirmed() {
    let mut contract = new_contract();
    pre_delegate(&mut contract, "alice.near");
    // attacker overwrites the entry after the operator verified alice's signature
    pre_delegate(&mut contract, "mallory.near");
    testing_env!(context("operator.near").attached_deposit(1).build());
    contract.operator_confirm_delegated_evm_address(ETH.to_string(), "alice.near".to_string());
}
