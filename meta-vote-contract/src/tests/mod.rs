use super::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::serde_json;
use near_sdk::testing_env;
use near_sdk::{test_utils::VMContextBuilder, AccountId};
mod utils;
use std::str::FromStr;
use utils::*;
const E6: u128 = 1_000_000;
const E20: u128 = 100_000_000_000_000_000_000;
const E24: u128 = 1_000_000_000_000_000_000_000_000;

fn new_metavote_contract() -> MetaVoteContract {
    MetaVoteContract::new(
        owner_account(),
        owner_account(),
        MIN_UNBOND_PERIOD,
        MAX_UNBOND_PERIOD,
        U128::from(MIN_DEPOSIT_AMOUNT),
        MAX_LOCKING_POSITIONS,
        MAX_VOTING_POSITIONS,
        mpdao_token_account(),
        meta_pool_account(),
        U128::from(6_000 * E20), // 0.6 Near
        "prev-gov-token.testnet".into(),
    )
}

fn setup_new_test() -> MetaVoteContract {
    let call_context = get_context(
        &mpdao_token_account(),
        ntoy(TEST_INITIAL_BALANCE),
        0,
        to_ts(GENESIS_TIME_IN_DAYS),
    );
    testing_env!(call_context.clone());
    new_metavote_contract()
}

#[test]
fn test_single_deposit() {
    let mut contract = setup_new_test();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(2 * E6);
    let msg: String = "30".to_owned();

    contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());
    assert_eq!(1, contract.voters.len(), "Voter was not created!");

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(1, voter.locking_positions.len(), "Locking position was not created!");

    let vote_power = calculate_voting_power(MpDAOAmount::from(amount), msg.parse::<Days>().unwrap());
    assert_eq!(
        vote_power, voter.available_voting_power,
        "Incorrect voting power calculation!"
    );

    let voters = contract.get_voters(0, 10);
    assert_eq!(voters.len(), 1);
    let locking_position = &voters.first().unwrap().locking_positions;
    assert_eq!(locking_position.len(), 1);
    let vote_position = &voters.first().unwrap().vote_positions;
    assert_eq!(vote_position.len(), 0);

    assert_eq!(contract.total_voting_power, vote_power);
}

#[test]
fn test_multiple_deposit_same_locking_period() {
    let mut contract = setup_new_test();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(2 * E6);
    let msg: String = "30".to_owned();

    contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());

    let new_amount = U128::from(5 * E6);
    contract.ft_on_transfer(sender_id.clone(), new_amount.clone(), msg.clone());

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(1, voter.locking_positions.len(), "Locking position was not created!");

    let total_vote_power = calculate_voting_power(MpDAOAmount::from(amount.clone()), msg.parse::<Days>().unwrap())
        + calculate_voting_power(MpDAOAmount::from(new_amount.clone()), msg.parse::<Days>().unwrap());

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, to_ts(GENESIS_TIME_IN_DAYS));
    testing_env!(context.clone());
    assert_eq!(
        U128::from(total_vote_power),
        contract.get_available_voting_power(sender_id.as_str().to_string()),
        "Incorrect voting power calculation!"
    );

    let locked_balance = u128::from(amount) + u128::from(new_amount);
    assert_eq!(
        U128::from(locked_balance),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance sum!"
    );
    assert_eq!(
        U128::from(0),
        contract.get_balance(sender_id.as_str().to_string()),
        "Incorrect balance!"
    );

    let voters = contract.get_voters(0, 10);
    assert_eq!(voters.len(), 1);
    let locking_position = &voters.first().unwrap().locking_positions;
    assert_eq!(locking_position.len(), 1);
    let vote_position = &voters.first().unwrap().vote_positions;
    assert_eq!(vote_position.len(), 0);

    assert_eq!(contract.total_voting_power, total_vote_power);
}

#[test]
fn test_multiple_deposit_diff_locking_period() {
    let mut contract = setup_new_test();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(2 * E6);
    let msg: String = "30".to_owned();
    contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());

    let new_amount = U128::from(5 * E6);
    let new_msg: String = "200".to_owned();
    contract.ft_on_transfer(sender_id.clone(), new_amount.clone(), new_msg.clone());

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(2, voter.locking_positions.len(), "Locking position was not created!");

    let total_vote_power = calculate_voting_power(MpDAOAmount::from(amount), msg.parse::<Days>().unwrap())
        + calculate_voting_power(MpDAOAmount::from(new_amount), new_msg.parse::<Days>().unwrap());

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, to_ts(GENESIS_TIME_IN_DAYS));
    testing_env!(context.clone());
    assert_eq!(
        U128::from(total_vote_power),
        contract.get_available_voting_power(sender_id.as_str().to_string()),
        "Incorrect voting power calculation!"
    );

    let locked_balance = u128::from(amount) + u128::from(new_amount);
    assert_eq!(
        U128::from(locked_balance),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance sum!"
    );
    assert_eq!(
        U128::from(0),
        contract.get_balance(sender_id.as_str().to_string()),
        "Incorrect balance!"
    );

    let voters = contract.get_voters(0, 10);
    assert_eq!(voters.len(), 1);
    let locking_position = &voters.first().unwrap().locking_positions;
    assert_eq!(locking_position.len(), 2);
    let vote_position = &voters.first().unwrap().vote_positions;
    assert_eq!(vote_position.len(), 0);

    assert_eq!(contract.total_voting_power, total_vote_power);
}

#[test]
fn test_unlock_position() {
    let mut contract = setup_new_test();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(2 * E6);
    let msg: String = "30".to_owned();
    contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, to_ts(GENESIS_TIME_IN_DAYS));
    testing_env!(context.clone());

    assert_eq!(
        amount,
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        U128::from(0),
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();
    contract.unlock_position(index);
    assert_eq!(1, voter.locking_positions.len(), "Locking position was not created!");

    let unlocking_started_at = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .unlocking_started_at;
    assert!(unlocking_started_at.is_some(), "Position is not unlocked!");
    assert_eq!(
        U128::from(0),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        amount,
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(voter.available_voting_power, 0, "Voting power was not removed!");

    let expected_total_voting_power = 0;
    assert_eq!(contract.total_voting_power, expected_total_voting_power);
}

#[test]
fn test_unlock_partial_position() {
    let mut contract = setup_new_test();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(2 * E6);
    let msg: String = "30".to_owned();
    contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());

    let new_amount = U128::from(5 * E6);
    let new_msg: String = "200".to_owned();
    contract.ft_on_transfer(sender_id.clone(), new_amount.clone(), new_msg.clone());

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, to_ts(GENESIS_TIME_IN_DAYS));
    testing_env!(context.clone());

    let total_amount = U128::from(u128::from(amount) + u128::from(new_amount));
    assert_eq!(
        total_amount,
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        U128::from(0),
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );

    // Partially removing the last (second) locking position.
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .last()
        .unwrap()
        .index
        .unwrap();
    let third_amount = U128::from(4 * E6);
    contract.unlock_partial_position(index, third_amount);
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(3, voter.locking_positions.len(), "Locking position was not created!");

    let unlocking_started_at = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .last()
        .unwrap()
        .unlocking_started_at;
    assert!(unlocking_started_at.is_some(), "Position is not unlocked!");
    let locked_amount = U128::from(u128::from(amount) + u128::from(new_amount) - u128::from(third_amount));
    assert_eq!(
        locked_amount,
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        third_amount,
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let total_vote_power = calculate_voting_power(MpDAOAmount::from(amount), msg.parse::<Days>().unwrap())
        + calculate_voting_power(
            MpDAOAmount::from(new_amount) - MpDAOAmount::from(third_amount),
            new_msg.parse::<Days>().unwrap(),
        );
    assert_eq!(
        voter.available_voting_power, total_vote_power,
        "Voting power was not removed!"
    );

    assert_eq!(contract.total_voting_power, total_vote_power);
}

fn generate_lock_position_context(locking_days: u16, amount: Balance) -> (MetaVoteContract, AccountId) {
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS);

    let context = get_context(&mpdao_token_account(), ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
    testing_env!(context.clone());
    let mut contract = new_metavote_contract();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(amount);
    let msg: String = locking_days.to_string();
    contract.ft_on_transfer(sender_id.clone(), amount, msg);
    (contract, sender_id)
}

fn generate_relock_position_context() -> (MetaVoteContract, AccountId) {
    const LOCKING_PERIOD: u16 = 100;
    const AMOUNT: Balance = 10 * E6;
    let (mut contract, sender_id) = generate_lock_position_context(LOCKING_PERIOD, AMOUNT);
    let timestamp_1 = to_ts(GENESIS_TIME_IN_DAYS + 5);
    let timestamp_2 = to_ts(GENESIS_TIME_IN_DAYS + 5 + LOCKING_PERIOD as u64);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_1);
    testing_env!(context.clone());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();
    contract.unlock_position(index);
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(
        locking_position.unlocking_started_at.unwrap(),
        nanos_to_millis(timestamp_1),
        "Incorrect unlocking started at date."
    );
    assert_eq!(
        locking_position.unlocking_started_at.unwrap() + locking_position.locking_period_millis(),
        nanos_to_millis(timestamp_2),
        "Incorrect unlocking finish date."
    );
    assert_eq!(
        U128::from(0),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        AMOUNT,
        contract.get_unlocking_balance(sender_id.as_str().to_string()).0,
        "Incorrect unlocking balance!"
    );
    (contract, sender_id)
}

#[test]
#[should_panic(expected = "The new locking period should be greater than 88 days.")]
fn test_relock_position_1() {
    let (mut contract, sender_id) = generate_relock_position_context();
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS + 5 + 12);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
    testing_env!(context.clone());
    contract.relock_position(0, 30, U128::from(0));
    // should panic
}

fn prepare_locking_position_extend_days(initial_locking_days: u16, amount: u128) -> (MetaVoteContract, AccountId) {
    let (contract, sender_id) = generate_lock_position_context(initial_locking_days, amount);
    let five_days_after = to_ts(GENESIS_TIME_IN_DAYS + 5);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, five_days_after);
    testing_env!(context.clone());
    (contract, sender_id)
}

fn do_locking_position_extend_days(
    contract: &mut MetaVoteContract,
    sender_id: &AccountId,
    new_locking_days: u16,
    amount: Balance,
) {
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    const INDEX: u32 = 0;
    let locking_position = voter.locking_positions.get(INDEX).unwrap();
    println!("{:?}", locking_position);
    let now = get_current_epoch_millis();
    let unlocking_date = now + locking_position.locking_period_millis();
    let remaining = unlocking_date - now;
    println!("{} {} {}", now, unlocking_date, millis_to_days(remaining),);
    let old_voting_power = locking_position.voting_power;
    let old_total_voting_power = contract.total_voting_power;

    contract.locking_position_extend_days(INDEX, new_locking_days);

    // check
    let voter_new = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(INDEX).unwrap();
    println!("{:?}", locking_position);
    assert_eq!(locking_position.locking_period, new_locking_days);
    assert!(locking_position.is_locked());
    assert_eq!(locking_position.amount, amount);
    let expected_new_voting_power = calculate_voting_power(amount, new_locking_days);
    assert_eq!(locking_position.voting_power, expected_new_voting_power);
    assert_eq!(
        contract.total_voting_power,
        old_total_voting_power - old_voting_power + expected_new_voting_power
    );
    assert_eq!(voter_new.balance, voter.balance);
    assert_eq!(
        voter_new.available_voting_power,
        voter.available_voting_power - old_voting_power + expected_new_voting_power
    );

    let expected_total_voting_power = voter_new.available_voting_power;
    assert_eq!(contract.total_voting_power, expected_total_voting_power);
}

#[test]
fn test_locking_position_extend_days_1() {
    let amount = 10 * E6;
    let (mut contract, sender_id) = prepare_locking_position_extend_days(30, amount);
    do_locking_position_extend_days(&mut contract, &sender_id, 165, amount)
}

#[test]
#[should_panic(expected = "new auto-lock period should be greater than previous one")]
fn test_locking_position_extend_days_fail() {
    let amount = 10 * E6;
    let (mut contract, sender_id) = prepare_locking_position_extend_days(30, amount);
    do_locking_position_extend_days(&mut contract, &sender_id, 29, amount)
}

#[test]
#[should_panic(expected = "position should be locked in order to extend time")]
fn test_locking_position_extend_days_fail_2() {
    let amount = 10 * E6;
    let (mut contract, sender_id) = generate_relock_position_context();
    do_locking_position_extend_days(&mut contract, &sender_id, 60, amount)
}

#[test]
fn test_relock_position_2() {
    let (mut contract, sender_id) = generate_relock_position_context();
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS + 5 + 12);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
    testing_env!(context.clone());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(voter.available_voting_power, 0, "Voting power should be 0.");

    let amount = locking_position.amount;
    let locking_period: Days = 89;
    contract.relock_position(0, locking_period, U128::from(0));

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(
        voter.available_voting_power,
        calculate_voting_power(amount, locking_period),
        "Voting power of Voter is incorrect."
    );
    assert_eq!(
        U128::from(locking_position.amount),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        U128::from(0),
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );
    assert!(
        locking_position.unlocking_started_at.is_none(),
        "Unlocking started should be None."
    );
}

#[test]
fn test_relock_position_3() {
    let (mut contract, sender_id) = generate_relock_position_context();
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS + 5 + 177);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
    testing_env!(context.clone());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(voter.available_voting_power, 0, "Voting power should be 0.");

    let amount = locking_position.amount;
    let locking_period: Days = 30;
    contract.relock_position(0, locking_period, U128::from(0));

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(
        voter.available_voting_power,
        calculate_voting_power(amount, locking_period),
        "Voting power of Voter is incorrect."
    );
    assert_eq!(
        U128::from(locking_position.amount),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        U128::from(0),
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );
    assert!(
        locking_position.unlocking_started_at.is_none(),
        "Unlocking started should be None."
    );
}

#[test]
#[should_panic(expected = "The new locking period should be greater than 77 days.")]
fn test_relock_partial_position_1() {
    let (mut contract, sender_id) = generate_relock_position_context();
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS + 5 + 23);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
    testing_env!(context.clone());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    contract.relock_partial_position(index, U128::from(locking_position.amount - 2 * E6), 30, U128::from(0));
}

#[test]
fn test_relock_partial_position_2() {
    let (mut contract, sender_id) = generate_relock_position_context();
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS + 5 + 23);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
    testing_env!(context.clone());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(voter.available_voting_power, 0, "Voting power should be 0.");

    let keep_amount = 2 * E6;
    let relock_amount = locking_position.amount - keep_amount;
    let locking_period: Days = 89;
    contract.relock_partial_position(index, U128::from(relock_amount), locking_period, U128::from(0));

    // The Unlocking is index 0, and the Relocked is index 1.
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .last()
        .unwrap()
        .index
        .unwrap();
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(voter.locking_positions.len(), 2, "Incorrect locking position.");
    assert_eq!(
        voter.available_voting_power,
        calculate_voting_power(relock_amount, locking_period),
        "Voting power of Voter is incorrect."
    );
    assert_eq!(
        U128::from(relock_amount),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        U128::from(keep_amount),
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );
    assert!(
        locking_position.unlocking_started_at.is_none(),
        "Unlocking started should be None."
    );
}

#[test]
fn test_relock_partial_position_3() {
    let (mut contract, sender_id) = generate_relock_position_context();
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS + 5 + 177);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
    testing_env!(context.clone());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(voter.available_voting_power, 0, "Voting power should be 0.");

    let keep_amount = 2 * E6;
    let relock_amount = locking_position.amount - keep_amount;
    let locking_period: Days = 30;
    contract.relock_partial_position(index, U128::from(relock_amount), locking_period, U128::from(0));

    // The Unlocking is index 0, and the Relocked is ALSO index 0.
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    let locking_position = voter.locking_positions.get(index).unwrap();
    assert_eq!(voter.locking_positions.len(), 1, "Incorrect locking position.");
    assert_eq!(
        voter.available_voting_power,
        calculate_voting_power(relock_amount, locking_period),
        "Voting power of Voter is incorrect."
    );
    assert_eq!(voter.balance, keep_amount, "Voter balance is incorrect.");
    assert_eq!(
        U128::from(keep_amount),
        contract.get_balance(sender_id.as_str().to_string()),
        "Incorrect free balance!"
    );
    assert_eq!(
        U128::from(relock_amount),
        contract.get_locked_balance(sender_id.as_str().to_string()),
        "Incorrect locked balance!"
    );
    assert_eq!(
        U128::from(0),
        contract.get_unlocking_balance(sender_id.as_str().to_string()),
        "Incorrect unlocking balance!"
    );
    assert!(
        locking_position.unlocking_started_at.is_none(),
        "Unlocking started should be None."
    );

    contract.unlock_position(index);
    let locking_period: Days = 38;
    let keep_amount_1 = keep_amount - (1 * E6);
    let keep_amount_2 = keep_amount - keep_amount_1;
    contract.relock_position(index, locking_period, U128::from(keep_amount_1));

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(voter.balance, keep_amount_2, "Incorrect voter balance.");
    assert_eq!(
        voter.available_voting_power,
        calculate_voting_power(relock_amount + keep_amount_1, locking_period),
        "Voting power of Voter is incorrect."
    );

    // Relock from balance
    let locking_period: Days = 278;
    contract.relock_from_balance(locking_period, U128::from(keep_amount_2));
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(voter.balance, 0, "Incorrect voter balance.");
    assert_eq!(
        voter.locking_positions.len(),
        2,
        "Incorrect number of Locking Positions."
    );
}

#[test]
fn test_clear_fully_unlocked_positions() {
    const UNBOND_PERIOD: u16 = 30;
    const MPDAO_AMOUNT: Balance = 2 * E6;
    let (mut contract, sender_id) = generate_lock_position_context(UNBOND_PERIOD, MPDAO_AMOUNT);
    let unlock_started_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 5);
    let clear_positions_timestamp = to_ts(GENESIS_TIME_IN_DAYS + 5 + 60);

    let new_amount = U128::from(5 * E6);
    let new_msg: String = "32".to_owned();
    contract.ft_on_transfer(sender_id.clone(), new_amount.clone(), new_msg.clone());

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(2, voter.locking_positions.len(), "Locking position was not created!");

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, unlock_started_timestamp);
    testing_env!(context.clone());

    contract.unlock_position(0);
    contract.unlock_position(1);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, clear_positions_timestamp);
    testing_env!(context.clone());

    let position_index_list: Vec<PositionIndex> = vec![0, 1];
    contract.clear_locking_position(position_index_list);

    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(0, voter.locking_positions.len(), "Locking position was not deleted!");
    assert_eq!(
        MpDAOAmount::from(MPDAO_AMOUNT) + MpDAOAmount::from(new_amount),
        voter.balance,
        "Incorrect balance!"
    );
    assert_eq!(
        MpDAOAmount::from(MPDAO_AMOUNT) + MpDAOAmount::from(new_amount),
        MpDAOAmount::from(contract.get_balance(sender_id.as_str().to_string())),
        "Incorrect balance!"
    );

    let voters = contract.get_voters(0, 10);
    assert_eq!(voters.len(), 1);
    let locking_position = &voters.first().unwrap().locking_positions;
    assert_eq!(locking_position.len(), 0);
    let vote_position = &voters.first().unwrap().vote_positions;
    assert_eq!(vote_position.len(), 0);
}

#[test]
#[should_panic(expected = "Not enough free voting power to unlock! You have 0, required 20000000000000000000000000.")]
fn test_unlock_position_without_voting_power() {
    const UNBOND_PERIOD: u16 = 120;
    const MPDAO_AMOUNT: Balance = 10 * E6;
    let (mut contract, sender_id) = generate_lock_position_context(UNBOND_PERIOD, MPDAO_AMOUNT);

    let timestamp_1 = to_ts(GENESIS_TIME_IN_DAYS + 5);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_1);
    testing_env!(context.clone());
    let index = contract
        .get_all_locking_positions(sender_id.as_str().to_string())
        .first()
        .unwrap()
        .index
        .unwrap();

    let vote = calculate_voting_power(MpDAOAmount::from(MPDAO_AMOUNT), UNBOND_PERIOD);
    contract.vote(U128::from(vote), votable_account().as_str().to_string(), "0".to_owned());
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(voter.available_voting_power, 0, "Incorrect Voting Power calculation.");
    assert_eq!(
        U128::from(vote),
        contract.get_votes_for_object(
            sender_id.as_str().to_string(),
            votable_account().as_str().to_string(),
            "0".to_owned()
        ),
        "Incorrect votes for votable object"
    );
    contract.unlock_position(index);
}

#[test]
fn test_rebalance_increase_and_decrease() {
    const LOCKING_PERIOD: u16 = 100;
    const AMOUNT: Balance = 10 * E6;
    let (mut contract, sender_id) = generate_lock_position_context(LOCKING_PERIOD, AMOUNT);
    let timestamp_1 = to_ts(GENESIS_TIME_IN_DAYS + 5);

    // New context: the voter is doing the call now!
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_1);
    testing_env!(context.clone());

    let vote = calculate_voting_power(MpDAOAmount::from(AMOUNT), LOCKING_PERIOD);
    let contract_address = votable_account();
    let votable_object_id = "0".to_owned();

    contract.vote(
        U128::from(vote),
        contract_address.as_str().to_string(),
        votable_object_id.clone(),
    );

    // Decrease votes.
    let delta_1 = 5 * E6;
    let decreased_votes = U128::from(vote - delta_1);
    contract.rebalance(
        decreased_votes,
        contract_address.as_str().to_string(),
        votable_object_id.clone(),
    );
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(
        voter.available_voting_power, delta_1,
        "Incorrect Voting Power calculation."
    );

    let votes_for_address =
        voter.get_vote_position_for_address(&sender_id.as_str().to_string(), &contract_address.as_str().to_string());
    let votes = votes_for_address.get(&votable_object_id).unwrap();

    // Increase votes.
    let delta_2 = 1 * E6;
    let additional_votes = U128::from(votes.voting_power + delta_2);
    contract.rebalance(
        additional_votes,
        contract_address.as_str().to_string(),
        votable_object_id.clone(),
    );
    let voter = contract.internal_get_voter(&sender_id.as_str().to_string());
    assert_eq!(
        voter.available_voting_power,
        delta_1 - delta_2,
        "Incorrect Voting Power calculation."
    );

    let votes_for_address =
        voter.get_vote_position_for_address(&sender_id.as_str().to_string(), &contract_address.as_str().to_string());
    let votes = votes_for_address.get(&votable_object_id).unwrap();
    assert_eq!(
        votes.voting_power,
        u128::from(additional_votes),
        "Incorrect Voting Power calculation."
    );

    let voters = contract.get_voters(0, 10);
    assert_eq!(voters.len(), 1);
    let locking_position = &voters.first().unwrap().locking_positions;
    assert_eq!(locking_position.len(), 1);
    let vote_position = &voters.first().unwrap().vote_positions;
    assert_eq!(vote_position.len(), 1);
}

struct User {
    numeric_id: u8,
    locked_mpdao: u128,
    unbond_days: Days,
    contract_address: ContractAddress,
    votable_object_id: VotableObjId,
}
impl User {
    pub fn account_id(&self) -> AccountId {
        voter_account_id(self.numeric_id)
    }
}

fn internal_prepare_multi_voter_contract() -> (MetaVoteContract, Vec<User>) {
    let users = vec![
        User {
            numeric_id: 0,
            locked_mpdao: 10 * E6,
            unbond_days: 30,
            contract_address: compose_account("app_1").as_str().to_string(),
            votable_object_id: "1".to_string(),
        },
        User {
            numeric_id: 1,
            locked_mpdao: 1 * E6,
            unbond_days: 45,
            contract_address: compose_account("app_1").as_str().to_string(),
            votable_object_id: "1".to_string(),
        },
        User {
            numeric_id: 2,
            locked_mpdao: 24 * E6,
            unbond_days: 200,
            contract_address: compose_account("app_1").as_str().to_string(),
            votable_object_id: "2".to_string(),
        },
        User {
            numeric_id: 3,
            locked_mpdao: 8 * E6,
            unbond_days: 300,
            contract_address: compose_account("app_2").as_str().to_string(),
            votable_object_id: "1".to_string(),
        },
    ];

    let mut votes_1_1 = 0_u128;
    let mut votes_1_2 = 0_u128;
    let mut votes_2_1 = 0_u128;
    let timestamp_0 = to_ts(GENESIS_TIME_IN_DAYS);
    let timestamp_1 = to_ts(GENESIS_TIME_IN_DAYS + 5);

    testing_env!(get_context(
        &mpdao_token_account(),
        ntoy(TEST_INITIAL_BALANCE),
        0,
        timestamp_0,
    ));
    let mut contract = new_metavote_contract();

    for user in users.iter() {
        let context = get_context(&mpdao_token_account(), ntoy(TEST_INITIAL_BALANCE), 0, timestamp_0);
        testing_env!(context.clone());

        let sender_id: AccountId = user.account_id();
        let amount = U128::from(user.locked_mpdao);
        let msg: String = user.unbond_days.to_string();
        contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());

        // New context: the voter is doing the call now!
        let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_1);
        testing_env!(context.clone());

        let voting_power = calculate_voting_power(u128::from(amount), user.unbond_days.clone());
        assert_eq!(
            u128::from(contract.get_available_voting_power(sender_id.as_str().to_string())),
            voting_power,
            "Incorrect voting power for user."
        );
        let votes_to_use = user.locked_mpdao;
        let remaining = voting_power - votes_to_use;
        contract.vote(
            U128::from(votes_to_use),
            user.contract_address.clone(),
            user.votable_object_id.clone(),
        );

        if user.numeric_id == 0 || user.numeric_id == 1 {
            votes_1_1 += votes_to_use;
        } else if user.numeric_id == 2 {
            votes_1_2 += votes_to_use;
        } else if user.numeric_id == 3 {
            votes_2_1 += votes_to_use;
        }

        assert_eq!(
            U128::from(votes_to_use),
            contract.get_used_voting_power(sender_id.as_str().to_string()),
            "Incorrect used voting power."
        );
        assert_eq!(
            U128::from(remaining),
            contract.get_available_voting_power(sender_id.as_str().to_string()),
            "Incorrect remaining voting power."
        );
    }

    // Unvote and vote again to test contract total voting consistency.
    let user = users.get(3).unwrap();
    let sender_id: AccountId = user.account_id();
    let context = get_context(&sender_id, ntoy(TEST_INITIAL_BALANCE), 0, timestamp_1);
    testing_env!(context.clone());
    contract.unvote(user.contract_address.clone(), user.votable_object_id.clone());
    contract.vote(
        U128::from(user.locked_mpdao.clone()),
        user.contract_address.clone(),
        user.votable_object_id.clone(),
    );

    assert_eq!(
        contract.get_total_votes(compose_account_string("app_1"), "1".to_string()),
        U128::from(votes_1_1),
        "Incorrect vote count for project 1, object 1."
    );
    assert_eq!(
        contract.get_total_votes(compose_account_string("app_1"), "2".to_string()),
        U128::from(votes_1_2),
        "Incorrect vote count for project 1, object 2."
    );
    assert_eq!(
        contract.get_total_votes(compose_account_string("app_2"), "1".to_string()),
        U128::from(votes_2_1),
        "Incorrect vote count for project 2, object 1."
    );

    let voters = contract.get_voters(0, 10);
    assert_eq!(voters.len(), 4);
    let locking_position = &voters.first().unwrap().locking_positions;
    assert_eq!(locking_position.len(), 1);
    let vote_position = &voters.first().unwrap().vote_positions;
    assert_eq!(vote_position.len(), 1);

    (contract, users)
}

#[test]
fn test_multi_voter_contract() {
    internal_prepare_multi_voter_contract();
}

/// For mpDAO Claims
fn internal_distribute_100_mpdao_for_claims(contract: &mut MetaVoteContract, users: &Vec<User>) {
    let sender_id: AccountId = operator_account();
    let initial_accumulated_distributed = contract.accumulated_mpdao_distributed_for_claims;
    let initial_unclaimed = contract.total_unclaimed_mpdao;
    const AMOUNT: u128 = 100 * E6;
    let mut msg = String::from("for-claims:");
    msg.push_str(
        &serde_json::to_string(&vec![
            (users[0].account_id().to_string(), 10),
            (users[1].account_id().to_string(), 20),
            (users[2].account_id().to_string(), 40),
            (users[3].account_id().to_string(), 30),
        ])
        .unwrap(),
    );

    println!("{}", mpdao_token_account());
    set_context_caller(&mpdao_token_account());
    contract.ft_on_transfer(sender_id, AMOUNT.into(), msg);
    assert_eq!(
        contract.accumulated_mpdao_distributed_for_claims,
        initial_accumulated_distributed + AMOUNT,
        "accumulated_distributed_for_claims not correct"
    );
    assert_eq!(
        contract.total_unclaimed_mpdao,
        initial_unclaimed + AMOUNT,
        "contract.total_unclaimed_meta not correct"
    );
}

/// For stNear Claims
fn internal_distribute_300_stnear_for_claims(contract: &mut MetaVoteContract, users: &Vec<User>) {
    let sender_id: AccountId = operator_account();
    let initial_accumulated_distributed = contract.accum_distributed_stnear_for_claims;
    let initial_unclaimed = contract.total_unclaimed_stnear;
    const AMOUNT: u128 = 3000040 * E20; // 300.0040
    let mut msg = String::from("for-claims:");
    msg.push_str(
        &serde_json::to_string(&vec![
            (users[0].account_id().to_string(), 1500010), // 150.0010
            (users[1].account_id().to_string(), 0500012), // 50.0012
            (users[2].account_id().to_string(), 0800008), // 80.0008
            (users[3].account_id().to_string(), 0200010), // 20.0010
        ])
        .unwrap(),
    );

    set_context_caller(&meta_pool_account());
    contract.ft_on_transfer(sender_id, AMOUNT.into(), msg);
    assert_eq!(
        contract.accum_distributed_stnear_for_claims,
        initial_accumulated_distributed + AMOUNT,
        "accum_distributed_stnear_for_claims not correct"
    );
    assert_eq!(
        contract.total_unclaimed_stnear,
        initial_unclaimed + AMOUNT,
        "contract.total_unclaimed_stnear not correct"
    );
}

#[test]
fn test_deposit_for_claims() {
    let (mut contract, users) = internal_prepare_multi_voter_contract();
    let _ = internal_distribute_100_mpdao_for_claims(&mut contract, &users);
    let _ = internal_distribute_300_stnear_for_claims(&mut contract, &users);
}

#[test]
#[should_panic(expected = "total to distribute 101000000 != total_amount sent 100000000")]
fn distribute_too_much_mpdao() {
    let (mut contract, users) = internal_prepare_multi_voter_contract();
    set_context_caller(&owner_account());
    let amount = 100 * E6;
    let mut msg = String::from("for-claims:");
    msg.push_str(
        &serde_json::to_string(&vec![
            (users[0].account_id().to_string(), 10),
            (users[1].account_id().to_string(), 20),
            (users[2].account_id().to_string(), 40),
            (users[3].account_id().to_string(), 31),
        ])
        .unwrap(),
    );
    set_context_caller(&mpdao_token_account());
    contract.ft_on_transfer(operator_account(), amount.into(), msg);
}

#[test]
#[should_panic(
    expected = "total to distribute 300012900000000000000000000 != total_amount sent 300003000000000000000000000"
)]
fn distribute_too_much_stnear() {
    let (mut contract, users) = internal_prepare_multi_voter_contract();
    set_context_caller(&owner_account());
    const AMOUNT: u128 = 3000030 * E20; // 300.0030
    let mut msg = String::from("for-claims:");
    msg.push_str(
        &serde_json::to_string(&vec![
            (users[0].account_id().to_string(), 1500010), // 150.0010
            (users[1].account_id().to_string(), 0500012), // 50.0012
            (users[2].account_id().to_string(), 0800008), // 80.0008
            (users[3].account_id().to_string(), 0200099), // 20.0099 too much
        ])
        .unwrap(),
    );
    set_context_caller(&meta_pool_account());
    contract.ft_on_transfer(operator_account(), AMOUNT.into(), msg);
}

fn prepare_contract_with_claims() -> (MetaVoteContract, Vec<User>) {
    let (mut contract, users) = internal_prepare_multi_voter_contract();
    internal_distribute_100_mpdao_for_claims(&mut contract, &users);
    internal_distribute_300_stnear_for_claims(&mut contract, &users);
    (contract, users)
}
#[test]
fn test_distribute_claims() {
    prepare_contract_with_claims();
}

#[test]
#[should_panic(expected = "you don't have enough claimable mpDAO")]
fn test_claim_too_much_meta() {
    let (mut contract, users) = prepare_contract_with_claims();
    set_context_caller(&users[2].account_id());
    contract.claim_and_lock((41 * E6).into(), 30);
}

#[test]
#[should_panic(expected = "you don't have enough claimable stNEAR")]
fn test_claim_too_much_stnear() {
    let (mut contract, users) = prepare_contract_with_claims();
    set_context_caller(&users[2].account_id());
    contract.claim_stnear((81 * E24).into());
}

#[test]
fn test_claim_mpdao() {
    let (mut contract, users) = prepare_contract_with_claims();

    // total claim
    {
        let unclaimed_pre = contract.total_unclaimed_mpdao;
        let caller = users[2].account_id();
        // let user_record_pre = contract.get_voter_info(&caller);
        // println!("{:?}", user_record_pre);
        set_context_caller(&caller);
        let claim_balance_pre = contract.get_claimable_mpdao(&caller.as_str().to_string()).0;
        let claim_amount = 40 * E6;
        let duration = 165;
        contract.claim_and_lock(claim_amount.into(), duration);
        assert_eq!(
            contract.total_unclaimed_mpdao,
            unclaimed_pre - claim_amount,
            "total_unclaimed_meta"
        );
        let claim_balance_post = contract.get_claimable_mpdao(&caller.as_str().to_string()).0;
        assert_eq!(claim_balance_post, claim_balance_pre.saturating_sub(claim_amount));
        let user_record_post = contract.get_voter_info(&caller.as_str().to_string());
        // println!("{:?}", user_record_post);
        assert_eq!(user_record_post.locking_positions.len(), 2);
        let pos = &user_record_post.locking_positions[1];
        assert_eq!(pos.locking_period, duration);
        assert_eq!(pos.is_unlocked, false);
        assert_eq!(pos.is_unlocking, false);
        let expected_vp = claim_amount * u128::from(duration) / 60 * E18;
        assert_eq!(pos.voting_power.0, expected_vp);
    }

    // partial claim
    {
        let unclaimed_pre = contract.total_unclaimed_mpdao;
        let caller = users[1].account_id();
        // let user_record_pre = contract.get_voter_info(&caller);
        // println!("{:?}", user_record_pre);
        set_context_caller(&caller);
        let claim_balance_pre = contract.get_claimable_meta(&caller.as_str().to_string()).0;
        let claim_amount = 6 * E6;
        contract.claim_and_lock(claim_amount.into(), 30);
        assert_eq!(
            contract.total_unclaimed_mpdao,
            unclaimed_pre - claim_amount,
            "total_unclaimed_meta"
        );
        // let user_record_post = contract.get_voter_info(&caller);
        // println!("{:?}", user_record_post);
        let claim_balance_post = contract.get_claimable_meta(&caller.as_str().to_string()).0;
        assert_eq!(claim_balance_post, claim_balance_pre - claim_amount);
    }
}

#[test]
fn test_claim_stnear() {
    let (mut contract, users) = prepare_contract_with_claims();

    // total claim
    {
        let unclaimed_pre = contract.total_unclaimed_stnear;
        let caller = users[2].account_id();
        // let user_record_pre = contract.get_voter_info(&caller);
        // println!("{:?}", user_record_pre);
        set_context_caller(&caller);
        let claim_balance_pre = contract.get_claimable_stnear(&caller.as_str().to_string()).0;
        let claim_amount = 800008 * E20;
        contract.claim_stnear(claim_amount.into());
        assert_eq!(
            contract.total_unclaimed_stnear,
            unclaimed_pre - claim_amount,
            "total_unclaimed_stnear"
        );
        let claim_balance_post = contract.get_claimable_stnear(&caller.as_str().to_string()).0;
        assert_eq!(claim_balance_post, claim_balance_pre.saturating_sub(claim_amount));
    }

    // partial claim
    {
        let unclaimed_pre = contract.total_unclaimed_stnear;
        let caller = users[1].account_id();
        // let user_record_pre = contract.get_voter_info(&caller);
        // println!("{:?}", user_record_pre);
        set_context_caller(&caller);
        let claim_balance_pre = contract.get_claimable_stnear(&caller.as_str().to_string()).0;
        let claim_amount = 50 * E6;
        contract.claim_stnear(claim_amount.into());
        assert_eq!(
            contract.total_unclaimed_stnear,
            unclaimed_pre - claim_amount,
            "total_unclaimed_stnear"
        );
        // let user_record_post = contract.get_voter_info(&caller);
        // println!("{:?}", user_record_post);
        let claim_balance_post = contract.get_claimable_stnear(&caller.as_str().to_string()).0;
        assert_eq!(claim_balance_post, claim_balance_pre - claim_amount);
    }
}

#[test]
/// This issue was discovered by Rodrigo.
fn test_transfer_unlock_relock() {
    let mut contract = setup_new_test();

    let sender_id: AccountId = voter_account();
    let amount = U128::from(5 * E6);
    let msg: String = "30".to_owned();

    contract.ft_on_transfer(sender_id.clone(), amount.clone(), msg.clone());

    set_context_caller(&sender_id);
    contract.unlock_partial_position(0, U128::from(3 * E6));

    testing_env!(get_context(
        &sender_id,
        ntoy(TEST_INITIAL_BALANCE),
        0,
        to_ts(GENESIS_TIME_IN_DAYS + 10),
    ));
    contract.relock_partial_position(1, U128::from(2 * E6), 30, U128::from(0));

    // let res = contract.get_all_locking_positions(sender_id);
    let res = contract
        .get_locking_position(1, sender_id.as_str().to_string())
        .unwrap();

    // println!("{:?}", res);
    // println!("{:?}", res.unwrap().amount);

    // let voter = contract.internal_get_staker(sender_id);
    assert_eq!(&res.amount, &U128::from(1 * E6), "res.amount Calculation error");

    assert_eq!(
        &res.voting_power,
        &U128::from(1 * E24 / 2),
        "res.voting_power Calculation error"
    );
}

// NEW TEST

fn ntoy(v: u128) -> u128 {
    v * 10u128.pow(24)
}

fn account(s: &str) -> AccountId {
    AccountId::from_str(s).expect("Invalid NEAR account ID format")
}

fn get_context_2(predecessor: &str, timestamp_ms: u64) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder
        .predecessor_account_id(account(predecessor))
        .block_timestamp(timestamp_ms * 1_000_000);
    builder
}

fn new_contract_with_user(voter_id: &str, available_power: u128) -> MetaVoteContract {
    let mut contract = MetaVoteContract::new(
        account("owner.testnet"),
        account("operator.testnet"),
        30,
        300,
        U128::from(1000000),
        32,
        32,
        account("mpdao-token.testnet"),
        account("meta-v2.pool.testnet"),
        U128::from(100000),
        "metavote.testnet".to_string(),
    );

    let voter_id = voter_id.to_string();
    let mut voter = Voter::new(&voter_id);
    voter.available_voting_power = available_power;

    contract.voters.insert(&voter_id.to_string(), &voter);
    contract
}

#[test]
fn test_rebalance_increase_and_decrease_2() {
    let voter_id = "alice.testnet";
    let contract_address = "initiatives".to_string();
    let object_id = "42|Project X".to_string();

    println!("[INIT] Creando contrato con 1000 VP disponibles");
    let mut contract = new_contract_with_user(voter_id, 1000);

    println!("[STEP] Ejecutando vote(400)");
    testing_env!(get_context_2(voter_id, 1_000_000).build());
    contract.vote(U128(400), contract_address.clone(), object_id.clone());

    let vp1 = contract
        .voters
        .get(&voter_id.to_string())
        .unwrap()
        .available_voting_power;
    println!("[STATE] VP restante después de votar: {}", vp1);

    println!("[STEP] Ejecutando rebalance(600)");
    testing_env!(get_context_2(voter_id, 1_100_000).build());
    contract.rebalance(U128(600), contract_address.clone(), object_id.clone());

    let vp2 = contract
        .voters
        .get(&voter_id.to_string())
        .unwrap()
        .available_voting_power;
    println!("[STATE] VP restante después de rebalance (aumento): {}", vp2);

    println!("[STEP] Ejecutando rebalance(200)");
    contract.rebalance(U128(200), contract_address.clone(), object_id.clone());

    let voter = contract.voters.get(&voter_id.to_string()).unwrap();
    let votes = voter
        .get_vote_position_for_address(&voter_id.to_string(), &contract_address)
        .get(&object_id)
        .unwrap();
    println!("[STATE] VP final del objeto: {}", votes.voting_power);
    println!("[STATE] VP disponible final: {}", voter.available_voting_power);

    assert_eq!(votes.voting_power, 200);
}

#[test]
fn test_revalidate_vote_updates_timestamp() {
    let voter_id = "bob.testnet";
    let contract_address = "initiatives".to_string();
    let object_id = "43|Project Y".to_string();

    println!("[INIT] Creando contrato con 500 VP disponibles");
    let mut contract = new_contract_with_user(voter_id, 500);

    println!("[STEP] Ejecutando vote(500)");
    testing_env!(get_context_2(voter_id, 1_000_000).build());
    contract.vote(U128(500), contract_address.clone(), object_id.clone());

    let original = contract
        .voters
        .get(&voter_id.to_string())
        .unwrap()
        .get_vote_position_for_address(&voter_id.to_string(), &contract_address)
        .get(&object_id)
        .unwrap()
        .created_at;
    println!("[STATE] Timestamp original del voto: {}", original);

    println!("[STEP] Ejecutando revalidate_vote()");
    testing_env!(get_context_2(voter_id, 1_500_000).build());
    contract.revalidate_vote(contract_address.clone(), object_id.clone());

    let updated = contract
        .voters
        .get(&voter_id.to_string())
        .unwrap()
        .get_vote_position_for_address(&voter_id.to_string(), &contract_address)
        .get(&object_id)
        .unwrap()
        .created_at;
    println!("[STATE] Timestamp actualizado: {}", updated);

    assert!(updated > original);
}

#[test]
fn test_purge_votes_by_list_removes_votes() {
    let bot_id = "bot-account.testnet";
    let voter_id = "carol.testnet";
    let contract_address = "initiatives".to_string();
    let object_id = "44|Project Z".to_string();

    println!("[INIT] Creando contrato con 300 VP disponibles");
    let mut contract = new_contract_with_user(voter_id, 300);

    println!("[STEP] Ejecutando vote(300)");
    testing_env!(get_context_2(voter_id, 1_000_000).build());
    contract.vote(U128(300), contract_address.clone(), object_id.clone());

    println!("[STATE] Ejecutando purge_votes_by_list() como bot");
    testing_env!(get_context_2(bot_id, 1_200_000).build());
    contract.purge_votes_by_list(vec![(
        voter_id.to_string(),
        contract_address.clone(),
        object_id.clone(),
    )]);

    let voter = contract.voters.get(&voter_id.to_string()).unwrap();
    let has_votes = voter.vote_positions.get(&contract_address).is_some();
    println!("[RESULT] ¿Aún tiene votos? {}", has_votes);

    assert!(!has_votes);
}
