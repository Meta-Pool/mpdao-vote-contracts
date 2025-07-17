#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh

echo DEPLOYING $NEAR_ENV META VOTE
NEAR_ENV=testnet near deploy $METAVOTE_CONTRACT_ADDRESS $METAVOTE_WASM \
    --initFunction new --initArgs "$ARGS_INIT_META_VOTE"

# register so the contract can receive mpDAO
NEAR_ENV=testnet near call $MPDAO_TOKEN_ADDRESS storage_deposit '{"account_id":"'$METAVOTE_CONTRACT_ADDRESS'"}' --accountId $OWNER_ID --amount 0.0125
