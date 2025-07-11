use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

//ARF
//NEAR SDK re-exports its own version of borsh, and by using #[derive(BorshDeserialize)] without specifying #[borsh(crate = "near_sdk::borsh")], the compiler may look for the derive in the global borsh or another version — which breaks the macro.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct LockingPosition {
    pub amount: MpDAOAmount,
    pub locking_period: Days,
    pub voting_power: u128,
    pub unlocking_started_at: Option<EpochMillis>,
}

impl LockingPosition {
    pub(crate) fn locking_period_millis(&self) -> u64 {
        days_to_millis(self.locking_period)
    }

    pub(crate) fn new(
        amount: MpDAOAmount,
        locking_period: Days,
        voting_power: u128,
        unlocking_started_at: Option<EpochMillis>,
    ) -> Self {
        LockingPosition {
            amount,
            locking_period,
            voting_power,
            unlocking_started_at,
        }
    }

    pub(crate) fn is_locked(&self) -> bool {
        self.unlocking_started_at.is_none()
    }

    pub(crate) fn is_unlocking(&self) -> bool {
        match self.unlocking_started_at {
            Some(date) => get_current_epoch_millis() <= (date + self.locking_period_millis()),
            None => false,
        }
    }

    pub(crate) fn is_unlocked(&self) -> bool {
        match self.unlocking_started_at {
            Some(date) => get_current_epoch_millis() > (date + self.locking_period_millis()),
            None => false,
        }
    }

    pub(crate) fn to_json(&self, index: Option<PositionIndex>) -> LockingPositionJSON {
        LockingPositionJSON {
            index,
            amount: U128Json(U128::from(self.amount)),
            locking_period: self.locking_period,
            voting_power: U128Json(U128::from(self.amount)),
            unlocking_started_at: self.unlocking_started_at,
            is_unlocked: self.is_unlocked(),
            is_unlocking: self.is_unlocking(),
            is_locked: self.is_locked(),
        }
    }
}

impl MetaVoteContract {
    fn increase_locking_position(
        &mut self,
        voter: &mut Voter,
        index: u64,
        mpdao_amount: MpDAOAmount,
        unbond_days: Days,
    ) {
        let voting_power = utils::calculate_voting_power(mpdao_amount, unbond_days);
        let mut current_position = voter.get_position(index);
        current_position.amount += mpdao_amount;
        current_position.voting_power += voting_power;

        voter.locking_positions.replace(index, &current_position);
        voter.available_voting_power += voting_power;
        self.total_voting_power += voting_power;
    }

    pub(crate) fn internal_create_locking_position(
        &mut self,
        voter: &mut Voter,
        mpdao_amount: MpDAOAmount,
        unbond_days: Days,
    ) {
        assert!(
            (voter.locking_positions.len() as u8) < self.max_locking_positions,
            "The max number of locking positions is {}",
            self.max_locking_positions
        );
        // double-check it does not exists
        assert!(
            voter.find_locked_position(unbond_days).is_none(),
            "a locking-position for {} days already exists",
            unbond_days
        );
        let voting_power = utils::calculate_voting_power(mpdao_amount, unbond_days);
        let locking_position = LockingPosition::new(mpdao_amount, unbond_days, voting_power, None);
        voter.locking_positions.push(&locking_position);
        voter.available_voting_power += voting_power;
        self.total_voting_power += voting_power;
    }

    pub(crate) fn deposit_locking_position(
        &mut self,
        mpdao_amount: MpDAOAmount,
        unbond_days: Days,
        voter_id: &String,
        voter: &mut Voter,
    ) {
        assert!(
            unbond_days >= self.min_unbond_period && unbond_days <= self.max_unbond_period,
            "Unbound period must be between {} and {} days",
            self.min_unbond_period,
            self.max_unbond_period
        );

        match voter.find_locked_position(unbond_days) {
            Some(index) => {
                // Deposit into existing locking position.
                self.increase_locking_position(voter, index, mpdao_amount, unbond_days);
            }
            None => {
                self.internal_create_locking_position(voter, mpdao_amount, unbond_days);
            }
        };
        self.voters.insert(&voter_id, &voter);
    }

    pub(crate) fn create_unlocking_position(
        &mut self,
        voter: &mut Voter,
        mpdao_amount: MpDAOAmount,
        unbond_days: Days,
        voting_power: u128,
    ) {
        // TODO: you can split this function into increase and create unlocking position
        // to avoid multiple unlocking positions. Or not, be careful with the rounding
        // in the days.
        assert!(
            (voter.locking_positions.len() as u8) < self.max_locking_positions,
            "The max number of locking positions is {}",
            self.max_locking_positions
        );
        let unlocking_position = LockingPosition::new(
            mpdao_amount,
            unbond_days,
            voting_power,
            Some(get_current_epoch_millis()),
        );
        voter.locking_positions.push(&unlocking_position);
    }
}
