export RUSTFLAGS='-C link-arg=-s'
cargo build -p meta-vote-contract --target wasm32-unknown-unknown --release
cargo build -p kv-store-contract --target wasm32-unknown-unknown --release
cargo build -p mpip-contract --target wasm32-unknown-unknown --release
cargo build -p test-meta-token --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/meta_vote_contract.wasm res/
cp target/wasm32-unknown-unknown/release/mpip_contract.wasm res/
cp target/wasm32-unknown-unknown/release/test_meta_token.wasm res/
echo =========================================================
echo RUST version:
rustc --version
echo WARN: If rustc version is 1.82 or 1.83 after deploy you will get Deserialization ERROR!!!
echo WARN: The error is "wasm execution failed with error: CompilationError(PrepareError(Deserialization))"
echo =========================================================
