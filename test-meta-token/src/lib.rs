/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::events::{self, FtBurn};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore, resolver::FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{PanicOnDefault};
use near_sdk::collections::LazyOption;
use near_sdk::env::predecessor_account_id;
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, log, near_bindgen, require, AccountId, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
    minters: Vec<AccountId>,
}

const DATA_IMAGE_SVG_ICON: &str = r#"data:image/svg+xml,%3csvg width='96' height='96' viewBox='0 0 96 96' fill='none' xmlns='http://www.w3.org/2000/svg'%3e%3crect width='96' height='96' rx='48' fill='white'/%3e%3cpath d='M29.2241 28.7456C28.396 27.9423 27.0094 28.5289 27.0091 29.6825L27 66.6773C26.9997 67.8501 28.4257 68.4286 29.2426 67.5872L48.6529 47.5943L29.2241 28.7456Z' fill='%23231B51'/%3e%3cpath d='M66.7759 28.7456C67.604 27.9423 68.9906 28.5289 68.9909 29.6825L69 66.6773C69.0003 67.8501 67.5743 68.4286 66.7574 67.5872L47.3471 47.5943L66.7759 28.7456Z' fill='%23231B51'/%3e%3c/svg%3e"#;

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "metapool.app DAO Governance Token".to_string(),
                symbol: "mpDAO".to_string(),
                icon: Some(DATA_IMAGE_SVG_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 6,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        require!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b't'),
            metadata: LazyOption::new(b'm', Some(&metadata)),
            owner_id: owner_id.clone(),
            minters: vec![],
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("initial mint"),
        }
        .emit();
        this
    }

    fn assert_owner_calling(&self) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "can only be called by the owner"
        );
    }
    // returns account ID of the owner.
    pub fn get_owner_id(self) -> AccountId {
        self.owner_id
    }
    #[payable]
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        self.assert_owner_calling();
        assert!(env::is_valid_account_id(owner_id.as_bytes()));
        self.owner_id = owner_id.into();
    }

    // owner can add/remove minters
    #[payable]
    pub fn add_minter(&mut self, account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner_calling();
        if let Some(_) = self.minters.iter().position(|x| *x == account_id) {
            //found
            panic!("already in the list");
        }
        self.minters.push(account_id);
    }

    #[payable]
    pub fn remove_minter(&mut self, account_id: &AccountId) {
        assert_one_yocto();
        self.assert_owner_calling();
        if let Some(inx) = self.minters.iter().position(|x| x == account_id) {
            //found
            let _removed = self.minters.swap_remove(inx);
        } else {
            panic!("not a minter")
        }
    }

    pub fn get_minters(self) -> Vec<AccountId> {
        self.minters
    }

    pub fn assert_minter(&self, account_id: &AccountId) {
        assert!(self.minters.contains(&account_id), "not a minter");
    }

    // minters can mint
    #[payable]
    pub fn ft_mint(&mut self, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        self.assert_minter(&env::predecessor_account_id());
        self.token
            .internal_deposit(&env::predecessor_account_id(), amount.into());
        events::FtMint {
            owner_id: &env::predecessor_account_id(),
            amount: &amount,
            memo: memo.as_deref(),
        }
        .emit();
    }

    // anyone can burn their tokens
    #[payable]
    pub fn ft_burn(&mut self, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        self.token
            .internal_withdraw(&predecessor_account_id(), amount.into());
        FtBurn {
            owner_id: &predecessor_account_id(),
            amount: &amount,
            memo: memo.as_deref(),
        }
        .emit();
    }
}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            log!("Account @{} burned {}", sender_id, burned_amount);
        }
        used_amount.into()
    }
}

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.token.internal_storage_unregister(force) {
            log!("Closed @{} with {}", account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
