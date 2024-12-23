#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

echo transfer_extra_balance to owner $OWNER_ID --accountId $1
near call $METAVOTE_CONTRACT_ADDRESS transfer_extra_balance --accountId $OPERATOR_ID --gas 50000000000000
