MPDAO_DECIMALS="000000"
export NEAR_ENV=testnet
MPDAO_TOKEN_CONTRACT=mpdao-token.testnet
META_VOTE_CONTRACT=v1.mpdao-vote.testnet
OWNER_ACCOUNT=mpdao-vote.testnet
if [ $# -ne 1 ]; then
  echo "Error: Please provide mpDAO avail amount (in mpDAO)"
  exit 1
fi
echo "update mpDAO avail amount to $1 mpDAO ( $1 $MPDAO_DECIMALS raw )"
near call $META_VOTE_CONTRACT update_mpdao_avail_to_sell \
     '{"mpdao_avail_to_sell":"'$1$MPDAO_DECIMALS'"}' \
     --accountId $OWNER_ACCOUNT --depositYocto 1 --gas 50000000000000
