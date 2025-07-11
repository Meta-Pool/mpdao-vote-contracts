use crate::*;
use near_contract_standards::fungible_token::Balance;
use near_sdk::json_types::U128;
use near_sdk::{env, log, serde_json, PromiseOrValue};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

const E20: Balance = 100_000_000_000_000_000_000;

#[near]
impl FungibleTokenReceiver for MetaVoteContract {
    // receiving mpDAO or stNEAR to distribute
    // verifies the caller is mpdao_token_contract_address or stnear_token_contract_address
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let amount = amount.0;

        // deposit for-claims, msg == "for-claims" means mpDAO to be later distributed to voters
        // it could be stNEAR or mpDAO (checked at fn distribute_for_claims)
        if msg.len() >= 11 && &msg[..11] == "for-claims:" {
            match serde_json::from_str(&msg[11..]) {
                Ok(info) => self.distribute_for_claims(amount, &info),
                Err(_) => panic!("Err parsing msg for-claims"),
            };
        }
        // else, user deposit of mpDAO to bond for x days
        else {
            assert_eq!(
                env::predecessor_account_id(),
                self.mpdao_token_contract_address,
                "You can only bond mpDAO-token, contract:{}",
                self.mpdao_token_contract_address.to_string()
            );
            let (voter_id, days) = if msg.len() >= 1 && &msg[..1] == "[" {
                // deposit & bond for others
                match serde_json::from_str::<(String, u16)>(&msg) {
                    Ok(voter_and_days) => {
                        // sending user wants to lock for others. Normally meta pool DAO granting locked mpDAO to collaborators
                        voter_and_days // assign to voter_id, days
                    }
                    Err(_) => {
                        panic!("Err parsing msg, expected [voter_id,days]")
                    }
                }
            } else {
                // self-deposit & bond
                match msg.parse::<Days>() {
                    Ok(days) => (sender_id.to_string(), days),
                    Err(_) => panic!("Err parsing bonding_days from msg. Must be u16"),
                }
            };

            self.assert_min_deposit_amount(amount);
            log!("DEPOSIT: {} mpDAO deposited from {}", amount, &voter_id,);
            let mut voter = self.internal_get_voter(&voter_id);
            self.deposit_locking_position(amount, days, &voter_id, &mut voter);
        }
        // Return unused amount
        PromiseOrValue::Value(U128::from(0))
    }
}

impl MetaVoteContract {
    // distributes stNEAR or mpDAO between existent voters
    // called from ft_on_transfer
    pub(crate) fn distribute_for_claims(
        &mut self,
        total_amount: u128,
        distribute_info: &Vec<(String, u64)>,
    ) {
        let mut total_distributed = 0;
        let token_address = env::predecessor_account_id();

        // Meta Token
        if token_address == self.mpdao_token_contract_address {
            for item in distribute_info {
                // in case of mpDAO, item.1 is integer mpDAO - mpDAO has 6 decimals
                let amount = item.1 as u128 * 1_000_000;
                self.add_claimable_mpdao(&item.0, amount);
                total_distributed += amount;
            }
            self.accumulated_mpdao_distributed_for_claims += total_distributed;

        // stNear Token
        } else if token_address == self.stnear_token_contract_address {
            for item in distribute_info {
                // in case of stNEAR, item.1 is stNEAR amount * 1e4 (4 decimal places)
                // so we multiply by 1e20 to get yocto-stNEAR
                let amount = item.1 as u128 * E20;
                self.add_claimable_stnear(&item.0, amount);
                total_distributed += amount;
            }
            self.accum_distributed_stnear_for_claims += total_distributed;
        } else {
            panic!("Unknown token address: {}", token_address);
        }
        assert!(
            total_distributed == total_amount,
            "total to distribute {} != total_amount sent {}",
            total_distributed,
            total_amount
        );
    }
}
