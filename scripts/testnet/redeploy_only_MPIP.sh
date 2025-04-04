#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh

echo mpip-contract: $MPIP_CONTRACT_ADDRESS
ls -l $MPIP_WASM

# Redeploy Contract
echo Re-DEPLOY ONLY
set -ex
NEAR_ENV=testnet near deploy $MPIP_CONTRACT_ADDRESS $MPIP_WASM
