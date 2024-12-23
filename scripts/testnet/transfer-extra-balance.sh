PAYER_ACCOUNT=$1

MPDAO_DECIMALS="000000"
export NEAR_ENV=testnet
MPDAO_TOKEN_CONTRACT=mpdao-token.testnet
META_VOTE_CONTRACT=v1.mpdao-vote.testnet
OWNER_ACCOUNT=mpdao-vote.testnet
echo transfer_extra_balance to owner $OWNER_ACCOUNT --accountId $PAYER_ACCOUNT
near call $META_VOTE_CONTRACT transfer_extra_balance --accountId $PAYER_ACCOUNT --gas 50000000000000
