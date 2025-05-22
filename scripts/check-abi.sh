#!/bin/bash
set -e

# Function to retrieve all Cargo.toml paths in the ./examples directory
get_example_dirs() {
  find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs -n1 dirname | sort
}

# Get the root directory of the git repository
ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR" || exit

# Check contract ABI by crate name
check_abi() {
  local CRATE_PATH=$1

  echo "Checking contract $CRATE_PATH"

  echo

  cd "$CRATE_PATH"

  cargo stylus export-abi

  echo

  echo "Done!"

  echo

  cd "$ROOT_DIR"
}


for CRATE_PATH in $(get_example_dirs); do
  check_abi "$CRATE_PATH"
done
