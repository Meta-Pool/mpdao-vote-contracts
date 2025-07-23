use near_sdk::{
    assert_one_yocto, collections::unordered_map::UnorderedMap, env, json_types::U64, near,
    require, AccountId, BorshStorageKey, PanicOnDefault,
};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct KVStoreContract {
    pub owner_id: AccountId,
    pub operator_id: AccountId,
    pub associated_user_data: UnorderedMap<String, String>, // account => encrypted_data
}

#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
pub enum StorageKey {
    AssociatedUserData,
}

#[near]
impl KVStoreContract {
    #[init]
    pub fn new(owner_id: AccountId, operator_id: AccountId) -> Self {
        require!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner_id,
            operator_id,
            associated_user_data: UnorderedMap::new(StorageKey::AssociatedUserData),
        }
    }

    // ***************
    // * owner config
    // ***************
    #[payable]
    pub fn set_operator_id(&mut self, operator_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.operator_id = operator_id;
    }
    #[payable]
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_only_owner();
        self.owner_id = owner_id;
    }

    // ***************
    // * assert
    // ***************
    pub(crate) fn assert_only_owner(&self) {
        require!(
            self.owner_id == env::predecessor_account_id(),
            "Only the owner can call this function."
        );
    }
    pub(crate) fn assert_operator(&self) {
        require!(
            self.operator_id == env::predecessor_account_id(),
            "Only the operator can call this function."
        );
    }

    // ***************
    // * operator set fns
    // ***************

    // save a key=>value
    pub fn set_value(&mut self, key: String, value: String) {
        self.assert_operator();
        self.associated_user_data.insert(&key, &value);
    }
    // send a vec of (key, value) to store
    pub fn set_values(&mut self, kv_array: Vec<(String, String)>) {
        self.assert_operator();
        for kv in kv_array {
            // set associated user data
            self.associated_user_data.insert(&kv.0, &kv.1);
        }
    }
    pub fn remove_value(&mut self, key: String) -> Option<String> {
        self.assert_operator();
        self.associated_user_data.remove(&key)
    }

    /*********************/
    /*   Get functions   */
    /*********************/

    /// get key=>value or default
    pub fn get_value(&self, key: String) -> Option<String> {
        self.associated_user_data.get(&key)
    }

    /// Returns a list of (key,value)
    pub fn get_values(&self, from_index: u32, limit: u32) -> Vec<(String, String)> {
        let keys = self.associated_user_data.keys_as_vector();
        let voters_len = keys.len() as u64;
        let start = from_index as u64;
        let limit = limit as u64;
        let mut results = Vec::<(String, String)>::new();
        for index in start..std::cmp::min(start + limit, voters_len) {
            let key = keys.get(index).unwrap();
            let value = self.associated_user_data.get(&key).unwrap();
            results.push((key.to_string(), value));
        }
        results
    }

    /**********************/
    /*   View functions   */
    /**********************/

    pub fn get_owner_id(&self) -> String {
        self.owner_id.to_string()
    }
    pub fn get_operator_id(&self) -> String {
        self.operator_id.to_string()
    }
    pub fn get_kv_count(&self) -> U64 {
        self.associated_user_data.len().into()
    }
}
