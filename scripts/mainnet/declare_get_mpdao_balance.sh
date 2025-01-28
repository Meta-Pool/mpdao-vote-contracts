if [ -z "$MPDAO_TOKEN_ADDRESS" ]; then
  __dir=$(dirname "$0")
  . $__dir/mainnet-set-vars.sh
fi

# Function to get mpDAO ft_balance of an account
# Input is accountId, output is exported BALANCE
get_mpdao_balance() {
  if [ $# -ne 1 ]; then
    echo "Error: Please provide exactly account_id"
    echo "Usage: get_mpdao_balance <account_id>"
    return 1
  fi

  local ACCOUNT=$1
  local RESULT=$(near view $MPDAO_TOKEN_ADDRESS ft_balance_of '{"account_id":"'$ACCOUNT'"}')
  # get only the last line
  RESULT=$(echo "$RESULT" | tail -n 1)
  # remove enclosing single quotes
  RESULT=$(echo "$RESULT" | tr -d "'")
  # convert to number and divide by 1e6, add thousand separator
  BALANCE=$(echo "scale=0; $RESULT / 1000000" | bc)
  local WITH_SEPARATORS=$(echo $BALANCE | sed ':a;s/\B[0-9]\{3\}\>/,&/;ta')
  echo $ACCOUNT mpDAO balance: $WITH_SEPARATORS
}

# Example usage:
# get_mpdao_balance <account_id>
# example:
# get_mpdao_balance meta-pool-dao.near
# echo $BALANCE
