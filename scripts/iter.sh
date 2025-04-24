#!/bin/bash
set -e

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0x6ac4839Bfe169CadBBFbDE3f29bd8459037Bf64e

mydir=$(git rev-parse --show-toplevel)
cd "$mydir" || exit

# Retrieve all Cargo.toml paths in the `./examples` directory.
get_example_manifest_paths () {
  find ./examples -maxdepth 2 -type f -name "Cargo.toml"
}

for CRATE_NAME in $(get_example_manifest_paths)
do
  cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --manifest-path "$CRATE_NAME"
  cargo test --features e2e --manifest-path "$CRATE_NAME"
done