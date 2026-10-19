use crate::*;
use near_sdk::json_types::U64;

/// Voting power used in an MPIP vote stays locked until the voting period ends. A lock can
/// never be longer than this, whatever the MPIP contract asks for.
pub(crate) const MAX_MPIP_LOCK_MS: u64 = 90 * 24 * 60 * 60 * 1000;

/// Reserved key space inside `timestamp_storage` (no extra storage collection is needed)
const MPIP_LOCK_KEY: &str = "mpip-lock";

impl MetaVoteContract {
    fn mpip_lock_key(voter_id: &String) -> CryptoHash {
        Self::compose_key(voter_id, &MPIP_LOCK_KEY.to_string(), &String::new())
    }

    pub(crate) fn internal_mpip_lock_until(&self, voter_id: &String) -> u64 {
        self.timestamp_storage
            .get(&Self::mpip_lock_key(voter_id))
            .unwrap_or(0)
    }

    /// Voting power that backs an MPIP vote cannot be unlocked, delegated, or taken away from
    /// a delegate while that vote is open. MPIP reads voting power with a one-way call, so
    /// without this lock the same stake could vote again through another account.
    pub(crate) fn assert_not_mpip_locked(&self, voter_id: &String) {
        let lock_until = self.internal_mpip_lock_until(voter_id);
        require!(
            lock_until <= env::block_timestamp_ms(),
            format!(
                "Voting power of {} is locked by an MPIP vote until {} (ms since epoch).",
                voter_id, lock_until
            )
        );
    }
}

#[near_bindgen]
impl MetaVoteContract {
    /// The only contract allowed to lock voting power (see lock_and_get_mpip_voting_power)
    #[payable]
    pub fn set_mpip_contract_id(&mut self, mpip_contract_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.mpip_contract_id = Some(mpip_contract_id);
    }

    /// Called by the MPIP contract when a voter votes. Returns the same value as
    /// `get_mpip_voting_power` and locks that voter until `lock_until_ms` (the end of the
    /// voting period): while locked the voter cannot unlock positions or delegate, and
    /// delegated voting power cannot be taken away from the voter.
    #[payable]
    pub fn lock_and_get_mpip_voting_power(
        &mut self,
        voter_id: VoterId,
        lock_until_ms: U64,
    ) -> U128String {
        assert_one_yocto();
        require!(
            self.mpip_contract_id.as_ref() == Some(&env::predecessor_account_id()),
            "Only the MPIP contract can lock voting power."
        );
        let now = env::block_timestamp_ms();
        let lock_until = std::cmp::min(lock_until_ms.0, now + MAX_MPIP_LOCK_MS);
        let key = Self::mpip_lock_key(&voter_id);
        // a lock is never shortened
        if lock_until > self.timestamp_storage.get(&key).unwrap_or(0) {
            self.timestamp_storage.insert(&key, &lock_until);
        }
        log!("MPIP-LOCK: {} locked until {}", voter_id, lock_until);
        self.get_mpip_voting_power(voter_id)
    }

    /// Until when (ms since epoch) the voter's voting power is locked by an MPIP vote, 0 if never
    pub fn get_mpip_lock_until(&self, voter_id: VoterId) -> U64 {
        U64(self.internal_mpip_lock_until(&voter_id))
    }

    pub fn get_mpip_contract_id(&self) -> Option<AccountId> {
        self.mpip_contract_id.clone()
    }
}
