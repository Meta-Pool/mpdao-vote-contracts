#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

set -ex
#near state $METAVOTE_CONTRACT_ADDRESS
##| jq -r '.values[0].value' | base64 -d >contract.wasm

curl -s -X POST \
  -H 'Content-Type: application/json' \
  --data '{
    "jsonrpc": "2.0",
    "id": "dontcare",
    "method": "query",
    "params": {
      "request_type": "view_code",
      "finality": "final",
      "account_id": "'$METAVOTE_CONTRACT_ADDRESS'"
    }
  }' \
  'https://rpc.mainnet.near.org' | \
  jq -r '.result.code_base64' | \
  base64 -d > $METAVOTE_CONTRACT_ADDRESS.wasm

