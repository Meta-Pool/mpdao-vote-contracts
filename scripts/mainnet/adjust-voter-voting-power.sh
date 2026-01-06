#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

# Script to call operator_recompute_available_vp which internally calls adjust_voter_voting_power
# This recomputes a voter's available voting power from scratch and removes votes if needed

if [ -z "$1" ]; then
  echo "Usage: $0 <voter_id>"
  echo "Example: $0 user.near"
  echo "Example: $0 0xABCDEF1234567890ABCDEF1234567890ABCDEF12.evmp.near"
  exit 1
fi

VOTER_ID=$1

echo "Adjusting voting power for voter: $VOTER_ID"
echo "Contract: $METAVOTE_CONTRACT_ADDRESS"
echo "Operator: $OPERATOR_ID"
echo ""

set -x

NEAR_ENV=mainnet near call $METAVOTE_CONTRACT_ADDRESS \
  operator_recompute_available_vp '{"voter_id":"'$VOTER_ID'"}' \
  --useAccount $OPERATOR_ID
