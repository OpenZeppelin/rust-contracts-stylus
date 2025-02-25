#!/bin/bash
set -e

source scripts/utils.sh

MYDIR=$(realpath "$(dirname "$0")")
cd "$MYDIR"
cd ..

# Optimize contract's wasm binary by crate name.
opt_wasm () {
  local CONTRACT_CRATE_NAME=$1
  local CONTRACT_BIN_NAME="${CONTRACT_CRATE_NAME//-/_}.wasm"
  local CONTRACT_OPT_BIN_NAME="${CONTRACT_CRATE_NAME//-/_}_opt.wasm"

  echo
  echo "Optimizing $CONTRACT_CRATE_NAME WASM binary"
  # https://rustwasm.github.io/book/reference/code-size.html#use-the-wasm-opt-tool
  wasm-opt -O3 -o ./target/wasm32-unknown-unknown/release/"$CONTRACT_OPT_BIN_NAME" ./target/wasm32-unknown-unknown/release/"$CONTRACT_BIN_NAME"
}

build_contract

# Optimize contract's wasm for gas usage.
for CRATE_NAME in $(get_example_crate_names)
do
  opt_wasm "$CRATE_NAME"
done

export RPC_URL=http://localhost:8547

# No need to compile benchmarks with `--release`
# since this only runs the benchmarking code and the contracts have already been compiled with `--release`.
cargo run -p benches
echo "This benchmarks measure gas execution cost,
 the 21000 EVM base gas fee is omitted."
echo
echo "To measure non cached contract's gas usage correctly,
 benchmarks should run on a clean instance of the nitro test node."
echo
echo "Finished running benches!"
