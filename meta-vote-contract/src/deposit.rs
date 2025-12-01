use crate::buy_and_lock::{ReceiveTokenOptions, TokenAndAmount};
use crate::*;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, near_bindgen, serde_json, PromiseOrValue};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

const E20: Balance = 100_000_000_000_000_000_000;

/// this struct can be sent in msg of ft_transfer_call
/// in order to distribute part locked / part unlocked
#[near_bindgen]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ForClaimsInfo {
    pub bpu: u16,                 //0-10000 basis points to be distributed as unlocked
    pub data: Vec<(String, u64)>, // account, amount pairs
}

#[near_bindgen]
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

        log!(
            "Received FT token: {} {} from {}",
            env::predecessor_account_id(),
            amount,
            sender_id
        );

        // if msg == "for-claims:{bpu:x,[['account',amount],...]}"
        // means tokens to be later distributed to voters (deposit for-claims)
        // it could be stNEAR or mpDAO (checked at fn distribute_for_claims)
        // bpu = basis points unlocked (0-10000), applies to mpDAO only
        if msg.len() >= 11 && &msg[..11] == "for-claims:" {
            match serde_json::from_str::<ForClaimsInfo>(&msg[11..]) {
                Ok(info) => self.distribute_for_claims(amount, info),
                Err(_) => panic!("Err parsing msg for-claims"),
            };
        }
        // if we're receiving mpDAO
        // then it is a deposit & lock [& vote] (for sender or others)
        else if env::predecessor_account_id() == self.mpdao_token_contract_address {
            // lock: user deposit of mpDAO to bond for x days
            self.assert_min_deposit_amount(amount);
            // check msg format:
            // {...} as ReceiveTokenOptions
            // or JSON array: ["voter_id",days]
            // or just number of days as string: "30"
            if msg.len() >= 1 && &msg[..1] == "{" {
                // new format, lock & optionally vote:
                // msg is JSON object ReceiveTokenOptions
                let options: ReceiveTokenOptions = near_sdk::serde_json::from_str(&msg)
                    .unwrap_or_else(|_| {
                        env::panic_str("Invalid msg format. Must be JSON ReceiveTokenOptions")
                    });
                self.lock_and_optionally_vote(sender_id, amount, &options);
            } else {
                // old format:
                // check if msg is JSON array: ["voter_id",days]`
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
                    // assume msg is number of days for lock
                    // self-deposit & bond
                    match msg.parse::<Days>() {
                        Ok(days) => (sender_id.to_string(), days),
                        Err(_) => panic!("Err parsing bonding_days from msg. Must be u16"),
                    }
                };
                // lock mpDAO for signer or others
                let mut voter = self.internal_get_voter(&voter_id);
                self.deposit_locking_position(amount, days, &voter_id, &mut voter);
            }
        } else {
            // receiving other tokens, check for valid buy, lock [and vote] commands
            self.internal_ft_token_received(
                sender_id,
                TokenAndAmount {
                    token: env::predecessor_account_id(),
                    amount,
                },
                msg,
            );
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
        distribute_info: ForClaimsInfo,
    ) {
        // bpu=basis points unlocked. How much is unlocked vs locked
        assert!(distribute_info.bpu <= 10000);
        let mut total_distributed = 0_u128;
        let token_address = env::predecessor_account_id();

        // Meta Token
        if token_address == self.mpdao_token_contract_address {
            for item in &distribute_info.data {
                // in case of mpDAO, item.1 is integer mpDAO - mpDAO has 6 decimals
                let total_mpdao_amount = item.1 as u128 * 1_000_000;
                let unlocked_amount = apply_bp(total_mpdao_amount, distribute_info.bpu);
                let locked_amount = total_mpdao_amount - unlocked_amount;
                if unlocked_amount > 0 {
                    // portion to be distributed as unlocked
                    let existing = self
                        .claimable_unlocked_mpdao
                        .get(&item.0)
                        .unwrap_or_default();
                    self.claimable_unlocked_mpdao
                        .insert(&item.0, &(existing + unlocked_amount));
                    self.total_unclaimed_unlocked_mpdao += unlocked_amount;
                    self.accumulated_unlocked_mpdao_distributed_for_claims += unlocked_amount;
                };
                if locked_amount > 0 {
                    // add locking claim
                    self.add_claimable_mpdao(&item.0, locked_amount);
                }
                total_distributed += total_mpdao_amount;
            }
            self.accumulated_mpdao_distributed_for_claims += total_distributed;

        // stNear Token
        } else if token_address == self.stnear_token_contract_address {
            if distribute_info.bpu != 10000 {
                panic!("stNEAR cannot be distributed as locked yet, bpu must be 100%");
            }
            for item in &distribute_info.data {
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
