#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh

# re-Deploy and call state MIGRATION
echo RE-DEPLOY AND MIGRATION
set -ex
NEAR_ENV=testnet near deploy $METAVOTE_CONTRACT_ADDRESS $METAVOTE_WASM --initFunction migrate --initArgs {}
