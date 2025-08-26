echo "use cargo near build to build the contracts"
cd meta-vote-contract
cargo near build non-reproducible-wasm && cp ../target/near/meta_vote_contract/meta_vote_contract.wasm ../res/
cd ..

