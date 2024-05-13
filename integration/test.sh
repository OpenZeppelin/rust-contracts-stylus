#!/bin/zsh
set -o pipefail

deploy_contract () {
  CONTRACT_CRATE_NAME=$1
  CONTRACT_BIN_NAME="${CONTRACT_CRATE_NAME/-/_}.wasm"
  PRIVATE_KEY=0x5744b91fe94e38f7cde31b0cc83e7fa1f45e31c053d015b9fb8c9ab3298f8a2d
  LOCAL_NODE_HOST=http://localhost:8547

  echo "Deploying contract $CONTRACT_CRATE_NAME."

  DEPLOY_OUTPUT=$(cargo stylus deploy --wasm-file-path target/wasm32-unknown-unknown/release/"$CONTRACT_BIN_NAME" -e $LOCAL_NODE_HOST --private-key $PRIVATE_KEY) || exit $?

  echo "Contract $CONTRACT_CRATE_NAME successfully deployed to the local nitro node ($LOCAL_NODE_HOST)."

  # extract randomly created contract deployment address
  DEPLOYMENT_ADDRESS="$(echo "$DEPLOY_OUTPUT" | grep 'Deploying program to address' | grep -oE "(0x)?[0-9a-fA-F]{40}")"

  if [[ -z "$DEPLOYMENT_ADDRESS" ]]
  then
    echo "Error: Couldn't retrieve deployment address for a contract $CONTRACT_CRATE_NAME."
    exit 1
  fi
}

cargo build --release --profile release --target wasm32-unknown-unknown

deploy_contract erc721-example
export ERC721_DEPLOYMENT_ADDRESS=$DEPLOYMENT_ADDRESS

RUST_TEST_THREADS=1 cargo test -p integration