#!/bin/bash
set -e

# Navigate to project root
cd "$(dirname "$(realpath "$0")")/.."

cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

export RPC_URL=http://localhost:8547
export DEPLOYER_ADDRESS=0xD678cE107b0627e08393680bE6cA41F583b53AA4

# If any arguments are set, just pass them as-is to the cargo test command
if [[ $# -eq 0 ]]; then
    cargo test --features e2e --test "*"
else
    cargo test --features e2e "$@"
fi
