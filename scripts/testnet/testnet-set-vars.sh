#!/bin/bash
set -e
export NEAR_ENV="testnet"

BASE_METAVOTE_ADDRESS=mpdao-vote.testnet
METAVOTE_VERSION=v1
METAVOTE_CONTRACT_ADDRESS=$METAVOTE_VERSION.$BASE_METAVOTE_ADDRESS
METAVOTE_WASM="res/meta_vote_contract.wasm"
OLD_METAVOTE_CONTRACT=metavote.testnet

echo $NEAR_ENV $METAVOTE_CONTRACT_ADDRESS $(date)

YOCTO_UNITS="000000000000000000000000"
TOTAL_PREPAID_GAS="300000000000000"

MPIP_CONTRACT_ADDRESS="v4-mpip.testnet"
MPIP_WASM="res/mpip_contract.wasm"

# 1st Deploy Contract & init
OWNER_ID=$METAVOTE_CONTRACT_ADDRESS
OPERATOR_ID=$BASE_METAVOTE_ADDRESS
MPDAO_TOKEN_ADDRESS="mpdao-token.testnet"
STNEAR_TESTNET_TOKEN_ADDRESS="meta-v2.pool.testnet"
        # owner_id: AccountId,
        # min_unbond_period: Days,
        # max_unbond_period: Days,
        # min_deposit_amount: U128String,
        # max_locking_positions: u8,
        # max_voting_positions: u8,
        # mpdao_token_contract_address: ContractAddress,
        # stnear_token_contract_address: ContractAddress,
        # registration_cost: U128String,
ARGS_INIT_META_VOTE=$(cat <<EOA
{
"owner_id":"$OWNER_ID","operator_id":"$OPERATOR_ID",
"min_unbond_period":30, "max_unbond_period":300, "min_deposit_amount":"1000000",
"max_locking_positions": 32, "max_voting_positions": 32,
"mpdao_token_contract_address":"$MPDAO_TOKEN_ADDRESS","stnear_token_contract_address":"$STNEAR_TESTNET_TOKEN_ADDRESS",
"prev_governance_contract":"$OLD_METAVOTE_CONTRACT",
"registration_cost":"100000"
}
EOA
)

echo $NEAR_ENV $METAVOTE_CONTRACT_ADDRESS $OWNER_ID

KV_STORE_CONTRACT_ADDRESS=kv-store.testnet
KV_STORE_WASM=target/wasm32-unknown-unknown/release/kv_store_contract.wasm
