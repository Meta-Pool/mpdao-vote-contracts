use crate::*;
use near_sdk::CryptoHash;

#[inline]
pub fn get_current_epoch_millis() -> EpochMillis {
    env::block_timestamp() / 1_000_000
}

#[inline]
pub fn days_to_millis(days: Days) -> u64 {
    (days as u64) * 24 * 60 * 60 * 1_000
}

#[inline]
pub fn millis_to_days(millis: u64) -> Days {
    (millis / (24 * 60 * 60 * 1_000)) as Days
}

#[inline]
/// returns amount * numerator/denominator
pub fn proportional(amount: u128, numerator: u128, denominator: u128) -> u128 {
    (U256::from(amount) * U256::from(numerator) / U256::from(denominator)).as_u128()
}

pub fn generate_hash_id(id: &String) -> CryptoHash {
    env::keccak256_array(id.as_bytes())
}

pub fn pseudo_near_address(external_address: &String) -> String {
    format!("{}.evmp.near", external_address)
}

pub fn assert_at_least_1_mpdao(mpdao_amount: MpDAOAmount) {
    assert!(
        mpdao_amount >= ONE_MPDAO,
        "amount should be at least 1 mpDAO"
    ); // at least 1 mpDAO
}

/// Voting power is proportional to unbond_period
/// i.e: 30->0.5x, 60(default)->1, 120->2, 180->3, 240->4, 300->5x –Step: 30days
/// formula for multiplier is: unbond_days/60
/// formula for voting power is: govTokenLocked * unbond_days / 60
pub fn calculate_voting_power(mpdao_amount: MpDAOAmount, unbond_days: Days) -> u128 {
    // voting power is u128 with 24 decimals (NEAR standard) and mpdao_amount has 6 decimals
    let base_vp = mpdao_amount.checked_mul(E18).expect("vp overflow"); // convert to 24 decimals voting power
    assert!(unbond_days < 3600); // put a limit to unbond_days
    proportional(base_vp, unbond_days.into(), 60) // apply multiplier
}
