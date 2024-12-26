#!/bin/bash
set -e

# Move to project root
cd "$(dirname "$(realpath "$0")")/.."

# Build the project
cargo build --release --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort

# Set RPC_URL environment variable
export RPC_URL=http://localhost:8547

# Run tests based on arguments
if [[ $# -eq 0 ]]; then
    cargo test --features std,e2e --test "*"
else
    cargo test --features std,e2e "$@"
fi
