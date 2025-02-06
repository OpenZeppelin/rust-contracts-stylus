#!/bin/bash
set -e

source scripts/utils.sh
mydir=$(dirname "$0")
cd "$mydir" || exit
cd ..

# Check contract wasm binary by crate name
check_wasm () {
  local CONTRACT_CRATE_NAME=$1
  local CONTRACT_BIN_NAME="${CONTRACT_CRATE_NAME//-/_}.wasm"

  echo
  echo "Checking contract $CONTRACT_CRATE_NAME"
  cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc --wasm-file ./target/wasm32-unknown-unknown/release/"$CONTRACT_BIN_NAME"
}

build_contract
for CRATE_NAME in $(get_example_crate_names)
do
  check_wasm "$CRATE_NAME"
done
