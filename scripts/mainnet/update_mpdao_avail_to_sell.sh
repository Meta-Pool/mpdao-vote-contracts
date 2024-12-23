#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

if [ $# -ne 1 ]; then
  echo "Error: Please provide mpDAO avail amount (in mpDAO)"
  exit 1
fi
echo "update mpDAO avail amount to $1 mpDAO ( $1 $MPDAO_DECIMALS raw )"
set -ex
near call $METAVOTE_CONTRACT_ADDRESS update_mpdao_avail_to_sell \
     '{"mpdao_avail_to_sell":"'$1$MPDAO_DECIMALS'"}' \
     --accountId $OWNER_ID --depositYocto 1 --gas 50000000000000
