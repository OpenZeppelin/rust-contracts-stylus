#!/bin/bash
# Run the end-to-end tests against the local devnet (`./scripts/devnet.sh up`).
#
# Any arguments are passed to `cargo test`, e.g. `-p erc20-example`.
set -e

# Navigate to project root
cd "$(dirname "$(realpath "$0")")/.."

cargo build --release --target wasm32-unknown-unknown

# If any arguments are set, just pass them as-is to the cargo test command
if [[ $# -eq 0 ]]; then
    cargo test --features e2e --test "*"
else
    cargo test --features e2e "$@"
fi
