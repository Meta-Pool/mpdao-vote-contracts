use near_sdk::{ext_contract, AccountId};
use near_sdk::json_types::U128;

#[allow(dead_code)]
#[ext_contract(ext_ft)]
pub trait FungibleTokenCore {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String
    );

    fn ft_transfer(
        &mut self,
        receiver_id: String,
        amount: U128,
        memo: Option<String>
    );
}

#[allow(dead_code)]
#[ext_contract(ext_self)]
pub trait SelfMetaVote {
    fn after_transfer_mpdao_callback(
        &mut self,
        voter_id: AccountId,
        amount: U128
    );
}
