#!/bin/bash
set -e

source ./scripts/helpers.sh

# Get the root directory of the git repository
ROOT_DIR=$(git rev-parse --show-toplevel)
cd "$ROOT_DIR" || exit

# nested example projects are not able to use the installed toolchain unless
# explicitly instructed to do so
TOOLCHAIN_ARG=""
if [ -n "$RUST_TOOLCHAIN" ]; then
  TOOLCHAIN_ARG="+$RUST_TOOLCHAIN"
fi

# Check contract wasm binary by crate path
check_wasm() {
  local CRATE_PATH=$1

  echo "Checking formatting for $CRATE_PATH"

  cd "$CRATE_PATH"

  cargo $TOOLCHAIN_ARG fmt --all --check

  cd "$ROOT_DIR"
}


# no need to set TOOLCHAIN_ARG as the toolchain is overriden automatically
# format main crates
cargo fmt --all --check

# format examples
for CRATE_PATH in $(get_example_dirs); do
  check_wasm "$CRATE_PATH"
done
