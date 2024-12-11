MPDAO_DECIMALS="000000"
export NEAR_ENV=testnet
MPDAO_TOKEN_CONTRACT=mpdao-token.testnet
META_VOTE_CONTRACT=v1.mpdao-vote.testnet
OWNER_ACCOUNT=mpdao-vote.testnet
if [ $# -ne 3 ]; then
  echo "Error: Please provide exactly 3 arguments."
  echo "voter_id, mpDAO-amount, bonding-days"
  if [ $# -eq 1 ]; then
    near view $META_VOTE_CONTRACT get_voter_info '{"voter_id":"'$1'"}'
  fi
  exit 1
fi
echo VESTING for $1 $2 mpDAO for $3 days
# near call $MPDAO_TOKEN_CONTRACT ft_transfer_call \
#       '{"receiver_id":"'$META_VOTE_CONTRACT'","amount":"'$2$MPDAO_DECIMALS'","msg":"[\"'$1'\",'$3']"}' \
#       --accountId $OWNER_ACCOUNT --depositYocto 1 --gas 150000000000000
# near view $META_VOTE_CONTRACT get_voter_info '{"voter_id":"'$1'"}'
near call $MPDAO_TOKEN_CONTRACT ft_transfer \
     '{"receiver_id":"'$OWNER_ACCOUNT'","amount":"'$2$MPDAO_DECIMALS'"}' \
     --accountId $META_VOTE_CONTRACT --depositYocto 1 --gas 150000000000000
