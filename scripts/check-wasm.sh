#!/bin/bash
# Check that every example contract can be deployed and activated on a Stylus
# chain: `cargo stylus check` compresses the WASM, verifies the size against
# the chain's limits and simulates the activation.
#
# The check runs against `RPC_URL` when set, otherwise against the local
# devnet, which is started for the duration of the script if it is not
# running.
set -e

mydir=$(dirname "$0")
cd "$mydir" || exit
cd ..

if [ -z "${RPC_URL:-}" ]; then
  STARTED_DEVNET=false
  if ! ./scripts/devnet.sh status | grep -q '^running'; then
    ./scripts/devnet.sh up
    STARTED_DEVNET=true
  fi
  RPC_URL=$(cat "${CARGO_TARGET_DIR:-target}/devnet/rpc")
  if $STARTED_DEVNET; then
    trap './scripts/devnet.sh down' EXIT
  fi
fi

# Check contract wasm binary by crate name
check_wasm() {
  local CONTRACT_CRATE_NAME=$1
  local CONTRACT_BIN_NAME="${CONTRACT_CRATE_NAME//-/_}.wasm"

  echo
  echo "Checking contract $CONTRACT_CRATE_NAME"
  cargo stylus check -e "$RPC_URL" --wasm-file ./target/wasm32-unknown-unknown/release/"$CONTRACT_BIN_NAME"
}

# Retrieve all alphanumeric contract's crate names in `./examples` directory.
get_example_crate_names() {
  # shellcheck disable=SC2038
  # NOTE: optimistically relying on the 'name = ' string at Cargo.toml file
  find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs grep -m 1 'name = ' | grep -oE '".*"' | tr -d "'\""
}

cargo build --release --target wasm32-unknown-unknown

for CRATE_NAME in $(get_example_crate_names); do
  check_wasm "$CRATE_NAME"
done
