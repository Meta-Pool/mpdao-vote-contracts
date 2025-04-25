# #!/bin/bash
# __dir=$(dirname "$0")
# . $__dir/mainnet-set-vars.sh

export NEAR_ENV=mainnet
METAVOTE_CONTRACT_ADDRESS="meta-vote.near"
METAVOTE_OWNER="meta-pool-dao.near"
METAVOTE_WASM="contracts/res/meta_vote_contract.wasm"

REQUIRED_ARGS=1
if [ $# -ne $REQUIRED_ARGS ]; then
  echo "Error: Please provide exactly $REQUIRED_ARGS arguments."
  exit 1
fi

# Call function
near view $METAVOTE_CONTRACT_ADDRESS get_all_locking_positions '{"voter_id":"'$1'"}'
set -ex
echo near call $METAVOTE_CONTRACT_ADDRESS reset_user_migration '{"account_id":"'$1'"}' \
      --accountId $METAVOTE_OWNER --gas 150000000000000
# sleep 2
near view $METAVOTE_CONTRACT_ADDRESS get_all_locking_positions '{"voter_id":"'$1'"}'