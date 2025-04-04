#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh

OWNER_ID="mp-dev-shared.testnet"
OPERATOR_ID="mp-dev-shared.testnet"
VOTING_PERIOD=5
# PROPOSAL THRESHOLD - MIN VOTING POWER 10.000
MIN_VOTING_POWER_AMOUNT="1"$YOCTO_UNITS
# QUORUM FLOOR 5%
QUORUM_FLOOR=500
MPIP_STORAGE_COST="1"$YOCTO_UNITS
# QUORUM FLOOR 5%
QUORUM_FLOOR=500
# Deploy MPIP Contract
ARGS_INIT_MPIP=$(cat <<EOA
{
"admin_id":"$OWNER_ID",
"operator_id":"$OPERATOR_ID",
"meta_token_contract_address":"$MPDAO_TOKEN_ADDRESS",
"meta_vote_contract_address":"$METAVOTE_CONTRACT_ADDRESS",
"voting_period":"$VOTING_PERIOD",
"min_voting_power_amount":"$MIN_VOTING_POWER_AMOUNT",
"mpip_storage_near":"$MPIP_STORAGE_COST",
"quorum_floor":$QUORUM_FLOOR
}
EOA
)

set -ex
NEAR_ENV=testnet near deploy $MPIP_CONTRACT_ADDRESS $MPIP_WASM \
    --initFunction new \
    --initArgs "$ARGS_INIT_MPIP" \
    --accountId $MPIP_CONTRACT_ADDRESS

