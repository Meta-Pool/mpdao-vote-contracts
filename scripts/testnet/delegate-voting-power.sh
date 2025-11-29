#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh

DELEGATED_CONTRACT_CODE="delegated"

SIX_ZEROS="000000"
TWENTY_FOUR_ZEROS=$SIX_ZEROS$SIX_ZEROS$SIX_ZEROS$SIX_ZEROS

SIGNER=$1

if [ $# -ne 3 ]; then
  echo "Error: Please provide exactly 3 arguments."
  echo "delegator-account, delegate-account, voting-power (millions)"
  if [ $# -eq 1 ]; then
    near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$1'"}'
  fi
  exit 1
fi
echo DELEGATING $3 million VP from $1 to $2
near call $METAVOTE_CONTRACT_ADDRESS vote \
      '{"voting_power":"'$3$SIX_ZEROS$TWENTY_FOUR_ZEROS'","contract_address":"'$DELEGATED_CONTRACT_CODE'","votable_object_id":"'$2'"}' \
      --accountId $SIGNER --gas 10$SIX_ZEROS$SIX_ZEROS
sleep 2
set -ex
echo --------- $1
near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$1'"}'
echo --------- $2
near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$2'"}'
