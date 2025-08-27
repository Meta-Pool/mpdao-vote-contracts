#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh


# Redeploy Contract
echo Re-DEPLOY ONLY
#METAVOTE_WASM=res/downloaded-testnet-mpdao-vote.wasm
NEAR_ENV=testnet near deploy $METAVOTE_CONTRACT_ADDRESS $METAVOTE_WASM
