#!/bin/bash
__dir=$(dirname "$0")
. $__dir/mainnet-set-vars.sh

# to confirm that a pub(crate) cant be called externally
# even if the implementation has #[near_bindgen]
set -ex
near call $METAVOTE_CONTRACT_ADDRESS transfer_mpdao_to_voter \
      '{"voter_id":"--","amount":"'$1$MPDAO_DECIMALS'"}' \
      --accountId $OWNER_ID --gas 150000000000000
