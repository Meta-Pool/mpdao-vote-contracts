echo "use cargo near build to build the contracts"
echo "checking for errors"
cargo check --target wasm32-unknown-unknown
echo "just used cargo check --target wasm32-unknown-unknown"
echo "use `cargo near build` ON EACH CONTRACT FOLDER, to build the contracts"
