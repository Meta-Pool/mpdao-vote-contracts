#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh
. $__dir/declare_get_mpdao_balance.sh

#NEAR_ENV=mainnet near view mpdao-vote get_used_voting_power '{"voter_id":"vhieu.testnet"}'
#NEAR_ENV=mainnet near view mpdao-vote.near get_contract_info '{}'

#NEAR_ENV=mainnet near view mpdao-vote.near get_votes_by_app '{"app_or_contract_address":"initiatives"}'
#NEAR_ENV=mainnet near view meta-pipeline.near get_folders
#NEAR_ENV=mainnet near view meta-pipeline.near get_projects_in_folder '{"folder_id":6}'

#near view mpdao-vote.near get_contract_info '{}'
#near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$1'"}'

# near view $METAVOTE_CONTRACT_ADDRESS get_voters_count
# near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"non-existent.near"}'
# near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"luciotato.near"}'

# curl -X POST https://free.rpc.fastnear.com -H "Content-Type: application/json" -d '{
#   "jsonrpc": "2.0",
#   "id": 1,
#   "method": "query",
#   "params": {
#     "request_type": "call_function",
#     "finality": "final",
#     "account_id": "mpdao-vote.near",
#     "method_name": "get_voter_info",
#     "args_base64": "'$(echo -n '{"voter_id": "luciotato.near"}' | base64)'"
#   }
# }'

near view $METAVOTE_CONTRACT_ADDRESS get_balance '{"voter_id":"luciotato.near"}'

# curl -X POST https://free.rpc.fastnear.com -H "Content-Type: application/json" -d '{
#   "jsonrpc": "2.0",
#   "id": 1,
#   "method": "query",
#   "params": {
#     "request_type": "call_function",
#     "finality": "final",
#     "account_id": "mpdao-vote.near",
#     "method_name": "get_balance",
#     "args_base64": "'$(echo -n '{"voter_id": "luciotato.near"}' | base64)'"
#   }
# }'

# get_near_balance $METAVOTE_CONTRACT_ADDRESS

# get_mpdao_balance meta-pool-dao.near
# #echo $BALANCE
