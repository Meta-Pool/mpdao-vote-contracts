use crate::*;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::json_types::U64;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{assert_one_yocto, near_bindgen, Promise};

/// for a given token: store amount received and enable/disable in contract's state
#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenInfo {
    pub enabled: bool,         // if token can be used to buy mpDAO
    pub token_decimals: u8,    // e.g. 24 for NEAR, 18 for ETH, 6 for USDT/USDC
    pub amount_received: u128, // amount of tokens received
}
impl TokenInfo {
    pub fn compute_mpdao_amount(&self, token_amount: u128, price: &MpdaoPrice) -> u128 {
        price.assert_not_stale();
        // mpdao_per_token_e9 is in e9, token_amount is in token_decimals, result is in e6 (mpdao has 6 decimals)
        let mpdao_amount = proportional(
            token_amount,
            price.mpdao_per_token_e9 as u128,
            // 9 decimals has the price, 6 has mpdao
            10u128.pow((self.token_decimals + 9 - 6) as u32),
        );
        assert!(mpdao_amount > 0);
        mpdao_amount
    }
}

/// Store price info in contract's state for a given token
#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MpdaoPrice {
    pub mpdao_per_token_e9: u64, // price in mpDAO with 6 decimals, e.g. 1000000 = 1 mpDAO per token
    pub updated_at_ms: u64,      // price last updated timestamp in milliseconds
}
impl MpdaoPrice {
    pub fn assert_not_stale(&self) {
        assert!(
            env::block_timestamp_ms() < self.updated_at_ms + 10 * MINUTES_IN_MS,
            "price is stale"
        );
    }
}

/// items to receive in batch update
#[near_bindgen]
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdatePriceJsonItem {
    pub token_contract: AccountId,
    pub mpdao_per_token_e9: U64,
}

/// this struct can be sent in msg of ft_transfer_call
/// in order to buy & lock [and vote]
#[near_bindgen]
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ReceiveTokenOptions {
    pub days: u16,
    pub beneficiary: Option<String>, // if buy & lock for others
    pub contract_address: Option<ContractAddress>, // if buy, lock & vote
    pub votable_object_id: Option<VotableObjId>, // if buy, lock & vote
}

// internal struct to pass token and amount
pub struct TokenAndAmount {
    pub token: AccountId,
    pub amount: u128,
}

#[near_bindgen]
impl MetaVoteContract {
    // ************************************************************************
    // * Buy & Lock [and vote] mpDAO with other tokens (USDT, USDC, stNEAR)
    // * Sell received tokens to owner
    // ************************************************************************

    /// configure a token that can be used to buy mpDAO
    /// token_decimals: e.g. 24 for NEAR, 18 for ETH, 6 for USDT/USDC
    /// token is disabled by default
    #[payable]
    pub fn set_token_info(&mut self, token_address: &AccountId, token_decimals: u8) {
        self.assert_only_owner();
        // sanity check token_decimals >= 3
        require!(
            token_decimals >= 3,
            "token_decimals should be greater than or equal to 3"
        );
        let amount_received = {
            if let Some(existent) = self.token_info.get(token_address) {
                require!(
                    existent.amount_received == 0 || existent.token_decimals == token_decimals,
                    "token_decimals cannot be changed if amount_received > 0"
                );
                existent.amount_received
            } else {
                // first setup
                0
            }
        };
        self.token_info.insert(
            token_address,
            &TokenInfo {
                enabled: false,
                token_decimals,
                amount_received,
            },
        );
    }

    /// enable/disable a token that can be used to buy mpDAO
    #[payable]
    pub fn enable_token(&mut self, token_address: &AccountId, enabled: bool) {
        self.assert_only_owner();
        if let Some(mut token_info) = self.token_info.get(token_address) {
            token_info.enabled = enabled;
            self.token_info.insert(token_address, &token_info);
        } else {
            env::panic_str("Token not found");
        }
    }

    /// get all token addresses configured
    pub fn get_token_info_keys(&self) -> Vec<AccountId> {
        self.token_info.keys().collect()
    }
    /// get token info and price for a given token address
    pub fn get_token_info(&self, token_address: &AccountId) -> Option<TokenInfo> {
        self.token_info.get(token_address)
    }
    /// get current price for a given token address
    pub fn get_token_price(&self, token_address: &AccountId) -> Option<MpdaoPrice> {
        self.mpdao_prices.get(token_address)
    }

    /// batch update prices for tokens configured
    #[payable]
    pub fn update_mpdao_prices(&mut self, prices: Vec<UpdatePriceJsonItem>) {
        assert_one_yocto();
        self.assert_operator();
        for price in prices {
            // sanity check price > 0
            require!(
                price.mpdao_per_token_e9.0 > 0,
                "mpdao_per_token_e9 should be greater than 0"
            );
            // sanity check token is setup
            let token_info = self.token_info.get(&price.token_contract);
            require!(
                token_info.is_some(),
                format!("token {} is not setup", price.token_contract.as_str())
            );
            self.mpdao_prices.insert(
                &price.token_contract,
                &MpdaoPrice {
                    mpdao_per_token_e9: price.mpdao_per_token_e9.0,
                    updated_at_ms: env::block_timestamp_ms(),
                },
            );
        }
    }

    /// Buy & Lock [and vote] mpDAO with NEAR
    #[payable]
    pub fn buy_lock_and_vote(
        &mut self,
        days: u16,
        contract_address: ContractAddress,
        votable_object_id: VotableObjId,
    ) {
        let options = ReceiveTokenOptions {
            days,
            beneficiary: None,
            contract_address: Some(contract_address),
            votable_object_id: Some(votable_object_id),
        };
        self.receive_sell_lock_and_vote(
            env::predecessor_account_id(),
            TokenAndAmount {
                token: near_as_account_id(),
                amount: env::attached_deposit(),
            },
            options,
        )
    }

    /// buy & lock [and vote] with other tokens
    /// receiving tokens from ft_transfer_call
    pub(crate) fn internal_ft_token_received(
        &mut self,
        sender_id: AccountId,
        token_and_amount: TokenAndAmount,
        msg: String,
    ) {
        assert!(
            // it is not possible, but let's close this route anyway
            token_and_amount.token != near_as_account_id(),
            "impossible: invalid ft: near"
        );

        let options: ReceiveTokenOptions = near_sdk::serde_json::from_str(&msg)
            .unwrap_or_else(|_| env::panic_str("Invalid msg format. Must be JSON."));

        self.receive_sell_lock_and_vote(sender_id, token_and_amount, options)
    }

    // call after receiving payment
    fn receive_sell_lock_and_vote(
        &mut self,
        sender_id: AccountId,
        token_and_amount: TokenAndAmount,
        options: ReceiveTokenOptions,
    ) {
        require!(
            token_and_amount.amount > 0,
            "Amount of tokens sent must be greater than 0"
        );
        // token is setup & enabled?
        let token_info = self.token_info.get(&token_and_amount.token);
        require!(token_info.is_some(), "Token not found");
        let mut token_info = token_info.unwrap();
        require!(token_info.enabled, "Token not enabled");

        // find price for token
        let price = self.mpdao_prices.get(&token_and_amount.token);
        require!(price.is_some(), " Price for token not found");
        let price = price.unwrap();

        // compute how much mpDAO to give for token_and_amount
        let mpdao_amount = token_info.compute_mpdao_amount(token_and_amount.amount, &price);
        require!(mpdao_amount > 0, "mpDAO amount to buy is zero");
        // enough mpDAO to sell?
        require!(
            self.mpdao_avail_to_sell >= mpdao_amount,
            format!(
                "Not enough mpDAO available to sell {}, we have {}",
                mpdao_amount, self.mpdao_avail_to_sell
            )
        );
        // sold
        self.mpdao_avail_to_sell -= mpdao_amount;

        // update received amount for token
        token_info.amount_received += token_and_amount.amount;
        self.token_info.insert(&token_and_amount.token, &token_info);

        // lock for sender or others
        let voter_id = options.beneficiary.unwrap_or(sender_id.to_string());
        let mut voter = self.internal_get_voter(&voter_id);
        self.deposit_locking_position(mpdao_amount, options.days, &voter_id, &mut voter);

        // if it is also a vote command, vote
        let voting_power = utils::calculate_voting_power(mpdao_amount, options.days);
        if let (Some(contract_address), Some(votable_object_id)) =
            (options.contract_address, options.votable_object_id)
        {
            self.internal_vote(
                &voter_id,
                voting_power.into(),
                contract_address,
                votable_object_id,
            );
        }

        log!(
            "buy_lock_and_vote: {} {} mpDAO {} vp",
            voter_id,
            mpdao_amount,
            voting_power
        );
    }

    #[payable]
    pub fn update_mpdao_avail_to_sell(&mut self, mpdao_avail_to_sell: U128String) {
        self.assert_only_owner();
        self.mpdao_avail_to_sell = mpdao_avail_to_sell.0;
    }

    // If extra NEAR balance (from buy_lock_and_vote with NEAR)
    // transfer to owner
    pub fn transfer_extra_near_balance(&mut self) -> U128String {
        let storage_cost = env::storage_usage() as u128 * env::storage_byte_cost();
        let extra_balance = env::account_balance() - storage_cost;
        if extra_balance >= 6 * ONE_NEAR {
            // only if there's more than 6 NEAR to transfer, and leave 5 extra NEAR to backup the storage an extra 500kb
            let extra = extra_balance - 5 * ONE_NEAR;
            // update amount_received for NEAR
            let mut token = self
                .token_info
                .get(&near_as_account_id())
                .expect("NEAR token not configured");
            token.amount_received = token.amount_received.saturating_sub(extra);
            self.token_info.insert(&near_as_account_id(), &token);
            Promise::new(self.owner_id.clone()).transfer(extra);
            extra.into()
        } else {
            0.into()
        }
    }

    #[private]
    pub fn resolve_transfer_received_tokens(&mut self, token_address: &AccountId) {
        let mut token_info = self
            .token_info
            .get(&token_address)
            .expect("Token not found");
        token_info.amount_received = 0;
        self.token_info.insert(&token_address, &token_info);
    }

    // transfer any received tokens (stNEAR, USDT, USDC) to owner
    pub fn transfer_received_tokens(&mut self, token_address: &AccountId) -> Promise {
        self.assert_operator();
        let token_info = self
            .token_info
            .get(&token_address)
            .expect("Token not found");
        require!(
            token_info.amount_received >= 1000,
            "Not enough received tokens to transfer"
        );
        ext_ft_core::ext(token_address.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(
                self.owner_id.clone(),
                token_info.amount_received.into(),
                Some("Transfer received tokens".to_string()),
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .resolve_transfer_received_tokens(token_address),
            )
    }
}
