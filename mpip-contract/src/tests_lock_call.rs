use crate::*;
use near_sdk::json_types::{U128, U64};
use near_sdk::test_utils::{get_created_receipts, VMContextBuilder};
use near_sdk::{testing_env, PromiseResult, RuntimeFeesConfig, VMConfig};
use std::collections::HashMap;

const T0: u64 = 10 * 24 * 3600 * 1_000_000_000; // block timestamp in ns

fn acct(name: &str) -> AccountId {
    name.parse().unwrap()
}

fn builder(predecessor: &str) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .current_account_id(acct("mpip.near"))
        .predecessor_account_id(acct(predecessor))
        .block_timestamp(T0);
    builder
}

#[test]
fn voting_locks_the_voter_in_meta_vote_until_the_voting_period_ends() {
    testing_env!(builder("admin.near").build());
    let mut contract = MpipContract::new(
        acct("admin.near"),
        acct("operator.near"),
        acct("meta.near"),
        acct("metavote.near"),
        U64(7 * 24 * 3600 * 1000),
        U128(1),
        U128(0),
        2500,
    );
    contract.internal_create_proposal(
        0,
        "title".to_string(),
        "short".to_string(),
        "body".to_string(),
        "data".to_string(),
        "extra".to_string(),
    );
    testing_env!(
        builder("mpip.near").build(),
        VMConfig::test(),
        RuntimeFeesConfig::test(),
        HashMap::default(),
        vec![PromiseResult::Successful(b"\"1000\"".to_vec())]
    );
    contract.start_voting_period_callback(0);

    testing_env!(builder("alice.near").build());
    contract.vote_proposal(0, VoteType::For, String::new());

    // the function name prints as text, the JSON arguments as a list of byte values
    let receipts = format!("{:?}", get_created_receipts());
    let method = "function_name: \"lock_and_get_mpip_voting_power\"".to_string();
    assert!(receipts.contains(&method));
    let vote_end = T0 / 1_000_000 + 7 * 24 * 3600 * 1000;
    let args = format!(
        "{{\"voter_id\":\"alice.near\",\"lock_until_ms\":\"{}\"}}",
        vote_end
    );
    assert!(receipts.contains(&format!("{:?}", args.as_bytes())));
}
