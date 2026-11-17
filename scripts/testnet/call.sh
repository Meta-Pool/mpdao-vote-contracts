#!/bin/bash
__dir=$(dirname "$0")
. $__dir/testnet-set-vars.sh


# call view function
EVM_ADDRESS_OWNER=0xf06B9633c6a6b255C80B4900f693797F43393ea3
# NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$EVM_ADDRESS_1'.evmp.near"}'
EVM_ADDRESS_ACC0=0x81f41569F7d9b61ED2c7c348673C7e0D2590044D
EVM_ADDRESS_ACC1=0xD3D1C049384e72DBA025BB25E87830b10bc5568e
# NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS get_voter_info '{"voter_id":"'$EVM_ADDRESS_2'.evmp.near"}'

DELEGATE="asimov.testnet"

# NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
# pre_delegate_evm_address '{"evm_address":"'$EVM_ADDRESS_ACC0'","signature":"xx"}' \
#     --useAccount $DELEGATE --depositYocto 1 

# NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
# operator_confirm_delegated_evm_address '{"evm_address":"'$EVM_ADDRESS_ACC0'","expected_account_id":"'$DELEGATE'"}' \
#     --useAccount $OPERATOR_ID --depositYocto 1 

# REMOVE DELEGATION
# NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
# remove_delegated_evm_address '{"evm_address":"'$EVM_ADDRESS_ACC0'.evmp.near"}' \
#     --useAccount $DELEGATE --depositYocto 1 

NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS \
    get_delegating_evm_addresses '{"account_id":"'$DELEGATE'"}'

NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
    vote_delegated '{"evm_address":"'$EVM_ADDRESS_ACC0'","voting_power":"2'$YOCTO_UNITS'","contract_address":"xx","votable_object_id":"xx1"}' \
     --useAccount $DELEGATE

NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
    vote_delegated '{"evm_address":"'$EVM_ADDRESS_ACC0'","voting_power":"1'$YOCTO_UNITS'","contract_address":"xx","votable_object_id":"xx2"}' \
     --useAccount $DELEGATE

NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
    vote_delegated '{"evm_address":"'$EVM_ADDRESS_ACC0'","voting_power":"3'$YOCTO_UNITS'","contract_address":"xx","votable_object_id":"xx3"}' \
     --useAccount $DELEGATE

NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
    vote_delegated '{"evm_address":"'$EVM_ADDRESS_ACC0'","voting_power":"5'$YOCTO_UNITS'","contract_address":"xx","votable_object_id":"xx4"}' \
     --useAccount $DELEGATE

NEAR_ENV=testnet near call $METAVOTE_CONTRACT_ADDRESS \
    vote_delegated '{"evm_address":"'$EVM_ADDRESS_ACC0'","voting_power":"3'$YOCTO_UNITS'","contract_address":"xx","votable_object_id":"xx5"}' \
     --useAccount $DELEGATE

NEAR_ENV=testnet near view $METAVOTE_CONTRACT_ADDRESS \
    get_voter_info '{"voter_id":"'$EVM_ADDRESS_ACC0'.evmp.near"}' \
