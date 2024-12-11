use near_sdk::ONE_NEAR;
// use std::fs;
// use meta_tools::types::{EpochMillis, VaultId};
// use meta_tools::utils::proportional;
// use meta_tools::bond::BondLoaderJSON;
use near_units::parse_near;
use tokio::try_join;
// use json;
use std::{
    str::{self, FromStr}, time::SystemTime
};
// use near_sdk::json_types::{U128, U64};

// use workspaces::network::Sandbox;
use near_gas::*;
use near_workspaces::{
    network::Sandbox,
    types::{KeyType, NearToken, SecretKey},
    Account, AccountId, Contract, DevNetwork, Worker,
};

// use meta_test_utils::now::Now;
// use meta_test_utils::now;

const METAVOTE_FILEPATH: &str = "../res/meta_vote_contract.wasm";
const MPIP_FILEPATH: &str = "../res/mpip_contract.wasm";
const MPDAO_TEST_TOKEN_FILEPATH: &str = "../res/test_meta_token.wasm";

pub const E6: u128 = 1_000_000;
pub const E24: u128 = 1_000_000_000_000_000_000_000_000;

pub const ONE_MPDAO: u128 = 1_000_000;

fn mpdao_as_u128_string(mpdao_amount: u64) -> String {
    format!("{}000000", mpdao_amount)
}
fn micro_stnear_as_u128_string(micro_stnear: u64) -> String {
    format!("{}000000000000000000", micro_stnear)
}

async fn ft_transfer(
    nep_141_contract: &Contract,
    source: &Account,
    receiver: &Account,
    amount_string: &String,
) -> anyhow::Result<()> {
    // send mpdao to voter
    let res = source
        .call(nep_141_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
           "receiver_id": receiver.id(),
            "amount": amount_string
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        panic!(
            "Transfer {} to {} {} err: {:?}\n",
            source.id(),
            receiver.id(),
            amount_string,
            res
        );
    }
    Ok(())
}

async fn ft_balance(nep_141_contract: &Contract, account: &Account) -> anyhow::Result<String> {
    let res = nep_141_contract
        .view("ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": account.id()
        }))
        .await?;
    let res: serde_json::Value = serde_json::from_slice(&res.result)?;
    Ok(String::from(res.as_str().unwrap()))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("init sandbox");
    let worker = near_workspaces::sandbox().await?;

    // Creating Accounts.
    println!("Creating Accounts");
    let owner = create_account(&worker, "owner").await?;
    let voter = create_account(&worker, "voter").await?;
    let third_user = create_account(&worker, "third").await?;
    let buy_vote_user = create_account(&worker, "ape").await?;
    let proposer = create_account(&worker, "proposer").await?;
    let operator = create_account(&worker, "operator").await?;

    ///////////////////////////////////////
    // Stage 1: Deploy relevant contracts
    ///////////////////////////////////////

    let mpdao_token_contract = create_nep141_token(&owner, &worker, "200000000000000").await?;
    let stnear_token_contract =
        create_nep141_token(&owner, &worker, "2000000000000000000000000000").await?;
    let metavote_contract = create_metavote(
        &owner,
        &operator,
        mpdao_token_contract.id(),
        stnear_token_contract.id(),
        &worker,
    )
    .await?;
    let mpip_contract = create_mpip(
        &owner,
        mpdao_token_contract.id(),
        metavote_contract.id(),
        &worker,
    )
    .await?;

    println!("mpDAO token Contract: {}", mpdao_token_contract.id());
    println!("stNEAR token Contract: {}", stnear_token_contract.id());

    println!("Meta vote Contract: {}", metavote_contract.id());
    println!("MPIPs Contract: {}", mpip_contract.id());

    println!("Owner: {}", owner.id());
    println!("Voter: {}", voter.id());
    println!("Proposer: {}", proposer.id());

    register_storage_for(
        &mpdao_token_contract,
        &stnear_token_contract,
        &metavote_contract,
        &mpip_contract,
        &owner,
        &voter,
        &proposer,
    )
    .await?;

    // send some mpdao to voter
    {
        let amount = mpdao_as_u128_string(15);
        ft_transfer(&mpdao_token_contract, &owner, &voter, &amount).await?;
        // check balance
        let balance = ft_balance(&mpdao_token_contract, &voter).await?;
        println!("mpDAO balance of {}: {:?}\n", voter.id(), &balance);
        assert_eq!(balance, amount);
    }

    // send some mpdao to proposer
    {
        let amount = mpdao_as_u128_string(15);
        ft_transfer(&mpdao_token_contract, &owner, &proposer, &amount).await?;
    }

    // get_available_voting_power, must be zero
    let res = metavote_contract
        .view("get_available_voting_power")
        .args_json(serde_json::json!({
            "voter_id": voter.id()
        }))
        .await?;
    let res: serde_json::Value = serde_json::from_slice(&res.result)?;
    assert_eq!(res.as_str().unwrap(), "0");

    // -----------
    {
        // lock mpDAO for others
        let locked_mpdao = 2;
        let days = 240;
        let args = serde_json::json!({
            "receiver_id": metavote_contract.id(),
            "amount": mpdao_as_u128_string(locked_mpdao),
            "msg": format!(r#"["{}",{}]"#,third_user.id(),days)
        });
        let res = owner
            .call(mpdao_token_contract.id(), "ft_transfer_call")
            .args_json(&args)
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            println!(
                "for locking Transfer {} mpdao, days={} ERR: {:?}\n",
                locked_mpdao, days, res
            );
        }

        // verify voting power
        let res = owner
            .call(metavote_contract.id(), "get_available_voting_power")
            .args_json(serde_json::json!({
                "voter_id": third_user.id()
            }))
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        let res = &res.raw_bytes().unwrap().clone();
        let res = str::from_utf8(res).unwrap();
        let res = json::parse(&res)?;
        let vp: u128 = res.to_string().parse().unwrap();
        let expected_vp: u128 = locked_mpdao as u128 * 4 * E24;
        println!("delegate for others vp: {:?}", vp);
        assert_eq!(vp, expected_vp, "Vp <> expected_vp");
    }

    // -----------
    // test buy lock & vote
    // -----------
    {

        let mpdao_per_near = 12 * ONE_NEAR; // price: the user gets 12 mpdao per each NEAR

        let args = serde_json::json!({
            "mpdao_per_near_e24": mpdao_per_near.to_string(),
        });
        let res = operator
            .call(metavote_contract.id(), "update_mpdao_per_near_e24")
            .args_json(&args)
            .gas(NearGas::from_tgas(30))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            println!(
                "update_mpdao_per_near_e24 ERR: {:?}\n",
                res
            );
        }

        let near_amount = 10;
        let locked_mpdao =  near_amount * mpdao_per_near / ONE_NEAR * ONE_MPDAO;
        println!("expected locked_mpdao: {:?}", locked_mpdao);

        let res = owner
            .call(metavote_contract.id(), "update_mpdao_avail_to_sell")
            .args_json(serde_json::json!({
                "mpdao_avail_to_sell": (locked_mpdao -1).to_string(),
            }))
            .gas(NearGas::from_tgas(30))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            println!(
                "update_mpdao_avail_to_sell ERR: {:?}\n",
                res
            );
        }

        let grants8_object_id = "1234|mpDAO Grants #8 - August 2024-asking-user.near";

        let days = 240;
        let args = serde_json::json!({
            "days": days,
            "contract_address": "initiatives",
            "votable_object_id": grants8_object_id
        });

        // should fail
        let res = buy_vote_user
            .call(metavote_contract.id(), "buy_lock_and_vote")
            .args_json(&args)
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_near(near_amount))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() == 0 {
            println!(
                "expected Err:Not enough mpDAO available to sell, but OK: {:?}\n",
                res
            );
        }

        let res = owner
            .call(metavote_contract.id(), "update_mpdao_avail_to_sell")
            .args_json(serde_json::json!({
                "mpdao_avail_to_sell": locked_mpdao.to_string(),
            }))
            .gas(NearGas::from_tgas(30))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            println!(
                "update_mpdao_avail_to_sell ERR: {:?}\n",
                res
            );
        }

        // should work
        let res = buy_vote_user
            .call(metavote_contract.id(), "buy_lock_and_vote")
            .args_json(&args)
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_near(near_amount))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            println!(
                "buy_lock_and_vote ERR: {:?}\n",
                res
            );
        }
        // else {
        //     println!(
        //         "buy_lock_and_vote: {:?}\n",
        //         res
        //     );
        // }
        // verify voting power
        let res = owner
            .call(metavote_contract.id(), "get_used_voting_power")
            .args_json(serde_json::json!({
                "voter_id": buy_vote_user.id()
            }))
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        let res = &res.raw_bytes().unwrap().clone();
        let res = str::from_utf8(res).unwrap();
        let res = json::parse(&res)?;
        let vp: u128 = res.to_string().parse().unwrap();
        let expected_vp: u128 = locked_mpdao * days/60 * E24 / E6;
        println!("buy_lock_and_vote vp: {:?}", vp);
        assert_eq!(vp, expected_vp, "Vp <> expected_vp");

        // unvote and starts unlocking (to not mess with vote proposal test)
        let res = buy_vote_user
            .call(metavote_contract.id(), "unvote")
            .args_json(serde_json::json!({
                "contract_address": "initiatives",
                "votable_object_id": grants8_object_id
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        if res.failures().len() > 0 {
            panic!("unvote ERR: {:?}\n", res);
        }
        let res = buy_vote_user
            .call(metavote_contract.id(), "unlock_position")
            .args_json(serde_json::json!({
                "index": 0
            }))
            .gas(NearGas::from_tgas(200))
            // .deposit(NearToken::from_millinear(3))
            .transact()
            .await?;
        if res.failures().len() > 0 {
            panic!("unlock_position ERR: {:?}\n", res);
        }
    }

    // -----------
    // test lock-in votes filters
    // -----------
    {
        let grants8_object_id = "1234|mpDAO Grants #8 - August 2024-asking-user.near";
        let grants7_object_id = "187|mpDAO Grants #7 - June 2024-asking-user.near";

        // vote in grants #8
        let res = third_user
            .call(metavote_contract.id(), "vote")
            .args_json(serde_json::json!({
                "contract_address": "initiatives",
                "voting_power": mpdao_as_u128_string(200),
                "votable_object_id": grants8_object_id
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            panic!("voting ERR: {:?}\n", res);
        }

        // vote in grants #7
        let res = third_user
            .call(metavote_contract.id(), "vote")
            .args_json(serde_json::json!({
                "contract_address": "initiatives",
                "voting_power": mpdao_as_u128_string(100),
                "votable_object_id": grants7_object_id
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            panic!("voting ERR: {:?}\n", res);
        }

        // lock un-voting
        let now = SystemTime::now();
        let now_unix_ms = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let res = operator
            .call(metavote_contract.id(), "set_lock_in_vote_filters")
            .args_json(serde_json::json!({
                "end_timestamp_ms": now_unix_ms + 1000*60*60,
                "votable_numeric_id": 8,
                "votable_address":"initiatives"
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            panic!("set_lock_in_vote_filters ERR: {:?}\n", res);
        }

        // un-vote should fail
        let res = third_user
            .call(metavote_contract.id(), "unvote")
            .args_json(serde_json::json!({
                "contract_address": "initiatives",
                "votable_object_id": grants8_object_id
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() == 0 {
            panic!("EXPECTING unvote ERR but it was ok: {:?}\n", res);
        }
        let formatted_res = format!("{:?}", res);
        let expected = "you can not remove votes here until timestamp_ms";
        if !formatted_res.contains(expected) {
            panic!("unvote test, expecting '{}' in {}", expected, formatted_res)
        }

        // unvote of the old grant should work
        let res = third_user
            .call(metavote_contract.id(), "unvote")
            .args_json(serde_json::json!({
                "contract_address": "initiatives",
                "votable_object_id": grants7_object_id
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        if res.failures().len() > 0 {
            panic!("unvote old grant 7 ERR: {:?}\n", res);
        }

        // set restrictions as passed
        let res = operator
            .call(metavote_contract.id(), "set_lock_in_vote_filters")
            .args_json(serde_json::json!({
                "end_timestamp_ms": now_unix_ms - 1000,
                "votable_numeric_id": 8,
                "votable_address":"initiatives"
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            panic!("set restrictions as passed ERR: {:?}\n", res);
        }

        // unvote now should work
        let res = third_user
            .call(metavote_contract.id(), "unvote")
            .args_json(serde_json::json!({
                "contract_address": "initiatives",
                "votable_object_id": grants8_object_id
            }))
            .gas(NearGas::from_tgas(200))
            .transact()
            .await?;
        if res.failures().len() > 0 {
            panic!("unvote 2 ERR: {:?}\n", res);
        }
    }

    {
        // send stNEAR to distribute
        let micro_distribute_stnear = 123400;
        let days = 240;
        let args = serde_json::json!({
            "receiver_id": metavote_contract.id(),
            "amount": micro_stnear_as_u128_string(micro_distribute_stnear),
            "msg": format!(r#"for-claims:[["{}",{}]]"#,third_user.id(),1234)
        });
        let res = owner
            .call(stnear_token_contract.id(), "ft_transfer_call")
            .args_json(&args)
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            println!(
                "for-claims Transfer {} stnear, days={} ERR: {:?}\n",
                args, days, res
            );
        }

        // verify claims
        let res = owner
            .call(metavote_contract.id(), "get_claimable_stnear")
            .args_json(serde_json::json!({
                "voter_id": third_user.id()
            }))
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        let res = &res.raw_bytes().unwrap().clone();
        let res = str::from_utf8(res).unwrap();
        let res = json::parse(&res)?;
        let claimable_stnear: u128 = res.to_string().parse().unwrap();
        let expected_stnear: u128 = micro_distribute_stnear as u128 * E24 / 1000000;
        println!("claimable_stnear: {:?}", claimable_stnear);
        assert_eq!(claimable_stnear, expected_stnear, "Vp <> expected_vp");
    }

    // -----------------
    // register_delegate
    // -----------------
    let delegate1 = create_account(&worker, "delegate-1").await?;
    //storage_deposit(&delegate1, &mpdao_token_contract).await?;
    let evm_address1 = "0xSomeEvmAddress1";
    register_delegate(
        &metavote_contract,
        &delegate1,
        &evm_address1.into(),
        &operator,
    )
    .await?;

    // delegator must have it
    let res = get_delegations(&metavote_contract, &delegate1).await?;
    let array = res.as_array().expect("no array");
    assert!(array.len() == 1);
    assert!(array[0] == evm_address1);

    // get signature
    let res = get_delegation_signature(&metavote_contract, &evm_address1.into()).await?;
    assert!(res.eq(&serde_json::json!([
        delegate1.id().to_string(),
        format!("SOME-SIGNATURE-FOR-DELEGATE-TO-{}", delegate1.id()),
    ])));

    // delegator 2, same evmAddress
    let delegate2 = create_account(&worker, "delegate-2").await?;
    //storage_deposit(&delegate2, &mpdao_token_contract).await?;
    // register again with another account, should remove from delegate 1
    register_delegate(
        &metavote_contract,
        &delegate2,
        &evm_address1.into(),
        &operator,
    )
    .await?;

    // old delegator must be empty now
    let res = get_delegations(&metavote_contract, &delegate1).await?;
    assert!(res.as_array().expect("no array").len() == 0);

    // signature must be from new delegator
    let res = get_delegation_signature(&metavote_contract, &evm_address1.into()).await?;
    let expected_signature = format!("SOME-SIGNATURE-FOR-DELEGATE-TO-{}", delegate2.id());
    let expected_json = &serde_json::json!([delegate2.id().to_string(), expected_signature]);
    println!("expected {:?}", expected_json);
    assert!(res.eq(expected_json));

    // new delegator must have it
    let res = get_delegations(&metavote_contract, &delegate2).await?;
    let array = res.as_array().expect("no array");
    assert!(array.len() == 1);
    assert!(array[0] == evm_address1);

    // add another
    let evm_address2 = "0xSomeEvmAddress02";
    register_delegate(
        &metavote_contract,
        &delegate2,
        &evm_address2.into(),
        &operator,
    )
    .await?;

    // new delegator must have it
    let res = get_delegations(&metavote_contract, &delegate2).await?;
    let array = res.as_array().expect("no array");
    assert!(array.len() == 2);
    assert!(array[0] == evm_address1);
    assert!(array[1] == evm_address2);

    // --------------
    // create_proposal
    // --------------
    let res = proposer
        .call(mpip_contract.id(), "create_proposal")
        .args_json(serde_json::json!({
            "title": "title1",
            "short_description": "short_description1",
            "body": "body1",
            "data": "data1",
            "extra": "extra1"
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    assert!(res.is_failure(), "Not enough deposit");

    let res = proposer
        .call(mpip_contract.id(), "create_proposal")
        .args_json(serde_json::json!({
            "title": "title1",
            "short_description": "short_description1",
            "body": "body1",
            "data": "data1",
            "extra": "extra1"
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    // Due to Workspaces nuances, this is the way to see if a receipt in the tx failed.
    assert!(
        res.is_success() && res.receipt_failures().len() == 1,
        "Not enough voting power"
    );

    let res = proposer
        .call(mpip_contract.id(), "get_proposals")
        .args_json(serde_json::json!({
            "from_index": 0,
            "limit": 100
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    assert!(res.len() == 0);

    let res = proposer
        .call(mpip_contract.id(), "get_user_proposals_ids")
        .args_json(serde_json::json!({
            "proposer_id": voter.id()
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    assert!(res.len() == 0);

    ///////////////////////////////////////
    // Stage 2: Creating Proposals
    ///////////////////////////////////////

    // voter locks mpDAO
    let locked_mpdao = 3;
    let days = 60;
    let args = serde_json::json!({
        "receiver_id": metavote_contract.id(),
        "amount": mpdao_as_u128_string(locked_mpdao),
        "msg": format!(r#"{}"#,days)
    });
    let res = voter
        .call(mpdao_token_contract.id(), "ft_transfer_call")
        .args_json(&args)
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    //println!("args {:?}\n {:?}\n", args, res);
    if res.failures().len() > 0 {
        println!(
            "for locking Transfer {} mpdao, days={} ERR: {:?}\n",
            locked_mpdao, days, res
        );
    }

    // verify voting power
    let res = owner
        .call(metavote_contract.id(), "get_available_voting_power")
        .args_json(serde_json::json!({
            "voter_id": voter.id()
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    let vp: u128 = res.to_string().parse().unwrap();
    let expected_vp: u128 = locked_mpdao as u128 * 1 * E24;
    println!("vp: {:?}\n", vp);
    assert_eq!(vp, expected_vp, "Vp <> expected_vp");

    // proposer locks mpdao
    let res = proposer
        .call(mpdao_token_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": metavote_contract.id(),
            "amount": mpdao_as_u128_string(3),
            "msg": "120"
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("lock 3 mpdao unbond 120 d ERR: {:?}\n", res);
    }

    let res = owner
        .call(metavote_contract.id(), "get_available_voting_power")
        .args_json(serde_json::json!({
            "voter_id": proposer.id()
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    let vp: u128 = res.to_string().parse().unwrap();
    let expected_vp: u128 = 3 as u128 * 2 * E24;
    println!("(2) vp: {:?}\n", vp);
    assert_eq!(vp, expected_vp, "(2) vp <> expected_vp");

    // voter locks 2nd position: 0 day to test withdraw
    {
        let locked_mpdao = 5;
        let days = 0;
        let args = serde_json::json!({
            "receiver_id": metavote_contract.id(),
            "amount": mpdao_as_u128_string(locked_mpdao),
            "msg": format!(r#"{}"#,days)
        });
        let res = proposer
            .call(mpdao_token_contract.id(), "ft_transfer_call")
            .args_json(&args)
            .gas(NearGas::from_tgas(200))
            .deposit(NearToken::from_yoctonear(1))
            .transact()
            .await?;
        //println!("args {:?}\n {:?}\n", args, res);
        if res.failures().len() > 0 {
            println!(
                "for locking Transfer {} mpdao, days={} ERR: {:?}\n",
                locked_mpdao, days, res
            );
        }
    }

    // create proposal
    let res = proposer
        .call(mpip_contract.id(), "create_proposal")
        .args_json(serde_json::json!({
            "title": "title1",
            "short_description": "short_description1",
            "body": "body1",
            "data": "data1",
            "extra": "extra1"
        }))
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    // Due to Workspaces nuances, this is the way to see if a receipt in the tx failed.
    assert!(
        res.is_success() && res.receipt_failures().len() == 0,
        "Not enough voting power"
    );

    let res = proposer
        .call(mpip_contract.id(), "get_proposals")
        .args_json(serde_json::json!({
            "from_index": 0,
            "limit": 100
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    assert!(res.len() == 1);

    ///////////////////////////////////////
    // Stage 3: Voting Proposals
    ///////////////////////////////////////

    let res = voter
        .call(mpip_contract.id(), "get_proposal")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    println!("Proposal 1: {:?}\n", res);

    let res = voter
        .call(mpip_contract.id(), "get_proposal_state")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    assert_eq!(res, "Draft");

    let res = proposer
        .call(mpip_contract.id(), "start_voting_period")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("Starting Voting Period ERR: {:?}\n", res);
    }

    // let res = voter
    //     .call(mpip_contract.id(), "get_proposal")
    //     .args_json(serde_json::json!({
    //         "mpip_id": 0
    //     }))
    //     .gas(NearGas::from_tgas(200))
    //     // .deposit(NearToken::from_millinear(3))
    //     .transact()
    //     .await?;
    // let res = &res.raw_bytes().unwrap().clone();
    // let res = str::from_utf8(res).unwrap();
    // let res = json::parse(&res)?;
    // println!("Proposal ACA: {:?}\n", res);

    let res = voter
        .call(mpip_contract.id(), "get_proposal_state")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    assert_eq!(res, "VotingProcess");

    let res = voter
        .call(mpip_contract.id(), "get_proposal_votes")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    assert_eq!(res["for_votes"], "0");
    assert_eq!(res["against_votes"], "0");
    assert_eq!(res["abstain_votes"], "0");

    // Voting AGAINST

    let res = voter
        .call(mpip_contract.id(), "vote_proposal")
        .args_json(serde_json::json!({
            "mpip_id": 0,
            "vote": "Against",
            "memo": ""
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    // let res = &res.raw_bytes().unwrap().clone();
    // let res = str::from_utf8(res).unwrap();
    // let res = json::parse(&res)?;
    if res.failures().len() > 0 {
        println!("vote_proposal ERR: {:?}\n", res);
    }

    let res = voter
        .call(mpip_contract.id(), "get_proposal_votes")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;

    let num: u128 = format!("{}", parse_near!("3 N")).parse().unwrap();
    let aga: u128 = res["against_votes"].to_string().parse().unwrap();
    assert_eq!(aga, num, "(2) against_votes<>expected");
    assert_eq!(res["for_votes"], "0");
    assert_eq!(res["abstain_votes"], "0");

    // Remove Vote
    let _res = voter
        .call(mpip_contract.id(), "remove_vote_proposal")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    // let res = &res.raw_bytes().unwrap().clone();
    // let res = str::from_utf8(res).unwrap();
    // let res = json::parse(&res)?;

    // Voting FOR

    let res = voter
        .call(mpip_contract.id(), "vote_proposal")
        .args_json(serde_json::json!({
            "mpip_id": 0,
            "vote": "For",
            "memo": ""
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    // let res = &res.raw_bytes().unwrap().clone();
    // let res = str::from_utf8(res).unwrap();
    // let res = json::parse(&res)?;
    if res.failures().len() > 0 {
        panic!("vote_proposal 2 ERR: {:?}\n", res);
    }

    let res = voter
        .call(mpip_contract.id(), "get_proposal_votes")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;

    let num: u128 = format!("{}", parse_near!("3 N")).parse().unwrap();
    let for_votes: u128 = res["for_votes"].to_string().parse().unwrap();
    assert_eq!(for_votes, num, "(1) for_votes<>expected");
    assert_eq!(res["against_votes"], "0");
    assert_eq!(res["abstain_votes"], "0");

    // proposer starts unlocking position 1
    let res = proposer
        .call(metavote_contract.id(), "unlock_position")
        .args_json(serde_json::json!({
            "index": 1
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        panic!("unlock_position ERR: {:?}\n", res);
    }
    // let res = &res.raw_bytes().unwrap().clone();
    // let res = str::from_utf8(res).unwrap();
    // let res = json::parse(&res)?;

    // ------------------------------------------
    println!("start worker.fast_forward");
    let blocks_to_advance = 3000;
    worker.fast_forward(blocks_to_advance).await?;
    println!("end worker.fast_forward");
    // ------------------------------------------

    let res = voter
        .call(mpip_contract.id(), "get_proposal_state")
        .args_json(serde_json::json!({
            "mpip_id": 0
        }))
        .gas(NearGas::from_tgas(200))
        // .deposit(NearToken::from_millinear(3))
        .transact()
        .await?;
    let res = &res.raw_bytes().unwrap().clone();
    let res = str::from_utf8(res).unwrap();
    let res = json::parse(&res)?;
    println!("FINAL proposal state: {:?}\n", res);
    assert_eq!(res, "Accepted");

    // -------------------------
    println!("WITHDRAW ALL");
    let balance_pre = ft_balance(&mpdao_token_contract, &proposer).await?;
    println!("balance_pre: {}\n", balance_pre);
    // proposer withdraws all mpdao tokens (position 1 should be unlocked)
    let res = proposer
        .call(metavote_contract.id(), "withdraw_all")
        .gas(NearGas::from_tgas(200))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        panic!("withdraw_all ERR: {:?}\n", res);
    }
    let balance_post: String = ft_balance(&mpdao_token_contract, &proposer).await?;
    println!("balance_post: {}\n", balance_post);
    let delta = u128::from_str(&balance_post).unwrap() - u128::from_str(&balance_pre).unwrap();
    println!("delta: {}\n", delta);
    assert!(delta.to_string() == mpdao_as_u128_string(5));

    Ok(())
}

async fn create_nep141_token(
    owner: &Account,
    worker: &Worker<impl DevNetwork>,
    amount_string: &str,
) -> anyhow::Result<Contract> {
    let token_contract_wasm = std::fs::read(MPDAO_TEST_TOKEN_FILEPATH)?;
    let token_contract = worker.dev_deploy(&token_contract_wasm).await?;
    let args = serde_json::json!({
        "owner_id": owner.id(),
        "total_supply": amount_string
    });
    println!("init token contract {}", args);
    let res = token_contract
        .call("new_default_meta")
        .args_json(args)
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("token contract new_default_meta result: {:#?}", res);
        panic!("err on init")
    }

    Ok(token_contract)
}

async fn create_metavote(
    owner: &Account,
    operator: &Account,
    mpdao_token_contract: &AccountId,
    stnear_contract: &AccountId,
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<Contract> {
    let metavote_contract_wasm = std::fs::read(METAVOTE_FILEPATH)?;
    let metavote_contract = worker.dev_deploy(&metavote_contract_wasm).await?;

    let res = metavote_contract
        .call("new")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "operator_id": operator.id(),
            "min_unbond_period": 0,
            "max_unbond_period": 300,
            "min_deposit_amount": mpdao_as_u128_string(1),
            "max_locking_positions": 20,
            "max_voting_positions": 40,
            "mpdao_token_contract_address": mpdao_token_contract,
            "stnear_token_contract_address": stnear_contract,
            "registration_cost":"0",
            "prev_governance_contract":mpdao_token_contract,
        }))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("METAVOTE init error: {:#?}", res);
        panic!()
    }

    Ok(metavote_contract)
}

async fn create_mpip(
    owner: &Account,
    mpdao_token_contract_address: &AccountId,
    metavote_contract_address: &AccountId,
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<Contract> {
    let mpip_contract_wasm = std::fs::read(MPIP_FILEPATH)?;
    let mpip_contract = worker.dev_deploy(&mpip_contract_wasm).await?;

    let res = mpip_contract
        .call("new")
        .args_json(serde_json::json!({
            "admin_id": owner.id(),
            "operator_id": owner.id(),
            "meta_token_contract_address": mpdao_token_contract_address,
            "meta_vote_contract_address": metavote_contract_address,
            "voting_period": "20000", // milliseconds
            "min_voting_power_amount": format!("{}", parse_near!("3 N")),
            "mpip_storage_near": "300000000000000",
            "quorum_floor": 1000
        }))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("MPIPs init err: {:#?}", res);
        panic!()
    }

    Ok(mpip_contract)
}

async fn storage_deposit(account: &Account, token_contract: &Contract) -> anyhow::Result<()> {
    println!("storage_deposit {}", account.id());
    let res = account
        .call(token_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": account.id(),
        }))
        .gas(NearGas::from_tgas(50))
        .deposit(NearToken::from_millinear(250))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("storage_deposit {}: {:?}\n", account.id(), res);
        Err(anyhow::Error::msg("storage_deposit failure"))
    } else {
        Ok(())
    }
}

async fn register_storage_for(
    metatoken_contract: &Contract,
    stnear_contract: &Contract,
    metavote_contract: &Contract,
    mpip_contract: &Contract,
    owner: &Account,
    voter: &Account,
    proposer: &Account,
) -> anyhow::Result<()> {
    try_join!(
        storage_deposit(&owner, &metatoken_contract),
        storage_deposit(&voter, &metatoken_contract),
        storage_deposit(&metavote_contract.as_account(), &metatoken_contract),
        storage_deposit(&metavote_contract.as_account(), &stnear_contract),
        storage_deposit(&mpip_contract.as_account(), &metatoken_contract),
        storage_deposit(&proposer, &metatoken_contract)
    )
    .expect("err st storage_deposit");
    Ok(())
}

async fn register_delegate(
    metavote_contract: &Contract,
    delegate1: &Account,
    evm_address: &String,
    operator: &Account,
) -> anyhow::Result<()> {
    let res = delegate1
        .call(metavote_contract.id(), "pre_delegate_evm_address")
        .args_json(serde_json::json!({
            "evm_address": evm_address,
            "signature": format!("SOME-SIGNATURE-FOR-DELEGATE-TO-{}",delegate1.id())
        }))
        .gas(NearGas::from_tgas(75))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("pre_delegate_evm_address ERR: {:?}\n", res);
        return Ok(());
    }

    // register_delegate
    let res = operator
        .call(
            metavote_contract.id(),
            "operator_confirm_delegated_evm_address",
        )
        .args_json(serde_json::json!({
            "evm_address": evm_address,
        }))
        .gas(NearGas::from_tgas(75))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    if res.failures().len() > 0 {
        println!("operator_confirm_delegated_evm_address ERR: {:?}\n", res);
        return Err(anyhow::Error::msg("operator_confirm_delegated_evm_address"));
    }
    Ok(())
}

async fn get_delegation_signature(
    metavote_contract: &Contract,
    evm_address: &String,
) -> anyhow::Result<serde_json::Value> {
    let res = metavote_contract
        .view("get_delegation_signature")
        .args_json(serde_json::json!({
            "evm_address": evm_address,
        }))
        .await?;
    let res: serde_json::Value = serde_json::from_slice(&res.result)?;
    println!("signature for {} {:?}", evm_address, res);
    Ok(res)
}

async fn get_delegations(
    metavote_contract: &Contract,
    account: &Account,
) -> anyhow::Result<serde_json::Value> {
    let res = metavote_contract
        .view("get_delegating_evm_addresses")
        .args_json(serde_json::json!({
            "account_id": account.id(),
        }))
        .await?;
    let res: serde_json::Value = serde_json::from_slice(&res.result)?;
    println!("delegations for {} {:?}", account.id(), res);
    Ok(res)
}

pub(crate) const DEV_ACCOUNT_SEED: &str = "testificate";

pub async fn create_account(worker: &Worker<Sandbox>, name: &str) -> anyhow::Result<Account> {
    let sk = SecretKey::from_seed(KeyType::ED25519, DEV_ACCOUNT_SEED);
    //    let (id, sk) = self.dev_generate().await;
    let account_id = AccountId::from_str(&format!("{}.registrar", name)).unwrap();
    let account = worker.create_tla(account_id, sk).await?;
    Ok(account.into_result()?)
}
