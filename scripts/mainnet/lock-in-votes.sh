#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

if [ $# -eq 2 ]; then
  echo Lock-in Grants Round $1 until $2
  NEW_LOCK_TIMESTAMP_MS=$(date -d "$2" +%s%3N)
  near call $METAVOTE_CONTRACT_ADDRESS set_lock_in_vote_filters \
        '{"end_timestamp_ms":'$NEW_LOCK_TIMESTAMP_MS',"votable_numeric_id":'$1',"votable_address":"initiatives"}' \
        --accountId $OPERATOR_ID --gas 150000000000000
fi

if [ $# -ne 2 ]; then
  echo "To set please provide exactly 2 arguments: Round #, date-ISO"
  echo "where date-ISO is when the lock-in will end"
  echo "Example: $0 8 2025-02-09T19:00:00Z"
fi

# check current status
RESULT=$(near view $METAVOTE_CONTRACT_ADDRESS get_lock_in_vote_filters)
# result example: [ 1726754400000, 'initiatives', 8 ]
echo "Current Lock-in Grants Round: $RESULT"
# parse the result into 3 variables, skip first line
# remove the [ and ] characters
RESULT=$(tail -n +2 <<< "$RESULT")
RESULT=$(echo $RESULT | sed 's/\[//g' | sed 's/\]//g')
IFS=', ' read -r -a array <<< $RESULT
echo ${array}
TIMESTAMP_MS=${array[0]}
ROUND=${array[2]}
TIMESTAMP=$(( $TIMESTAMP_MS / 1000 ))
# show timestamp in human readable format
TIMESTAMP_DATE=$(date -d @$TIMESTAMP)
echo "Current Lock-in is: Grants Round #$ROUND until $TIMESTAMP_DATE"
