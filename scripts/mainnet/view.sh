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
near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"luciotato.near"}'
get_near_balance $METAVOTE_CONTRACT_ADDRESS

get_mpdao_balance meta-pool-dao.near
#echo $BALANCE
