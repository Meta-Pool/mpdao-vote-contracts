#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::{types::*, utils::proportional, E18, ONE_MPDAO};
use near_sdk::{testing_env, Gas, MockedBlockchain, PromiseResult, PublicKey, VMContext};
use near_sdk::{AccountId, NearToken};
use std::convert::TryFrom;

use super::E6;

pub type Balance = u128;
pub const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000;

pub const E24: u128 = ONE_NEAR;
pub const LOCKUP_NEAR: u128 = 1000;
pub const GENESIS_TIME_IN_DAYS: u64 = 500;
pub const YEAR: u64 = 365;
pub const TEST_INITIAL_BALANCE: u128 = 100;

pub const MIN_UNBOND_PERIOD: Days = 30;
pub const MAX_UNBOND_PERIOD: Days = 300;

pub const MIN_DEPOSIT_AMOUNT: u128 = 1 * E6; // 1 mpDao

pub const MAX_LOCKING_POSITIONS: u8 = 20;
pub const MAX_VOTING_POSITIONS: u8 = 100;

/// Returns the system account used for testing purposes.
///
/// # Evolution:
/// Previously, we used `AccountId::new_unchecked(...)`, but this method was removed
/// from the public NEAR SDK API because it allowed constructing account IDs without validation.
/// This was unsafe and discouraged in favor of proper parsing.
///
/// We now use `"account_id".parse::<AccountId>().unwrap()`, which relies on the `FromStr`
/// implementation for `AccountId`. This ensures the account ID string is validated at runtime,
/// which is appropriate for test environments.
///
/// Using `.unwrap()` is acceptable here since we control the input and know it's valid.
/// In production code, prefer handling the potential error explicitly.
///
/// # Example:
/// ```
/// let system = system_account();
/// assert_eq!(system.as_str(), "system.metavote.near");
/// ```
pub fn system_account() -> AccountId {
    "system.metavote.near".parse::<AccountId>().unwrap()
}

pub fn contract_account() -> AccountId {
    "contract.metavote.near".parse::<AccountId>().unwrap()
}

pub fn treasury_account() -> AccountId {
    "treasury.metavote.near".parse::<AccountId>().unwrap()
}

pub fn owner_account() -> AccountId {
    "owner.metavote.near".parse::<AccountId>().unwrap()
}

pub fn non_owner() -> AccountId {
    "non_owner.metavote.near".parse::<AccountId>().unwrap()
}

pub fn developer_account() -> AccountId {
    "developer.metavote.near".parse::<AccountId>().unwrap()
}

pub fn operator_account() -> AccountId {
    "operator.metavote.near".parse::<AccountId>().unwrap()
}

pub fn mpdao_token_account() -> AccountId {
    "mpdao-token.metavote.near".parse::<AccountId>().unwrap()
}

pub fn meta_pool_account() -> AccountId {
    "meta-pool.metavote.near".parse::<AccountId>().unwrap()
}

pub fn voter_account() -> AccountId {
    "voter.metavote.near".parse::<AccountId>().unwrap()
}

/// Returns a synthetic voter account ID for testing, e.g. `voter_acc_1.near`.
///
/// # Notes:
/// This replaces the deprecated `AccountId::new_unvalidated(...)` approach with a
/// validated one using `.parse::<AccountId>()`.
///
/// While `.unwrap()` is acceptable in tests where inputs are controlled, make sure
/// that the resulting account ID string is always valid according to NEAR account naming rules.
pub fn voter_account_id(id: u8) -> AccountId {
    format!("voter_acc_{}.near", id).parse::<AccountId>().unwrap()
}

pub fn compose_account_string(suffix: &str) -> String {
    format!("account_{}.near", suffix)
}

pub fn compose_account(suffix: &str) -> AccountId {
    compose_account_string(suffix).parse::<AccountId>().unwrap()
}
pub fn votable_account() -> AccountId {
    "votable.metavote.near".parse::<AccountId>().unwrap()
}

pub fn ntoy(near_amount: u128) -> u128 {
    return near_amount * 10u128.pow(24);
}

pub fn yton(yoctos_amount: u128) -> f64 {
    return yoctos_amount as f64 / 10u128.pow(24) as f64;
}
//convert yocto to f64 NEAR truncate to 4 dec places
pub fn ytof(yoctos_amount: u128) -> f64 {
    let four_dec_f: f64 = ((yoctos_amount / 10u128.pow(20)) as u32).into();
    return four_dec_f / 10000.0;
}

pub fn to_nanos(num_days: u64) -> u64 {
    return num_days * 86400_000_000_000;
}

#[inline]
pub fn nanos_to_millis(nanoseconds: u64) -> EpochMillis {
    nanoseconds / 1_000_000
}

pub fn to_ts(num_days: u64) -> u64 {
    // 2018-08-01 UTC in nanoseconds
    1533081600_000_000_000 + to_nanos(num_days)
}

pub fn assert_almost_eq_with_max_delta(left: u128, right: u128, max_delta: u128) {
    assert!(
        std::cmp::max(left, right) - std::cmp::min(left, right) < max_delta,
        "Left {} is not even close to Right {} within delta {}",
        left,
        right,
        max_delta
    );
}

pub fn assert_almost_eq(left: u128, right: u128) {
    assert_almost_eq_with_max_delta(left, right, ntoy(10));
}

pub fn get_context(
    predecessor_account_id: &AccountId,
    account_balance: u128,
    account_locked_balance: u128,
    block_timestamp: u64,
) -> VMContext {
    let ed: PublicKey = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp".parse().unwrap();
    let seed: [u8; 32] = [0; 32];
    VMContext {
        current_account_id: contract_account(),
        signer_account_id: predecessor_account_id.clone(),
        signer_account_pk: ed,
        predecessor_account_id: predecessor_account_id.clone(),
        input: vec![],
        block_index: 1,
        block_timestamp,
        epoch_height: 1,
        account_balance: NearToken::from_yoctonear(account_balance),
        account_locked_balance: NearToken::from_yoctonear(account_locked_balance),
        storage_usage: 10u64.pow(6),
        attached_deposit: NearToken::from_yoctonear(0),
        prepaid_gas: Gas::from_gas(10u64.pow(15)),
        random_seed: seed,
        view_config: None,
        output_data_receivers: Vec::new(),
    }
}

pub fn set_context_caller(predecessor_account_id: &AccountId) {
    testing_env!(get_context(
        predecessor_account_id,
        ntoy(TEST_INITIAL_BALANCE),
        0,
        to_ts(GENESIS_TIME_IN_DAYS),
    ));
}

// /// Voting power is proportional to unbond_period
// /// i.e: 30->0.5x, 60(default)->1, 120->2, 180->3, 240->4, 300->5x –Step: 30days
// /// formula for multiplier is: unbond_days/60
// /// formula for voting power is: govTokenLocked * unbond_days / 60
// pub fn calculate_voting_power(mpdao_amount: MpDAOAmount, unbond_days: Days) -> u128 {
//     assert!(mpdao_amount >= ONE_MPDAO); // at least 1 mpDAO, 1_000_0000
//                                         // voting power is u128 with 24 decimals (NEAR standard) and mpdao_amount has 6 decimals
//     let base_vp = mpdao_amount.checked_mul(E18).expect("vp overflow"); // convert to 24 decimals voting power
//     assert!(unbond_days < 3600); // put a limit to unbond_days
//     proportional(base_vp, unbond_days.into(), 60) // apply multiplier
// }
