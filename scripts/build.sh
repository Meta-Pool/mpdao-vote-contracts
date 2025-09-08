rustup override set 1.81
export RUSTFLAGS='-C link-arg=-s'
echo =========================================================
rustc_version=$(rustc --version | awk '{print $2}')
echo RUST version:  $rustc_version
echo =========================================================
echo WARN: If rustc version is 1.82 or higher, after deploy you will get Deserialization ERROR!!!
echo WARN: teh worng rust version will compile, but the wasm binary will not work.
echo WARN: You will get "CompilationError(PrepareError(Deserialization))" when running in the blockchain
echo =========================================================
GOOD_VERSION="1.81"
if [[ $(echo -e "$rustc_version\n$GOOD_VERSION" | sort -V | head -n1) != "$GOOD_VERSION" ]]; then
    exit 1
fi

cargo build -p meta-vote-contract --target wasm32-unknown-unknown --release
cp -u target/wasm32-unknown-unknown/release/meta_vote_contract.wasm res/

cargo build -p kv-store-contract --target wasm32-unknown-unknown --release
cp -u target/wasm32-unknown-unknown/release/kv_store_contract.wasm res/

cargo build -p mpip-contract --target wasm32-unknown-unknown --release
cp -u target/wasm32-unknown-unknown/release/mpip_contract.wasm res/

# cargo build -p test-meta-token --target wasm32-unknown-unknown --release
# cp target/wasm32-unknown-unknown/release/test_meta_token.wasm res/
