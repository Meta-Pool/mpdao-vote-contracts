set -ex
rustup override set 1.80
# RUSTFLAGS optimizations for NEAR smart contracts:
# -C link-arg=-s: Strip all symbols from the resulting wasm (reduces file size)
# -C target-cpu=generic: Use generic CPU features for better compatibility
# -C opt-level=z: Optimize for size rather than speed (critical for NEAR contracts)
# -C lto=fat: Enable Link Time Optimization for better size reduction
# -C codegen-units=1: Use single codegen unit for better optimization
export RUSTFLAGS='-C link-arg=-s -C target-cpu=generic -C opt-level=z -C lto=fat -C codegen-units=1'
echo =========================================================
rustc_version=$(rustc --version | awk '{print $2}')
echo RUST version:  $rustc_version
echo =========================================================
echo WARN: If rustc version is 1.82 or higher, after deploy you will get Deserialization ERROR!!!
echo WARN: The error is "wasm execution failed with error: CompilationError(PrepareError(Deserialization))"
echo =========================================================
GOOD_VERSION="1.80"
if [[ $(echo -e "$rustc_version\n$GOOD_VERSION" | sort -V | head -n1) != "$GOOD_VERSION" ]]; then
    exit 1
fi
echo "use cargo near build to build the contracts"
## meta-vote contract
cd meta-vote-contract
cargo near build reproducible-wasm && cp ../target/near/meta_vote_contract/meta_vote_contract.wasm ../res/
cd ..
