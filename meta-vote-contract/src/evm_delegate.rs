use crate::*;
use near_sdk::assert_one_yocto;

#[near]
impl MetaVoteContract {
    // ************************
    // * Delegate EVM address *
    // ************************

    #[payable]
    /// called from the user account, with a ECDSA signature (ethereum signatures & the evm account)
    /// if confirmed by the operator, it moves to "delegated"
    pub fn pre_delegate_evm_address(&mut self, evm_address: String, signature: String) {
        assert_one_yocto();
        // minimal checks to avoid common mistakes (e.g. send with .evmp.near)
        require!(!evm_address.contains("."), "evm_address can not contain dots");
        let account_id = env::predecessor_account_id();
        self.evm_pre_delegation
            .insert(evm_address, (account_id.into(), signature));
    }

    pub fn get_pre_delegate_evm_address(&self, evm_address: String) -> Option<&(String, String)> {
        self.evm_pre_delegation.get(&evm_address)
    }

    #[payable]
    pub fn operator_remove_pre_delegate_evm_address(&mut self, evm_address: String) {
        assert_one_yocto();
        self.assert_operator();
        self.evm_pre_delegation.remove(&evm_address);
    }

    #[payable]
    pub fn operator_confirm_delegated_evm_address(&mut self, evm_address: String) {
        assert_one_yocto();
        self.assert_operator();
        if let Some(pre_delegation) = self.evm_pre_delegation.remove(&evm_address) {
            let account_id = pre_delegation.0;
            let evm_signature = pre_delegation.1;
            if let Some(existing_delegation) = self.evm_delegation_signatures.get(&evm_address) {
                // this evm_address was already delegated
                if existing_delegation.0.eq(&account_id) {
                    // to the the same, nothing to do
                    // just removed the pre-delegation
                    return;
                } else {
                    // it was delegated to another near address, get the list of all delegations for that address
                    let mut previous_delegate_addresses = self.evm_delegates.get(&existing_delegation.0).unwrap();
                    // remove evm_address from the previous account id list
                    previous_delegate_addresses.retain(|x| !x.eq(&evm_address));
                    // update the old delegate addresses list
                    if previous_delegate_addresses.len() == 0 {
                        self.evm_delegates.remove(&existing_delegation.0);
                    } else {
                        self.evm_delegates
                            .insert(&existing_delegation.0, &previous_delegate_addresses);
                    }
                }
            }
            // get current delegations for account_id
            let mut delegated_addresses = self.evm_delegates.get(&account_id).unwrap_or_default();
            // add this one
            delegated_addresses.push(evm_address.clone());
            // save
            self.evm_delegates.insert(&account_id, &delegated_addresses);
            // also save & keep the signature
            self.evm_delegation_signatures
                .insert(evm_address, (account_id, evm_signature));
        }
    }

    #[payable]
    pub fn delegated_claim_stnear(&mut self, evm_address: EvmAddress, amount: U128) -> Promise {
        assert_one_yocto();
        // verify delegation and compose the pseudo near account
        let pseudo_account = self.verify_delegate(&evm_address);
        // claim
        self.claim_stnear_internal(&pseudo_account, &env::predecessor_account_id().to_string(), amount.0)

        // self.remove_claimable_stnear(&pseudo_account, amount.0);
        // // transfer to delegate
        // self.transfer_claimable_stnear_to_receiver(
        //     &pseudo_account,
        //     &env::predecessor_account_id().into(),
        //     amount.0,
        // )
    }

    #[payable]
    pub fn delegated_claim_and_bond_mpdao(&mut self, evm_address: EvmAddress, amount: U128, locking_period: u16) {
        assert_one_yocto();
        // verify delegation and compose the pseudo near account
        let pseudo_account = self.verify_delegate(&evm_address);
        self.claim_and_bond_internal(
            &pseudo_account,
            &env::predecessor_account_id().to_string(),
            amount.0,
            locking_period,
        );
    }

    // local fn: verify delegation and compose pseudo account
    fn verify_delegate(&self, evm_address: &EvmAddress) -> String {
        // get delegations for predecessor_account_id
        let delegations = self
            .evm_delegates
            .get(&env::predecessor_account_id().into())
            .unwrap_or_default();
        // make sure predecessor_account_id() is the delegate
        assert!(
            delegations.contains(&evm_address),
            "{} is not delegated to {}",
            &evm_address,
            &env::predecessor_account_id()
        );
        // compose the pseudo near account
        utils::pseudo_near_address(&evm_address)
    }

    pub fn vote_delegated(
        &mut self,
        evm_address: EvmAddress,
        voting_power: U128,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        // verify delegation and compose the pseudo near account
        let pseudo_account = self.verify_delegate(&evm_address);
        self.internal_vote(&pseudo_account, voting_power, contract_address, votable_object_id)
    }

    pub fn unvote_delegated(
        &mut self,
        evm_address: EvmAddress,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        // verify delegation and compose the pseudo near account
        let pseudo_account = self.verify_delegate(&evm_address);
        self.internal_unvote(&pseudo_account, &contract_address, &votable_object_id)
    }

    #[payable]
    pub fn remove_delegated_evm_address(&mut self, evm_address: String) {
        assert_one_yocto();
        if let Some(existing_delegation) = self.evm_delegation_signatures.get(&evm_address) {
            let predecessor = env::predecessor_account_id().as_str().to_string();
            // this evm_address is delegated
            if existing_delegation.0.eq(&predecessor) {
                // remove from signatures
                self.evm_delegation_signatures.remove(&evm_address);
                // remove from delegate's vector
                let mut delegated_addresses = self.evm_delegates.get(&predecessor).unwrap();
                // remove this one
                delegated_addresses.retain(|x| !x.eq(&evm_address));
                // save
                self.evm_delegates.insert(&predecessor, &delegated_addresses);
            } else {
                env::panic_str("not delegated to you");
            }
        } else {
            env::panic_str("not delegated");
        }
    }

    // --------
    // view fns
    // --------

    /// get delegated evm addresses for a near account
    pub fn get_delegating_evm_addresses(&self, account_id: AccountId) -> Vec<EvmAddress> {
        self.evm_delegates.get(&account_id.into()).unwrap_or_default()
    }

    /// batch get all delegates
    /// get all registered near account delegates and their evm addresses
    pub fn get_delegates(&self, from_index: u32, limit: u32) -> Vec<(String, Vec<EvmAddress>)> {
        let keys = self.evm_delegates.keys_as_vector();
        let keys_len = keys.len() as u32;
        require!(from_index < keys_len, "from_index greater than evm_delegates");
        let after_last = std::cmp::min(from_index + limit, keys_len);

        let mut results = vec![];
        for index in from_index..after_last {
            let account_id = keys.get(index as u64).unwrap();
            let delegated_addresses = self.evm_delegates.get(&account_id).unwrap();
            results.push((account_id, delegated_addresses));
        }
        results
    }

    // return the delegate (near account) for an specific evm address or null
    pub fn get_delegate(&self, evm_address: EvmAddress) -> Option<String> {
        if let Some(delegation) = self.evm_delegation_signatures.get(&evm_address) {
            Some(delegation.0.to_string())
        } else {
            None
        }
    }

    /// returns [near_account, delegation_signature], e.g: ["alice.near”, ”xxxxxxxxxxxxxxxx”]
    /// for external verification of the validity of delegations
    /// The message to validate against the signature is: “delegate to alice.near”
    pub fn get_delegation_signature(&self, evm_address: String) -> &(String, String) {
        self.evm_delegation_signatures.get(&evm_address).unwrap()
    }
}
