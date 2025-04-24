#!/bin/bash
set -e

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

mydir=$(git rev-parse --show-toplevel)
cd "$mydir" || exit

# # Check contract wasm binary by crate name
# check_wasm () {
#   local CONTRACT_CRATE_NAME=$1
#   local CONTRACT_BIN_NAME="${CONTRACT_CRATE_NAME//-/_}.wasm"

#   echo
#   echo "Checking contract $CONTRACT_CRATE_NAME"
#   cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc --wasm-file ./target/wasm32-unknown-unknown/release/"$CONTRACT_BIN_NAME"
# }

# Retrieve all Cargo.toml paths in the `./examples` directory.
get_example_manifest_paths () {
  find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs grep 'name = ' | grep -oE '".*"' | tr -d "'\""
}

# cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

for CRATE_NAME in $(get_example_manifest_paths``)
do
  echo "$mydir/$CRATE_NAME"
done
