MPDAO_DECIMALS="000000"
E24="000000000000000000000000"
export NEAR_ENV=testnet
MPDAO_TOKEN_CONTRACT=mpdao-token.testnet
META_VOTE_CONTRACT=v1.mpdao-vote.testnet
OWNER_ACCOUNT=mpdao-vote.testnet
OPERATOR_ACCOUNT=operator.mpdao-vote.testnet
if [ $# -ne 1 ]; then
  echo "Error: Please provide mpDAO per NEAR amount (in mpDAO)"
  exit 1
fi
NEAR_USD_PRICE=7
echo considering NEAR_USD_PRICE=$NEAR_USD_PRICE usd
echo "update mpDAO per NEAR to $1 mpDAO per NEAR"
echo "($NEAR_USD_PRICE / $1 = $(echo "scale=4; $NEAR_USD_PRICE / $1" | bc)) ) mpDAO/usd price approx"
read -p "Press enter to continue"
near call $META_VOTE_CONTRACT update_mpdao_per_near_e24 \
     '{"mpdao_per_near_e24":"'$1$E24'"}' \
     --accountId $OPERATOR_ACCOUNT --depositYocto 1 --gas 50000000000000
