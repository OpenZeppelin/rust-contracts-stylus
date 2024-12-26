#!/bin/bash
set -e

# Navigate to project root
cd "$(dirname "$(realpath "$0")")/.."

cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

export RPC_URL=http://localhost:8547

# If any arguments are set, just pass them as-is to the cargo test command
if [[ $# -eq 0 ]]; then
    cargo test --features std,e2e --test "*"
else
    cargo test --features std,e2e "$@"
fi
