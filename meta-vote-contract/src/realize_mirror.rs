use crate::*;
use near_sdk::{log, near_bindgen, require};

#[near_bindgen]
impl MetaVoteContract {
    /// True after `realize_mirror` has converted this Ethereum address.
    pub fn is_eth_mirror_realized(&self, external_address: EvmAddress) -> bool {
        self.realized_eth_mirrors
            .get(&external_address)
            .copied()
            .unwrap_or(false)
    }

    /// Operator only. Converts `{eth}.evmp.near` into real locks on `get_delegate`.
    /// The realized mark is stored before the positions are created. A second call reverts.
    /// The holder cannot call this.
    pub fn realize_mirror(&mut self, external_address: EvmAddress) {
        self.assert_operator();
        require!(
            !self.is_eth_mirror_realized(external_address.clone()),
            "eth address already realized"
        );
        let delegate_id = self
            .get_delegate(external_address.clone())
            .expect("no confirmed delegate");
        let pseudo_id = utils::pseudo_near_address(&external_address);

        // Written before the positions are created. A revert rolls the mark back with the call.
        self.realized_eth_mirrors
            .insert(external_address.clone(), true);

        let pseudo = self.internal_get_voter(&pseudo_id);
        let mut snaps: Vec<(u128, Days)> = Vec::new();
        let mut prev_vp: u128 = 0;
        for lp in pseudo.locking_positions.iter() {
            require!(lp.is_locked(), "mirror position is not locked");
            snaps.push((lp.amount, lp.locking_period));
            prev_vp += lp.voting_power;
        }
        require!(!snaps.is_empty(), "nothing to realize");

        let mut vote_snaps: Vec<(String, String, u128, u64)> = Vec::new();
        let addresses: Vec<String> = pseudo.vote_positions.keys_as_vector().to_vec();
        for address in &addresses {
            let pos = pseudo.vote_positions.get(address).unwrap();
            let objects: Vec<String> = pos.keys_as_vector().to_vec();
            for object in objects {
                let power = pos.get(&object).unwrap();
                let timestamp = self.get_vote_timestamp(&pseudo_id, address, &object);
                vote_snaps.push((address.clone(), object, power, timestamp));
            }
        }

        let mut delegate = self.internal_get_voter(&delegate_id);
        for (amount, days) in snaps {
            self.deposit_locking_position(amount, days, &delegate_id, &mut delegate);
        }

        for (address, object, power, timestamp) in vote_snaps {
            let mut dest = delegate.get_vote_position_for_address(&delegate_id, &address);
            let existing = dest.get(&object).unwrap_or(0);
            dest.insert(&object, &(existing + power));
            delegate.vote_positions.insert(&address, &dest);
            if existing == 0 {
                let hash_key = Self::compose_key(&delegate_id, &address, &object);
                self.timestamp_storage.insert(&hash_key, &timestamp);
            }
            self.remove_vote_timestamp(&pseudo_id, &address, &object);
        }
        self.adjust_voter_voting_power(&delegate_id, &mut delegate);
        self.voters.insert(&delegate_id, &delegate);

        let mut pseudo = self.internal_get_voter(&pseudo_id);
        pseudo.locking_positions.clear();
        for address in addresses {
            pseudo.vote_positions.remove(&address);
        }
        pseudo.available_voting_power = 0;
        self.total_voting_power -= prev_vp;
        self.voters.insert(&pseudo_id, &pseudo);

        log!(
            "REALIZE: {} moved onto {} (vp {})",
            pseudo_id,
            delegate_id,
            prev_vp
        );
    }
}
