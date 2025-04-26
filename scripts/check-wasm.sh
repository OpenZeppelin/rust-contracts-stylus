#!/bin/bash
set -e

# Get the root directory of the git repository
ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR" || exit

# Check contract wasm binary by crate name
check_wasm() {
  local CRATE_PATH=$1

  echo
  echo "Checking contract $CRATE_PATH"

  cd "$CRATE_PATH"

  cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

  cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc

  cd "$ROOT_DIR"
}

# Function to retrieve all Cargo.toml paths in the ./examples directory
get_example_dirs() {
  find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs -n1 dirname | sort
}

for CRATE_PATH in $(get_example_dirs); do
  check_wasm "$CRATE_PATH"
done
