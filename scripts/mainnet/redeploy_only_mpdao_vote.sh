#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

echo meta-vote-contract: $METAVOTE_CONTRACT_ADDRESS 
ls -l $METAVOTE_WASM

# Redeploy Contract
echo Re-DEPLOY ONLY
NEAR_ENV=mainnet near deploy $METAVOTE_CONTRACT_ADDRESS $METAVOTE_WASM
