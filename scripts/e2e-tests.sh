#!/bin/bash
set -e

source scripts/utils.sh

# Navigate to project root
cd "$(dirname "$(realpath "$0")")/.."

build_contract

export RPC_URL=http://localhost:8547

# If any arguments are set, just pass them as-is to the cargo test command
if [[ $# -eq 0 ]]; then
    cargo test --features e2e --test "*"
else
    cargo test --features e2e "$@"
fi
