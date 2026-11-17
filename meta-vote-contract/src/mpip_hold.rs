use crate::*;
use near_sdk::json_types::U64;
use near_sdk::{assert_one_yocto, near_bindgen};

/// Quick fix for "MPIP vote recycling": a position can be sold for USDT (and its mpDAO burned) while
/// the MPIP ballot it already voted on keeps counting its voting power, so the same stake can vote
/// again through a new account. MPIP now registers a "sell hold" together with every vote, and
/// `sell_locking_position` is refused until the ballot ends.
///
/// The state is kept under raw storage keys so this fix needs no state migration.
const MPIP_CONTRACT_KEY: &[u8] = b"mpip_contract_id";
const MPIP_HOLD_PREFIX: &str = "mpip_hold:";

fn hold_key(voter_id: &str) -> Vec<u8> {
    format!("{}{}", MPIP_HOLD_PREFIX, voter_id).into_bytes()
}

impl MetaVoteContract {
    pub(crate) fn mpip_sell_hold_until_ms(voter_id: &str) -> u64 {
        env::storage_read(&hold_key(voter_id))
            .map(|bytes| u64::from_le_bytes(bytes.try_into().expect("bad hold value")))
            .unwrap_or(0)
    }

    pub(crate) fn assert_not_on_mpip_hold(voter_id: &str) {
        let until = Self::mpip_sell_hold_until_ms(voter_id);
        require!(
            env::block_timestamp_ms() >= until,
            format!(
                "cannot sell: the account has an MPIP vote in progress until timestamp_ms {}",
                until
            )
        );
    }
}

#[near_bindgen]
impl MetaVoteContract {
    /// owner only: the MPIP contract allowed to register sell holds
    #[payable]
    pub fn set_mpip_contract_id(&mut self, mpip_contract_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        env::storage_write(MPIP_CONTRACT_KEY, mpip_contract_id.as_str().as_bytes());
    }

    pub fn get_mpip_contract_id(&self) -> Option<AccountId> {
        env::storage_read(MPIP_CONTRACT_KEY).map(|b| {
            String::from_utf8(b)
                .expect("bad mpip contract id")
                .parse()
                .expect("bad mpip contract id")
        })
    }

    pub fn get_mpip_sell_hold(&self, voter_id: VoterId) -> U64 {
        U64(Self::mpip_sell_hold_until_ms(&voter_id))
    }

    /// Called by the MPIP contract when `voter_id` votes. Registers the hold and returns the
    /// same value as `get_mpip_voting_power`, in the same receipt, so the voting power that is
    /// counted is exactly the one that gets held.
    #[payable]
    pub fn mpip_vote_power_and_hold(
        &mut self,
        voter_id: VoterId,
        hold_until_ms: U64,
    ) -> U128String {
        assert_one_yocto();
        let mpip = self.get_mpip_contract_id();
        require!(mpip.is_some(), "MPIP contract is not configured");
        require!(
            Some(env::predecessor_account_id()) == mpip,
            "Only the MPIP contract can call this function."
        );
        let current = Self::mpip_sell_hold_until_ms(&voter_id);
        if hold_until_ms.0 > current {
            env::storage_write(&hold_key(&voter_id), &hold_until_ms.0.to_le_bytes());
        }
        self.get_mpip_voting_power(voter_id)
    }
}
