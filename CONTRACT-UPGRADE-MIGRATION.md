# Contract upgrade and migration

What was deployed on NEAR testnet on 29 Oct 2026, and what to repeat for mainnet. Mainnet was not upgraded.

## Testnet result

Account: `v1.mpdao-vote.testnet` (the vote contract `meta-pool-ui` uses in testnet mode).

Transaction: https://explorer.testnet.near.org/transactions/Ch8sbrnUE7NQS7D49Wesnnpom9bnfymSCWxWnXCZ31DG

Code hash: `4hEUFxA9bycyhaLarkqSzHzTiwwnGdjy7VGwBi7JdAfU`

The receipt deployed the wasm and called `migrate` with `{}`. It succeeded. Views before and after:

| Check | Before | After |
|---|---|---|
| `get_operator_id` | `operator.mpdao-vote.testnet` | `operator.mpdao-vote.testnet` |
| `get_voters_count` | 67 | 67 |
| `get_total_voting_power` | `56086941780202666666666666666665` | `56086941780202666666666666666665` |
| `is_eth_mirror_realized` | method missing | `false` |
| `get_sell_price_guard` | method missing | empty guard (`days: []`, payouts `0`) |

`migrate` reads the previous state and appends two fields:

- `sell_price_guard`, initialized empty. The testnet wasm did not have this field yet.
- `realized_eth_mirrors`, an empty map. `realize_mirror` refuses a second conversion of the same Ethereum address, and `operator_mirror_lps` skips an address once it is in this map.

A failed `migrate` reverts the whole receipt, including the code deploy. The two earlier attempts did not change the contract.

## Why the first attempts failed

1. The file that was uploaded was a wasm from September, not the build just produced. `CARGO_TARGET_DIR` points at a sandbox cache, so `res/meta_vote_contract.wasm` and `target/wasm32-unknown-unknown/release/` were stale. That old `migrate` does not match the live state (`Not all bytes read`).
2. The next real wasm treated `sell_price_guard` as already stored. It is not on this contract (`get_sell_price_guard` was missing), so deserialization ran out of bytes (`Unexpected length of input`).

## Build

Use Rust 1.81. `rust-toolchain.toml` pins it. `scripts/build.sh` refuses 1.82 or newer: the wasm compiles, then the chain rejects it with `CompilationError(PrepareError(Deserialization))`.

```sh
cd mpdao-vote-contracts
CARGO_TARGET_DIR=/tmp/mpdao-wasm-target \
  RUSTFLAGS='-C link-arg=-s' \
  rustup run 1.81 cargo build -p meta-vote-contract --target wasm32-unknown-unknown --release
```

Confirm the binary before deploying:

```sh
rustup run 1.81 rustc --version
# rustc 1.81.0

strings /tmp/mpdao-wasm-target/wasm32-unknown-unknown/release/meta_vote_contract.wasm \
  | grep "eth address already realized"
```

The testnet wasm was 608629 bytes, sha256 `36e1860fa59f8f4165375937c036856e23308c31dd55d15bad70528bff2271ab`.

## Deploy command that worked

`near` here is near-cli-rs 0.23.6. The shell scripts under `scripts/testnet` and `scripts/mainnet` still call the old `near deploy` form. Use this instead, and pass the wasm path from the build above, not a stale `res/` copy.

```sh
near contract deploy v1.mpdao-vote.testnet \
  use-file /tmp/mpdao-wasm-target/wasm32-unknown-unknown/release/meta_vote_contract.wasm \
  with-init-call migrate json-args '{}' \
  prepaid-gas '300 Tgas' \
  attached-deposit '0 NEAR' \
  network-config testnet \
  sign-with-legacy-keychain \
  send
```

The signer is the contract account. The key is `~/.near-credentials/testnet/v1.mpdao-vote.testnet.json`.

## Mainnet, when testnet behavior is accepted

Do this only after the testnet freeze and realize path has been exercised. As of this note, mainnet has not been upgraded.

Account: `mpdao-vote.near`. Owner in `scripts/mainnet/mainnet-set-vars.sh`: `meta-pool-dao.near`. Operator: `operator.mpdao-vote.near`.

Checked on 29 Oct 2026, before any mainnet deploy:

| Check | Result |
|---|---|
| `get_operator_id` | `operator.mpdao-vote.near` |
| `get_voters_count` | 2148 |
| `get_total_voting_power` | `671378051059835750000000000000000` |
| `get_sell_price_guard` | method missing |
| `is_eth_mirror_realized` | method missing |
| `get_claimable_unlocked_mpdao` | method exists |

That is the same shape as testnet was: state ends at the unlocked-mpDAO fields, so the current `OldState` in `meta-vote-contract/src/migrate.rs` matches. Re-run those views immediately before the mainnet deploy. If `get_sell_price_guard` exists by then, stop. `OldState` would have to include `sell_price_guard` or `migrate` will fail with `Unexpected length of input`. If some other field was appended, stop as well. A wrong `OldState` reverts, but only if the receipt is one deploy-plus-migrate. Do not deploy the code in a separate transaction from `migrate`.

Record `get_operator_id`, `get_voters_count`, and `get_total_voting_power` first. After the transaction they must be unchanged, `is_eth_mirror_realized` must return `false` for an address that was not realized, and `get_sell_price_guard` must return an empty guard.

Build with the same Rust 1.81 command. Deploy with `network-config mainnet` and the `mpdao-vote.near` key. Do not point this at any other account.

```sh
near contract deploy mpdao-vote.near \
  use-file /tmp/mpdao-wasm-target/wasm32-unknown-unknown/release/meta_vote_contract.wasm \
  with-init-call migrate json-args '{}' \
  prepaid-gas '300 Tgas' \
  attached-deposit '0 NEAR' \
  network-config mainnet \
  sign-with-legacy-keychain \
  send
```

## Not done

The Ethereum voting proxy was not upgraded. Sepolia is `0xb98211076dD150164b20702CD36ABe8f2eB7C027`. The script is `evm-mpdao-vote-contracts/scripts/testnet/upgrade-voting-power-v2.js`. The deployer key is `DEPLOYER_PRIV_KEY_ENCRYPTED` and the decrypt script asks for a password. Mainnet Ethereum (`0xF8709E3e8ED3dB79744579D3FA1795b514104113`) stays on `VotingPowerV1` until that testnet upgrade has been used.

The buyback bot change is in the repo only. It was not restarted against testnet or mainnet.
