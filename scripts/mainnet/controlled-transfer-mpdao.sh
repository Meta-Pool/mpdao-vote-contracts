#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

if [ $# -ne 2 ]; then
  echo "Error: Please provide exactly 2 arguments."
  echo "receiver_id, mpDAO-amount"
  exit 1
fi
echo $1 will receive $2 mpDAO
echo $1 must be registered as a safe destination

set -ex
# NEAR_ENV=mainnet near call $MPDAO_TOKEN_ADDRESS \
#     storage_deposit '{"account_id":"'$1'"}' \
#     --accountId operator.meta-pool.near \
#     --deposit 0.00125

NEAR_ENV=mainnet near call meta-pool-dao-safe.near \
    ft_transfer '{"token_contract_id":"'$MPDAO_TOKEN_ADDRESS'","receiver_id":"'$1'","amount":"'$2$MPDAO_DECIMALS'"}' \
    --accountId operator.meta-pool-dao-safe.near
