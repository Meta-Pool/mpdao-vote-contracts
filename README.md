# mpDAO Vote Contract

![Vote Logo](media/logo.png)

Implementation of a general voting system for metapool.app DAO using the mpDAO token.

## Testing

For integration testing see the repo metapool/mpdao-vote-contracts-integration-testing

## Contract functions

### Deposit & Locking

* Use ft_transfer_call on mpDAO token NEP-141 contract, in the `msg` field set the number of days for the unbonding period

* to see how this contract process deposits check `fn ft_on_transfer()`

### Getting a list of paginated Voters

Getting all the voters with pagination using `testnet`:

```rs
NEAR_ENV=testnet near view metavote.testnet get_voters '{"from_index": 0, "limit": 100}'
```

### View Contract Functions

```rs
pub fn get_voters(&self, from_index: u32, limit: u32) -> Vec<VoterJSON>;

pub fn get_balance(&self, voter_id: VoterId) -> U128;

pub fn get_locked_balance(&self, voter_id: VoterId) -> U128;

pub fn get_unlocking_balance(&self, voter_id: VoterId) -> U128;

pub fn get_available_voting_power(&self, voter_id: VoterId) -> U128;

pub fn get_used_voting_power(&self, voter_id: VoterId) -> U128;

pub fn get_locking_period(&self) -> (Days, Days);

pub fn get_all_locking_positions(
    &self,
    voter_id: VoterId
) -> Vec<LockingPositionJSON>;

pub fn get_locking_position(
    &self,
    index: PositionIndex,
    voter_id: VoterId
) -> Option<LockingPositionJSON>;

pub fn get_total_votes(
    &self,
    contract_address: ContractAddress,
    votable_object_id: VotableObjId
) -> U128;

// votes by app (contract)
// returns [[votable_bj_id, vote_amount],[votable_bj_id, vote_amount]...]
pub fn get_votes_by_app(&self, app_or_contract_address: String) -> Vec<(String, U128String)>;

pub fn get_votes_by_voter(
    &self,
    voter_id: VoterId
) -> Vec<VotableObjectJSON>;

pub fn get_votes_for_object(
    &self,
    voter_id: VoterId,
    contract_address: ContractAddress,
    votable_object_id: VotableObjId
) -> U128;
```

### Mutating Contract state

```rs
// *************
// * Unlocking *
// *************

pub fn unlock_position(&mut self, index: PositionIndex);

pub fn unlock_partial_position(&mut self, index: PositionIndex, amount: U128);

// ***********
// * Re-Lock *
// ***********

pub fn relock_position(
    &mut self,
    index: PositionIndex,
    locking_period: Days,
    amount_from_balance: U128
);

pub fn relock_partial_position(
    &mut self,
    index: PositionIndex,
    amount_from_position: U128,
    locking_period: Days,
    amount_from_balance: U128
);

pub fn relock_from_balance(
    &mut self,
    locking_period: Days,
    amount_from_balance: U128
);


// ************
// * Withdraw *
// ************

#[payable]
pub fn withdraw(
    &mut self,
    position_index_list: Vec<PositionIndex>,
    amount_from_balance: U128
);

#[payable]
pub fn withdraw_all(&mut self);

// **********
// * Voting *
// **********

pub fn vote(
    &mut self,
    voting_power: U128,
    contract_address: ContractAddress,
    votable_object_id: VotableObjId
);

pub fn rebalance(
    &mut self,
    voting_power: U128,
    contract_address: ContractAddress,
    votable_object_id: VotableObjId
);

pub fn unvote(
    &mut self,
    contract_address: ContractAddress,
    votable_object_id: VotableObjId
);
```

### Locking, re-locking, Unbonding process

To Lock funds into the mpDAO Vote contract, the user must define an amount in the $mpDAO token and a number of days (between 30 and 300 days) as the unbonding period.

![Locking and Unbonding](media/process.png)

### Consider:

- To reclaim the mpDAO tokens, you will have start the unbonding and then wait the unbonding period.

- During the unbonding period, you can not vote and you will not get paid.

- During the unbonding period, you can chose to re-lock the tokens to be able vote and get paid again. After the voting is over, you can also chose to continue the unbonding process where you left.

- Implement **NEP264** for cross-contract calls: https://github.com/near/near-sdk-rs/issues/740
- Release notes for `near-sdk = "4.0.0"`: https://github.com/near/near-sdk-rs/discussions/797


### Deploying a new instance of mpDAO Vote

To deploy check scripts/testnet and scripts/mainnet
