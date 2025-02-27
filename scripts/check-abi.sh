#!/bin/bash
set -e

source scripts/utils.sh

mydir=$(dirname "$0")
cd "$mydir" || exit
cd ..

# Check contract ABI by crate name
check_abi () {
  local CONTRACT_CRATE_NAME=$1

  echo
  echo "Checking contract ABI: $CONTRACT_CRATE_NAME"
}

build_contract
for CRATE_NAME in $(get_example_crate_names)
do
  check_abi "$CRATE_NAME"
done
