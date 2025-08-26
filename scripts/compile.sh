echo "use cargo near build to build the contracts"
echo "checking for errors"
cargo check --target wasm32-unknown-unknown
echo "just used cargo check --target wasm32-unknown-unknown"
echo "use -- cargo near build -- ON EACH CONTRACT FOLDER, to build the contracts"
# export RUSTFLAGS='-C link-arg=-s'
# cargo build -p meta-vote-contract --target wasm32-unknown-unknown --release
# cargo build -p kv-store-contract --target wasm32-unknown-unknown --release
# cargo build -p mpip-contract --target wasm32-unknown-unknown --release
# cargo build -p test-meta-token --target wasm32-unknown-unknown --release
# cp target/wasm32-unknown-unknown/release/meta_vote_contract.wasm res/
# cp target/wasm32-unknown-unknown/release/mpip_contract.wasm res/
# cp target/wasm32-unknown-unknown/release/test_meta_token.wasm res/
# echo =========================================================
# rustc_version=$(rustc --version | awk '{print $2}')
# echo RUST version:  $rustc_version
# echo =========================================================
# echo WARN: If rustc version is 1.82 or 1.83 after deploy you will get Deserialization ERROR!!!
# echo WARN: The error is "wasm execution failed with error: CompilationError(PrepareError(Deserialization))"
# echo =========================================================
# BAD_VERSION="1.82"
# if [[ $(echo -e "$rustc_version\n$BAD_VERSION" | sort -V | head -n1) == "$BAD_VERSION" ]]; then
#     exit 1
# fi
