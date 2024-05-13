#!/bin/zsh
set -o pipefail

export ALICE_PRIV_KEY=0x5744b91fe94e38f7cde31b0cc83e7fa1f45e31c053d015b9fb8c9ab3298f8a2d
export BOB_PRIV_KEY=0xa038232e463efa8ad57de6f88cd3c68ed64d1981daff2dcc015bce7eaf53db9d
export RPC_URL=${RPC_URL:-http://localhost:8547}

deploy_contract () {
  CONTRACT_CRATE_NAME=$1
  CONTRACT_BIN_NAME="${CONTRACT_CRATE_NAME/-/_}.wasm"
  PRIVATE_KEY=$ALICE_PRIV_KEY
  RPC_URL=$RPC_URL

  echo "Deploying contract $CONTRACT_CRATE_NAME."

  DEPLOY_OUTPUT=$(cargo stylus deploy --wasm-file-path target/wasm32-unknown-unknown/release/"$CONTRACT_BIN_NAME" -e $RPC_URL --private-key $PRIVATE_KEY) || exit $?

  echo "Contract $CONTRACT_CRATE_NAME successfully deployed to the local nitro node ($RPC_URL)."

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