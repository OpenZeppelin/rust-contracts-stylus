#!/bin/bash
set -o pipefail

# make sure we will be running script from the project root.
mydir=$(dirname "$0")
cd "$mydir" || exit
cd ..

# Deploy contract by rust crate name.
# Sets $DEPLOYMENT_ADDRESS environment variable after successful deployment.
deploy_contract () {
  local CONTRACT_CRATE_NAME=$1
  local CONTRACT_BIN_NAME="${CONTRACT_CRATE_NAME//-/_}.wasm"
  local PRIVATE_KEY=$ALICE_PRIV_KEY
  local RPC_URL=$RPC_URL

  echo "Deploying contract $CONTRACT_CRATE_NAME."

  DEPLOY_OUTPUT=$(cargo stylus deploy --wasm-file-path ./target/wasm32-unknown-unknown/release/"$CONTRACT_BIN_NAME" -e "$RPC_URL" --private-key "$PRIVATE_KEY") || exit $?

  echo "Contract $CONTRACT_CRATE_NAME successfully deployed to the stylus environment ($RPC_URL)."

  # extract randomly created contract deployment address
  # NOTE: optimistically relying on the 'Deploying program to address' string in output
  DEPLOYMENT_ADDRESS="$(echo "$DEPLOY_OUTPUT" | grep 'Deploying program to address' | grep -oE "(0x)?[0-9a-fA-F]{40}")"

  if [[ -z "$DEPLOYMENT_ADDRESS" ]]
  then
    echo "Error: Couldn't retrieve deployment address for a contract $CONTRACT_CRATE_NAME."
    exit 1
  fi

  DEPLOYMENT_ADDRESS_ENV_VAR_NAME="$(echo "$CRATE_NAME" | tr '-' '_' | tr '[:lower:]' '[:upper:]')_DEPLOYMENT_ADDRESS"

  # export dynamically created variable
  set -a
  printf -v "$DEPLOYMENT_ADDRESS_ENV_VAR_NAME" "%s" "$DEPLOYMENT_ADDRESS"
  set +a
}

# Retrieve all contract's crate names in `./examples` directory.
get_example_crate_names () {
  # NOTE: optimistically relying on the 'name = ' string at Cargo.toml file
  find ./examples -type f -print0 -name "Cargo.toml" | xargs -0 grep 'name = ' | grep -oE '".*"' | tr -d "'\""
}

export ALICE_PRIV_KEY=${ALICE_PRIV_KEY:-0x5744b91fe94e38f7cde31b0cc83e7fa1f45e31c053d015b9fb8c9ab3298f8a2d}
export BOB_PRIV_KEY=${BOB_PRIV_KEY:-0xa038232e463efa8ad57de6f88cd3c68ed64d1981daff2dcc015bce7eaf53db9d}
export RPC_URL=${RPC_URL:-http://localhost:8547}

cargo build --release --target wasm32-unknown-unknown

# TODO: deploy contracts asynchronously
for CRATE_NAME in $(get_example_crate_names)
do
  deploy_contract "$CRATE_NAME"
done

# TODO: run tests in parallel when concurrency scope will be per test/contract
RUST_TEST_THREADS=1 cargo test -p e2e-tests --all-features