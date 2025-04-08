#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh

NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS get_contract_info
# call view function
EVM_ACC0="0xf06B9633c6a6b255C80B4900f693797F43393ea3"
NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$EVM_ACC0'.evmp.near"}'

NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS voter_total_voting_power '{"voter_id":"'$EVM_ACC0'.evmp.near"}'
# VOTER_ID_2="0x0B438De1DCa9FBa6D14F17c1F0969ECc73C8186F.evmp.near"
# NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$VOTER_ID_2'"}'

# NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS get_voters '{"from_index":0,"limit":10}'

NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"asimov.testnet"}'
set -ex
NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS voter_total_voting_power '{"voter_id":"asimov.testnet"}'