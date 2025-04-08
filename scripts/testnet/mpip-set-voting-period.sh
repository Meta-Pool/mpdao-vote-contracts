#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh

OWNER_ID="mp-dev-shared.testnet"
OPERATOR_ID="mp-dev-shared.testnet"
# Convert 2 hours to milliseconds
TWO_HOURS_MS=$((2 * 60 * 60 * 1000))
VOTING_PERIOD=$TWO_HOURS_MS

set -ex
NEAR_ENV=testnet near call $MPIP_CONTRACT_ADDRESS update_voting_period '{"new_value":"'$VOTING_PERIOD'"}' --accountId $OPERATOR_ID
