#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

#NEAR_ENV=mainnet near view mpdao-vote get_used_voting_power '{"voter_id":"vhieu.testnet"}'
#NEAR_ENV=mainnet near view mpdao-vote.near get_contract_info '{}'

#NEAR_ENV=mainnet near view mpdao-vote.near get_votes_by_app '{"app_or_contract_address":"initiatives"}'
#NEAR_ENV=mainnet near view meta-pipeline.near get_folders
#NEAR_ENV=mainnet near view meta-pipeline.near get_projects_in_folder '{"folder_id":6}'

set -ex
near view mpdao-vote.near get_contract_info '{}'
#near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$1'"}'
