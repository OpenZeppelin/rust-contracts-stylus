#!/bin/bash
set -e

# Get the root directory of the git repository
ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR" || exit

# Check contract wasm binary by crate name
check_wasm() {
  local CRATE_PATH=$1

  echo "Checking formatting for $CRATE_PATH"

  cd "$CRATE_PATH"

  cargo fmt --all --check

  cd "$ROOT_DIR"
}

# Function to retrieve all Cargo.toml paths in the ./examples directory
get_example_dirs() {
  find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs -n1 dirname | sort
}

# format main crates
cargo fmt --all --check

# format examples
for CRATE_PATH in $(get_example_dirs); do
  check_wasm "$CRATE_PATH"
done
